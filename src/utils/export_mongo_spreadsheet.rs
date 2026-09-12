//! MongoDB workbook export — port of `src/mongo/mongoExport.ts`.
//!
//! Same five sheets and column widths as the reference, written with
//! `rust_xlsxwriter` behind a worker thread (the app builds the workbook off the
//! UI thread, exactly like the PM2 export).

use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet};

use crate::core::mongo_models::MongoAggregationResult;

/// Top slow queries written to the workbook (reference slices at 2500).
const MAX_QUERY_ROWS: usize = 2500;
/// `FF064E3B` — emerald-900 header fill.
const HEADER_FILL: Color = Color::RGB(0x064E3B);
const HEADER_TEXT: Color = Color::RGB(0xFFFFFF);

fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_size(11)
        .set_font_color(HEADER_TEXT)
        .set_background_color(HEADER_FILL)
        .set_border(FormatBorder::Thin)
}

fn text_format() -> Format {
    Format::new().set_font_size(11)
}

fn number_format() -> Format {
    Format::new().set_font_size(11).set_align(FormatAlign::Right)
}

fn set_widths(worksheet: &mut Worksheet, widths: &[f64]) -> Result<(), String> {
    for (index, width) in widths.iter().enumerate() {
        worksheet
            .set_column_width(index as u16, *width)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn write_header(worksheet: &mut Worksheet, headers: &[&str]) -> Result<(), String> {
    let format = header_format();
    for (index, header) in headers.iter().enumerate() {
        worksheet
            .write_string_with_format(0, index as u16, *header, &format)
            .map_err(|error| error.to_string())?;
        worksheet
            .set_row_height(0, 18.0)
            .map_err(|error| error.to_string())?;
    }
    worksheet.set_freeze_panes(1, 0).map_err(|error| error.to_string())?;
    Ok(())
}

/// Builds the MongoDB workbook for the current result.
pub fn build_workbook(result: &MongoAggregationResult, source_label: Option<&str>) -> Result<Workbook, String> {
    let mut workbook = Workbook::new();
    let properties = rust_xlsxwriter::DocProperties::new()
        .set_title("MongoDB Log Analyzer Report")
        .set_author("MongoDB Log Analyzer")
        .set_subject(source_label.unwrap_or("MongoDB Log Analyzer Report"))
        .set_comment(match source_label {
            Some(label) => format!("Source: {label}"),
            None => "Source: MongoDB Log Analyzer".to_string(),
        });
    workbook.set_properties(&properties);

    build_patterns_sheet(&mut workbook, result)?;
    build_queries_sheet(&mut workbook, result)?;
    build_collections_sheet(&mut workbook, result)?;
    if !result.errors.is_empty() {
        build_errors_sheet(&mut workbook, result)?;
    }
    if !result.users.is_empty() {
        build_users_sheet(&mut workbook, result)?;
    }
    Ok(workbook)
}

fn build_patterns_sheet(workbook: &mut Workbook, result: &MongoAggregationResult) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Query Patterns")
        .map_err(|error| error.to_string())?;
    set_widths(
        worksheet,
        &[
            32.0, 14.0, 22.0, 55.0, 12.0, 16.0, 14.0, 14.0, 14.0, 14.0, 14.0, 18.0, 45.0,
        ],
    )?;
    write_header(
        worksheet,
        &[
            "Namespace",
            "Operation",
            "Plan Summary",
            "Query Fingerprint",
            "Count",
            "Total Time (s)",
            "Avg (ms)",
            "P95 (ms)",
            "Max (ms)",
            "COLLSCANs",
            "Scan Ratio",
            "Avg Docs Examined",
            "Suggested Index",
        ],
    )?;

    let text = text_format();
    let number = number_format();
    for (index, pattern) in result.patterns.iter().enumerate() {
        let row = index as u32 + 1;
        let cells: [String; 13] = [
            pattern.ns.clone(),
            pattern.op.clone(),
            pattern.plan_summary.clone(),
            pattern.fingerprint.clone(),
            pattern.count.to_string(),
            js_round2(pattern.total_duration_ms as f64 / 1000.0),
            js_number(pattern.avg_duration_ms),
            pattern.p95_duration_ms.to_string(),
            pattern.max_duration_ms.to_string(),
            pattern.collscan_count.to_string(),
            js_number(pattern.scan_ratio),
            js_number(pattern.avg_docs_examined),
            if pattern.index_suggestion.is_empty() {
                "N/A".to_string()
            } else {
                pattern.index_suggestion.clone()
            },
        ];
        for (column, value) in cells.iter().enumerate() {
            if column == 4
                || column == 5
                || column == 6
                || column == 7
                || column == 8
                || column == 9
                || column == 10
                || column == 11
            {
                if let Ok(parsed) = value.parse::<f64>() {
                    worksheet
                        .write_number_with_format(row, column as u16, parsed, &number)
                        .map_err(|error| error.to_string())?;
                    continue;
                }
            }
            worksheet
                .write_string_with_format(row, column as u16, value, &text)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn build_queries_sheet(workbook: &mut Workbook, result: &MongoAggregationResult) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Slow Queries")
        .map_err(|error| error.to_string())?;
    set_widths(
        worksheet,
        &[
            26.0, 18.0, 14.0, 15.0, 14.0, 32.0, 22.0, 16.0, 16.0, 14.0, 14.0, 12.0, 14.0, 24.0,
        ],
    )?;
    write_header(
        worksheet,
        &[
            "Timestamp",
            "User",
            "Connection",
            "Duration (ms)",
            "Operation",
            "Namespace",
            "Plan",
            "Docs Examined",
            "Keys Examined",
            "Returned",
            "Scan Ratio",
            "Yields",
            "ResLen (B)",
            "Remote IP",
        ],
    )?;

    let text = text_format();
    let number = number_format();
    for (index, query) in result.slow_queries.iter().take(MAX_QUERY_ROWS).enumerate() {
        let row = index as u32 + 1;
        for (column, value) in [
            (0_u16, query.timestamp.clone()),
            (1, query.user.clone().unwrap_or_else(|| "system".to_string())),
            (2, query.ctx.clone()),
            (4, query.op.clone()),
            (5, query.ns.clone()),
            (6, query.plan_summary.clone()),
            (13, query.remote.clone().unwrap_or_else(|| "unknown".to_string())),
        ] {
            worksheet
                .write_string_with_format(row, column, &value, &text)
                .map_err(|error| error.to_string())?;
        }
        for (column, value) in [
            (3_u16, f64::from(query.duration_ms)),
            (7, query.docs_examined as f64),
            (8, query.keys_examined as f64),
            (9, query.nreturned as f64),
            (10, round_to_tenth(query.scan_ratio)),
            (11, query.num_yields as f64),
            (12, query.reslen as f64),
        ] {
            worksheet
                .write_number_with_format(row, column, value, &number)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn build_collections_sheet(
    workbook: &mut Workbook,
    result: &MongoAggregationResult,
) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Collections")
        .map_err(|error| error.to_string())?;
    set_widths(
        worksheet,
        &[32.0, 15.0, 16.0, 18.0, 18.0, 18.0, 15.0, 22.0, 15.0],
    )?;
    write_header(
        worksheet,
        &[
            "Namespace",
            "Query Count",
            "Total Time (s)",
            "Avg Duration (ms)",
            "P95 Duration (ms)",
            "Max Duration (ms)",
            "COLLSCANs",
            "Total Docs Examined",
            "Scan Ratio",
        ],
    )?;

    let text = text_format();
    let number = number_format();
    for (index, collection) in result.collections.iter().enumerate() {
        let row = index as u32 + 1;
        worksheet
            .write_string_with_format(row, 0, &collection.ns, &text)
            .map_err(|error| error.to_string())?;
        let numbers: [(u16, f64); 8] = [
            (1, collection.query_count as f64),
            (2, round_2(collection.total_duration_ms as f64 / 1000.0)),
            (3, collection.avg_duration_ms),
            (4, collection.p95_duration_ms as f64),
            (5, collection.max_duration_ms as f64),
            (6, collection.collscan_count as f64),
            (7, collection.total_docs_examined as f64),
            (8, collection.scan_ratio),
        ];
        for (column, value) in numbers {
            worksheet
                .write_number_with_format(row, column, value, &number)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn build_errors_sheet(workbook: &mut Workbook, result: &MongoAggregationResult) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Errors & Warnings")
        .map_err(|error| error.to_string())?;
    set_widths(worksheet, &[26.0, 12.0, 16.0, 14.0, 14.0, 60.0])?;
    write_header(
        worksheet,
        &["Timestamp", "Severity", "Component", "ID", "Occurrences", "Message"],
    )?;

    let text = text_format();
    let number = number_format();
    for (index, error) in result.errors.iter().enumerate() {
        let row = index as u32 + 1;
        for (column, value) in [
            (0_u16, error.timestamp.clone()),
            (1, error.severity.clone()),
            (2, error.component.clone()),
        ] {
            worksheet
                .write_string_with_format(row, column, &value, &text)
                .map_err(|error| error.to_string())?;
        }
        match error.id {
            Some(id) => worksheet
                .write_number_with_format(row, 3, f64::from(id), &number)
                .map_err(|error| error.to_string())?,
            None => worksheet
                .write_string_with_format(row, 3, "N/A", &text)
                .map_err(|error| error.to_string())?,
        };
        worksheet
            .write_number_with_format(row, 4, error.count as f64, &number)
            .map_err(|error| error.to_string())?;
        worksheet
            .write_string_with_format(row, 5, &error.msg, &text)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn build_users_sheet(workbook: &mut Workbook, result: &MongoAggregationResult) -> Result<(), String> {
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("User Activity")
        .map_err(|error| error.to_string())?;
    set_widths(
        worksheet,
        &[
            22.0, 14.0, 24.0, 26.0, 14.0, 14.0, 16.0, 14.0, 14.0, 14.0, 16.0, 14.0, 14.0, 14.0,
            24.0, 24.0,
        ],
    )?;
    write_header(
        worksheet,
        &[
            "Username",
            "Auth DB",
            "Client App",
            "Client IPs",
            "Slow Queries",
            "COLLSCANs",
            "Total Time (s)",
            "Avg (ms)",
            "P95 (ms)",
            "Max (ms)",
            "Docs Examined",
            "Scan Ratio",
            "Auth Success",
            "Auth Fails",
            "First Active",
            "Last Active",
        ],
    )?;

    let text = text_format();
    let number = number_format();
    for (index, user) in result.users.iter().enumerate() {
        let row = index as u32 + 1;
        for (column, value) in [
            (0_u16, user.user_name.clone()),
            (1, blank_to_na(&user.auth_db)),
            (2, blank_to_na(&user.app_name)),
            (3, blank_to_na(&user.client_ips.join(", "))),
            (14, blank_to_na(&user.first_active)),
            (15, blank_to_na(&user.last_active)),
        ] {
            worksheet
                .write_string_with_format(row, column, &value, &text)
                .map_err(|error| error.to_string())?;
        }
        for (column, value) in [
            (4, user.slow_query_count as f64),
            (5, user.collscan_count as f64),
            (6, round_2(user.total_duration_ms as f64 / 1000.0)),
            (7, user.avg_duration_ms),
            (8, user.p95_duration_ms as f64),
            (9, user.max_duration_ms as f64),
            (10, user.total_docs_examined as f64),
            (11, user.scan_ratio),
            (12, user.auth_success_count as f64),
        ] {
            worksheet
                .write_number_with_format(row, column, value, &number)
                .map_err(|error| error.to_string())?;
        }
        worksheet
            .write_number_with_format(row, 13, user.auth_fail_count as f64, &number)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn blank_to_na(value: &str) -> String {
    if value.is_empty() {
        "N/A".to_string()
    } else {
        value.to_string()
    }
}

/// `Math.round(x * 100) / 100`.
fn round_2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// `Math.round(x * 10) / 10`.
fn round_to_tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

/// JS `Number` printing for a float column.
fn js_number(value: f64) -> String {
    format!("{value}")
}

/// `Math.round(x * 100) / 100` rendered the way Excel stores it.
fn js_round2(value: f64) -> String {
    js_number(round_2(value))
}

/// The Mongo export file name (reference default is `mongo-analysis.xlsx`).
fn export_file_name() -> String {
    let ts = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    format!("mongo-analysis-{ts}.xlsx")
}

/// Opens the Excel save dialog (or `PM2_ANALYZER_EXPORT_DIR` for tests).
pub fn save_target() -> Option<std::path::PathBuf> {
    let name = export_file_name();
    if let Ok(dir) = std::env::var("PM2_ANALYZER_EXPORT_DIR") {
        return Some(std::path::PathBuf::from(dir).join(name));
    }
    rfd::FileDialog::new()
        .set_file_name(&name)
        .add_filter("Excel Workbook", &["xlsx"])
        .save_file()
}

/// Writes the workbook for `result` to `path` (app + parity runner).
pub fn write_export(
    result: &MongoAggregationResult,
    source_label: Option<&str>,
    path: &std::path::Path,
) -> Result<(), String> {
    let mut workbook = build_workbook(result, source_label)?;
    workbook.save(path).map_err(|error| error.to_string())
}
