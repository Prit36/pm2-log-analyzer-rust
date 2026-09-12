//! MongoDB aggregation result types — port of `src/mongo/types.ts`.
//!
//! The kernel serializes exactly this JSON shape, so the structs are a direct
//! serde mirror and decode with one pass.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Free-form document captured from a log line (`command`, `locks`, ...).
pub type MongoDoc = Value;

pub type MongoSlowQueries = Vec<MongoSlowQuery>;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoSummary {
    pub total_lines: u64,
    pub slow_query_count: u64,
    pub collscan_count: u64,
    pub avg_duration_ms: f64,
    pub p50_duration_ms: u32,
    pub p90_duration_ms: u32,
    pub p95_duration_ms: u32,
    pub p99_duration_ms: u32,
    pub max_duration_ms: u32,
    pub total_docs_examined: u64,
    pub total_keys_examined: u64,
    pub total_returned: u64,
    pub overall_scan_ratio: f64,
    pub unique_patterns: u64,
    pub unique_collections: u64,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoSlowQuery {
    pub id: String,
    pub timestamp: String,
    pub epoch_ms: f64,
    pub severity: String,
    pub component: String,
    pub ctx: String,
    pub ns: String,
    pub db: String,
    pub collection: String,
    pub op: String,
    pub duration_ms: u32,
    pub planning_time_micros: Option<u32>,
    pub plan_summary: String,
    pub is_collscan: bool,
    pub keys_examined: u64,
    pub docs_examined: u64,
    pub nreturned: u64,
    pub scan_ratio: f64,
    pub num_yields: u64,
    pub reslen: u64,
    pub query_hash: Option<String>,
    pub plan_cache_key: Option<String>,
    pub remote: Option<String>,
    pub command: MongoDoc,
    pub originating_command: Option<MongoDoc>,
    pub locks: Option<MongoDoc>,
    pub storage: Option<MongoDoc>,
    pub fingerprint: String,
    pub index_suggestion: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoQueryPattern {
    pub id: String,
    pub ns: String,
    pub db: String,
    pub collection: String,
    pub op: String,
    pub fingerprint: String,
    pub plan_summary: String,
    pub is_collscan: bool,
    pub count: u64,
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: u32,
    pub max_duration_ms: u32,
    pub p50_duration_ms: u32,
    pub p90_duration_ms: u32,
    pub p95_duration_ms: u32,
    pub p99_duration_ms: u32,
    pub total_docs_examined: u64,
    pub avg_docs_examined: f64,
    pub total_keys_examined: u64,
    pub avg_keys_examined: f64,
    pub total_returned: u64,
    pub avg_returned: f64,
    pub scan_ratio: f64,
    pub collscan_count: u64,
    pub index_suggestion: String,
    pub example_query: MongoSlowQuery,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoCollectionMetric {
    pub ns: String,
    pub collection: String,
    pub db: String,
    pub query_count: u64,
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub max_duration_ms: u32,
    pub p95_duration_ms: u32,
    pub collscan_count: u64,
    pub total_docs_examined: u64,
    pub total_returned: u64,
    pub scan_ratio: f64,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoTimeBucket {
    pub time_key: String,
    pub hour_label: String,
    pub query_count: u64,
    pub collscan_count: u64,
    pub avg_duration_ms: f64,
    pub p95_duration_ms: u32,
    pub max_duration_ms: u32,
    pub ops: std::collections::BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoDriverInfo {
    pub driver_name: String,
    pub driver_version: String,
    pub platform: String,
    pub os_name: String,
    pub os_version: String,
    pub count: u64,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoClientIp {
    pub ip: String,
    pub count: u64,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoConnectionStats {
    pub accepted: u64,
    pub ended: u64,
    pub peak_concurrent: u64,
    pub auth_success: u64,
    pub auth_failed: u64,
    pub drivers: Vec<MongoDriverInfo>,
    pub client_ips: Vec<MongoClientIp>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoErrorInfo {
    pub timestamp: String,
    pub severity: String,
    pub component: String,
    pub id: Option<u32>,
    pub msg: String,
    pub attr: Option<MongoDoc>,
    pub count: u64,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoCheckpointInfo {
    pub timestamp: String,
    pub thread: Option<String>,
    pub msg: String,
    pub bytes_written: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoUserTopCollection {
    pub ns: String,
    pub count: u64,
    pub total_duration_ms: u64,
    pub collscan_count: u64,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoUserActivity {
    pub user_name: String,
    pub auth_db: String,
    pub app_name: String,
    pub client_ips: Vec<String>,
    pub total_operations: u64,
    pub slow_query_count: u64,
    pub collscan_count: u64,
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: u32,
    pub max_duration_ms: u32,
    pub p95_duration_ms: u32,
    pub total_docs_examined: u64,
    pub total_keys_examined: u64,
    pub total_returned: u64,
    pub scan_ratio: f64,
    pub first_active: String,
    pub last_active: String,
    pub auth_success_count: u64,
    pub auth_fail_count: u64,
    pub operations: std::collections::BTreeMap<String, u64>,
    pub top_collections: Vec<MongoUserTopCollection>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoAggregationResult {
    pub summary: MongoSummary,
    pub patterns: Vec<MongoQueryPattern>,
    pub slow_queries: MongoSlowQueries,
    pub collections: Vec<MongoCollectionMetric>,
    pub time_buckets: Vec<MongoTimeBucket>,
    pub connections: MongoConnectionStats,
    pub errors: Vec<MongoErrorInfo>,
    pub checkpoints: Vec<MongoCheckpointInfo>,
    pub dates: Vec<String>,
    pub operations: Vec<String>,
    pub users: Vec<MongoUserActivity>,
    pub user_names: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MongoPlanFilter {
    #[default]
    All,
    CollscanOnly,
    IxscanOnly,
}

impl MongoPlanFilter {
    /// Kernel wire value (`0 = all, 1 = collscan only, 2 = ixscan only`).
    pub fn code(self) -> u8 {
        match self {
            MongoPlanFilter::All => 0,
            MongoPlanFilter::CollscanOnly => 1,
            MongoPlanFilter::IxscanOnly => 2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MongoPlanFilter::All => "All Plans",
            MongoPlanFilter::CollscanOnly => "COLLSCAN only",
            MongoPlanFilter::IxscanOnly => "Index scans only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MongoSortField {
    #[default]
    TotalDurationMs,
    AvgDurationMs,
    P95DurationMs,
    MaxDurationMs,
    Count,
    CollscanCount,
    TotalDocsExamined,
    ScanRatio,
    Collection,
}

impl MongoSortField {
    pub fn label(self) -> &'static str {
        match self {
            MongoSortField::TotalDurationMs => "total ms",
            MongoSortField::AvgDurationMs => "avg ms",
            MongoSortField::P95DurationMs => "p95 ms",
            MongoSortField::MaxDurationMs => "max ms",
            MongoSortField::Count => "count",
            MongoSortField::CollscanCount => "collscans",
            MongoSortField::TotalDocsExamined => "docs scanned",
            MongoSortField::ScanRatio => "scan ratio",
            MongoSortField::Collection => "collection",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MongoSlowQuerySortField {
    #[default]
    Timestamp,
    DurationMs,
    DocsExamined,
    KeysExamined,
    Nreturned,
    ScanRatio,
    Collection,
}

impl MongoSlowQuerySortField {
    pub fn label(self) -> &'static str {
        match self {
            MongoSlowQuerySortField::Timestamp => "time",
            MongoSlowQuerySortField::DurationMs => "duration",
            MongoSlowQuerySortField::DocsExamined => "docs",
            MongoSlowQuerySortField::KeysExamined => "keys",
            MongoSlowQuerySortField::Nreturned => "returned",
            MongoSlowQuerySortField::ScanRatio => "scan ratio",
            MongoSlowQuerySortField::Collection => "collection",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MongoSortDirection {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MongoFilters {
    pub operation: String,
    pub plan_filter: MongoPlanFilter,
    pub min_duration_ms: u32,
    pub collection: String,
    pub search_query: String,
    pub high_scan_ratio_only: bool,
    pub sort_field: MongoSortField,
    pub sort_direction: MongoSortDirection,
    pub slow_sort_field: MongoSlowQuerySortField,
    pub slow_sort_direction: MongoSortDirection,
    pub date_filter: String,
    pub user_filter: String,
}

impl Default for MongoFilters {
    fn default() -> Self {
        Self {
            operation: "all".to_string(),
            plan_filter: MongoPlanFilter::All,
            min_duration_ms: 0,
            collection: "all".to_string(),
            search_query: String::new(),
            high_scan_ratio_only: false,
            sort_field: MongoSortField::TotalDurationMs,
            sort_direction: MongoSortDirection::Desc,
            slow_sort_field: MongoSlowQuerySortField::DurationMs,
            slow_sort_direction: MongoSortDirection::Desc,
            date_filter: "all".to_string(),
            user_filter: "all".to_string(),
        }
    }
}

impl MongoFilters {
    /// `countActiveMongoFilters` — the reset-button badge.
    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if !self.search_query.trim().is_empty() {
            count += 1;
        }
        if self.plan_filter != MongoPlanFilter::All {
            count += 1;
        }
        if self.operation != "all" {
            count += 1;
        }
        if self.collection != "all" {
            count += 1;
        }
        if self.user_filter != "all" {
            count += 1;
        }
        if self.min_duration_ms > 0 {
            count += 1;
        }
        if self.high_scan_ratio_only {
            count += 1;
        }
        if self.date_filter != "all" {
            count += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_kernel_shape() {
        let json = r#"{
            "summary":{"totalLines":3,"slowQueryCount":2,"avgDurationMs":12.5},
            "patterns":[{"id":"pat-0","ns":"db.coll","collection":"coll","count":2,
                "indexSuggestion":"","exampleQuery":{"id":"q1","durationMs":5,"command":{"find":"coll"}}}],
            "slowQueries":[{"id":"q1","timestamp":"2026-01-02T03:04:05.000Z","op":"find",
                "durationMs":5,"isCollscan":true,"command":{"find":"coll","filter":{}}}],
            "collections":[{"ns":"db.coll","queryCount":2}],
            "timeBuckets":[{"timeKey":"03","hourLabel":"03:00","queryCount":2,"ops":{"find":2}}],
            "connections":{"accepted":4,"clientIps":[{"ip":"127.0.0.1","count":4}]},
            "errors":[{"timestamp":"t","severity":"E","component":"QUERY","msg":"boom","count":1}],
            "checkpoints":[{"timestamp":"t","msg":"WiredTiger checkpoint"}],
            "dates":["2026-01-02"],"operations":["find"],"users":[],"userNames":[]
        }"#;
        let result: MongoAggregationResult = serde_json::from_str(json).expect("decodes");
        assert_eq!(result.summary.total_lines, 3);
        assert_eq!(result.summary.slow_query_count, 2);
        assert_eq!(result.summary.unique_patterns, 0, "missing keys default");
        assert_eq!(result.patterns[0].collection, "coll");
        assert_eq!(result.slow_queries[0].command["find"], "coll");
        assert_eq!(result.connections.client_ips[0].count, 4);
        assert_eq!(result.time_buckets[0].ops["find"], 2);
        assert_eq!(result.errors[0].component, "QUERY");
        assert_eq!(result.user_names.len(), 0);
    }
}
