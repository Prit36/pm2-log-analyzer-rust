//! Persistent per-shard columnar store + reaggregation.

use crate::kernels::pm2::normalize::{normalize_path, NormalizeMode};
use crate::kernels::pm2::parse::{parse_line_bytes, LineKind, Method};
use crate::kernels::pm2::relhist::RelHist;
use hashbrown::{HashMap, HashTable};
use memchr::memchr;
use rapidhash::v3::{rapidhash_v3_seeded, RapidSecrets};

const LINE_EXTEND: usize = 256 * 1024;
const INVALID_RELHIST_KEY: i16 = i16::MIN;
/// Reusable ingest window. Keeps Wasm peak memory bounded.
pub const INGEST_CAP: usize = 32 * 1024 * 1024;

/// Secondary seed for the 128-bit path fingerprint (h1 = rapidhash, h2 = rapidhash_seeded).
const HASH2_SEED: u64 = 0x9E37_79B9_7F4A_7C15;
static HASH2_SECRETS: RapidSecrets = RapidSecrets::seed_cpp(HASH2_SEED);

#[inline(always)]
fn hash_bytes(b: &[u8]) -> u64 {
    rapidhash::v3::rapidhash_v3(b)
}

#[inline(always)]
fn hash2_bytes(b: &[u8]) -> u64 {
    rapidhash_v3_seeded(b, &HASH2_SECRETS)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct PackedEntry {
    pub path_id: u32,
    pub duration: f32,
    pub meta: u32, // status:16, method:3, hour:5, date_id:8
}

impl PackedEntry {
    #[inline(always)]
    pub fn new(path_id: u32, duration: f32, status: u16, method: u8, hour: u8, date_id: u16) -> Self {
        let meta = (status as u32)
            | ((method as u32) << 16)
            | (((hour.min(31)) as u32) << 19)
            | (((date_id & 0xFF) as u32) << 24);
        Self {
            path_id,
            duration,
            meta,
        }
    }

    #[inline(always)]
    pub fn status(self) -> u16 {
        self.meta as u16
    }

    #[inline(always)]
    pub fn method(self) -> u8 {
        ((self.meta >> 16) & 0x7) as u8
    }

    #[inline(always)]
    pub fn hour(self) -> u8 {
        ((self.meta >> 19) & 0x1F) as u8
    }

    #[inline(always)]
    pub fn date_id(self) -> u16 {
        ((self.meta >> 24) & 0xFF) as u16
    }
}

#[derive(Clone)]
struct CronEv {
    event: u8,
    name: Vec<u8>,
    ts: Option<Vec<u8>>,
    duration_ms: Option<f32>,
}

pub struct EndpointAcc {
    pub method: u8,
    pub sketch: RelHist,
    pub count: u32,
    pub sum: f64,
    pub min: f32,
    pub max: f32,
    pub error_count: u32,
}

/// Filter-independent summary exposed to the native coordinator.
pub struct SummaryAcc {
    pub sum: f64,
    pub max: f32,
    pub errors: u32,
    pub slow: u32,
    pub sketch: RelHist,
}

/// Reaggregation result in structured form: the native driver merges these
/// directly instead of encoding/decoding the wire. `norm_id` indexes the shard's
/// normalization arena for `mode` (read back via [`Engine::norm_path`]).
pub struct EnginePartial {
    pub mode: u8,
    pub matched: u32,
    pub unmatched: u32,
    pub endpoints: Vec<(u32, EndpointAcc)>,
    pub summary: Option<SummaryAcc>,
}

#[derive(Clone)]
pub struct HourlyAcc {
    pub count: u32,
    pub error_count: u32,
    pub sum: f64,
    pub max: f32,
    pub sketch: RelHist,
}

pub struct DailyAcc {
    pub date: [u8; 10],
    pub count: u32,
    pub error_count: u32,
    pub slow_count: u32,
    pub sum: f64,
    pub max: f32,
    pub sketch: RelHist,
    pub hourly: [HourlyAcc; 24],
}

pub struct Engine {
    ingest: Vec<u8>,
    carry: Vec<u8>,
    /// Absolute file offset of carry[0], if carry non-empty.
    carry_abs: u64,

    path_bytes: Vec<u8>,
    path_off: Vec<u32>,
    path_len: Vec<u16>,
    /// 128-bit fingerprint per path: h1 is the table key (also kept so the hash
    /// table can rehash on growth), h2 verifies the hit without touching the cold
    /// path arena (~2^-128 false-match probability, matching the previous native
    /// implementation).
    path_hash1: Vec<u64>,
    path_hash2: Vec<u64>,
    path_table: HashTable<u32>,

    entries: Vec<PackedEntry>,
    /// Cached RelHist bucket keys; avoids recomputing duration.ln() on each filter pass.
    hist_keys: Vec<i16>,

    dates: Vec<[u8; 10]>,
    last_date: [u8; 10],
    last_date_id: u16,

    unmatched_count: u32,
    unmatched_sample: Vec<Vec<u8>>,
    cron_events: Vec<CronEv>,
    methods_mask: u8,

    norm_bytes: [Vec<u8>; 3],
    norm_off: [Vec<u32>; 3],
    norm_len: [Vec<u16>; 3],
    norm_table: [HashTable<u32>; 3],
    path_to_norm: [Vec<u32>; 3],
    mode_ready: [bool; 3],

    /// Filter-independent summary computed once after parse.
    summary_sum: f64,
    summary_max: f32,
    summary_errors: u32,
    summary_slow: u32,
    summary_sketch: RelHist,
    summary_ready: bool,

    hourly_accs: [HourlyAcc; 24],
    daily_accs: Vec<DailyAcc>,

    shard_start: u64,
    shard_end: u64,
    file_size: u64,
    skip_partial: bool,
    parsing: bool,
    last_path_id: Option<u32>,
    path_cache: [(u64, u32); 1024],
}

impl Engine {
    pub fn new() -> Self {
        Self {
            ingest: Vec::new(),
            carry: Vec::new(),
            carry_abs: 0,
            path_bytes: Vec::new(),
            path_off: Vec::new(),
            path_len: Vec::new(),
            path_hash1: Vec::new(),
            path_hash2: Vec::new(),
            path_table: HashTable::new(),
            entries: Vec::new(),
            hist_keys: Vec::new(),
            dates: Vec::new(),
            last_date: [0u8; 10],
            last_date_id: 0,
            unmatched_count: 0,
            unmatched_sample: Vec::new(),
            cron_events: Vec::new(),
            methods_mask: 0,
            norm_bytes: [Vec::new(), Vec::new(), Vec::new()],
            norm_off: [Vec::new(), Vec::new(), Vec::new()],
            norm_len: [Vec::new(), Vec::new(), Vec::new()],
            norm_table: [HashTable::new(), HashTable::new(), HashTable::new()],
            path_to_norm: [Vec::new(), Vec::new(), Vec::new()],
            mode_ready: [false; 3],
            summary_sum: 0.0,
            summary_max: 0.0,
            summary_errors: 0,
            summary_slow: 0,
            summary_sketch: RelHist::new(),
            summary_ready: false,
            hourly_accs: std::array::from_fn(|_| HourlyAcc {
                count: 0,
                error_count: 0,
                sum: 0.0,
                max: 0.0,
                sketch: RelHist::new(),
            }),
            daily_accs: Vec::new(),
            shard_start: 0,
            shard_end: 0,
            file_size: 0,
            skip_partial: false,
            parsing: false,
            last_path_id: None,
            path_cache: [(0, u32::MAX); 1024],
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn hit_count(&self) -> usize {
        self.entries.len()
    }

    pub fn unmatched_count(&self) -> u32 {
        self.unmatched_count
    }

    pub fn path_count(&self) -> usize {
        self.path_off.len()
    }

    pub fn methods_mask(&self) -> u8 {
        self.methods_mask
    }

    /// Grow ingest to `len` and return pointer for JS to `memory.set` into.
    pub fn ingest_ptr(&mut self, len: u32) -> u32 {
        let n = (len as usize).min(INGEST_CAP);
        if self.ingest.len() < n {
            self.ingest.resize(n, 0);
        }
        self.ingest.as_mut_ptr() as u32
    }

    pub fn begin_shard(&mut self, start: u64, end: u64, file_size: u64) {
        self.reset_columns();
        self.shard_start = start;
        self.shard_end = end;
        self.file_size = file_size;
        self.skip_partial = start > 0;
        self.parsing = true;
        self.carry.clear();
        self.carry_abs = 0;
        let span = end.saturating_sub(start);
        // The stress corpus stores about one hit per 276 input bytes. Keep a
        // small safety margin without reserving roughly twice the live entries.
        let estimate = span
            .saturating_div(256)
            .saturating_add(65536)
            .min(usize::MAX as u64) as usize;
        self.entries.reserve(estimate);
        self.path_off.reserve(8192);
        self.path_len.reserve(8192);
        self.path_bytes.reserve(262144);
        let path_off = &self.path_off;
        let path_len = &self.path_len;
        let path_bytes = &self.path_bytes;
        self.path_table.reserve(8192, |&id| {
            let off = path_off[id as usize] as usize;
            let len = path_len[id as usize] as usize;
            hash_bytes(&path_bytes[off..off + len])
        });
    }

    fn reset_columns(&mut self) {
        self.path_bytes.clear();
        self.path_off.clear();
        self.path_len.clear();
        self.path_hash1.clear();
        self.path_hash2.clear();
        self.path_table.clear();
        self.path_cache = [(0, u32::MAX); 1024];
        self.entries.clear();
        self.hist_keys.clear();
        self.dates.clear();
        self.last_date = [0u8; 10];
        self.last_date_id = 0;
        self.unmatched_count = 0;
        self.unmatched_sample.clear();
        self.cron_events.clear();
        self.methods_mask = 0;
        for m in 0..3 {
            self.norm_bytes[m].clear();
            self.norm_off[m].clear();
            self.norm_len[m].clear();
            self.norm_table[m].clear();
            self.path_to_norm[m].clear();
            self.mode_ready[m] = false;
        }
        self.summary_sum = 0.0;
        self.summary_max = 0.0;
        self.summary_errors = 0;
        self.summary_slow = 0;
        self.summary_sketch = RelHist::new();
        self.summary_ready = false;
        self.hourly_accs = std::array::from_fn(|_| HourlyAcc {
            count: 0,
            error_count: 0,
            sum: 0.0,
            max: 0.0,
            sketch: RelHist::new(),
        });
        self.daily_accs.clear();
        self.last_path_id = None;
    }

    /// Native convenience: copy `bytes` into the ingest window and feed them at `abs_off`.
    pub fn feed_bytes(&mut self, bytes: &[u8], abs_off: u64) -> u32 {
        let len = bytes.len();
        if self.ingest.len() < len {
            self.ingest.resize(len, 0);
        }
        self.ingest[..len].copy_from_slice(bytes);
        self.feed(len as u32, abs_off)
    }

    /// Zero-copy feed: process `bytes` (whose first byte sits at absolute `abs_off`)
    /// directly from the caller's buffer. Semantics match `feed`.
    pub fn feed_slice(&mut self, bytes: &[u8], abs_off: u64) -> u32 {
        self.feed_view(bytes, abs_off)
    }

    /// Feed `len` bytes already written at ingest[0..len] starting at absolute `abs_off`.
    pub fn feed(&mut self, len: u32, abs_off: u64) -> u32 {
        let len = (len as usize).min(self.ingest.len());
        let ingest = std::mem::take(&mut self.ingest);
        let fed = self.feed_view(&ingest[..len], abs_off);
        self.ingest = ingest;
        fed
    }

    fn feed_view(&mut self, view: &[u8], abs_off: u64) -> u32 {
        if self.carry.is_empty() {
            self.feed_view_only(view, abs_off)
        } else {
            self.feed_view_with_carry(view, abs_off)
        }
    }

    /// Common path: no carry — SIMD memchr newline scan over the view.
    fn feed_view_only(&mut self, view: &[u8], abs_off: u64) -> u32 {
        let before = self.entries.len();
        let len = view.len();
        let chunk_end = abs_off + len as u64;
        let at_file_end = chunk_end >= self.file_size;
        let extend_limit = self.shard_end + LINE_EXTEND as u64;

        let mut i = 0usize;
        if self.skip_partial {
            match memchr(b'\n', view) {
                Some(nl) => {
                    i = nl + 1;
                    self.skip_partial = false;
                }
                None => {
                    if !at_file_end {
                        self.carry.extend_from_slice(view);
                        self.carry_abs = abs_off;
                    }
                    return 0;
                }
            }
        }

        let mut line_start = i;
        for nl in memchr::memchr_iter(b'\n', &view[i..]) {
            let line_end = i + nl;
            let abs_line_start = abs_off + line_start as u64;
            if abs_line_start >= self.shard_end {
                line_start = line_end + 1;
                break;
            }
            self.accept_line(view, line_start, line_end);
            line_start = line_end + 1;
        }

        if line_start < len {
            let abs_line_start = abs_off + line_start as u64;
            if !at_file_end && abs_line_start < extend_limit {
                self.carry.clear();
                self.carry.extend_from_slice(&view[line_start..]);
                self.carry_abs = abs_line_start;
            } else if at_file_end && abs_line_start < self.shard_end {
                self.accept_line(view, line_start, len);
            }
        }

        (self.entries.len() - before) as u32
    }

    /// Rare path: leftover partial line from the previous chunk.
    fn feed_view_with_carry(&mut self, view: &[u8], abs_off: u64) -> u32 {
        let before = self.entries.len();
        let len = view.len();
        let mut carry = std::mem::take(&mut self.carry);
        let carry_abs = self.carry_abs;

        let chunk_end = abs_off + len as u64;
        let at_file_end = chunk_end >= self.file_size;
        let extend_limit = self.shard_end + LINE_EXTEND as u64;

        let total = carry.len() + len;
        let buf_abs = carry_abs;
        let byte_at = |idx: usize| -> u8 {
            if idx < carry.len() {
                carry[idx]
            } else {
                view[idx - carry.len()]
            }
        };

        let mut i = 0usize;
        if self.skip_partial {
            while i < total && byte_at(i) != b'\n' {
                i += 1;
            }
            if i >= total {
                if !at_file_end {
                    carry.extend_from_slice(view);
                    self.carry = carry;
                    self.carry_abs = carry_abs;
                }
                return 0;
            }
            i += 1;
            self.skip_partial = false;
        }

        while i < total {
            let line_start = i;
            let abs_line_start = buf_abs + line_start as u64;
            if abs_line_start >= self.shard_end {
                break;
            }
            while i < total && byte_at(i) != b'\n' {
                i += 1;
            }
            let line_end = i;
            let has_nl = i < total && byte_at(i) == b'\n';
            if has_nl {
                i += 1;
            }

            if !has_nl && !at_file_end {
                if abs_line_start < extend_limit {
                    self.carry.clear();
                    self.carry_abs = abs_line_start;
                    if line_start < carry.len() {
                        self.carry.extend_from_slice(&carry[line_start..]);
                        self.carry.extend_from_slice(view);
                    } else {
                        let s = line_start - carry.len();
                        self.carry.extend_from_slice(&view[s..]);
                    }
                }
                break;
            }

            if line_end <= carry.len() {
                self.accept_line(&carry, line_start, line_end);
            } else if line_start >= carry.len() {
                let s = line_start - carry.len();
                let e = line_end - carry.len();
                self.accept_line(view, s, e);
            } else {
                let mut line = Vec::with_capacity(line_end - line_start);
                line.extend_from_slice(&carry[line_start..]);
                line.extend_from_slice(&view[..line_end - carry.len()]);
                self.accept_line(&line, 0, line.len());
            }

            if !has_nl {
                break;
            }
        }

        (self.entries.len() - before) as u32
    }

    pub fn end_shard(&mut self) {
        if !self.carry.is_empty() {
            let abs_line_start = self.carry_abs;
            if abs_line_start < self.shard_end {
                let buf = std::mem::take(&mut self.carry);
                self.accept_line(&buf, 0, buf.len());
            }
            self.carry.clear();
        }
        self.parsing = false;
        self.ingest.clear();
        self.build_summary_and_meta();
    }

    /// One-time single-pass computation of summary, hourly stats, and daily stats.
    fn build_summary_and_meta(&mut self) {
        if self.summary_ready {
            return;
        }

        let mut sum_sum = 0.0f64;
        let mut sum_max = 0.0f32;
        let mut sum_errors = 0u32;
        let mut sum_slow = 0u32;
        let mut sum_sketch = RelHist::new();

        let mut hourly_buckets: [HourlyAcc; 24] = std::array::from_fn(|_| HourlyAcc {
            count: 0,
            error_count: 0,
            sum: 0.0,
            max: 0.0,
            sketch: RelHist::new(),
        });

        let mut daily_accs: Vec<DailyAcc> = self
            .dates
            .iter()
            .map(|&d| DailyAcc {
                date: d,
                count: 0,
                error_count: 0,
                slow_count: 0,
                sum: 0.0,
                max: 0.0,
                sketch: RelHist::new(),
                hourly: std::array::from_fn(|_| HourlyAcc {
                    count: 0,
                    error_count: 0,
                    sum: 0.0,
                    max: 0.0,
                    sketch: RelHist::new(),
                }),
            })
            .collect();

        use crate::kernels::pm2::relhist::relhist_key;

        self.hist_keys.reserve(self.entries.len());
        let hist_keys = &mut self.hist_keys;
        for entry in &self.entries {
            let d = entry.duration;
            let st = entry.status();
            let h = entry.hour() as usize;
            let is_err = st >= 400;
            let is_slow = d >= 3000.0;
            let compact_key = relhist_key(d)
                .map(|key| key.clamp((i16::MIN + 1) as i32, i16::MAX as i32) as i16)
                .unwrap_or(INVALID_RELHIST_KEY);
            hist_keys.push(compact_key);
            let k = (compact_key != INVALID_RELHIST_KEY).then_some(compact_key as i32);

            sum_sum += d as f64;
            if d > sum_max {
                sum_max = d;
            }
            if is_err {
                sum_errors += 1;
            }
            if is_slow {
                sum_slow += 1;
            }
            if let Some(key) = k {
                sum_sketch.accept_key(key);
            }

            if h < 24 {
                let hb = &mut hourly_buckets[h];
                hb.count += 1;
                hb.sum += d as f64;
                if d > hb.max {
                    hb.max = d;
                }
                if is_err {
                    hb.error_count += 1;
                }
                if let Some(key) = k {
                    hb.sketch.accept_key(key);
                }
            }

            let did = entry.date_id();
            if did != 0 {
                let didx = (did - 1) as usize;
                if let Some(da) = daily_accs.get_mut(didx) {
                    da.count += 1;
                    da.sum += d as f64;
                    if d > da.max {
                        da.max = d;
                    }
                    if is_err {
                        da.error_count += 1;
                    }
                    if is_slow {
                        da.slow_count += 1;
                    }
                    if let Some(key) = k {
                        da.sketch.accept_key(key);
                    }

                    if h < 24 {
                        let dh = &mut da.hourly[h];
                        dh.count += 1;
                        dh.sum += d as f64;
                        if d > dh.max {
                            dh.max = d;
                        }
                        if is_err {
                            dh.error_count += 1;
                        }
                        if let Some(key) = k {
                            dh.sketch.accept_key(key);
                        }
                    }
                }
            }
        }

        self.summary_sum = sum_sum;
        self.summary_max = sum_max;
        self.summary_errors = sum_errors;
        self.summary_slow = sum_slow;
        self.summary_sketch = sum_sketch;
        self.summary_ready = true;

        self.hourly_accs = hourly_buckets;
        self.daily_accs = daily_accs;
    }

    /// Filter-independent hour-of-day request statistics.
    pub fn hourly_accs(&self) -> &[HourlyAcc; 24] {
        &self.hourly_accs
    }

    /// Per-date summary accumulators (in first-seen order).
    pub fn daily_accs(&self) -> &[DailyAcc] {
        &self.daily_accs
    }

    /// Encode filter-independent hour-of-day request statistics.
    pub fn hourly_wire(&self) -> Vec<u8> {
        encode_hourly_vec(&self.hourly_accs)
    }

    /// Encode list of unique dates seen in logs.
    pub fn dates_wire(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.dates.len() as u32).to_le_bytes());
        for d in &self.dates {
            write_bytes(&mut out, d);
        }
        out
    }

    /// Encode daily summary stats and per-date hourly breakdown.
    pub fn daily_wire(&self) -> Vec<u8> {
        encode_daily_vec(&self.daily_accs)
    }

    /// Summary wire for coordinator cache (same fields as PM2P summary block).
    pub fn summary_wire(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.summary_sum.to_le_bytes());
        out.extend_from_slice(&self.summary_max.to_le_bytes());
        out.extend_from_slice(&self.summary_errors.to_le_bytes());
        out.extend_from_slice(&self.summary_slow.to_le_bytes());
        let wire = self.summary_sketch.to_wire();
        out.extend_from_slice(&(wire.len() as u32).to_le_bytes());
        out.extend_from_slice(&wire);
        out
    }

    fn intern_date(&mut self, d: [u8; 10]) -> u16 {
        if let Some(pos) = self.dates.iter().position(|&x| x == d) {
            let id = (pos + 1) as u16;
            self.last_date = d;
            self.last_date_id = id;
            id
        } else {
            self.dates.push(d);
            let id = self.dates.len() as u16;
            self.last_date = d;
            self.last_date_id = id;
            id
        }
    }

    fn accept_line(&mut self, buf: &[u8], line_start: usize, line_end: usize) {
        match parse_line_bytes(buf, line_start, line_end) {
            LineKind::Empty => {}
            LineKind::Cron {
                event,
                name,
                ts,
                duration_ms,
            } => {
                self.cron_events.push(CronEv {
                    event,
                    name,
                    ts,
                    duration_ms,
                });
            }
            LineKind::Http {
                method,
                path_start,
                path_end,
                status,
                duration_ms,
                hour,
                date,
            } => {
                let path = &buf[path_start..path_end];
                let pid = self.intern_path(path);
                let date_id = match date {
                    Some(d) => {
                        if d == self.last_date && self.last_date_id != 0 {
                            self.last_date_id
                        } else {
                            self.intern_date(d)
                        }
                    }
                    None => 0,
                };
                self.entries.push(PackedEntry::new(
                    pid,
                    duration_ms,
                    status,
                    method as u8,
                    hour.unwrap_or(255),
                    date_id,
                ));
                self.methods_mask |= 1u8 << (method as u8);
            }
            LineKind::Unmatched => {
                self.unmatched_count += 1;
                if self.unmatched_sample.len() < 40 {
                    let end = (line_start + 500).min(line_end);
                    self.unmatched_sample.push(buf[line_start..end].to_vec());
                }
            }
        }
    }

    /// Test / small-buffer helper: copies through the ingest window.
    pub fn parse_shard(
        &mut self,
        buf: &[u8],
        shard_start: usize,
        shard_end: usize,
        file_size: usize,
    ) -> usize {
        self.begin_shard(shard_start as u64, shard_end as u64, file_size as u64);
        let read_end = (shard_end + LINE_EXTEND).min(file_size);
        let mut off = shard_start;
        while off < read_end {
            let take = (read_end - off)
                .min(INGEST_CAP)
                .min(buf.len().saturating_sub(off - shard_start));
            if take == 0 {
                break;
            }
            let src_off = off - shard_start;
            let _ = self.ingest_ptr(take as u32);
            self.ingest[..take].copy_from_slice(&buf[src_off..src_off + take]);
            self.feed(take as u32, off as u64);
            off += take;
        }
        self.end_shard();
        self.entries.len()
    }

    fn intern_path(&mut self, path: &[u8]) -> u32 {
        if let Some(last_id) = self.last_path_id {
            let last_id_usize = last_id as usize;
            if last_id_usize < self.path_off.len() {
                let off = self.path_off[last_id_usize] as usize;
                let len = self.path_len[last_id_usize] as usize;
                if path.len() == len && &self.path_bytes[off..off + len] == path {
                    return last_id;
                }
            }
        }
        let h1 = hash_bytes(path);
        let h2 = hash2_bytes(path);
        let slot = (h1 as usize) & 1023;
        let (cached_hash, cached_id) = self.path_cache[slot];
        if cached_hash == h1 && cached_id != u32::MAX {
            let id_usize = cached_id as usize;
            if id_usize < self.path_hash2.len() && self.path_hash2[id_usize] == h2 {
                self.last_path_id = Some(cached_id);
                return cached_id;
            }
        }

        let path_hash1 = &self.path_hash1;
        let path_hash2 = &self.path_hash2;
        if let Some(&id) = self
            .path_table
            .find(h1, |&id| path_hash1[id as usize] == h1 && path_hash2[id as usize] == h2)
        {
            self.path_cache[slot] = (h1, id);
            self.last_path_id = Some(id);
            return id;
        }

        let next_id = self.path_off.len() as u32;
        let off = self.path_bytes.len() as u32;
        self.path_bytes.extend_from_slice(path);
        self.path_off.push(off);
        self.path_len.push(path.len() as u16);
        self.path_hash1.push(h1);
        self.path_hash2.push(h2);
        self.mode_ready = [false; 3];

        let path_hash1 = &self.path_hash1;
        self.path_table
            .insert_unique(h1, next_id, |&id| path_hash1[id as usize]);
        self.path_cache[slot] = (h1, next_id);
        self.last_path_id = Some(next_id);
        next_id
    }

    fn path_slice(&self, id: usize) -> &[u8] {
        let off = self.path_off[id] as usize;
        let len = self.path_len[id] as usize;
        &self.path_bytes[off..off + len]
    }

    pub fn path_bytes_of(&self, path_id: usize) -> Option<Vec<u8>> {
        if path_id >= self.path_off.len() {
            return None;
        }
        Some(self.path_slice(path_id).to_vec())
    }

    fn intern_norm_into(
        norm_bytes: &mut Vec<u8>,
        norm_off: &mut Vec<u32>,
        norm_len: &mut Vec<u16>,
        norm_table: &mut HashTable<u32>,
        path: &[u8],
    ) -> u32 {
        let hash = hash_bytes(path);
        if let Some(&id) = norm_table.find(hash, |&id| {
            let off = norm_off[id as usize] as usize;
            let len = norm_len[id as usize] as usize;
            &norm_bytes[off..off + len] == path
        }) {
            return id;
        }

        let next_id = norm_off.len() as u32;
        let off = norm_bytes.len() as u32;
        norm_bytes.extend_from_slice(path);
        norm_off.push(off);
        norm_len.push(path.len() as u16);

        norm_table.insert_unique(hash, next_id, |&id| {
            let off = norm_off[id as usize] as usize;
            let len = norm_len[id as usize] as usize;
            hash_bytes(&norm_bytes[off..off + len])
        });
        next_id
    }

    /// Borrowed normalized path bytes (native coordinator merge).
    pub fn norm_path(&self, mode: u8, norm_id: usize) -> Option<&[u8]> {
        let m = mode as usize;
        if m > 2 {
            return None;
        }
        if m == NormalizeMode::Exact as usize {
            if norm_id >= self.path_off.len() {
                return None;
            }
            let off = self.path_off[norm_id] as usize;
            let len = self.path_len[norm_id] as usize;
            return Some(&self.path_bytes[off..off + len]);
        }
        if norm_id >= self.norm_off[m].len() {
            return None;
        }
        let off = self.norm_off[m][norm_id] as usize;
        let len = self.norm_len[m][norm_id] as usize;
        Some(&self.norm_bytes[m][off..off + len])
    }

    pub fn norm_path_bytes(&self, mode: u8, norm_id: usize) -> Option<Vec<u8>> {
        let m = mode as usize;
        if m > 2 {
            return None;
        }
        if m == NormalizeMode::Exact as usize {
            return self.path_bytes_of(norm_id);
        }
        if norm_id >= self.norm_off[m].len() {
            return None;
        }
        let off = self.norm_off[m][norm_id] as usize;
        let len = self.norm_len[m][norm_id] as usize;
        Some(self.norm_bytes[m][off..off + len].to_vec())
    }

    pub fn ensure_mode(&mut self, mode: u8) {
        let mode_enum = NormalizeMode::from_u8(mode);
        let m = mode_enum as usize;
        if mode_enum == NormalizeMode::Exact {
            self.mode_ready[m] = true;
            return;
        }
        if self.mode_ready[m] && self.path_to_norm[m].len() == self.path_off.len() {
            return;
        }
        self.norm_bytes[m].clear();
        self.norm_off[m].clear();
        self.norm_len[m].clear();
        self.norm_table[m].clear();
        self.path_to_norm[m].resize(self.path_off.len(), 0);
        let norm_bytes = &mut self.norm_bytes[m];
        let norm_off = &mut self.norm_off[m];
        let norm_len = &mut self.norm_len[m];
        let norm_table = &mut self.norm_table[m];
        let path_bytes = &self.path_bytes;
        let path_off = &self.path_off;
        let path_len = &self.path_len;
        let path_to_norm = &mut self.path_to_norm[m];
        for pid in 0..path_off.len() {
            let off = path_off[pid] as usize;
            let len = path_len[pid] as usize;
            let raw = &path_bytes[off..off + len];
            let norm_path = normalize_path(raw, mode_enum);
            let nid = Self::intern_norm_into(
                norm_bytes,
                norm_off,
                norm_len,
                norm_table,
                norm_path.as_ref(),
            );
            path_to_norm[pid] = nid;
        }
        self.mode_ready[m] = true;
    }

    pub fn finalize_paths(&mut self) {
        for m in 0u8..3 {
            self.ensure_mode(m);
        }
    }

    pub fn reaggregate(
        &mut self,
        normalize_mode: u8,
        status_family: u8,
        min_ms: f32,
        date_filter: &[u8],
        need_summary: bool,
    ) -> Vec<u8> {
        let partial = self.reaggregate_partial(
            normalize_mode,
            status_family,
            min_ms,
            date_filter,
            need_summary,
        );
        self.encode_partial(&partial)
    }

    /// Encode a structured partial exactly like [`Self::reaggregate`] does.
    pub fn encode_partial(&self, partial: &EnginePartial) -> Vec<u8> {
        let mode = partial.mode as usize;
        let (norm_bytes, norm_off, norm_len) = if mode == NormalizeMode::Exact as usize {
            (&self.path_bytes, &self.path_off, &self.path_len)
        } else {
            (&self.norm_bytes[mode], &self.norm_off[mode], &self.norm_len[mode])
        };
        encode_partial_vec(
            partial.mode,
            &partial.endpoints,
            norm_bytes,
            norm_off,
            norm_len,
            partial.summary.as_ref(),
            partial.matched,
            partial.unmatched,
        )
    }

    pub fn reaggregate_partial(
        &mut self,
        normalize_mode: u8,
        status_family: u8,
        min_ms: f32,
        date_filter: &[u8],
        need_summary: bool,
    ) -> EnginePartial {
        self.ensure_mode(normalize_mode);
        let mode = NormalizeMode::from_u8(normalize_mode) as usize;
        let (status_min, status_max) = match status_family {
            2 => (200u16, 299u16),
            3 => (300u16, 399u16),
            4 => (400u16, 499u16),
            5 => (500u16, 599u16),
            _ => (0u16, 0u16),
        };
        let filter_status = status_min != 0;
        let target_date_id: u16 = if date_filter.is_empty() {
            0
        } else {
            self.dates
                .iter()
                .position(|d| d == date_filter)
                .map(|p| (p + 1) as u16)
                .unwrap_or(u16::MAX)
        };

        // Dense slots for low-cardinality modes: (norm_id << 3) | method.
        // Sparse Vec<Option<Box<...>>> keeps the slot table ~n_norm×8 bytes while
        // only active endpoints pay for a heap allocation. A flat Vec<EndpointAcc>
        // here would commit roughly 2 KiB × n_norm×8 per shard.
        // 6 methods (OPTIONS dropped at parse) → max 3 bits used, slots 6..7 unused.
        let use_dense = mode != 0;
        let n_norm = self.norm_off[mode].len();
        let dense_len = if use_dense {
            n_norm.saturating_mul(8).saturating_add(8)
        } else {
            0
        };

        let mut dense: Vec<Option<Box<EndpointAcc>>> = if use_dense {
            let mut v = Vec::with_capacity(dense_len);
            v.resize_with(dense_len, || None);
            v
        } else {
            Vec::new()
        };
        let mut by_key: HashMap<u64, EndpointAcc> = if use_dense {
            HashMap::new()
        } else {
            HashMap::with_capacity((self.path_off.len() / 4).max(64))
        };

        let mut custom_summary = RelHist::new();
        let mut sum_max = 0.0f32;
        let mut sum_sum = 0.0f64;
        let mut sum_errors = 0u32;
        let mut sum_slow = 0u32;
        let use_cached_summary = target_date_id == 0 && need_summary && self.summary_ready;
        if use_cached_summary {
            sum_sum = self.summary_sum;
            sum_max = self.summary_max;
            sum_errors = self.summary_errors;
            sum_slow = self.summary_slow;
        }

        let n = self.entries.len();
        let path_to_norm = &self.path_to_norm[mode];
        let hist_keys = &self.hist_keys;
        let mut matched_count = 0u32;

        for i in 0..n {
            let e = self.entries[i];
            if target_date_id != 0 && e.date_id() != target_date_id {
                continue;
            }
            matched_count += 1;

            let duration_ms = e.duration;
            let status = e.status();

            if !use_cached_summary && need_summary {
                sum_sum += duration_ms as f64;
                if duration_ms > sum_max {
                    sum_max = duration_ms;
                }
                if status >= 400 {
                    sum_errors += 1;
                }
                if duration_ms >= 3000.0 {
                    sum_slow += 1;
                }
                let hist_key = hist_keys[i];
                if hist_key != INVALID_RELHIST_KEY {
                    custom_summary.accept_key(hist_key as i32);
                }
            }

            if duration_ms < min_ms {
                continue;
            }
            if filter_status && (status < status_min || status > status_max) {
                continue;
            }

            let method_code = e.method();
            let path_id = e.path_id as usize;
            let norm_id = if mode == NormalizeMode::Exact as usize {
                e.path_id
            } else {
                path_to_norm[path_id]
            };
            let key = ((norm_id as u64) << 3) | (method_code as u64);

            if use_dense {
                let idx = key as usize;
                if idx >= dense.len() {
                    continue;
                }
                let slot = &mut dense[idx];
                let entry = slot.get_or_insert_with(|| {
                    Box::new(EndpointAcc {
                        method: method_code,
                        sketch: RelHist::new(),
                        count: 0,
                        sum: 0.0,
                        min: f32::INFINITY,
                        max: f32::NEG_INFINITY,
                        error_count: 0,
                    })
                });
                let hist_key = hist_keys[i];
                if hist_key != INVALID_RELHIST_KEY {
                    entry.sketch.accept_key(hist_key as i32);
                }
                entry.count += 1;
                entry.sum += duration_ms as f64;
                if duration_ms < entry.min {
                    entry.min = duration_ms;
                }
                if duration_ms > entry.max {
                    entry.max = duration_ms;
                }
                if status >= 400 {
                    entry.error_count += 1;
                }
            } else {
                let entry = by_key.entry(key).or_insert_with(|| EndpointAcc {
                    method: method_code,
                    sketch: RelHist::new(),
                    count: 0,
                    sum: 0.0,
                    min: f32::INFINITY,
                    max: f32::NEG_INFINITY,
                    error_count: 0,
                });
                let hist_key = hist_keys[i];
                if hist_key != INVALID_RELHIST_KEY {
                    entry.sketch.accept_key(hist_key as i32);
                }
                entry.count += 1;
                entry.sum += duration_ms as f64;
                if duration_ms < entry.min {
                    entry.min = duration_ms;
                }
                if duration_ms > entry.max {
                    entry.max = duration_ms;
                }
                if status >= 400 {
                    entry.error_count += 1;
                }
            }
        }

        // Collect active dense accumulators; paths are encoded from the normalized arena below.
        let mut endpoints: Vec<(u32, EndpointAcc)> = Vec::new();
        if use_dense {
            for (idx, slot) in dense.into_iter().enumerate() {
                if let Some(e) = slot {
                    let norm_id = (idx >> 3) as u32;
                    endpoints.push((norm_id, *e));
                }
            }
        } else {
            for (key, e) in by_key {
                let norm_id = (key >> 3) as u32;
                endpoints.push((norm_id, e));
            }
        }

        let summary = if !need_summary {
            None
        } else if use_cached_summary {
            Some(SummaryAcc {
                sum: sum_sum,
                max: sum_max,
                errors: sum_errors,
                slow: sum_slow,
                sketch: self.summary_sketch.clone(),
            })
        } else {
            Some(SummaryAcc {
                sum: sum_sum,
                max: sum_max,
                errors: sum_errors,
                slow: sum_slow,
                sketch: custom_summary,
            })
        };

        EnginePartial {
            mode: mode as u8,
            matched: matched_count,
            unmatched: if target_date_id == 0 {
                self.unmatched_count
            } else {
                0
            },
            endpoints,
            summary,
        }
    }

    pub fn cron_wire(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.cron_events.len() as u32).to_le_bytes());
        for ev in &self.cron_events {
            out.push(ev.event);
            write_bytes(&mut out, &ev.name);
            match &ev.ts {
                Some(ts) => {
                    out.push(1);
                    write_bytes(&mut out, ts);
                }
                None => out.push(0),
            }
            match ev.duration_ms {
                Some(d) => {
                    out.push(1);
                    out.extend_from_slice(&d.to_le_bytes());
                }
                None => out.push(0),
            }
        }
        out
    }

    pub fn unmatched_sample_wire(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.unmatched_sample.len() as u32).to_le_bytes());
        for s in &self.unmatched_sample {
            write_bytes(&mut out, s);
        }
        out
    }
}

fn encode_hourly_vec(buckets: &[HourlyAcc]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + buckets.len() * 32);
    out.extend_from_slice(&0x504D3248u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(buckets.len() as u16).to_le_bytes());
    for bucket in buckets {
        out.extend_from_slice(&bucket.count.to_le_bytes());
        out.extend_from_slice(&bucket.error_count.to_le_bytes());
        out.extend_from_slice(&bucket.sum.to_le_bytes());
        out.extend_from_slice(&bucket.max.to_le_bytes());
        let wire = bucket.sketch.to_wire();
        out.extend_from_slice(&(wire.len() as u32).to_le_bytes());
        out.extend_from_slice(&wire);
    }
    out
}

fn encode_daily_vec(accs: &[DailyAcc]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + accs.len() * (32 + 24 * 32));
    out.extend_from_slice(&0x504D3244u32.to_le_bytes()); // PM2D
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(accs.len() as u16).to_le_bytes());
    for acc in accs {
        out.extend_from_slice(&acc.date); // 10 bytes
        out.extend_from_slice(&[0u8; 2]); // pad to 12 bytes
        out.extend_from_slice(&acc.count.to_le_bytes());
        out.extend_from_slice(&acc.error_count.to_le_bytes());
        out.extend_from_slice(&acc.slow_count.to_le_bytes());
        out.extend_from_slice(&acc.sum.to_le_bytes());
        out.extend_from_slice(&acc.max.to_le_bytes());
        let sk_wire = acc.sketch.to_wire();
        out.extend_from_slice(&(sk_wire.len() as u32).to_le_bytes());
        out.extend_from_slice(&sk_wire);
        // 24 hourly buckets for this day
        for h in &acc.hourly {
            out.extend_from_slice(&h.count.to_le_bytes());
            out.extend_from_slice(&h.error_count.to_le_bytes());
            out.extend_from_slice(&h.sum.to_le_bytes());
            out.extend_from_slice(&h.max.to_le_bytes());
            let h_wire = h.sketch.to_wire();
            out.extend_from_slice(&(h_wire.len() as u32).to_le_bytes());
            out.extend_from_slice(&h_wire);
        }
    }
    out
}

fn write_bytes(out: &mut Vec<u8>, b: &[u8]) {
    out.extend_from_slice(&(b.len() as u32).to_le_bytes());
    out.extend_from_slice(b);
}

fn encode_partial_vec(
    mode: u8,
    endpoints: &[(u32, EndpointAcc)],
    norm_bytes: &[u8],
    norm_off: &[u32],
    norm_len: &[u16],
    summary: Option<&SummaryAcc>,
    matched: u32,
    unmatched: u32,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(64 + endpoints.len() * 64);
    out.extend_from_slice(&0x504D3250u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.push(mode);
    let flags = if summary.is_some() { 1u8 } else { 0u8 };
    out.push(flags);
    out.extend_from_slice(&(endpoints.len() as u32).to_le_bytes());
    out.extend_from_slice(&matched.to_le_bytes());
    out.extend_from_slice(&unmatched.to_le_bytes());

    if let Some(acc) = summary {
        out.extend_from_slice(&acc.sum.to_le_bytes());
        out.extend_from_slice(&acc.max.to_le_bytes());
        out.extend_from_slice(&acc.errors.to_le_bytes());
        out.extend_from_slice(&acc.slow.to_le_bytes());
        let wire = acc.sketch.to_wire();
        out.extend_from_slice(&(wire.len() as u32).to_le_bytes());
        out.extend_from_slice(&wire);
    }

    for (nid, e) in endpoints {
        out.push(e.method);
        out.extend_from_slice(&[0, 0, 0]);
        out.extend_from_slice(&e.count.to_le_bytes());
        out.extend_from_slice(&e.sum.to_le_bytes());
        let min = if e.count > 0 { e.min } else { 0.0 };
        let max = if e.count > 0 { e.max } else { 0.0 };
        out.extend_from_slice(&min.to_le_bytes());
        out.extend_from_slice(&max.to_le_bytes());
        out.extend_from_slice(&e.error_count.to_le_bytes());
        let off = norm_off[*nid as usize] as usize;
        let len = norm_len[*nid as usize] as usize;
        write_bytes(&mut out, &norm_bytes[off..off + len]);
        let wire = e.sketch.to_wire();
        out.extend_from_slice(&(wire.len() as u32).to_le_bytes());
        out.extend_from_slice(&wire);
    }
    out
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub fn method_name(code: u8) -> &'static str {
    Method::from_code(code).map(|m| m.as_str()).unwrap_or("GET")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunked_matches_oneshot() {
        let sample = b"2026-07-24T00:00:10: GET /api/health 200 12.5 ms - 42\n\
socket connected\n\
2026-07-24T00:00:11: POST /api/x 201 3.1 ms - -\n";
        let mut a = Engine::new();
        a.parse_shard(sample, 0, sample.len(), sample.len());

        let mut b = Engine::new();
        b.begin_shard(0, sample.len() as u64, sample.len() as u64);
        let mut off = 0usize;
        while off < sample.len() {
            let take = (sample.len() - off).min(17);
            let _ = b.ingest_ptr(take as u32);
            b.ingest[..take].copy_from_slice(&sample[off..off + take]);
            b.feed(take as u32, off as u64);
            off += take;
        }
        b.end_shard();
        assert_eq!(a.hit_count(), b.hit_count());
        assert_eq!(a.unmatched_count(), b.unmatched_count());
        assert_eq!(a.hit_count(), 2);
        assert_eq!(a.unmatched_count(), 1);
    }

    #[test]
    fn feed_slice_matches_chunked_feed_at_every_boundary() {
        // Long lines > chunk size so every line spans at least one boundary, plus a
        // leading partial line for the skip_partial path and a trailing partial line
        // at EOF without a newline.
        let mut sample = Vec::new();
        sample.extend_from_slice(b"half a line before the shard\n");
        for i in 0..64 {
            let path = format!("/api/very/long/endpoint/segment/{i}/with/trailing/query?id={i}\n");
            sample.extend_from_slice(
                format!("2026-07-24T00:00:{:02}: GET {path} 200 {i}.5 ms - 42\n", i % 60)
                    .as_bytes(),
            );
            sample.extend_from_slice(b"socket noise line that is long enough to span a boundary\n");
        }
        sample.extend_from_slice(b"tail line without newline");

        let render = |engine: &mut Engine| {
            let mut bytes = engine.reaggregate(2, 0, 0.0, b"", true);
            bytes.push(engine.hit_count() as u8);
            bytes.push(engine.unmatched_count() as u8);
            bytes
        };

        let shard_start = 32u64; // mid-line
        let shard_end = sample.len() as u64 - 10; // mid-line
        let reference = {
            let mut engine = Engine::new();
            engine.begin_shard(shard_start, shard_end, sample.len() as u64);
            engine.feed_slice(&sample[shard_start as usize - 1..shard_end as usize], shard_start - 1);
            engine.end_shard();
            render(&mut engine)
        };

        for chunk in [1usize, 3, 16, 64, 257] {
            let mut engine = Engine::new();
            engine.begin_shard(shard_start, shard_end, sample.len() as u64);
            let mut off = shard_start - 1;
            while off < shard_end {
                let take = ((shard_end - off) as usize).min(chunk);
                engine.feed_slice(&sample[off as usize..off as usize + take], off);
                off += take as u64;
            }
            engine.end_shard();
            assert_eq!(render(&mut engine), reference, "feed_slice chunk {chunk}");

            let mut engine = Engine::new();
            engine.begin_shard(shard_start, shard_end, sample.len() as u64);
            let mut off = shard_start - 1;
            while off < shard_end {
                let take = ((shard_end - off) as usize).min(chunk);
                engine.feed_bytes(&sample[off as usize..off as usize + take], off);
                off += take as u64;
            }
            engine.end_shard();
            assert_eq!(render(&mut engine), reference, "feed_bytes chunk {chunk}");
        }
    }

    #[test]
    fn hourly_wire_uses_timestamp_hours() {
        let sample = b"2026-07-24T03:00:10: GET /api/a 200 12.5 ms - 42\n\
2026-07-24T15:00:11: POST /api/b 500 40 ms - 1\n\
40ms GET /api/c\n";
        let mut engine = Engine::new();
        engine.parse_shard(sample, 0, sample.len(), sample.len());

        let wire = engine.hourly_wire();
        assert_eq!(&wire[0..4], &0x504D3248u32.to_le_bytes());
        assert_eq!(u16::from_le_bytes(wire[4..6].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(wire[6..8].try_into().unwrap()), 24);

        let mut offset = 8usize;
        let mut records = [(0u32, 0u32, 0.0f64, 0.0f32); 24];
        for record in &mut records {
            record.0 = u32::from_le_bytes(wire[offset..offset + 4].try_into().unwrap());
            record.1 = u32::from_le_bytes(wire[offset + 4..offset + 8].try_into().unwrap());
            record.2 = f64::from_le_bytes(wire[offset + 8..offset + 16].try_into().unwrap());
            record.3 = f32::from_le_bytes(wire[offset + 16..offset + 20].try_into().unwrap());
            let sketch_len = u32::from_le_bytes(wire[offset + 20..offset + 24].try_into().unwrap()) as usize;
            offset += 24 + sketch_len;
        }

        assert_eq!(records[3].0, 1);
        assert_eq!(records[3].1, 0);
        assert!((records[3].2 - 12.5).abs() < 0.01);
        assert!((records[3].3 - 12.5).abs() < 0.01);
        assert_eq!(records[15].0, 1);
        assert_eq!(records[15].1, 1);
        assert!((records[15].2 - 40.0).abs() < 0.01);
        assert!((records[15].3 - 40.0).abs() < 0.01);
        assert_eq!(records[0].0, 0);

        assert_eq!(offset, wire.len());
    }

    #[test]
    fn multi_day_dates_and_reaggregate() {
        let sample = b"2026-08-14T10:00:00: GET /api/users 200 50 ms - 100\n\
2026-08-14T11:00:00: POST /api/orders 201 120 ms - 200\n\
2026-08-15T09:00:00: GET /api/users 200 40 ms - 100\n\
2026-08-15T10:00:00: GET /api/health 200 5 ms - 20\n";
        let mut engine = Engine::new();
        engine.parse_shard(sample, 0, sample.len(), sample.len());

        assert_eq!(engine.hit_count(), 4);
        assert_eq!(engine.dates.len(), 2);
        assert_eq!(engine.dates[0], *b"2026-08-14");
        assert_eq!(engine.dates[1], *b"2026-08-15");

        // reaggregate all days
        let all_wire = engine.reaggregate(0, 0, 0.0, b"", true);
        assert!(!all_wire.is_empty());

        // reaggregate day 1 only
        let day1_wire = engine.reaggregate(0, 0, 0.0, b"2026-08-14", true);
        assert!(!day1_wire.is_empty());

        // daily wire
        let d_wire = engine.daily_wire();
        assert!(!d_wire.is_empty());
        assert_eq!(&d_wire[0..4], &0x504D3244u32.to_le_bytes());
    }

    #[test]
    fn mid_line_chunk_boundary() {
        let sample = b"2026-07-24T00:00:10: GET /api/health 200 12.5 ms - 42\n";
        let split = 20; // inside timestamp
        let mut e = Engine::new();
        e.begin_shard(0, sample.len() as u64, sample.len() as u64);
        let _ = e.ingest_ptr(split as u32);
        e.ingest[..split].copy_from_slice(&sample[..split]);
        e.feed(split as u32, 0);
        let rest = sample.len() - split;
        let _ = e.ingest_ptr(rest as u32);
        e.ingest[..rest].copy_from_slice(&sample[split..]);
        e.feed(rest as u32, split as u64);
        e.end_shard();
        assert_eq!(e.hit_count(), 1);
        assert!(e.summary_ready);
        assert!(e.summary_sum > 0.0);
    }

    #[test]
    fn boundary_after_newline_keeps_first_line() {
        let sample = b"first line\n2026-07-24T00:00:10: GET /api/health 200 12.5 ms - 42\n";
        let start = 11u64;
        let mut e = Engine::new();
        e.begin_shard(start, sample.len() as u64, sample.len() as u64);
        let suffix = &sample[start as usize - 1..];
        let _ = e.ingest_ptr(suffix.len() as u32);
        e.ingest[..suffix.len()].copy_from_slice(suffix);
        e.feed(suffix.len() as u32, start - 1);
        e.end_shard();
        assert_eq!(e.hit_count(), 1);
        assert_eq!(e.unmatched_count(), 0);
    }

    #[test]
    fn offsets_above_u32_remain_exact() {
        let sample = b"partial shard prefix\n2026-07-24T00:00:10: GET /api/health 200 12.5 ms - 42\n";
        let start = 4_500_000_000u64;
        let end = start + sample.len() as u64;
        let mut e = Engine::new();
        e.begin_shard(start, end, end + 1);
        let _ = e.ingest_ptr(sample.len() as u32);
        e.ingest[..sample.len()].copy_from_slice(sample);
        e.feed(sample.len() as u32, start);
        e.end_shard();
        assert_eq!(e.hit_count(), 1);
        assert_eq!(e.unmatched_count(), 0);
    }
}
