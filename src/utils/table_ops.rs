//! Client-side table operations — ports of the helpers in
//! `src/utils/exportSpreadsheet.ts`.

use crate::core::models::{AggregatedEndpoint, CronAggregated};
use crate::store::{ApiSortKey, CronSortKey, SortDirection};
use crate::utils::format::{format_ms, format_num};

pub fn sort_api_endpoints(
    rows: &[AggregatedEndpoint],
    sort_key: ApiSortKey,
    sort_dir: SortDirection,
) -> Vec<AggregatedEndpoint> {
    let mut out = rows.to_vec();
    out.sort_by(|a, b| {
        let cmp = if sort_key == ApiSortKey::Path {
            a.path.cmp(&b.path)
        } else {
            let va = api_value(a, sort_key);
            let vb = api_value(b, sort_key);
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        };
        if sort_dir == SortDirection::Asc {
            cmp
        } else {
            cmp.reverse()
        }
    });
    out
}

fn api_value(row: &AggregatedEndpoint, key: ApiSortKey) -> f64 {
    match key {
        ApiSortKey::P95Ms => row.p95_ms,
        ApiSortKey::P99Ms => row.p99_ms,
        ApiSortKey::AvgMs => row.avg_ms,
        ApiSortKey::MaxMs => row.max_ms,
        ApiSortKey::Count => row.count as f64,
        ApiSortKey::ErrorCount => row.error_count as f64,
        ApiSortKey::Path => 0.0,
    }
}

pub fn filter_api_endpoints(
    rows: &[AggregatedEndpoint],
    methods: &[String],
    query: &str,
) -> Vec<AggregatedEndpoint> {
    let q = query.trim().to_lowercase();
    rows.iter()
        .filter(|r| {
            if !methods.is_empty() && !methods.iter().any(|m| m == r.method.as_str()) {
                return false;
            }
            if q.is_empty() {
                return true;
            }
            r.path.to_lowercase().contains(&q) || r.key.to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}

pub fn sort_cron_jobs(
    rows: &[CronAggregated],
    sort_key: CronSortKey,
    sort_dir: SortDirection,
) -> Vec<CronAggregated> {
    let mut out = rows.to_vec();
    out.sort_by(|a, b| {
        if sort_key == CronSortKey::Name {
            return if sort_dir == SortDirection::Asc {
                a.name.cmp(&b.name)
            } else {
                b.name.cmp(&a.name)
            };
        }
        let va = cron_value(a, sort_key, sort_dir);
        let vb = cron_value(b, sort_key, sort_dir);
        let cmp = (va - vb)
            .partial_cmp(&0.0)
            .unwrap_or(std::cmp::Ordering::Equal);
        if sort_dir == SortDirection::Asc {
            cmp
        } else {
            cmp.reverse()
        }
    });
    out
}

fn cron_value(row: &CronAggregated, key: CronSortKey, dir: SortDirection) -> f64 {
    let missing = if dir == SortDirection::Asc {
        f64::INFINITY
    } else {
        f64::NEG_INFINITY
    };
    match key {
        CronSortKey::P95Ms => row.p95_ms,
        CronSortKey::P99Ms => row.p99_ms,
        CronSortKey::AvgMs => row.avg_ms,
        CronSortKey::MaxMs => row.max_ms,
        CronSortKey::Runs => row.runs as f64,
        CronSortKey::Fails => row.fails as f64,
        CronSortKey::Starts => row.starts as f64,
        CronSortKey::LastDurationMs => row.last_duration_ms.unwrap_or(missing),
        CronSortKey::Name => 0.0,
    }
}

pub fn build_api_tsv(rows: &[AggregatedEndpoint]) -> String {
    let header = [
        "Method", "Endpoint", "Count", "Avg", "p95", "p99", "Max", "Min", "Errors",
    ];
    let mut lines = vec![header.join("\t")];
    for r in rows {
        lines.push(
            [
                r.method.as_str().to_string(),
                r.path.clone(),
                format_num(r.count),
                format_ms(r.avg_ms),
                format_ms(r.p95_ms),
                format_ms(r.p99_ms),
                format_ms(r.max_ms),
                format_ms(r.min_ms),
                format_num(r.error_count),
            ]
            .join("\t"),
        );
    }
    lines.join("\r\n")
}

pub fn build_cron_tsv(rows: &[CronAggregated]) -> String {
    let header = [
        "Cron Job",
        "Runs",
        "Starts",
        "Fails",
        "Avg",
        "p95",
        "p99",
        "Max",
        "Min",
        "Last Run",
        "Last Duration",
    ];
    let mut lines = vec![header.join("\t")];
    for r in rows {
        lines.push(
            [
                r.name.clone(),
                format_num(r.runs),
                format_num(r.starts),
                format_num(r.fails),
                format_ms(r.avg_ms),
                format_ms(r.p95_ms),
                format_ms(r.p99_ms),
                format_ms(r.max_ms),
                format_ms(r.min_ms),
                r.last_run_ts.clone().unwrap_or_else(|| "-".to_string()),
                r.last_duration_ms
                    .map(format_ms)
                    .unwrap_or_else(|| "-".to_string()),
            ]
            .join("\t"),
        );
    }
    lines.join("\r\n")
}
