use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Options => "OPTIONS",
            Method::Head => "HEAD",
        }
    }

    #[allow(dead_code)]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"GET" => Some(Method::Get),
            b"POST" => Some(Method::Post),
            b"PUT" => Some(Method::Put),
            b"PATCH" => Some(Method::Patch),
            b"DELETE" => Some(Method::Delete),
            b"OPTIONS" => Some(Method::Options),
            b"HEAD" => Some(Method::Head),
            _ => None,
        }
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Method::Get),
            1 => Some(Method::Post),
            2 => Some(Method::Put),
            3 => Some(Method::Patch),
            4 => Some(Method::Delete),
            5 => Some(Method::Options),
            6 => Some(Method::Head),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathNormMode {
    Raw,
    StripQuery,
    CollapseIds,
}

impl Default for PathNormMode {
    fn default() -> Self {
        PathNormMode::CollapseIds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusFamily {
    All,
    Success, // 2xx
    Redirect, // 3xx
    ClientError, // 4xx
    ServerError, // 5xx
    ErrorOnly, // 4xx + 5xx
}

impl Default for StatusFamily {
    fn default() -> Self {
        StatusFamily::All
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterOptions {
    pub method: Option<Method>,
    pub status_family: StatusFamily,
    pub min_duration_ms: f32,
    pub max_duration_ms: Option<f32>,
    pub search_query: String,
    pub path_norm_mode: PathNormMode,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            method: None,
            status_family: StatusFamily::All,
            min_duration_ms: 0.0,
            max_duration_ms: None,
            search_query: String::new(),
            path_norm_mode: PathNormMode::CollapseIds,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndpointStats {
    pub path: String,
    pub method: Method,
    pub total_calls: u64,
    pub error_calls: u64,
    pub failed_calls: u64,
    pub total_duration_ms: f64,
    pub min_duration_ms: f32,
    pub max_duration_ms: f32,
    pub p50_ms: f32,
    pub p90_ms: f32,
    pub p95_ms: f32,
    pub p99_ms: f32,
    pub status_counts: std::collections::HashMap<u16, u64>,
}

impl EndpointStats {
    pub fn error_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            (self.error_calls as f64 / self.total_calls as f64) * 100.0
        }
    }

    pub fn avg_duration_ms(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.total_duration_ms / self.total_calls as f64
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CronJobStats {
    pub name: String,
    pub total_runs: u64,
    pub starts: u64,
    pub total_success: u64,
    pub failed_runs: u64,
    pub total_failures: u64,
    pub total_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f32,
    pub max_duration_ms: f32,
    pub p95_ms: f32,
    pub p99_ms: f32,
    pub last_duration_ms: f32,
    pub last_status: String,
}

pub type CronStats = CronJobStats;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HourlyBucket {
    pub hour: u8,
    pub label: String,
    pub count: u64,
    pub error_count: u64,
    pub avg_ms: f32,
    pub p95_ms: f32,
    pub p99_ms: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogAnalysisSummary {
    pub total_file_size_bytes: u64,
    pub total_lines_parsed: u64,
    pub matched_http_requests: u64,
    pub unmatched_lines: u64,
    pub total_cron_events: u64,
    pub parse_duration_ms: u64,
    pub overall_p50_ms: f32,
    pub overall_p90_ms: f32,
    pub overall_p95_ms: f32,
    pub overall_p99_ms: f32,
    pub overall_error_count: u64,
    pub overall_error_rate: f64,
    pub endpoints: Vec<EndpointStats>,
    pub cron_jobs: Vec<CronStats>,
    pub hourly_buckets: Vec<HourlyBucket>,
    pub unmatched_samples: Vec<String>,
}

impl LogAnalysisSummary {
    pub fn avg_latency_ms(&self) -> f32 {
        let total_calls: u64 = self.endpoints.iter().map(|e| e.total_calls).sum();
        let total_dur: f64 = self.endpoints.iter().map(|e| e.total_duration_ms).sum();
        if total_calls > 0 {
            (total_dur / total_calls as f64) as f32
        } else {
            0.0
        }
    }

    pub fn p95_latency_ms(&self) -> f32 {
        self.overall_p95_ms
    }

    pub fn error_responses(&self) -> u64 {
        self.overall_error_count
    }

    pub fn slow_requests_3s(&self) -> u64 {
        self.endpoints.iter().map(|e| if e.max_duration_ms >= 3000.0 { e.total_calls } else { 0 }).sum()
    }
}

impl Default for LogAnalysisSummary {
    fn default() -> Self {
        Self {
            total_file_size_bytes: 0,
            total_lines_parsed: 0,
            matched_http_requests: 0,
            unmatched_lines: 0,
            total_cron_events: 0,
            parse_duration_ms: 0,
            overall_p50_ms: 0.0,
            overall_p90_ms: 0.0,
            overall_p95_ms: 0.0,
            overall_p99_ms: 0.0,
            overall_error_count: 0,
            overall_error_rate: 0.0,
            endpoints: Vec::new(),
            cron_jobs: Vec::new(),
            hourly_buckets: Vec::new(),
            unmatched_samples: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Path,
    Method,
    Calls,
    ErrorRate,
    AvgDuration,
    P50,
    P95,
    P99,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableSortState {
    pub column: SortColumn,
    pub direction: SortDirection,
}

impl Default for TableSortState {
    fn default() -> Self {
        Self {
            column: SortColumn::P95,
            direction: SortDirection::Descending,
        }
    }
}
