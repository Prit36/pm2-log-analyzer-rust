//! Native PM2 pipeline — mirrors the Wasm coordinator
//! (`src/workers/logParserWorker.ts` + `src/workers/shardParserWorker.ts`).
//!
//! Parsing runs the vendored kernel exactly as the browser shard workers do
//! (same shard split, same read window, same 16 MiB feed chunks). Aggregation
//! mirrors the JS coordinator: absorb shard meta in shard order, then merge the
//! per-shard partial wires with the reference merge semantics.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use chrono::{Local, NaiveDateTime, TimeZone};
use memmap2::Mmap;
use rayon::prelude::*;

use crate::core::models::*;
use crate::core::relhist_js::{
    read_f32, read_f64, read_u16, read_u32, RelHist, RelHistWire,
};
use crate::kernels::pm2::store::Engine;

pub const SHARD_MIN_BYTES: u64 = 8 * 1024 * 1024;
const FEED_CHUNK: usize = 16 * 1024 * 1024;
const LINE_EXTEND: u64 = 256 * 1024;
const UNMATCHED_SAMPLE_CAP: usize = 40;

const MAGIC_PM2P: u32 = 0x504d_3250;
const MAGIC_PM2H: u32 = 0x504d_3248;
const MAGIC_PM2D: u32 = 0x504d_3244;

/// Byte budget / cancellation probe shared with the UI thread.
pub struct JobControl {
    pub processed: AtomicU64,
    pub total: u64,
    pub cancelled: AtomicBool,
}

impl JobControl {
    pub fn new(total: u64) -> Self {
        Self {
            processed: AtomicU64::new(0),
            total,
            cancelled: AtomicBool::new(false),
        }
    }

    pub fn percent(&self) -> u32 {
        if self.total == 0 {
            return 0;
        }
        let done = self.processed.load(Ordering::Relaxed);
        ((done.saturating_mul(100) / self.total).min(99)) as u32
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub enum ParseError {
    Cancelled,
    Io(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Cancelled => write!(f, "Cancelled"),
            ParseError::Io(msg) => write!(f, "{msg}"),
        }
    }
}

// ── Source bytes ────────────────────────────────────────────────────────────

/// One loaded log source: a file on disk or in-memory bytes (paste / archive entry).
#[derive(Clone, PartialEq)]
pub enum LoadedSource {
    Path(PathBuf),
    Memory { name: String, bytes: Arc<Vec<u8>> },
}

impl LoadedSource {
    pub fn name(&self) -> String {
        match self {
            LoadedSource::Path(path) => path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned()),
            LoadedSource::Memory { name, .. } => name.clone(),
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            LoadedSource::Path(path) => std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
            LoadedSource::Memory { bytes, .. } => bytes.len() as u64,
        }
    }
}

enum SourcePart {
    Shared(Arc<Vec<u8>>),
    Mapped(Arc<Mmap>),
}

struct Source {
    parts: Vec<SourcePart>,
    ends: Vec<u64>,
}

impl Source {
    fn from_paths(paths: &[PathBuf]) -> Result<Source, ParseError> {
        let sources: Vec<LoadedSource> = paths
            .iter()
            .map(|p| LoadedSource::Path(p.clone()))
            .collect();
        Source::from_loaded(&sources)
    }

    fn from_loaded(sources: &[LoadedSource]) -> Result<Source, ParseError> {
        let mut parts: Vec<SourcePart> = Vec::with_capacity(sources.len());
        let mut part_ends: Vec<u64> = Vec::with_capacity(sources.len());
        let mut acc = 0u64;
        for source in sources {
            match source {
                LoadedSource::Path(path) => {
                    let file = std::fs::File::open(path).map_err(|e| {
                        ParseError::Io(format!("Failed to open {}: {e}", path.display()))
                    })?;
                    let len = file
                        .metadata()
                        .map_err(|e| {
                            ParseError::Io(format!("Failed to stat {}: {e}", path.display()))
                        })?
                        .len();
                    if len > 0 {
                        let map = unsafe { Mmap::map(&file) }.map_err(|e| {
                            ParseError::Io(format!("Failed to map {}: {e}", path.display()))
                        })?;
                        parts.push(SourcePart::Mapped(Arc::new(map)));
                        acc += len;
                        part_ends.push(acc);
                    }
                }
                LoadedSource::Memory { bytes, .. } => {
                    let len = bytes.len() as u64;
                    if len > 0 {
                        parts.push(SourcePart::Shared(bytes.clone()));
                        acc += len;
                        part_ends.push(acc);
                    }
                }
            }
        }
        Ok(Source {
            parts,
            ends: part_ends,
        })
    }

    pub fn len(&self) -> u64 {
        self.ends.last().copied().unwrap_or(0)
    }

    /// Walk `[start, end)` across source boundaries, yielding `(abs_offset, bytes)`.
    fn for_each_segment(&self, start: u64, end: u64, mut f: impl FnMut(u64, &[u8])) {
        let mut part_start = 0u64;
        for (i, part) in self.parts.iter().enumerate() {
            let part_end = self.ends[i];
            if part_start >= end {
                break;
            }
            if part_end > start {
                let seg_start = start.max(part_start);
                let seg_end = end.min(part_end);
                if seg_start < seg_end {
                    let local = (seg_start - part_start) as usize;
                    let local_end = (seg_end - part_start) as usize;
                    match part {
                        SourcePart::Shared(bytes) => f(seg_start, &bytes[local..local_end]),
                        SourcePart::Mapped(map) => f(seg_start, &map[local..local_end]),
                    }
                }
            }
            part_start = part_end;
        }
    }
}

// ── Shard orchestration ─────────────────────────────────────────────────────

/// Shard pool size. Mirrors the reference worker pool (4) by default; native runs can widen
/// it with `PM2_ANALYZER_SHARDS` since there is no per-worker Wasm memory cost here.
pub fn pool_size() -> usize {
    let hc = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    match std::env::var("PM2_ANALYZER_SHARDS").ok().and_then(|v| v.parse::<usize>().ok()) {
        Some(n) if n > 0 => n.clamp(1, 64),
        _ => hc.clamp(2, 4),
    }
}

pub fn shard_count_for(file_size: u64) -> usize {
    if file_size < SHARD_MIN_BYTES {
        1
    } else {
        pool_size()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ShardRange {
    pub start: u64,
    pub end: u64,
}

pub fn shard_ranges(file_size: u64, n: usize) -> Vec<ShardRange> {
    if n <= 1 {
        return vec![ShardRange {
            start: 0,
            end: file_size,
        }];
    }
    let chunk = file_size.div_ceil(n as u64);
    let mut out = Vec::new();
    for i in 0..n {
        let start = i as u64 * chunk;
        if start >= file_size {
            break;
        }
        let end = if i == n - 1 {
            file_size
        } else {
            file_size.min((i as u64 + 1) * chunk)
        };
        out.push(ShardRange { start, end });
    }
    out
}

struct Shard {
    engine: Engine,
    /// Prekicked partial wire produced at parse time (default filters).
    default_partial: Option<(NormalizeMode, Vec<u8>)>,
}

struct ShardParsed {
    shard_index: usize,
    engine: Engine,
    hit_count: u64,
    unmatched_count: u64,
    methods_mask: u8,
    cron_wire: Vec<u8>,
    unmatched_wire: Vec<u8>,
    hourly_wire: Vec<u8>,
    dates_wire: Vec<u8>,
    daily_wire: Vec<u8>,
    default_partial: Option<(NormalizeMode, Vec<u8>)>,
}

fn feed_shard(
    engine: &mut Engine,
    source: &Source,
    start: u64,
    end: u64,
    file_size: u64,
    control: &JobControl,
) -> Result<(), ParseError> {
    engine.begin_shard(start, end, file_size);
    let read_end = (end + LINE_EXTEND).min(file_size);
    let mut off = if start > 0 { start - 1 } else { start };
    while off < read_end {
        if control.cancelled.load(Ordering::Relaxed) {
            return Err(ParseError::Cancelled);
        }
        let take = ((read_end - off) as usize).min(FEED_CHUNK);
        let chunk_end = off + take as u64;
        source.for_each_segment(off, chunk_end, |abs, bytes| {
            engine.feed_bytes(bytes, abs);
        });
        control.processed.fetch_add(take as u64, Ordering::Relaxed);
        off = chunk_end;
    }
    engine.end_shard();
    Ok(())
}

fn parse_one_shard(
    source: &Source,
    range: ShardRange,
    shard_index: usize,
    file_size: u64,
    normalize_mode: NormalizeMode,
    control: &JobControl,
) -> Result<ShardParsed, ParseError> {
    let mut engine = Engine::new();
    feed_shard(
        &mut engine,
        source,
        range.start,
        range.end,
        file_size,
        control,
    )?;
    engine.ensure_mode(normalize_mode.code());
    let partial_wire = engine.reaggregate(normalize_mode.code(), 0, 0.0, b"", true);
    Ok(ShardParsed {
        shard_index,
        hit_count: engine.hit_count() as u64,
        unmatched_count: engine.unmatched_count() as u64,
        methods_mask: engine.methods_mask(),
        cron_wire: engine.cron_wire(),
        unmatched_wire: engine.unmatched_sample_wire(),
        hourly_wire: engine.hourly_wire(),
        dates_wire: engine.dates_wire(),
        daily_wire: engine.daily_wire(),
        default_partial: Some((normalize_mode, partial_wire)),
        engine,
    })
}

// ── Kernel ──────────────────────────────────────────────────────────────────

pub struct Pm2Kernel {
    shards: Vec<Shard>,
    pub file_size: u64,
    pub hit_count: u64,
    pub unmatched_count: u64,
    pub unmatched_sample: Vec<String>,
    pub cron_events: Vec<CronEventCompact>,
    pub methods: Vec<String>,
    pub dates: Vec<String>,
    pub hourly_stats: Vec<HourlyBucket>,
    pub daily_stats: Vec<DaySummary>,
    cached_summary: Option<LogSummary>,
}

impl Pm2Kernel {
    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }

    pub fn reaggregate(&mut self, opts: &ParseOptions) -> AggregatedResult {
        if self.shards.is_empty() {
            return EMPTY_RESULT;
        }
        let is_date_filtered = opts
            .date_filter
            .as_deref()
            .is_some_and(|d| !d.is_empty() && d != "all");
        let need_summary = self.cached_summary.is_none() || is_date_filtered;
        let use_prekick = !is_date_filtered
            && opts.status_family == StatusFamily::All
            && opts.min_ms == 0.0;
        let mode = opts.normalize_mode;
        let date_bytes: Vec<u8> = if is_date_filtered {
            opts.date_filter.clone().unwrap_or_default().into_bytes()
        } else {
            Vec::new()
        };

        let decoded: Vec<(u64, u64, AggPartial)> = self
            .shards
            .par_iter_mut()
            .map(|shard| {
                if use_prekick {
                    if let Some((pre_mode, wire)) = shard.default_partial.take() {
                        if pre_mode == mode {
                            return decode_pm2_partial(&wire);
                        }
                    }
                }
                let wire = shard.engine.reaggregate(
                    mode.code(),
                    opts.status_family.code(),
                    opts.min_ms as f32,
                    &date_bytes,
                    need_summary,
                );
                decode_pm2_partial(&wire)
            })
            .collect();

        let mut total_matched = 0u64;
        let mut total_unmatched = 0u64;
        let mut partials: Vec<AggPartial> = Vec::with_capacity(decoded.len());
        for (matched, unmatched, mut partial) in decoded {
            total_matched += matched;
            total_unmatched += unmatched;
            if !need_summary {
                partial.summary = None;
            }
            partials.push(partial);
        }

        let (api, built) = finish_api_from_partials(&partials, total_matched, total_unmatched);
        let cron = aggregate_cron(&self.cron_events, opts);

        let summary = if is_date_filtered {
            built.clone().unwrap_or_default()
        } else {
            self.cached_summary
                .clone()
                .or(built)
                .unwrap_or_default()
        };

        let mut starts = 0u64;
        let mut dones = 0u64;
        let mut fails = 0u64;
        if is_date_filtered {
            let date = opts.date_filter.as_deref().unwrap_or("");
            for e in &self.cron_events {
                if let Some(ts) = e.ts.as_deref() {
                    if !ts.starts_with(date) {
                        continue;
                    }
                }
                count_cron_event(e.event, &mut starts, &mut dones, &mut fails);
            }
        } else {
            for e in &self.cron_events {
                count_cron_event(e.event, &mut starts, &mut dones, &mut fails);
            }
        }
        let cron_summary = CronSummary {
            starts,
            dones,
            fails,
            jobs: cron.len() as u64,
            slowest_run: cron.iter().map(|c| c.max_ms).fold(0.0, f64::max),
        };

        let hourly_stats = if is_date_filtered {
            let date = opts.date_filter.as_deref().unwrap_or("");
            self.daily_stats
                .iter()
                .find(|d| d.date == date)
                .map(|d| d.hourly_stats.clone())
                .unwrap_or_else(|| self.hourly_stats.clone())
        } else {
            self.hourly_stats.clone()
        };

        let result = AggregatedResult {
            api,
            cron,
            summary,
            cron_summary,
            hourly_stats,
            methods: self.methods.clone(),
            unmatched_sample: self.unmatched_sample.clone(),
            unmatched_count: self.unmatched_count,
            dates: self.dates.clone(),
            daily_stats: self.daily_stats.clone(),
        };
        if !is_date_filtered {
            self.cached_summary = Some(result.summary.clone());
        }
        result
    }
}

fn count_cron_event(event: CronEventKind, starts: &mut u64, dones: &mut u64, fails: &mut u64) {
    match event {
        CronEventKind::Start => *starts += 1,
        CronEventKind::Done => *dones += 1,
        CronEventKind::Fail => *fails += 1,
    }
}

/// Parse loaded sources into a kernel, mirroring
/// `parseFilesSharded` / `parseBufferSharded` / `parseText`.
pub fn parse_sources(
    sources: &[LoadedSource],
    normalize_mode: NormalizeMode,
    control: &JobControl,
) -> Result<Pm2Kernel, ParseError> {
    parse_source(Source::from_loaded(sources)?, normalize_mode, control)
}

pub fn parse_paths(
    paths: &[PathBuf],
    normalize_mode: NormalizeMode,
    control: &JobControl,
) -> Result<Pm2Kernel, ParseError> {
    parse_source(Source::from_paths(paths)?, normalize_mode, control)
}

pub fn parse_text(
    text: &str,
    normalize_mode: NormalizeMode,
    control: &JobControl,
) -> Result<Pm2Kernel, ParseError> {
    let bytes = text.as_bytes().to_vec();
    parse_source(
        Source::from_loaded(&[LoadedSource::Memory {
            name: "paste".to_string(),
            bytes: Arc::new(bytes),
        }])?,
        normalize_mode,
        control,
    )
}

fn parse_source(
    source: Source,
    normalize_mode: NormalizeMode,
    control: &JobControl,
) -> Result<Pm2Kernel, ParseError> {
    let file_size = source.len();
    let ranges = shard_ranges(file_size, shard_count_for(file_size));
    let parsed: Result<Vec<ShardParsed>, ParseError> = ranges
        .par_iter()
        .enumerate()
        .map(|(i, range)| {
            parse_one_shard(&source, *range, i, file_size, normalize_mode, control)
        })
        .collect();
    let mut parsed = parsed?;
    parsed.sort_by_key(|p| p.shard_index);
    Ok(absorb_meta(parsed, file_size))
}

/// Raw shard wires for cross-checking against the JS coordinator.
pub struct ShardWireDump {
    pub shard_index: usize,
    pub hit_count: u64,
    pub unmatched_count: u64,
    pub methods_mask: u8,
    pub cron_wire: Vec<u8>,
    pub unmatched_wire: Vec<u8>,
    pub hourly_wire: Vec<u8>,
    pub dates_wire: Vec<u8>,
    pub daily_wire: Vec<u8>,
    pub partial_wire: Vec<u8>,
}

pub fn parse_paths_wires(
    paths: &[PathBuf],
    normalize_mode: NormalizeMode,
    control: &JobControl,
) -> Result<Vec<ShardWireDump>, ParseError> {
    let source = Source::from_paths(paths)?;
    let file_size = source.len();
    let ranges = shard_ranges(file_size, shard_count_for(file_size));
    let parsed: Result<Vec<ShardParsed>, ParseError> = ranges
        .par_iter()
        .enumerate()
        .map(|(i, range)| {
            parse_one_shard(&source, *range, i, file_size, normalize_mode, control)
        })
        .collect();
    let mut parsed = parsed?;
    parsed.sort_by_key(|p| p.shard_index);
    Ok(parsed
        .into_iter()
        .map(|p| ShardWireDump {
            shard_index: p.shard_index,
            hit_count: p.hit_count,
            unmatched_count: p.unmatched_count,
            methods_mask: p.methods_mask,
            cron_wire: p.cron_wire,
            unmatched_wire: p.unmatched_wire,
            hourly_wire: p.hourly_wire,
            dates_wire: p.dates_wire,
            daily_wire: p.daily_wire,
            partial_wire: p.default_partial.map(|(_, w)| w).unwrap_or_default(),
        })
        .collect())
}

fn absorb_meta(parsed: Vec<ShardParsed>, file_size: u64) -> Pm2Kernel {
    let mut hit_count = 0u64;
    let mut unmatched_count = 0u64;
    let mut methods_mask = 0u8;
    let mut unmatched_sample: Vec<String> = Vec::new();
    let mut cron_events: Vec<CronEventCompact> = Vec::new();
    let mut all_dates: Vec<String> = Vec::new();
    let mut hourly_partials: Vec<HourlyPartial> = Vec::new();
    let mut daily_partials: Vec<DailyPartial> = Vec::new();
    let mut shards: Vec<Shard> = Vec::with_capacity(parsed.len());

    for p in parsed {
        hit_count += p.hit_count;
        unmatched_count += p.unmatched_count;
        methods_mask |= p.methods_mask;
        hourly_partials.push(decode_hourly_wire(&p.hourly_wire));
        cron_events.extend(decode_cron_wire(&p.cron_wire));
        all_dates.extend(decode_dates_wire(&p.dates_wire));
        daily_partials.push(decode_daily_wire(&p.daily_wire));
        if unmatched_sample.len() < UNMATCHED_SAMPLE_CAP {
            for line in decode_unmatched_wire(&p.unmatched_wire) {
                if unmatched_sample.len() >= UNMATCHED_SAMPLE_CAP {
                    break;
                }
                unmatched_sample.push(line);
            }
        }
        shards.push(Shard {
            engine: p.engine,
            default_partial: p.default_partial,
        });
    }

    let hourly_stats = finalize_hourly_stats(&merge_hourly_partials(&hourly_partials));
    let daily_stats = finalize_daily_stats(&merge_daily_partials(&daily_partials));
    let mut dates = all_dates;
    dates.sort();
    dates.dedup();
    let methods = methods_from_mask(methods_mask);

    Pm2Kernel {
        shards,
        file_size,
        hit_count,
        unmatched_count,
        unmatched_sample,
        cron_events,
        methods,
        dates,
        hourly_stats,
        daily_stats,
        cached_summary: None,
    }
}

// ── Wire decoding ───────────────────────────────────────────────────────────

pub struct AggPartial {
    pub buckets: Vec<NormBucketWire>,
    pub summary: Option<SummaryWire>,
}

pub struct NormBucketWire {
    pub method: LogMethod,
    pub path: String,
    pub sketch: RelHistWire,
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub error_count: u64,
}

pub struct SummaryWire {
    pub sum: f64,
    pub max: f64,
    pub errors: u64,
    pub slow: u64,
    pub sketch: RelHistWire,
}

fn decode_pm2_partial(buf: &[u8]) -> (u64, u64, AggPartial) {
    let mut o = 0usize;
    assert_eq!(read_u32(buf, o), MAGIC_PM2P, "bad PM2P magic");
    o += 4;
    let version = read_u16(buf, o);
    o += 2;
    assert_eq!(version, 1, "unsupported PM2P version");
    o += 1; // mode
    let flags = buf[o];
    o += 1;
    let endpoint_count = read_u32(buf, o) as usize;
    o += 4;
    let matched = read_u32(buf, o) as u64;
    o += 4;
    let unmatched = read_u32(buf, o) as u64;
    o += 4;

    let mut summary = None;
    if flags & 1 != 0 {
        let sum = read_f64(buf, o);
        o += 8;
        let max = read_f32(buf, o) as f64;
        o += 4;
        let errors = read_u32(buf, o) as u64;
        o += 4;
        let slow = read_u32(buf, o) as u64;
        o += 4;
        o += 4; // sketch length
        let sketch = RelHistWire::decode(buf, &mut o);
        summary = Some(SummaryWire {
            sum,
            max,
            errors,
            slow,
            sketch,
        });
    }

    let mut buckets = Vec::with_capacity(endpoint_count);
    for _ in 0..endpoint_count {
        let method_code = buf[o] as usize;
        o += 4;
        let count = read_u32(buf, o) as u64;
        o += 4;
        let sum = read_f64(buf, o);
        o += 8;
        let min = read_f32(buf, o) as f64;
        o += 4;
        let max = read_f32(buf, o) as f64;
        o += 4;
        let error_count = read_u32(buf, o) as u64;
        o += 4;
        let path_len = read_u32(buf, o) as usize;
        o += 4;
        let path = String::from_utf8_lossy(&buf[o..o + path_len]).into_owned();
        o += path_len;
        o += 4; // sketch length
        let sketch = RelHistWire::decode(buf, &mut o);
        buckets.push(NormBucketWire {
            method: LogMethod::from_index(method_code),
            path,
            sketch,
            count,
            sum,
            min,
            max,
            error_count,
        });
    }

    (matched, unmatched, AggPartial { buckets, summary })
}

fn read_bytes(buf: &[u8], o: &mut usize) -> Vec<u8> {
    let len = read_u32(buf, *o) as usize;
    *o += 4;
    let out = buf[*o..*o + len].to_vec();
    *o += len;
    out
}

pub fn decode_cron_wire(buf: &[u8]) -> Vec<CronEventCompact> {
    let mut o = 0usize;
    let n = read_u32(buf, o) as usize;
    o += 4;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let event = match buf[o] {
            0 => CronEventKind::Start,
            1 => CronEventKind::Done,
            _ => CronEventKind::Fail,
        };
        o += 1;
        let name = String::from_utf8_lossy(&read_bytes(buf, &mut o)).into_owned();
        let has_ts = buf[o];
        o += 1;
        let ts = if has_ts != 0 {
            Some(String::from_utf8_lossy(&read_bytes(buf, &mut o)).into_owned())
        } else {
            None
        };
        let has_dur = buf[o];
        o += 1;
        let duration_ms = if has_dur != 0 {
            let d = read_f32(buf, o) as f64;
            o += 4;
            Some(d)
        } else {
            None
        };
        out.push(CronEventCompact {
            ts,
            event,
            name,
            duration_ms,
        });
    }
    out
}

pub fn decode_unmatched_wire(buf: &[u8]) -> Vec<String> {
    let mut o = 0usize;
    let n = read_u32(buf, o) as usize;
    o += 4;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(String::from_utf8_lossy(&read_bytes(buf, &mut o)).into_owned());
    }
    out
}

pub fn decode_dates_wire(buf: &[u8]) -> Vec<String> {
    if buf.len() < 4 {
        return Vec::new();
    }
    let mut o = 0usize;
    let n = read_u32(buf, o) as usize;
    o += 4;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(String::from_utf8_lossy(&read_bytes(buf, &mut o)).into_owned());
    }
    out
}

#[derive(Clone)]
pub struct HourlyBucketPartial {
    pub count: u64,
    pub error_count: u64,
    pub sum: f64,
    pub max: f64,
    pub sketch: RelHistWire,
}

pub struct HourlyPartial {
    pub buckets: Vec<HourlyBucketPartial>,
}

pub fn decode_hourly_wire(buf: &[u8]) -> HourlyPartial {
    let mut o = 0usize;
    assert_eq!(read_u32(buf, o), MAGIC_PM2H, "bad PM2H magic");
    o += 4;
    let version = read_u16(buf, o);
    assert_eq!(version, 1, "unsupported PM2H version");
    o += 2;
    let bucket_count = read_u16(buf, o) as usize;
    o += 2;
    let mut buckets = Vec::with_capacity(bucket_count);
    for _ in 0..bucket_count {
        let count = read_u32(buf, o) as u64;
        let error_count = read_u32(buf, o + 4) as u64;
        let sum = read_f64(buf, o + 8);
        let max = read_f32(buf, o + 16) as f64;
        o += 24;
        let sketch = RelHistWire::decode(buf, &mut o);
        buckets.push(HourlyBucketPartial {
            count,
            error_count,
            sum,
            max,
            sketch,
        });
    }
    HourlyPartial { buckets }
}

pub struct DayMerged {
    pub date: String,
    pub count: u64,
    pub error_count: u64,
    pub slow_count: u64,
    pub sum: f64,
    pub max: f64,
    pub sketch: RelHistWire,
    pub hourly: Vec<HourlyBucketPartial>,
}

pub struct DailyPartial {
    pub days: Vec<DayMerged>,
}

pub fn decode_daily_wire(buf: &[u8]) -> DailyPartial {
    if buf.len() < 8 {
        return DailyPartial { days: Vec::new() };
    }
    let mut o = 0usize;
    assert_eq!(read_u32(buf, o), MAGIC_PM2D, "bad PM2D magic");
    o += 4;
    let version = read_u16(buf, o);
    assert_eq!(version, 1, "unsupported PM2D version");
    o += 2;
    let day_count = read_u16(buf, o) as usize;
    o += 2;

    let mut days = Vec::with_capacity(day_count);
    for _ in 0..day_count {
        let date = String::from_utf8_lossy(&buf[o..o + 10]).into_owned();
        o += 12;
        let count = read_u32(buf, o) as u64;
        let error_count = read_u32(buf, o + 4) as u64;
        let slow_count = read_u32(buf, o + 8) as u64;
        let sum = read_f64(buf, o + 12);
        let max = read_f32(buf, o + 20) as f64;
        o += 28;
        let sketch = RelHistWire::decode(buf, &mut o);

        let mut hourly = Vec::with_capacity(24);
        for _ in 0..24 {
            let h_count = read_u32(buf, o) as u64;
            let h_error = read_u32(buf, o + 4) as u64;
            let h_sum = read_f64(buf, o + 8);
            let h_max = read_f32(buf, o + 16) as f64;
            o += 24;
            let h_sketch = RelHistWire::decode(buf, &mut o);
            hourly.push(HourlyBucketPartial {
                count: h_count,
                error_count: h_error,
                sum: h_sum,
                max: h_max,
                sketch: h_sketch,
            });
        }
        days.push(DayMerged {
            date,
            count,
            error_count,
            slow_count,
            sum,
            max,
            sketch,
            hourly,
        });
    }
    DailyPartial { days }
}

// ── Coordinator merge (finishApiFromPartials parity) ────────────────────────

struct Merged {
    method: LogMethod,
    path: String,
    sketch: RelHist,
    count: u64,
    sum: f64,
    min: f64,
    max: f64,
    error_count: u64,
}

fn finish_api_from_partials(
    partials: &[AggPartial],
    total_matched: u64,
    total_unmatched: u64,
) -> (Vec<AggregatedEndpoint>, Option<LogSummary>) {
    let mut rows: Vec<Merged> = Vec::new();
    let mut index: hashbrown::HashMap<String, usize> = hashbrown::HashMap::new();
    let mut sum_sketch: Option<RelHist> = None;
    let mut sum_sum = 0.0f64;
    let mut sum_max = 0.0f64;
    let mut sum_errors = 0u64;
    let mut sum_slow = 0u64;

    for p in partials {
        if let Some(s) = &p.summary {
            let sketch = sum_sketch.get_or_insert_with(RelHist::new);
            sketch.merge_wire(&s.sketch);
            sum_sum += s.sum;
            if s.max > sum_max {
                sum_max = s.max;
            }
            sum_errors += s.errors;
            sum_slow += s.slow;
        }
        for b in &p.buckets {
            let key = format!("{} {}", b.method.as_str(), b.path);
            if let Some(&idx) = index.get(&key) {
                let dest = &mut rows[idx];
                dest.sketch.merge_wire(&b.sketch);
                dest.count += b.count;
                dest.sum += b.sum;
                if b.min < dest.min {
                    dest.min = b.min;
                }
                if b.max > dest.max {
                    dest.max = b.max;
                }
                dest.error_count += b.error_count;
            } else {
                let mut sketch = RelHist::new();
                sketch.merge_wire(&b.sketch);
                index.insert(key.clone(), rows.len());
                rows.push(Merged {
                    method: b.method,
                    path: b.path.clone(),
                    sketch,
                    count: b.count,
                    sum: b.sum,
                    min: b.min,
                    max: b.max,
                    error_count: b.error_count,
                });
            }
        }
    }

    let api: Vec<AggregatedEndpoint> = rows
        .into_iter()
        .map(|v| {
            let c = v.count;
            let [p50, p90, p95, p99] = v.sketch.quantiles4();
            AggregatedEndpoint {
                key: format!("{} {}", v.method.as_str(), v.path),
                method: v.method,
                path: v.path,
                count: c,
                avg_ms: if c > 0 { v.sum / c as f64 } else { 0.0 },
                p50_ms: p50,
                p90_ms: p90,
                p95_ms: p95,
                p99_ms: p99,
                min_ms: if c > 0 { v.min } else { 0.0 },
                max_ms: if c > 0 { v.max } else { 0.0 },
                error_count: v.error_count,
            }
        })
        .collect();

    let summary = sum_sketch.map(|sketch| LogSummary {
        matched: total_matched,
        unmatched: total_unmatched,
        max: sum_max,
        avg: if total_matched > 0 {
            sum_sum / total_matched as f64
        } else {
            0.0
        },
        p95_ms: if total_matched == 0 {
            0.0
        } else {
            sketch.quantile(0.95)
        },
        errors: sum_errors,
        slow: sum_slow,
    });

    (api, summary)
}

// ── Cron (aggregateCron parity) ─────────────────────────────────────────────

fn should_skip_cron_event(ev: &CronEventCompact, date_filter: Option<&str>, q: &str) -> bool {
    if let (Some(date), Some(ts)) = (date_filter, ev.ts.as_deref()) {
        if !ts.starts_with(date) {
            return true;
        }
    }
    if !q.is_empty() && !ev.name.to_lowercase().contains(q) {
        return true;
    }
    false
}

fn parse_ts_ms(ts: &str) -> Option<i64> {
    let normalized = ts.replacen(' ', "T", 1);
    let ndt = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%dT%H:%M:%S").ok()?;
    Local
        .from_local_datetime(&ndt)
        .earliest()
        .map(|dt| dt.timestamp_millis())
}

fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((p / 100.0 * sorted.len() as f64).ceil() as isize - 1)
        .clamp(0, sorted.len() as isize - 1) as usize;
    sorted[idx]
}

pub fn aggregate_cron(events: &[CronEventCompact], options: &ParseOptions) -> Vec<CronAggregated> {
    let q = options.cron_query.trim().to_lowercase();
    let min_ms = options.cron_min_ms;
    let date_filter = options
        .date_filter
        .as_deref()
        .filter(|d| !d.is_empty() && *d != "all");

    struct Bucket {
        name: String,
        starts: u64,
        durations: Vec<f64>,
        fails: u64,
        last_run_ts: Option<String>,
        last_duration_ms: Option<f64>,
    }

    let mut order: Vec<String> = Vec::new();
    let mut buckets: hashbrown::HashMap<String, Bucket> = hashbrown::HashMap::new();
    let mut start_map: hashbrown::HashMap<String, Option<String>> = hashbrown::HashMap::new();

    for ev in events {
        if should_skip_cron_event(ev, date_filter, &q) {
            continue;
        }
        if !buckets.contains_key(&ev.name) {
            order.push(ev.name.clone());
            buckets.insert(
                ev.name.clone(),
                Bucket {
                    name: ev.name.clone(),
                    starts: 0,
                    durations: Vec::new(),
                    fails: 0,
                    last_run_ts: None,
                    last_duration_ms: None,
                },
            );
        }
        let bucket = buckets.get_mut(&ev.name).unwrap();
        match ev.event {
            CronEventKind::Start => {
                bucket.starts += 1;
                start_map.insert(ev.name.clone(), ev.ts.clone());
            }
            CronEventKind::Done | CronEventKind::Fail => {
                let dur = match ev.duration_ms {
                    Some(d) => {
                        if d >= min_ms {
                            Some(d)
                        } else {
                            None
                        }
                    }
                    None => {
                        let start_ts = start_map.get(&ev.name).cloned().flatten();
                        match (start_ts, ev.ts.as_deref()) {
                            (Some(s), Some(e)) => match (parse_ts_ms(&s), parse_ts_ms(e)) {
                                (Some(s_ms), Some(e_ms)) if e_ms >= s_ms => {
                                    let d = (e_ms - s_ms) as f64;
                                    if d >= min_ms {
                                        Some(d)
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            },
                            _ => None,
                        }
                    }
                };
                start_map.remove(&ev.name);
                if let Some(d) = dur {
                    bucket.durations.push(d);
                    bucket.last_duration_ms = Some(d);
                    if let Some(ts) = &ev.ts {
                        bucket.last_run_ts = Some(ts.clone());
                    }
                }
                if ev.event == CronEventKind::Fail {
                    bucket.fails += 1;
                }
            }
        }
    }

    let mut out = Vec::with_capacity(order.len());
    for name in order {
        let b = buckets.get(&name).unwrap();
        if options.cron_show_failed_only && b.fails == 0 {
            continue;
        }
        let mut sorted = b.durations.clone();
        sorted.sort_by(|a, c| a.partial_cmp(c).unwrap_or(std::cmp::Ordering::Equal));
        let runs = sorted.len() as u64;
        let sum: f64 = sorted.iter().sum();
        out.push(CronAggregated {
            name: b.name.clone(),
            runs,
            starts: b.starts,
            fails: b.fails,
            avg_ms: if runs > 0 { sum / runs as f64 } else { 0.0 },
            p50_ms: percentile_sorted(&sorted, 50.0),
            p90_ms: percentile_sorted(&sorted, 90.0),
            p95_ms: percentile_sorted(&sorted, 95.0),
            p99_ms: percentile_sorted(&sorted, 99.0),
            min_ms: sorted.first().copied().unwrap_or(0.0),
            max_ms: sorted.last().copied().unwrap_or(0.0),
            last_run_ts: b.last_run_ts.clone(),
            last_duration_ms: b.last_duration_ms,
        });
    }
    out
}

// ── Hourly / daily finalization ─────────────────────────────────────────────

pub fn merge_hourly_partials(partials: &[HourlyPartial]) -> HourlyPartial {
    let mut buckets: Vec<HourlyBucketPartial> = (0..24)
        .map(|_| HourlyBucketPartial {
            count: 0,
            error_count: 0,
            sum: 0.0,
            max: 0.0,
            sketch: RelHistWire::default(),
        })
        .collect();
    for partial in partials {
        for (hour, source) in partial.buckets.iter().enumerate().take(24) {
            let target = &mut buckets[hour];
            target.count += source.count;
            target.error_count += source.error_count;
            target.sum += source.sum;
            if source.max > target.max {
                target.max = source.max;
            }
            let mut merged = RelHist::new();
            merged.merge_wire(&target.sketch);
            merged.merge_wire(&source.sketch);
            target.sketch = merged.to_wire();
        }
    }
    HourlyPartial { buckets }
}

pub fn finalize_hourly_stats(partial: &HourlyPartial) -> Vec<HourlyBucket> {
    partial
        .buckets
        .iter()
        .enumerate()
        .map(|(hour, bucket)| {
            let mut sketch = RelHist::new();
            sketch.merge_wire(&bucket.sketch);
            HourlyBucket {
                hour: hour as u8,
                label: format!("{hour:02}:00"),
                count: bucket.count,
                error_count: bucket.error_count,
                avg_ms: if bucket.count > 0 {
                    (bucket.sum / bucket.count as f64).round()
                } else {
                    0.0
                },
                p95_ms: sketch.quantile(0.95).round(),
                p99_ms: sketch.quantile(0.99).round(),
                max_ms: bucket.max.round(),
            }
        })
        .collect()
}

pub fn merge_daily_partials(partials: &[DailyPartial]) -> Vec<DayMerged> {
    let mut map: hashbrown::HashMap<String, usize> = hashbrown::HashMap::new();
    let mut days: Vec<DayMerged> = Vec::new();

    for partial in partials {
        for d in &partial.days {
            if let Some(&idx) = map.get(&d.date) {
                let target = &mut days[idx];
                target.count += d.count;
                target.error_count += d.error_count;
                target.slow_count += d.slow_count;
                target.sum += d.sum;
                if d.max > target.max {
                    target.max = d.max;
                }
                let mut merged = RelHist::new();
                merged.merge_wire(&target.sketch);
                merged.merge_wire(&d.sketch);
                target.sketch = merged.to_wire();
                for h in 0..24 {
                    let Some(src) = d.hourly.get(h) else { continue };
                    let tgt = &mut target.hourly[h];
                    tgt.count += src.count;
                    tgt.error_count += src.error_count;
                    tgt.sum += src.sum;
                    if src.max > tgt.max {
                        tgt.max = src.max;
                    }
                    let mut hmerged = RelHist::new();
                    hmerged.merge_wire(&tgt.sketch);
                    hmerged.merge_wire(&src.sketch);
                    tgt.sketch = hmerged.to_wire();
                }
            } else {
                map.insert(d.date.clone(), days.len());
                days.push(DayMerged {
                    date: d.date.clone(),
                    count: d.count,
                    error_count: d.error_count,
                    slow_count: d.slow_count,
                    sum: d.sum,
                    max: d.max,
                    sketch: d.sketch.clone(),
                    hourly: d.hourly.clone(),
                });
            }
        }
    }
    days.sort_by(|a, b| a.date.cmp(&b.date));
    days
}

pub fn finalize_daily_stats(days: &[DayMerged]) -> Vec<DaySummary> {
    days.iter()
        .map(|d| {
            let mut sketch = RelHist::new();
            sketch.merge_wire(&d.sketch);
            let hourly_stats = d
                .hourly
                .iter()
                .enumerate()
                .map(|(hour, h)| {
                    let mut h_sketch = RelHist::new();
                    h_sketch.merge_wire(&h.sketch);
                    HourlyBucket {
                        hour: hour as u8,
                        label: format!("{hour:02}:00"),
                        count: h.count,
                        error_count: h.error_count,
                        avg_ms: if h.count > 0 {
                            (h.sum / h.count as f64).round()
                        } else {
                            0.0
                        },
                        p95_ms: h_sketch.quantile(0.95).round(),
                        p99_ms: h_sketch.quantile(0.99).round(),
                        max_ms: h.max.round(),
                    }
                })
                .collect();
            DaySummary {
                date: d.date.clone(),
                count: d.count,
                error_count: d.error_count,
                slow_count: d.slow_count,
                avg_ms: if d.count > 0 {
                    (d.sum / d.count as f64).round()
                } else {
                    0.0
                },
                p95_ms: sketch.quantile(0.95).round(),
                p99_ms: sketch.quantile(0.99).round(),
                max_ms: d.max.round(),
                hourly_stats,
            }
        })
        .collect()
}

pub fn methods_from_mask(mask: u8) -> Vec<String> {
    let mut out: Vec<String> = (0..METHODS.len())
        .filter(|i| mask & (1 << i) != 0)
        .map(|i| METHODS[i].as_str().to_string())
        .collect();
    out.sort();
    out
}
