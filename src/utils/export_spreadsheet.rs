//! Excel export — port of the reference `src/utils/exportSpreadsheet.ts`.
//!
//! Sheet layout, formats, table styles, frozen panes, totals rows, conditional formatting and
//! the embedded chart images mirror the reference workbook.

use std::path::PathBuf;

use rust_xlsxwriter::{
    Color, ConditionalFormatCell, ConditionalFormatCellRule, Format, FormatAlign,
    FormatBorder, Image, Table, TableColumn, TableFunction, TableStyle, Workbook, Worksheet,
};

use crate::core::models::{AggregatedEndpoint, CronAggregated, HourlyBucket, LogSummary};
use crate::store::{show_toast, AnalysisFilters, AnalysisStore, SortDirection};
use crate::utils::chart_renderer;
use crate::utils::format::{format_bytes, format_date, format_date_time, format_ms, format_num};

const MS_FMT: &str = "#,##0.0\" ms\"";
const TABLE_HEADER_ROW: u32 = 4; // 1-based row of the table header (0-based index 3).

// ── Sorting / filtering (mirrors the reference helpers) ─────────────────────

pub fn sort_api_endpoints(
    rows: &[AggregatedEndpoint],
    sort_key: crate::store::ApiSortKey,
    sort_dir: SortDirection,
) -> Vec<AggregatedEndpoint> {
    use crate::store::ApiSortKey;
    let mut out = rows.to_vec();
    out.sort_by(|a, b| {
        let cmp = match sort_key {
            ApiSortKey::Path => a.path.cmp(&b.path),
            ApiSortKey::Count => a.count.cmp(&b.count),
            ApiSortKey::AvgMs => a.avg_ms.total_cmp(&b.avg_ms),
            ApiSortKey::P95Ms => a.p95_ms.total_cmp(&b.p95_ms),
            ApiSortKey::P99Ms => a.p99_ms.total_cmp(&b.p99_ms),
            ApiSortKey::MaxMs => a.max_ms.total_cmp(&b.max_ms),
            ApiSortKey::ErrorCount => a.error_count.cmp(&b.error_count),
        };
        if sort_dir == SortDirection::Asc { cmp } else { cmp.reverse() }
    });
    out
}

pub fn filter_api_endpoints(
    rows: &[AggregatedEndpoint],
    methods: &[String],
    query: &str,
) -> Vec<AggregatedEndpoint> {
    let q = query.trim().to_lowercase();
    rows.iter()
        .filter(|r| methods.is_empty() || methods.iter().any(|m| m == r.method.as_str()))
        .filter(|r| {
            q.is_empty()
                || r.path.to_lowercase().contains(&q)
                || r.key.to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}

pub fn sort_cron_jobs(
    rows: &[CronAggregated],
    sort_key: crate::store::CronSortKey,
    sort_dir: SortDirection,
) -> Vec<CronAggregated> {
    use crate::store::CronSortKey;
    let mut out = rows.to_vec();
    out.sort_by(|a, b| {
        let cmp = match sort_key {
            CronSortKey::Name => a.name.cmp(&b.name),
            CronSortKey::Runs => a.runs.cmp(&b.runs),
            CronSortKey::Starts => a.starts.cmp(&b.starts),
            CronSortKey::Fails => a.fails.cmp(&b.fails),
            CronSortKey::AvgMs => a.avg_ms.total_cmp(&b.avg_ms),
            CronSortKey::P95Ms => a.p95_ms.total_cmp(&b.p95_ms),
            CronSortKey::P99Ms => a.p99_ms.total_cmp(&b.p99_ms),
            CronSortKey::MaxMs => a.max_ms.total_cmp(&b.max_ms),
            CronSortKey::LastDurationMs => {
                let av = a.last_duration_ms.unwrap_or(if sort_dir == SortDirection::Asc {
                    f64::INFINITY
                } else {
                    f64::NEG_INFINITY
                });
                let bv = b.last_duration_ms.unwrap_or(if sort_dir == SortDirection::Asc {
                    f64::INFINITY
                } else {
                    f64::NEG_INFINITY
                });
                av.total_cmp(&bv)
            }
        };
        if sort_dir == SortDirection::Asc { cmp } else { cmp.reverse() }
    });
    out
}

// ── Formats ─────────────────────────────────────────────────────────────────

fn title_format() -> Format {
    Format::new()
        .set_font_name("Calibri")
        .set_font_size(16)
        .set_bold()
        .set_font_color(Color::RGB(0x0F172A))
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter)
}

fn meta_format() -> Format {
    Format::new()
        .set_font_name("Calibri")
        .set_font_size(11)
        .set_font_color(Color::RGB(0x475569))
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter)
}

fn ms_format() -> Format {
    Format::new().set_num_format(MS_FMT)
}

fn method_style(method: &str) -> (u32, u32) {
    match method {
        "GET" => (0xDBEAFE, 0x1E40AF),
        "POST" => (0xD1FAE5, 0x065F46),
        _ => (0xFEF3C7, 0x92400E),
    }
}

fn method_format(method: &str) -> Format {
    let (fill, text) = method_style(method);
    Format::new()
        .set_font_name("Calibri")
        .set_font_size(11)
        .set_bold()
        .set_font_color(Color::RGB(text))
        .set_background_color(Color::RGB(fill))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
}

fn error_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(Color::RGB(0xDC2626))
}

fn kpi_header_format() -> Format {
    Format::new()
        .set_font_name("Calibri")
        .set_font_size(10)
        .set_bold()
        .set_font_color(Color::RGB(0x475569))
        .set_background_color(Color::RGB(0xF1F5F9))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
}

fn kpi_value_format() -> Format {
    Format::new()
        .set_font_name("Calibri")
        .set_font_size(12)
        .set_bold()
        .set_font_color(Color::RGB(0x0F172A))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
}

// ── Meta strings ────────────────────────────────────────────────────────────

fn now_locale_string() -> String {
    format_date_time(Some(&chrono::Local::now().to_rfc3339()))
}

fn api_sort_label(key: crate::store::ApiSortKey) -> &'static str {
    use crate::store::ApiSortKey;
    match key {
        ApiSortKey::P95Ms => "p95",
        ApiSortKey::P99Ms => "p99",
        ApiSortKey::AvgMs => "avg",
        ApiSortKey::MaxMs => "max",
        ApiSortKey::Count => "count",
        ApiSortKey::ErrorCount => "errors",
        ApiSortKey::Path => "endpoint",
    }
}

fn cron_sort_label(key: crate::store::CronSortKey) -> &'static str {
    use crate::store::CronSortKey;
    match key {
        CronSortKey::P95Ms => "p95",
        CronSortKey::P99Ms => "p99",
        CronSortKey::AvgMs => "avg",
        CronSortKey::MaxMs => "max",
        CronSortKey::Runs => "runs",
        CronSortKey::Fails => "fails",
        CronSortKey::Starts => "starts",
        CronSortKey::LastDurationMs => "last duration",
        CronSortKey::Name => "job",
    }
}

fn dir_label(dir: SortDirection) -> &'static str {
    match dir {
        SortDirection::Asc => "asc",
        SortDirection::Desc => "desc",
    }
}

fn build_api_filter_meta(
    filters: &AnalysisFilters,
    sort_key: crate::store::ApiSortKey,
    sort_dir: SortDirection,
    endpoint_count: usize,
    source_label: Option<&str>,
) -> String {
    let mut parts = vec![format!("Generated: {}", now_locale_string())];
    if let Some(label) = source_label {
        parts.push(format!("Source: {label}"));
    }
    parts.push(format!("Endpoints: {endpoint_count}"));
    parts.push(format!(
        "Sorted by: {} ({})",
        api_sort_label(sort_key),
        dir_label(sort_dir)
    ));
    if !filters.date_filter.is_empty() && filters.date_filter != "all" {
        parts.push(format!("Day: {}", format_date(Some(&filters.date_filter))));
    }
    if !filters.methods.is_empty() {
        parts.push(format!("Methods: {}", filters.methods.join(", ")));
    }
    if !filters.query.trim().is_empty() {
        parts.push(format!("Search: \"{}\"", filters.query.trim()));
    }
    if filters.status_family != crate::core::models::StatusFamily::All {
        parts.push(format!("Status: {}", filters.status_family.key()));
    }
    if filters.min_ms > 0.0 {
        parts.push(format!("Min Latency: ≥{}ms", filters.min_ms));
    }
    if filters.normalize_mode != crate::core::models::NormalizeMode::CollapseIds {
        parts.push(format!(
            "Normalize: {}",
            match filters.normalize_mode {
                crate::core::models::NormalizeMode::StripQuery => "Strip query",
                _ => "Exact path",
            }
        ));
    }
    parts.join("  |  ")
}

fn build_cron_filter_meta(
    filters: &AnalysisFilters,
    sort_key: crate::store::CronSortKey,
    sort_dir: SortDirection,
    job_count: usize,
    source_label: Option<&str>,
) -> String {
    let mut parts = vec![format!("Generated: {}", now_locale_string())];
    if let Some(label) = source_label {
        parts.push(format!("Source: {label}"));
    }
    parts.push(format!("Jobs: {job_count}"));
    parts.push(format!(
        "Sorted by: {} ({})",
        cron_sort_label(sort_key),
        dir_label(sort_dir)
    ));
    if !filters.cron_query.trim().is_empty() {
        parts.push(format!("Search: \"{}\"", filters.cron_query.trim()));
    }
    if filters.cron_min_ms > 0.0 {
        parts.push(format!("Min Duration: ≥{}ms", filters.cron_min_ms));
    }
    if filters.cron_show_failed_only {
        parts.push("Filter: Failures only".to_string());
    }
    if !filters.date_filter.is_empty() && filters.date_filter != "all" {
        parts.push(format!("Day: {}", format_date(Some(&filters.date_filter))));
    }
    parts.join("  |  ")
}

// ── Sheet helpers ───────────────────────────────────────────────────────────

fn style_title_meta(ws: &mut Worksheet, last_col: u16, title: &str, meta: &str) -> Result<(), String> {
    let err = |e: rust_xlsxwriter::XlsxError| e.to_string();
    ws.merge_range(0, 0, 0, last_col - 1, title, &title_format())
        .map_err(err)?;
    ws.set_row_height(0, 28.0).map_err(err)?;
    ws.merge_range(1, 0, 1, last_col - 1, meta, &meta_format())
        .map_err(err)?;
    ws.set_row_height(1, 18.0).map_err(err)?;
    ws.set_row_height(2, 8.0).map_err(err)?;
    ws.set_freeze_panes(4, 0).map_err(err)?;
    Ok(())
}

fn highlight_errors(ws: &mut Worksheet, col: u16, row_count: usize) -> Result<(), String> {
    if row_count == 0 {
        return Ok(());
    }
    let cf = ConditionalFormatCell::new()
        .set_rule(ConditionalFormatCellRule::GreaterThan(0))
        .set_format(&error_format());
    ws.add_conditional_format(
        TABLE_HEADER_ROW,
        col,
        TABLE_HEADER_ROW + row_count as u32 - 1,
        col,
        &cf,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// `ExcelJS` writes totals as `SUBTOTAL(109,Table[Column])`; `rust_xlsxwriter` writes the
/// equivalent short form `SUBTOTAL(109,[Column])`. Rewrite them so both files match.
fn write_totals(
    ws: &mut Worksheet,
    table: &str,
    cols: &[(u16, &str)],
    row_count: usize,
) -> Result<(), String> {
    let row = TABLE_HEADER_ROW + row_count as u32;
    for (col, name) in cols {
        ws.write_formula(row, *col, rust_xlsxwriter::Formula::new(format!(
            "SUBTOTAL(109,{table}[{name}])"
        )))
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Sheets ──────────────────────────────────────────────────────────────────

/// `ExcelJS` stores the column width verbatim; `rust_xlsxwriter` adds 5px of padding
/// (5/7 of a character). Subtract it so both files describe the same column width.
fn set_width(ws: &mut Worksheet, col: u16, width: f64) -> Result<(), String> {
    ws.set_column_width(col, width - 5.0 / 7.0)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn col_px(width: f64) -> f64 {
    (width * 7.0 + 5.0).round()
}

fn row_px(height: f64) -> f64 {
    (height * 4.0 / 3.0).round()
}

fn build_api_sheet(
    wb: &mut Workbook,
    rows: &[AggregatedEndpoint],
    sort_key: crate::store::ApiSortKey,
    sort_dir: SortDirection,
    filters: &AnalysisFilters,
    source_label: Option<&str>,
) -> Result<(), String> {
    let sorted_rows = sort_api_endpoints(rows, sort_key, sort_dir);
    let ws = wb.add_worksheet();
    ws.set_name("API Endpoints").map_err(|e| e.to_string())?;
    let widths = [10.0, 48.0, 10.0, 12.0, 12.0, 12.0, 12.0, 12.0, 10.0];
    for (i, w) in widths.iter().enumerate() {
        set_width(ws, i as u16, *w)?;
    }
    style_title_meta(
        ws,
        9,
        "PM2 Log Analyzer — API Endpoints",
        &build_api_filter_meta(
            filters,
            sort_key,
            sort_dir,
            sorted_rows.len(),
            source_label,
        ),
    )?;

    let headers = [
        ("Method", Some(TableFunction::Sum)),
        ("Endpoint", None),
        ("Count", Some(TableFunction::Sum)),
        ("Avg", None),
        ("p95", None),
        ("p99", None),
        ("Max", None),
        ("Min", None),
        ("Errors", Some(TableFunction::Sum)),
    ];
    let mut cols: Vec<TableColumn> = Vec::new();
    for (name, total) in headers {
        let mut col = TableColumn::new().set_header(name);
        if let Some(f) = total {
            col = col.set_total_function(f);
        }
        if name == "Method" {
            col = col.set_total_label("Total");
        }
        cols.push(col);
    }
    let table = Table::new()
        .set_name("ApiEndpoints")
        .set_style(TableStyle::Medium2)
        .set_total_row(true)
        .set_columns(&cols);
    ws.add_table(3, 0, 3 + sorted_rows.len() as u32 + 1, 8, &table)
        .map_err(|e| e.to_string())?;
    write_totals(ws, "ApiEndpoints", &[(2, "Count"), (8, "Errors")], sorted_rows.len())?;

    let ms = ms_format();
    for (i, r) in sorted_rows.iter().enumerate() {
        let row = TABLE_HEADER_ROW + i as u32;
        ws.write_string_with_format(row, 0, r.method.as_str(), &method_format(r.method.as_str()))
            .map_err(|e| e.to_string())?;
        ws.write_string(row, 1, &r.path).map_err(|e| e.to_string())?;
        ws.write_number(row, 2, r.count as f64)
            .map_err(|e| e.to_string())?;
        for (j, v) in [r.avg_ms, r.p95_ms, r.p99_ms, r.max_ms, r.min_ms].iter().enumerate() {
            ws.write_number_with_format(row, 3 + j as u16, *v, &ms)
                .map_err(|e| e.to_string())?;
        }
        ws.write_number(row, 8, r.error_count as f64)
            .map_err(|e| e.to_string())?;
    }
    highlight_errors(ws, 8, sorted_rows.len())?;
    Ok(())
}

fn build_daily_sheet(
    wb: &mut Workbook,
    daily: &[crate::core::models::DaySummary],
    source_label: Option<&str>,
) -> Result<(), String> {
    if daily.is_empty() {
        return Ok(());
    }
    let ws = wb.add_worksheet();
    ws.set_name("Daily Summary").map_err(|e| e.to_string())?;
    for (i, w) in [16.0, 14.0, 14.0, 14.0, 14.0, 14.0, 12.0, 14.0].iter().enumerate() {
        set_width(ws, i as u16, *w)?;
    }
    let mut meta_parts = vec![format!("Generated: {}", now_locale_string())];
    if let Some(label) = source_label {
        meta_parts.push(format!("Source: {label}"));
    }
    meta_parts.push(format!("Days: {}", daily.len()));
    style_title_meta(ws, 8, "PM2 Log Analyzer — Daily Summary", &meta_parts.join("  |  "))?;

    let headers = [
        ("Date", None, true),
        ("Requests", Some(TableFunction::Sum), false),
        ("Avg", None, false),
        ("P95", None, false),
        ("P99", None, false),
        ("Max", None, false),
        ("Errors", Some(TableFunction::Sum), false),
        ("Slow (≥3s)", Some(TableFunction::Sum), false),
    ];
    let mut cols: Vec<TableColumn> = Vec::new();
    for (name, total, label) in headers.iter() {
        let mut col = TableColumn::new().set_header(*name);
        if let Some(f) = total {
            col = col.set_total_function(f.clone());
        }
        if *label {
            col = col.set_total_label("Total");
        }
        cols.push(col);
    }
    let table = Table::new()
        .set_style(TableStyle::Medium2)
        .set_total_row(true)
        .set_columns(&cols);
    let last_col = (headers.len() - 1) as u16;
    ws.add_table(3, 0, 3 + daily.len() as u32 + 1, last_col, &table)
        .map_err(|e| e.to_string())?;

    let ms = ms_format();
    for (i, d) in daily.iter().enumerate() {
        let row = TABLE_HEADER_ROW + i as u32;
        ws.write_string(row, 0, format_date(Some(&d.date))).map_err(|e| e.to_string())?;
        ws.write_number(row, 1, d.count as f64).map_err(|e| e.to_string())?;
        for (j, v) in [d.avg_ms, d.p95_ms, d.p99_ms, d.max_ms].iter().enumerate() {
            ws.write_number_with_format(row, 2 + j as u16, *v, &ms)
                .map_err(|e| e.to_string())?;
        }
        ws.write_number(row, 6, d.error_count as f64).map_err(|e| e.to_string())?;
        ws.write_number(row, 7, d.slow_count as f64).map_err(|e| e.to_string())?;
    }
    highlight_errors(ws, 6, daily.len())?;
    Ok(())
}

fn build_cron_sheet(
    wb: &mut Workbook,
    rows: &[CronAggregated],
    sort_key: crate::store::CronSortKey,
    sort_dir: SortDirection,
    filters: &AnalysisFilters,
    source_label: Option<&str>,
) -> Result<(), String> {
    let sorted_rows = sort_cron_jobs(rows, sort_key, sort_dir);
    let ws = wb.add_worksheet();
    ws.set_name("Cron Jobs").map_err(|e| e.to_string())?;
    for (i, w) in [
        40.0, 10.0, 10.0, 10.0, 12.0, 12.0, 12.0, 12.0, 12.0, 22.0, 14.0,
    ]
    .iter()
    .enumerate()
    {
        set_width(ws, i as u16, *w)?;
    }
    style_title_meta(
        ws,
        11,
        "PM2 Log Analyzer — Cron Jobs",
        &build_cron_filter_meta(filters, sort_key, sort_dir, sorted_rows.len(), source_label),
    )?;

    let headers = [
        ("Cron Job", None, true),
        ("Runs", Some(TableFunction::Sum), false),
        ("Starts", Some(TableFunction::Sum), false),
        ("Fails", Some(TableFunction::Sum), false),
        ("Avg", None, false),
        ("p95", None, false),
        ("p99", None, false),
        ("Max", None, false),
        ("Min", None, false),
        ("Last Run", None, false),
        ("Last Duration", None, false),
    ];
    let mut cols: Vec<TableColumn> = Vec::new();
    for (name, total, label) in headers.iter() {
        let mut col = TableColumn::new().set_header(*name);
        if let Some(f) = total {
            col = col.set_total_function(f.clone());
        }
        if *label {
            col = col.set_total_label("Total");
        }
        cols.push(col);
    }
    let table = Table::new()
        .set_name("CronJobs")
        .set_style(TableStyle::Medium2)
        .set_total_row(true)
        .set_columns(&cols);
    let last_col = (headers.len() - 1) as u16;
    ws.add_table(3, 0, 3 + sorted_rows.len() as u32 + 1, last_col, &table)
        .map_err(|e| e.to_string())?;
    write_totals(
        ws,
        "CronJobs",
        &[(1, "Runs"), (2, "Starts"), (3, "Fails")],
        sorted_rows.len(),
    )?;

    let ms = ms_format();
    for (i, r) in sorted_rows.iter().enumerate() {
        let row = TABLE_HEADER_ROW + i as u32;
        ws.write_string(row, 0, &r.name).map_err(|e| e.to_string())?;
        ws.write_number(row, 1, r.runs as f64).map_err(|e| e.to_string())?;
        ws.write_number(row, 2, r.starts as f64).map_err(|e| e.to_string())?;
        ws.write_number(row, 3, r.fails as f64).map_err(|e| e.to_string())?;
        for (j, v) in [r.avg_ms, r.p95_ms, r.p99_ms, r.max_ms, r.min_ms].iter().enumerate() {
            ws.write_number_with_format(row, 4 + j as u16, *v, &ms)
                .map_err(|e| e.to_string())?;
        }
        match &r.last_run_ts {
            Some(ts) => {
                ws.write_string(row, 9, ts).map_err(|e| e.to_string())?;
            }
            None => {
                ws.write_string(row, 9, "-").map_err(|e| e.to_string())?;
            }
        }
        if let Some(d) = r.last_duration_ms {
            ws.write_number_with_format(row, 10, d, &ms).map_err(|e| e.to_string())?;
        }
    }
    highlight_errors(ws, 3, sorted_rows.len())?;
    Ok(())
}

fn classify_ms(ms: f64) -> usize {
    if ms < 50.0 {
        0
    } else if ms < 100.0 {
        1
    } else if ms < 300.0 {
        2
    } else if ms < 500.0 {
        3
    } else if ms < 1000.0 {
        4
    } else if ms < 3000.0 {
        5
    } else {
        6
    }
}

fn distribution_buckets(rows: &[AggregatedEndpoint]) -> [(String, u64); 7] {
    let mut buckets = [
        ("<50ms".to_string(), 0u64),
        ("50-100ms".to_string(), 0),
        ("100-300ms".to_string(), 0),
        ("300-500ms".to_string(), 0),
        ("500ms-1s".to_string(), 0),
        ("1s-3s".to_string(), 0),
        (">3s".to_string(), 0),
    ];
    for r in rows {
        if r.count == 0 {
            continue;
        }
        let c50 = (r.count as f64 * 0.5).round() as u64;
        let c90 = (r.count as f64 * 0.4).round() as u64;
        let c95 = (r.count as f64 * 0.05).round() as u64;
        let c99 = (r.count as f64 * 0.04).round() as u64;
        let c_max = r
            .count
            .saturating_sub(c50)
            .saturating_sub(c90)
            .saturating_sub(c95)
            .saturating_sub(c99);
        buckets[classify_ms(r.p50_ms)].1 += c50;
        buckets[classify_ms((r.p50_ms + r.p90_ms) / 2.0)].1 += c90;
        buckets[classify_ms((r.p90_ms + r.p95_ms) / 2.0)].1 += c95;
        buckets[classify_ms((r.p95_ms + r.p99_ms) / 2.0)].1 += c99;
        buckets[classify_ms(r.max_ms)].1 += c_max;
    }
    buckets
}

fn build_hourly_sheet(
    wb: &mut Workbook,
    hourly: &[HourlyBucket],
    api_rows: &[AggregatedEndpoint],
    filters: &AnalysisFilters,
    source_label: Option<&str>,
) -> Result<(), String> {
    if hourly.is_empty() && api_rows.is_empty() {
        return Ok(());
    }
    let ws = wb.add_worksheet();
    ws.set_name("Hourly & Distribution").map_err(|e| e.to_string())?;
    let widths = [15.0, 15.0, 15.0, 15.0, 15.0, 15.0, 6.0, 18.0, 18.0];
    for (i, w) in widths.iter().enumerate() {
        set_width(ws, i as u16, *w)?;
    }
    let mut meta_parts = vec![format!("Generated: {}", now_locale_string())];
    if let Some(label) = source_label {
        meta_parts.push(format!("Source: {label}"));
    }
    if !filters.date_filter.is_empty() && filters.date_filter != "all" {
        meta_parts.push(format!("Day: {}", format_date(Some(&filters.date_filter))));
    }
    style_title_meta(
        ws,
        9,
        "PM2 Log Analyzer — Hourly Trends & Distribution Data",
        &meta_parts.join("  |  "),
    )?;

    if !hourly.is_empty() {
        let headers = [
            ("Hour", None, true),
            ("Total Requests", Some(TableFunction::Sum), false),
            ("Avg Latency", None, false),
            ("P95 Latency", None, false),
            ("P99 Latency", None, false),
            ("Errors", Some(TableFunction::Sum), false),
        ];
        let mut cols: Vec<TableColumn> = Vec::new();
        for (name, total, label) in headers {
            let mut col = TableColumn::new().set_header(name);
            if let Some(f) = total {
                col = col.set_total_function(f);
            }
            if label {
                col = col.set_total_label("Total");
            }
            cols.push(col);
        }
        let table = Table::new()
            .set_name("HourlyTrends")
            .set_style(TableStyle::Medium2)
            .set_total_row(true)
            .set_columns(&cols);
        ws.add_table(3, 0, 3 + hourly.len() as u32 + 1, 5, &table)
            .map_err(|e| e.to_string())?;
        write_totals(ws, "HourlyTrends", &[(1, "Total Requests"), (5, "Errors")], hourly.len())?;
        let ms = ms_format();
        for (i, h) in hourly.iter().enumerate() {
            let row = TABLE_HEADER_ROW + i as u32;
            ws.write_string(row, 0, &h.label).map_err(|e| e.to_string())?;
            ws.write_number(row, 1, h.count as f64).map_err(|e| e.to_string())?;
            for (j, v) in [h.avg_ms, h.p95_ms, h.p99_ms].iter().enumerate() {
                ws.write_number_with_format(row, 2 + j as u16, *v, &ms)
                    .map_err(|e| e.to_string())?;
            }
            ws.write_number(row, 5, h.error_count as f64)
                .map_err(|e| e.to_string())?;
        }
        highlight_errors(ws, 5, hourly.len())?;
    }

    // Distribution table at column H (index 7).
    let buckets = distribution_buckets(api_rows);
    let table = Table::new()
        .set_name("LatencyDistribution")
        .set_style(TableStyle::Medium2)
        .set_total_row(true)
        .set_columns(&[
            TableColumn::new()
                .set_header("Latency Range")
                .set_total_label("Total"),
            TableColumn::new()
                .set_header("Request Count")
                .set_total_function(TableFunction::Sum),
        ]);
    ws.add_table(3, 7, 3 + buckets.len() as u32 + 1, 8, &table)
        .map_err(|e| e.to_string())?;
    write_totals(ws, "LatencyDistribution", &[(8, "Request Count")], buckets.len())?;
    for (i, (label, count)) in buckets.iter().enumerate() {
        let row = TABLE_HEADER_ROW + i as u32;
        ws.write_string(row, 7, label).map_err(|e| e.to_string())?;
        ws.write_number(row, 8, *count as f64)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn resolve_visual_kpis(
    api_rows: &[AggregatedEndpoint],
    summary: Option<&LogSummary>,
    filters: &AnalysisFilters,
) -> (u64, u64, String, f64, f64) {
    let is_filtered = !filters.methods.is_empty() || !filters.query.trim().is_empty();
    let sum_count: u64 = api_rows.iter().map(|r| r.count).sum();
    let sum_errors: u64 = api_rows.iter().map(|r| r.error_count).sum();
    let total_count = if is_filtered {
        sum_count
    } else {
        summary.map(|s| s.matched).unwrap_or(sum_count)
    };
    let total_errors = if is_filtered {
        sum_errors
    } else {
        summary.map(|s| s.errors).unwrap_or(sum_errors)
    };
    let error_rate = if total_count > 0 {
        format!("{:.2}", (total_errors as f64 / total_count as f64) * 100.0)
    } else {
        "0.00".to_string()
    };
    let avg_ms = if is_filtered {
        if total_count > 0 {
            api_rows.iter().map(|r| r.avg_ms * r.count as f64).sum::<f64>() / total_count as f64
        } else {
            0.0
        }
    } else {
        summary.map(|s| s.avg).unwrap_or(0.0)
    };
    let p95_ms = if is_filtered {
        api_rows.iter().map(|r| r.p95_ms).fold(0.0f64, f64::max)
    } else {
        summary.map(|s| s.p95_ms).unwrap_or(0.0)
    };
    (total_count, total_errors, error_rate, avg_ms, p95_ms)
}

fn build_visual_sheet(
    wb: &mut Workbook,
    api_rows: &[AggregatedEndpoint],
    hourly: &[HourlyBucket],
    summary: Option<&LogSummary>,
    daily: &[crate::core::models::DaySummary],
    filters: &AnalysisFilters,
    source_label: Option<&str>,
) -> Result<(), String> {
    let ws = wb.add_worksheet();
    ws.set_name("Visual Analytics").map_err(|e| e.to_string())?;
    for i in 0..15 {
        set_width(ws, i, if i == 7 { 4.0 } else { 13.0 })?;
    }

    let mut meta_parts = vec![format!("Generated: {}", now_locale_string())];
    if let Some(label) = source_label {
        meta_parts.push(format!("Source: {label}"));
    }
    if !filters.date_filter.is_empty() && filters.date_filter != "all" {
        meta_parts.push(format!("Day: {}", format_date(Some(&filters.date_filter))));
    }
    if !filters.methods.is_empty() {
        meta_parts.push(format!("Methods: {}", filters.methods.join(", ")));
    }
    if !filters.query.trim().is_empty() {
        meta_parts.push(format!("Search: \"{}\"", filters.query.trim()));
    }
    let (total_count, total_errors, error_rate, avg_ms, p95_ms) =
        resolve_visual_kpis(api_rows, summary, filters);
    meta_parts.push(format!("Total Requests: {}", format_num(total_count)));
    meta_parts.push(format!("Endpoints: {}", api_rows.len()));
    style_title_meta(
        ws,
        15,
        "PM2 Log Analyzer — Visual Analytics & Charts",
        &meta_parts.join("  |  "),
    )?;

    // KPI block: headers in row 4, values in row 5.
    ws.set_row_height(3, 20.0).map_err(|e| e.to_string())?;
    ws.set_row_height(4, 24.0).map_err(|e| e.to_string())?;
    let kpis: [(&str, String); 6] = [
        ("Total Requests", format_num(total_count)),
        ("Total Errors", format_num(total_errors)),
        ("Error Rate", format!("{}%", error_rate)),
        ("Avg Latency", format_ms(avg_ms)),
        ("P95 Latency", format_ms(p95_ms)),
        ("Unique Endpoints", api_rows.len().to_string()),
    ];
    for (i, (title, value)) in kpis.iter().enumerate() {
        let col = i as u16;
        ws.write_string_with_format(3, col, *title, &kpi_header_format())
            .map_err(|e| e.to_string())?;
        if i < 2 || i == 5 {
            let n: f64 = value.replace(',', "").parse().unwrap_or(0.0);
            ws.write_number_with_format(4, col, n, &kpi_value_format())
                .map_err(|e| e.to_string())?;
        } else {
            ws.write_string_with_format(4, col, value, &kpi_value_format())
                .map_err(|e| e.to_string())?;
        }
    }

    // Chart images (light theme, 2x PNG), positioned over the same cell ranges.
    let images = chart_renderer::generate_all(api_rows, hourly, daily);
    let place = |ws: &mut Worksheet,
                 png_b64: &str,
                 start_row: u32,
                 start_col: u16,
                 end_row: u32,
                 end_col: u16|
     -> Result<(), String> {
        let Some(bytes) = b64_decode(png_b64) else {
            return Ok(());
        };
        let mut image = Image::new_from_buffer(&bytes).map_err(|e| e.to_string())?;
        let width: f64 = (start_col..=end_col).map(|_| col_px(13.0)).sum();
        let height: f64 = (start_row..=end_row)
            .map(|r| if r == 0 { 28.0 } else if r == 1 { 18.0 } else { 20.0 })
            .map(row_px)
            .sum();
        image = image
            .set_scale_to_size(width, height, false)
            .set_alt_text("chart");
        ws.insert_image(start_row, start_col, &image)
            .map_err(|e| e.to_string())?;
        Ok(())
    };
    place(ws, &images.time_vs_latency, 7, 0, 23, 6)?;
    place(ws, &images.hourly_volume, 7, 8, 23, 15)?;
    place(ws, &images.distribution, 26, 0, 42, 6)?;
    if let Some(daily_trend) = &images.daily_trend {
        place(ws, daily_trend, 26, 8, 42, 15)?;
    }
    Ok(())
}

// ── Workbook ────────────────────────────────────────────────────────────────

fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let mut buf = 0u32;
    let mut bits = 0u32;
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ => continue,
        } as u32;
        buf = (buf << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Some(out)
}

pub struct ExportInput<'a> {
    pub api: &'a [AggregatedEndpoint],
    pub cron: &'a [CronAggregated],
    pub hourly: &'a [HourlyBucket],
    pub daily: &'a [crate::core::models::DaySummary],
    pub summary: Option<&'a LogSummary>,
    pub filters: &'a AnalysisFilters,
    pub source_label: Option<&'a str>,
}

/// Build the workbook in memory (used by the app and by the parity runner).
pub fn build_workbook(input: &ExportInput) -> Result<Workbook, String> {
    let mut wb = Workbook::new();
    let props = rust_xlsxwriter::DocProperties::new()
        .set_title("PM2 Log Analyzer Report")
        .set_author("PM2 Log Analyzer")
        .set_subject(input.source_label.unwrap_or("PM2 Log Analyzer Report"))
        .set_comment(match input.source_label {
            Some(label) => format!("Source: {label}"),
            None => "Source: PM2 Log Analyzer".to_string(),
        });
    wb.set_properties(&props);

    build_api_sheet(
        &mut wb,
        input.api,
        input.filters.sort_key,
        input.filters.sort_dir,
        input.filters,
        input.source_label,
    )?;
    if input.daily.len() > 1 {
        build_daily_sheet(&mut wb, input.daily, input.source_label)?;
    }
    if !input.cron.is_empty() {
        build_cron_sheet(
            &mut wb,
            input.cron,
            input.filters.cron_sort_key,
            input.filters.cron_sort_dir,
            input.filters,
            input.source_label,
        )?;
    }
    build_hourly_sheet(&mut wb, input.hourly, input.api, input.filters, input.source_label)?;
    build_visual_sheet(
        &mut wb,
        input.api,
        input.hourly,
        input.summary,
        input.daily,
        input.filters,
        input.source_label,
    )?;
    Ok(wb)
}

fn export_file_name() -> String {
    let ts = chrono::Local::now()
        .format("%Y-%m-%d-%H-%M-%S")
        .to_string();
    format!("pm2-analyzer-report-{ts}.xlsx")
}

fn build_source_label(store: &AnalysisStore) -> Option<String> {
    use crate::store::SourceKind;
    match store.source_kind() {
        SourceKind::File => store.file_name().map(|name| match store.file_size() {
            Some(size) => format!("{name} ({})", format_bytes(size)),
            None => name,
        }),
        SourceKind::Paste => Some("Pasted text".to_string()),
        _ => None,
    }
}

fn save_target() -> Option<PathBuf> {
    let name = export_file_name();
    if let Ok(dir) = std::env::var("PM2_ANALYZER_EXPORT_DIR") {
        return Some(PathBuf::from(dir).join(name));
    }
    rfd::FileDialog::new()
        .set_file_name(&name)
        .add_filter("Excel Workbook", &["xlsx"])
        .save_file()
}

pub fn export_spreadsheet_data(store: AnalysisStore) {
    let result = store.result();
    let filters = store.filters();
    let (api, cron) = match &result {
        Some(r) => (
            filter_api_endpoints(&r.api, &filters.methods, &filters.query),
            r.cron.clone(),
        ),
        None => (Vec::new(), Vec::new()),
    };
    if api.is_empty() && cron.is_empty() {
        show_toast(store, "Nothing to export yet");
        return;
    }
    let source_label = build_source_label(&store);
    let input = ExportInput {
        api: &api,
        cron: &cron,
        hourly: result.as_ref().map(|r| r.hourly_stats.as_slice()).unwrap_or(&[]),
        daily: result.as_ref().map(|r| r.daily_stats.as_slice()).unwrap_or(&[]),
        summary: result.as_ref().map(|r| &r.summary),
        filters: &filters,
        source_label: source_label.as_deref(),
    };
    let built = build_workbook(&input).and_then(|mut wb| {
        let Some(path) = save_target() else {
            return Err("cancelled".to_string());
        };
        wb.save(&path).map_err(|e| e.to_string())
    });
    match built {
        Ok(()) => show_toast(
            store,
            if !cron.is_empty() {
                "Excel downloaded — Visual Analytics + Data sheets"
            } else {
                "Excel downloaded — Visual Analytics + API sheets"
            },
        ),
        Err(msg) if msg == "cancelled" => {}
        Err(_) => show_toast(store, "Excel export failed"),
    }
}

/// Write the workbook for the given input to `path` (used by the parity runner).
pub fn write_export(input: &ExportInput, path: &std::path::Path) -> Result<(), String> {
    let mut wb = build_workbook(input)?;
    wb.save(path).map_err(|e| e.to_string())
}

pub fn export_mongo_spreadsheet_placeholder(store: AnalysisStore) {
    show_toast(store, "Export is not wired yet");
}
