//! Result + filter models mirroring `src/parser/types.ts` of the Wasm reference.

use serde::{Deserialize, Serialize};

pub const METHODS: [LogMethod; 6] = [
    LogMethod::Get,
    LogMethod::Post,
    LogMethod::Put,
    LogMethod::Patch,
    LogMethod::Delete,
    LogMethod::Head,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
}

impl LogMethod {
    pub fn index(self) -> u8 {
        match self {
            LogMethod::Get => 0,
            LogMethod::Post => 1,
            LogMethod::Put => 2,
            LogMethod::Patch => 3,
            LogMethod::Delete => 4,
            LogMethod::Head => 5,
        }
    }

    pub fn from_index(i: usize) -> LogMethod {
        METHODS.get(i).copied().unwrap_or(LogMethod::Get)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LogMethod::Get => "GET",
            LogMethod::Post => "POST",
            LogMethod::Put => "PUT",
            LogMethod::Patch => "PATCH",
            LogMethod::Delete => "DELETE",
            LogMethod::Head => "HEAD",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum NormalizeMode {
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "stripQuery")]
    StripQuery,
    #[serde(rename = "collapseIds")]
    CollapseIds,
}

impl NormalizeMode {
    /// 0 exact / 1 stripQuery / 2 collapseIds (kernel code).
    pub fn code(self) -> u8 {
        match self {
            NormalizeMode::Exact => 0,
            NormalizeMode::StripQuery => 1,
            NormalizeMode::CollapseIds => 2,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            NormalizeMode::Exact => "exact",
            NormalizeMode::StripQuery => "stripQuery",
            NormalizeMode::CollapseIds => "collapseIds",
        }
    }

    pub fn from_key(key: &str) -> Option<NormalizeMode> {
        match key {
            "exact" => Some(NormalizeMode::Exact),
            "stripQuery" => Some(NormalizeMode::StripQuery),
            "collapseIds" => Some(NormalizeMode::CollapseIds),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StatusFamily {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "2xx")]
    X2xx,
    #[serde(rename = "3xx")]
    X3xx,
    #[serde(rename = "4xx")]
    X4xx,
    #[serde(rename = "5xx")]
    X5xx,
}

impl StatusFamily {
    pub fn code(self) -> u8 {
        match self {
            StatusFamily::All => 0,
            StatusFamily::X2xx => 2,
            StatusFamily::X3xx => 3,
            StatusFamily::X4xx => 4,
            StatusFamily::X5xx => 5,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            StatusFamily::All => "all",
            StatusFamily::X2xx => "2xx",
            StatusFamily::X3xx => "3xx",
            StatusFamily::X4xx => "4xx",
            StatusFamily::X5xx => "5xx",
        }
    }

    pub fn from_key(key: &str) -> Option<StatusFamily> {
        match key {
            "all" => Some(StatusFamily::All),
            "2xx" => Some(StatusFamily::X2xx),
            "3xx" => Some(StatusFamily::X3xx),
            "4xx" => Some(StatusFamily::X4xx),
            "5xx" => Some(StatusFamily::X5xx),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseOptions {
    pub normalize_mode: NormalizeMode,
    pub method_filter: Option<Vec<String>>,
    pub status_family: StatusFamily,
    pub min_ms: f64,
    pub cron_query: String,
    pub cron_min_ms: f64,
    pub cron_show_failed_only: bool,
    pub date_filter: Option<String>,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            normalize_mode: NormalizeMode::CollapseIds,
            method_filter: None,
            status_family: StatusFamily::All,
            min_ms: 0.0,
            cron_query: String::new(),
            cron_min_ms: 0.0,
            cron_show_failed_only: false,
            date_filter: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedEndpoint {
    pub key: String,
    pub method: LogMethod,
    pub path: String,
    pub count: u64,
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
    pub min_ms: f64,
    pub error_count: u64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronAggregated {
    pub name: String,
    pub runs: u64,
    pub starts: u64,
    pub fails: u64,
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
    pub min_ms: f64,
    pub last_run_ts: Option<String>,
    pub last_duration_ms: Option<f64>,
}

#[derive(Clone, Default, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogSummary {
    pub matched: u64,
    pub unmatched: u64,
    pub max: f64,
    pub avg: f64,
    pub p95_ms: f64,
    pub errors: u64,
    pub slow: u64,
}

#[derive(Clone, Default, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronSummary {
    pub starts: u64,
    pub dones: u64,
    pub fails: u64,
    pub jobs: u64,
    pub slowest_run: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyBucket {
    pub hour: u8,
    pub label: String,
    pub count: u64,
    pub error_count: u64,
    pub avg_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaySummary {
    pub date: String,
    pub count: u64,
    pub error_count: u64,
    pub avg_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
    pub slow_count: u64,
    pub hourly_stats: Vec<HourlyBucket>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedResult {
    pub api: Vec<AggregatedEndpoint>,
    pub cron: Vec<CronAggregated>,
    pub summary: LogSummary,
    pub cron_summary: CronSummary,
    pub hourly_stats: Vec<HourlyBucket>,
    pub methods: Vec<String>,
    pub unmatched_sample: Vec<String>,
    pub unmatched_count: u64,
    pub dates: Vec<String>,
    pub daily_stats: Vec<DaySummary>,
}

impl Default for AggregatedResult {
    fn default() -> Self {
        Self {
            api: Vec::new(),
            cron: Vec::new(),
            summary: LogSummary::default(),
            cron_summary: CronSummary::default(),
            hourly_stats: Vec::new(),
            methods: Vec::new(),
            unmatched_sample: Vec::new(),
            unmatched_count: 0,
            dates: Vec::new(),
            daily_stats: Vec::new(),
        }
    }
}

pub const EMPTY_RESULT: AggregatedResult = AggregatedResult {
    api: Vec::new(),
    cron: Vec::new(),
    summary: LogSummary {
        matched: 0,
        unmatched: 0,
        max: 0.0,
        avg: 0.0,
        p95_ms: 0.0,
        errors: 0,
        slow: 0,
    },
    cron_summary: CronSummary {
        starts: 0,
        dones: 0,
        fails: 0,
        jobs: 0,
        slowest_run: 0.0,
    },
    hourly_stats: Vec::new(),
    methods: Vec::new(),
    unmatched_sample: Vec::new(),
    unmatched_count: 0,
    dates: Vec::new(),
    daily_stats: Vec::new(),
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CronEventKind {
    Start,
    Done,
    Fail,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronEventCompact {
    pub ts: Option<String>,
    pub event: CronEventKind,
    pub name: String,
    pub duration_ms: Option<f64>,
}
