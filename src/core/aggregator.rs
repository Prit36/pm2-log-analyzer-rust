use super::log_parser::{parse_line_bytes, skip_ansi, LineKind};
use super::models::{
    CronStats, EndpointStats, FilterOptions, LogAnalysisSummary, Method, PathNormMode,
    StatusFamily,
};
use super::path_norm::normalize_path_bytes;
use super::relhist::RelHist;
use memchr::memchr;
use rapidhash::RapidBuildHasher;
use rayon::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

type FastHashMap<K, V> = HashMap<K, V, RapidBuildHasher>;


#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct PackedEntry {
    pub path_id: u32,
    pub duration: f32,
    pub status: u16,
    pub method: u8,
    pub _pad: u8,
}

#[derive(Clone, Debug)]
pub struct CronEvent {
    pub name: String,
    pub is_success: bool,
    pub is_fail: bool,
    pub duration_ms: Option<f32>,
}

#[derive(Default)]
pub struct Engine {
    pub total_lines: u64,
    pub entries: Vec<PackedEntry>,
    pub path_bytes: Vec<u8>,
    pub path_off: Vec<u32>,
    pub path_len: Vec<u16>,
    pub path_index: FastHashMap<u64, u32>,
    pub unmatched_count: u64,
    pub unmatched_samples: Vec<String>,
    pub cron_events: Vec<CronEvent>,
    pub norm_maps: [Vec<u32>; 3],
    pub norm_unique_paths: [Vec<Vec<u8>>; 3],
    pub default_summary: std::sync::OnceLock<LogAnalysisSummary>,
}


impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern_path(&mut self, path: &[u8]) -> u32 {
        let hash = rapidhash::rapidhash(path);
        if let Some(&id) = self.path_index.get(&hash) {
            if self.path_slice(id) == path {
                return id;
            }
        }
        let id = self.path_off.len() as u32;
        let off = self.path_bytes.len() as u32;
        self.path_bytes.extend_from_slice(path);
        self.path_off.push(off);
        self.path_len.push(path.len() as u16);
        self.path_index.insert(hash, id);
        id
    }


    pub fn path_slice(&self, id: u32) -> &[u8] {
        let off = self.path_off[id as usize] as usize;
        let len = self.path_len[id as usize] as usize;
        &self.path_bytes[off..off + len]
    }

    pub fn merge(&mut self, mut other: Engine) {
        self.total_lines += other.total_lines;
        self.unmatched_count += other.unmatched_count;
        self.cron_events.append(&mut other.cron_events);

        if self.unmatched_samples.len() < 50 {
            let take = 50 - self.unmatched_samples.len();
            self.unmatched_samples
                .extend(other.unmatched_samples.clone().into_iter().take(take));
        }

        if other.entries.is_empty() {
            return;
        }

        let mut remap = Vec::with_capacity(other.path_off.len());
        for id in 0..other.path_off.len() as u32 {
            let slice = other.path_slice(id);
            remap.push(self.intern_path(slice));
        }

        self.entries.reserve(other.entries.len());
        for mut entry in other.entries {
            entry.path_id = remap[entry.path_id as usize];
            self.entries.push(entry);
        }
    }

    pub fn ensure_mode(&self, mode_idx: usize) -> (Vec<u32>, Vec<Vec<u8>>) {
        if !self.norm_maps[mode_idx].is_empty() {
            return (self.norm_maps[mode_idx].clone(), self.norm_unique_paths[mode_idx].clone());
        }
        let num_paths = self.path_off.len();

        if mode_idx == 0 {
            let raw_to_norm: Vec<u32> = (0..num_paths as u32).collect();
            let unique_norm_paths: Vec<Vec<u8>> = (0..num_paths as u32)
                .into_par_iter()
                .map(|id| self.path_slice(id).to_vec())
                .collect();
            return (raw_to_norm, unique_norm_paths);
        }

        let mode = match mode_idx {
            0 => PathNormMode::Raw,
            1 => PathNormMode::StripQuery,
            _ => PathNormMode::CollapseIds,
        };


        let normalized_paths: Vec<Vec<u8>> = (0..num_paths as u32)
            .into_par_iter()
            .map(|id| {
                let raw = self.path_slice(id);
                normalize_path_bytes(raw, mode).into_owned()
            })
            .collect();

        let mut norm_map: FastHashMap<Vec<u8>, u32> = FastHashMap::default();
        let mut unique_norm_paths: Vec<Vec<u8>> = Vec::new();
        let mut raw_to_norm: Vec<u32> = Vec::with_capacity(num_paths);

        for norm in normalized_paths {
            let norm_id = match norm_map.get(&norm) {
                Some(&nid) => nid,
                None => {
                    let nid = unique_norm_paths.len() as u32;
                    norm_map.insert(norm.clone(), nid);
                    unique_norm_paths.push(norm);
                    nid
                }
            };
            raw_to_norm.push(norm_id);
        }
        (raw_to_norm, unique_norm_paths)
    }


    pub fn finalize_paths(&mut self) {
        let (raw_to_norm, unique_norm_paths) = self.ensure_mode(2);
        self.norm_maps[2] = raw_to_norm;
        self.norm_unique_paths[2] = unique_norm_paths;
    }



    pub fn aggregate(
        &self,
        file_size: u64,
        elapsed_ms: u64,
        filters: &FilterOptions,
    ) -> LogAnalysisSummary {
        let is_default = filters.search_query.is_empty()
            && filters.method.is_none()
            && filters.status_family == StatusFamily::All
            && filters.min_duration_ms == 0.0
            && filters.max_duration_ms.is_none()
            && filters.path_norm_mode == PathNormMode::CollapseIds;

        if is_default {
            if let Some(cached) = self.default_summary.get() {
                let mut res = cached.clone();
                res.total_file_size_bytes = file_size;
                res.parse_duration_ms = elapsed_ms;
                return res;
            }
        }

        let res = self.compute_aggregate(file_size, elapsed_ms, filters);
        if is_default {
            let _ = self.default_summary.set(res.clone());
        }
        res
    }

    pub fn compute_aggregate(
        &self,
        file_size: u64,
        elapsed_ms: u64,
        filters: &FilterOptions,
    ) -> LogAnalysisSummary {

        let mode_idx = match filters.path_norm_mode {
            PathNormMode::Raw => 0,
            PathNormMode::StripQuery => 1,
            PathNormMode::CollapseIds => 2,
        };

        let (computed_raw_to_norm, computed_unique_paths);
        let (raw_to_norm, unique_norm_paths) = if !self.norm_maps[mode_idx].is_empty() {
            (&self.norm_maps[mode_idx], &self.norm_unique_paths[mode_idx])
        } else {
            let (r, u) = self.ensure_mode(mode_idx);
            computed_raw_to_norm = r;
            computed_unique_paths = u;
            (&computed_raw_to_norm, &computed_unique_paths)
        };

        let search_lower = if filters.search_query.is_empty() {
            None
        } else {
            Some(filters.search_query.to_lowercase())
        };

        let num_norm_paths = unique_norm_paths.len();

        // 2. Pre-evaluate Search Query against unique normalized paths
        let mut norm_allowed = vec![true; num_norm_paths];
        if let Some(ref q) = search_lower {
            for (nid, p) in unique_norm_paths.iter().enumerate() {
                let s = String::from_utf8_lossy(p).to_lowercase();
                if !s.contains(q) {
                    norm_allowed[nid] = false;
                }
            }
        }

        // 3. Parallel Rayon Chunk Aggregation over PackedEntry array
        let chunk_size = (self.entries.len() / rayon::current_num_threads().max(1)).max(50_000);
        let chunk_results: Vec<ChunkResult> = self
            .entries
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut local_slots: FastHashMap<u32, EndpointAcc> = FastHashMap::default();
                let mut local_sketch = RelHist::new();
                let mut local_http_count = 0u64;
                let mut local_error_count = 0u64;

                for entry in chunk {
                    let method_u8 = entry.method;
                    if let Some(ref m) = filters.method {
                        if *m as u8 != method_u8 {
                            continue;
                        }
                    }

                    match filters.status_family {
                        StatusFamily::All => {}
                        StatusFamily::Success => {
                            if !(200..300).contains(&entry.status) {
                                continue;
                            }
                        }
                        StatusFamily::Redirect => {
                            if !(300..400).contains(&entry.status) {
                                continue;
                            }
                        }
                        StatusFamily::ClientError => {
                            if !(400..500).contains(&entry.status) {
                                continue;
                            }
                        }
                        StatusFamily::ServerError => {
                            if entry.status < 500 {
                                continue;
                            }
                        }
                        StatusFamily::ErrorOnly => {
                            if entry.status < 400 {
                                continue;
                            }
                        }
                    }

                    if entry.duration < filters.min_duration_ms {
                        continue;
                    }

                    if let Some(max_d) = filters.max_duration_ms {
                        if entry.duration > max_d {
                            continue;
                        }
                    }

                    let raw_id = entry.path_id as usize;
                    let norm_id = raw_to_norm[raw_id];
                    if !norm_allowed[norm_id as usize] {
                        continue;
                    }

                    local_http_count += 1;
                    if entry.status >= 400 {
                        local_error_count += 1;
                    }
                    local_sketch.accept(entry.duration);

                    let slot_idx = (norm_id * 8) + (method_u8 as u32);
                    let slot = local_slots.entry(slot_idx).or_insert_with(|| EndpointAcc {
                        total_calls: 0,
                        error_calls: 0,
                        total_duration_ms: 0.0,
                        min_duration_ms: f32::MAX,
                        max_duration_ms: f32::MIN,
                        sketch: RelHist::new(),
                        status_buckets: StatusBuckets::default(),
                    });

                    slot.total_calls += 1;
                    if entry.status >= 400 {
                        slot.error_calls += 1;
                    }
                    slot.total_duration_ms += entry.duration as f64;
                    if entry.duration < slot.min_duration_ms {
                        slot.min_duration_ms = entry.duration;
                    }
                    if entry.duration > slot.max_duration_ms {
                        slot.max_duration_ms = entry.duration;
                    }
                    slot.sketch.accept(entry.duration);
                    slot.status_buckets.inc_status(entry.status);
                }

                ChunkResult {
                    slots: local_slots,
                    overall_sketch: local_sketch,
                    http_count: local_http_count,
                    error_count: local_error_count,
                }
            })
            .collect();

        // 4. Reduce chunk results
        let mut global_slots: FastHashMap<u32, EndpointAcc> = FastHashMap::default();
        let mut overall_sketch = RelHist::new();
        let mut filtered_http_count = 0u64;
        let mut filtered_error_count = 0u64;

        for res in chunk_results {
            overall_sketch.merge(&res.overall_sketch);
            filtered_http_count += res.http_count;
            filtered_error_count += res.error_count;

            for (idx, chunk_acc) in res.slots {
                match global_slots.get_mut(&idx) {
                    Some(global_acc) => {
                        global_acc.total_calls += chunk_acc.total_calls;
                        global_acc.error_calls += chunk_acc.error_calls;
                        global_acc.total_duration_ms += chunk_acc.total_duration_ms;
                        if chunk_acc.min_duration_ms < global_acc.min_duration_ms {
                            global_acc.min_duration_ms = chunk_acc.min_duration_ms;
                        }
                        if chunk_acc.max_duration_ms > global_acc.max_duration_ms {
                            global_acc.max_duration_ms = chunk_acc.max_duration_ms;
                        }
                        global_acc.sketch.merge(&chunk_acc.sketch);
                        global_acc.status_buckets.merge(&chunk_acc.status_buckets);
                    }
                    None => {
                        global_slots.insert(idx, chunk_acc);
                    }
                }
            }
        }

        // 5. Aggregate Cron Events
        let mut cron_map: HashMap<String, CronAcc> = HashMap::new();
        for cron in &self.cron_events {
            let entry = cron_map.entry(cron.name.clone()).or_insert_with(|| CronAcc {
                total_runs: 0,
                total_success: 0,
                total_failures: 0,
                total_duration_ms: 0.0,
                min_duration_ms: f32::MAX,
                max_duration_ms: f32::MIN,
                last_status: "unknown".to_string(),
            });

            entry.total_runs += 1;
            if cron.is_fail {
                entry.total_failures += 1;
                entry.last_status = "failed".to_string();
            } else if cron.is_success {
                entry.total_success += 1;
                entry.last_status = "success".to_string();
            }

            if let Some(dur) = cron.duration_ms {
                entry.total_duration_ms += dur as f64;
                if dur < entry.min_duration_ms {
                    entry.min_duration_ms = dur;
                }
                if dur > entry.max_duration_ms {
                    entry.max_duration_ms = dur;
                }
            }
        }

        let overall_p50 = overall_sketch.quantile(0.50);
        let overall_p90 = overall_sketch.quantile(0.90);
        let overall_p95 = overall_sketch.quantile(0.95);
        let overall_p99 = overall_sketch.quantile(0.99);

        // 6. Build EndpointStats list
        let mut endpoints: Vec<EndpointStats> = Vec::with_capacity(global_slots.len());
        for (idx, acc) in global_slots {
            let norm_id = (idx / 8) as usize;
            let method_u8 = (idx % 8) as u8;
            let method = Method::from_u8(method_u8).unwrap_or(Method::Get);
            let path_bytes = &unique_norm_paths[norm_id];
            let path_str = String::from_utf8_lossy(path_bytes).into_owned();

            endpoints.push(EndpointStats {
                path: path_str,
                method,
                total_calls: acc.total_calls,
                error_calls: acc.error_calls,
                total_duration_ms: acc.total_duration_ms,
                min_duration_ms: if acc.min_duration_ms == f32::MAX {
                    0.0
                } else {
                    acc.min_duration_ms
                },
                max_duration_ms: if acc.max_duration_ms == f32::MIN {
                    0.0
                } else {
                    acc.max_duration_ms
                },
                p50_ms: acc.sketch.quantile(0.50),
                p90_ms: acc.sketch.quantile(0.90),
                p95_ms: acc.sketch.quantile(0.95),
                p99_ms: acc.sketch.quantile(0.99),
                status_counts: acc.status_buckets.to_map(),
            });
        }


        let cron_jobs: Vec<CronStats> = cron_map
            .into_iter()
            .map(|(name, acc)| CronStats {
                name,
                total_runs: acc.total_runs,
                total_success: acc.total_success,
                total_failures: acc.total_failures,
                total_duration_ms: acc.total_duration_ms,
                avg_duration_ms: if acc.total_runs > 0 {
                    acc.total_duration_ms / acc.total_runs as f64
                } else {
                    0.0
                },
                min_duration_ms: if acc.min_duration_ms == f32::MAX {
                    0.0
                } else {
                    acc.min_duration_ms
                },
                max_duration_ms: if acc.max_duration_ms == f32::MIN {
                    0.0
                } else {
                    acc.max_duration_ms
                },
                last_status: acc.last_status,
            })
            .collect();

        let error_rate = if filtered_http_count > 0 {
            (filtered_error_count as f64 / filtered_http_count as f64) * 100.0
        } else {
            0.0
        };

        LogAnalysisSummary {
            total_file_size_bytes: file_size,
            total_lines_parsed: self.total_lines,
            matched_http_requests: filtered_http_count,
            unmatched_lines: self.unmatched_count,
            total_cron_events: self.cron_events.len() as u64,
            parse_duration_ms: elapsed_ms,
            overall_p50_ms: overall_p50,
            overall_p90_ms: overall_p90,
            overall_p95_ms: overall_p95,
            overall_p99_ms: overall_p99,
            overall_error_count: filtered_error_count,
            overall_error_rate: error_rate,
            endpoints,
            cron_jobs,
            unmatched_samples: self.unmatched_samples.clone(),
        }
    }
}

pub fn parse_log_buffer(buffer: &[u8]) -> Engine {
    if buffer.is_empty() {
        return Engine::default();
    }

    let num_cpus = rayon::current_num_threads().max(1);
    let chunk_size = (buffer.len() / num_cpus).max(4 * 1024 * 1024);

    let mut chunk_starts = Vec::with_capacity(num_cpus + 1);
    chunk_starts.push(0);

    let mut curr = chunk_size;
    while curr < buffer.len() {
        if let Some(pos) = memchr(b'\n', &buffer[curr..]) {
            let next_start = curr + pos + 1;
            chunk_starts.push(next_start);
            curr = next_start + chunk_size;
        } else {
            break;
        }
    }
    if *chunk_starts.last().unwrap() < buffer.len() {
        chunk_starts.push(buffer.len());
    }

    let chunks: Vec<&[u8]> = chunk_starts
        .windows(2)
        .map(|w| &buffer[w[0]..w[1]])
        .collect();

    let mut engine = chunks
        .into_par_iter()
        .map(|chunk| parse_chunk(chunk))
        .reduce(Engine::default, |mut a, b| {
            a.merge(b);
            a
        });
    engine.finalize_paths();
    engine
}

fn parse_chunk(chunk: &[u8]) -> Engine {
    let mut engine = Engine::new();
    let est_lines = chunk.len() / 80;
    engine.entries.reserve(est_lines);
    engine.path_bytes.reserve(chunk.len() / 4);
    engine.path_off.reserve(2048);
    engine.path_len.reserve(2048);
    engine.path_index.reserve(2048);
    let mut start = 0;

    while start < chunk.len() {
        let end = match memchr(b'\n', &chunk[start..]) {
            Some(pos) => start + pos,
            None => chunk.len(),
        };

        engine.total_lines += 1;
        match parse_line_bytes(chunk, start, end) {
            LineKind::Empty => {}
            LineKind::Http {
                method,
                path,
                status,
                duration_ms,
            } => {
                let pid = engine.intern_path(path);
                engine.entries.push(PackedEntry {
                    path_id: pid,
                    duration: duration_ms,
                    status,
                    method: method as u8,
                    _pad: 0,
                });
            }
            LineKind::Cron {
                event,
                name,
                duration_ms,
            } => {
                let name_clean = strip_ansi_bytes(name);
                engine.cron_events.push(CronEvent {
                    name: String::from_utf8_lossy(&name_clean).into_owned(),
                    is_success: event == 1,
                    is_fail: event == 2,
                    duration_ms,
                });
            }
            LineKind::Unmatched(sample) => {
                engine.unmatched_count += 1;
                if engine.unmatched_samples.len() < 10 {
                    let clean = strip_ansi_bytes(sample);
                    engine
                        .unmatched_samples
                        .push(String::from_utf8_lossy(&clean).into_owned());
                }
            }
        }

        start = end + 1;
    }

    engine
}

fn strip_ansi_bytes(buf: &[u8]) -> Cow<'_, [u8]> {
    if memchr(0x1B, buf).is_none() {
        return Cow::Borrowed(buf);
    }
    let mut out = Vec::with_capacity(buf.len());
    let mut i = 0;
    while i < buf.len() {
        if buf[i] == 0x1B && i + 1 < buf.len() && buf[i + 1] == b'[' {
            i = skip_ansi(buf, i, buf.len());
        } else {
            out.push(buf[i]);
            i += 1;
        }
    }
    Cow::Owned(out)
}

struct ChunkResult {
    slots: FastHashMap<u32, EndpointAcc>,
    overall_sketch: RelHist,
    http_count: u64,
    error_count: u64,
}

#[derive(Clone, Debug, Default)]

pub struct StatusBuckets {
    pub c2xx: u32,
    pub c3xx: u32,
    pub c4xx: u32,
    pub c500: u32,
    pub c502: u32,
    pub c503: u32,
    pub c504: u32,
    pub c_other: u32,
}

impl StatusBuckets {
    #[inline(always)]
    pub fn inc_status(&mut self, status: u16) {
        match status {
            200..=299 => self.c2xx += 1,
            300..=399 => self.c3xx += 1,
            400..=499 => self.c4xx += 1,
            500 => self.c500 += 1,
            502 => self.c502 += 1,
            503 => self.c503 += 1,
            504 => self.c504 += 1,
            _ => self.c_other += 1,
        }
    }

    pub fn merge(&mut self, other: &StatusBuckets) {
        self.c2xx += other.c2xx;
        self.c3xx += other.c3xx;
        self.c4xx += other.c4xx;
        self.c500 += other.c500;
        self.c502 += other.c502;
        self.c503 += other.c503;
        self.c504 += other.c504;
        self.c_other += other.c_other;
    }

    pub fn to_map(&self) -> HashMap<u16, u64> {
        let mut map = HashMap::with_capacity(8);
        if self.c2xx > 0 { map.insert(200, self.c2xx as u64); }
        if self.c3xx > 0 { map.insert(300, self.c3xx as u64); }
        if self.c4xx > 0 { map.insert(400, self.c4xx as u64); }
        if self.c500 > 0 { map.insert(500, self.c500 as u64); }
        if self.c502 > 0 { map.insert(502, self.c502 as u64); }
        if self.c503 > 0 { map.insert(503, self.c503 as u64); }
        if self.c504 > 0 { map.insert(504, self.c504 as u64); }
        if self.c_other > 0 { map.insert(599, self.c_other as u64); }
        map
    }
}

struct EndpointAcc {
    total_calls: u64,
    error_calls: u64,
    total_duration_ms: f64,
    min_duration_ms: f32,
    max_duration_ms: f32,
    sketch: RelHist,
    status_buckets: StatusBuckets,
}




struct CronAcc {
    total_runs: u64,
    total_success: u64,
    total_failures: u64,
    total_duration_ms: f64,
    min_duration_ms: f32,
    max_duration_ms: f32,
    last_status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_aggregate_end_to_end() {
        let sample_logs = b"2026-07-24T00:00:10: GET /api/v1/users/507f1f77bcf86cd799439011/profile 200 12.5 ms - 42\n\
2026-07-24T00:00:11: POST /api/v1/auth/login 401 50.0 ms - 12\n\
68064.174ms\tPOST /api/v1/auth/login\n\
2026-07-24T00:00:12: [cron] done sync_job 1500ms\n\
some unmatched line here\n";

        let engine = parse_log_buffer(sample_logs);
        assert_eq!(engine.total_lines, 5);
        assert_eq!(engine.entries.len(), 3);
        assert_eq!(engine.cron_events.len(), 1);
        assert_eq!(engine.unmatched_count, 1);

        let filters = FilterOptions::default();
        let summary = engine.aggregate(sample_logs.len() as u64, 10, &filters);

        assert_eq!(summary.matched_http_requests, 3);
        assert_eq!(summary.endpoints.len(), 2);

        let user_ep = summary.endpoints.iter().find(|e| e.path.contains(":id")).unwrap();
        assert_eq!(user_ep.path, "/api/v1/users/:id/profile");
        assert_eq!(user_ep.total_calls, 1);

        let cron_job = &summary.cron_jobs[0];
        assert_eq!(cron_job.name, "sync_job");
        assert_eq!(cron_job.total_runs, 1);
    }
}

