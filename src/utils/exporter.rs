use crate::core::models::{CronStats, EndpointStats, LogAnalysisSummary, Method};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError};
use std::fs::File;
use std::io::{BufWriter, Write};

const MS_FMT: &str = "#,##0.0\" ms\"";

fn method_badge_colors(m: &Method) -> (u32, u32) {
    match m {
        Method::Get => (0xDBEAFE, 0x1E40AF),
        Method::Post => (0xD1FAE5, 0x065F46),
        _ => (0xFEF3C7, 0x92400E),
    }
}

fn calibri(size: f64) -> Format {
    Format::new().set_font_name("Calibri").set_font_size(size)
}

fn build_api_sheet(ws: &mut rust_xlsxwriter::worksheet::Worksheet, eps: &[EndpointStats]) -> Result<(), XlsxError> {
    // Column widths: Method, Endpoint, Count, Avg, p50, p90, p95, p99, Errors
    for (i, w) in [10.0, 48.0, 10.0, 12.0, 12.0, 12.0, 12.0, 12.0, 10.0].iter().enumerate() {
        ws.set_column_width(i as u16, *w)?;
    }

    let title = calibri(16.0).set_bold().set_font_color(Color::RGB(0x0F172A));
    let meta = calibri(11.0).set_font_color(Color::RGB(0x475569));
    let header = calibri(11.0)
        .set_bold()
        .set_font_color(Color::RGB(0xFFFFFF))
        .set_background_color(Color::RGB(0x1F4E78))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let body = calibri(11.0);
    let ms_body = calibri(11.0).set_num_format(MS_FMT);
    let total = calibri(11.0).set_bold().set_border_top(FormatBorder::Thin);

    // Title row
    ws.merge_range(0, 0, 0, 8, "PM2 Log Analyzer — API Endpoints", &title)?;
    let meta_str = format!(
        "Generated: {}  |  Endpoints: {}  |  Sorted by: calls (desc)",
        now_timestamp(),
        eps.len()
    );
    ws.merge_range(1, 0, 1, 8, &meta_str, &meta)?;

    let headers = ["Method", "Endpoint", "Count", "Avg", "p50", "p90", "p95", "p99", "Errors"];
    for (c, h) in headers.iter().enumerate() {
        ws.write_string_with_format(3, c as u16, *h, &header)?;
    }

    let mut total_calls = 0u64;
    let mut total_errors = 0u64;
    for (i, ep) in eps.iter().enumerate() {
        let r = 4 + i as u32;
        let (bg, fg) = method_badge_colors(&ep.method);
        let badge = calibri(11.0)
            .set_bold()
            .set_font_color(Color::RGB(fg))
            .set_background_color(Color::RGB(bg))
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter);
        ws.write_string_with_format(r, 0, ep.method.as_str(), &badge)?;
        ws.write_string_with_format(r, 1, &ep.path, &body)?;
        ws.write_number_with_format(r, 2, ep.total_calls as f64, &body)?;
        ws.write_number_with_format(r, 3, ep.avg_duration_ms(), &ms_body)?;
        ws.write_number_with_format(r, 4, ep.p50_ms as f64, &ms_body)?;
        ws.write_number_with_format(r, 5, ep.p90_ms as f64, &ms_body)?;
        ws.write_number_with_format(r, 6, ep.p95_ms as f64, &ms_body)?;
        ws.write_number_with_format(r, 7, ep.p99_ms as f64, &ms_body)?;
        ws.write_number_with_format(r, 8, ep.error_calls as f64, &body)?;
        total_calls += ep.total_calls;
        total_errors += ep.error_calls;
    }

    if !eps.is_empty() {
        let tr = 4 + eps.len() as u32;
        ws.write_string_with_format(tr, 0, "Total", &total)?;
        ws.write_number_with_format(tr, 2, total_calls as f64, &total)?;
        ws.write_number_with_format(tr, 8, total_errors as f64, &total)?;
        ws.autofilter(3, 0, tr - 1, 8)?;
    }

    ws.set_freeze_panes(4, 0)?;
    Ok(())
}

fn build_cron_sheet(ws: &mut rust_xlsxwriter::worksheet::Worksheet, crons: &[CronStats]) -> Result<(), XlsxError> {
    // Column widths: Cron Job, Runs, Starts, Fails, Avg, Min, Max, Last Run
    for (i, w) in [40.0, 10.0, 10.0, 10.0, 12.0, 12.0, 12.0, 22.0].iter().enumerate() {
        ws.set_column_width(i as u16, *w)?;
    }

    let title = calibri(16.0).set_bold().set_font_color(Color::RGB(0x0F172A));
    let meta = calibri(11.0).set_font_color(Color::RGB(0x475569));
    let header = calibri(11.0)
        .set_bold()
        .set_font_color(Color::RGB(0xFFFFFF))
        .set_background_color(Color::RGB(0x1F4E78))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let body = calibri(11.0);
    let ms_body = calibri(11.0).set_num_format(MS_FMT);
    let total = calibri(11.0).set_bold().set_border_top(FormatBorder::Thin);

    ws.merge_range(0, 0, 0, 7, "PM2 Log Analyzer — Cron Jobs", &title)?;
    let meta_str = format!("Generated: {}  |  Jobs: {}", now_timestamp(), crons.len());
    ws.merge_range(1, 0, 1, 7, &meta_str, &meta)?;

    let headers = ["Cron Job", "Runs", "Starts", "Fails", "Avg", "Min", "Max", "Last Run"];
    for (c, h) in headers.iter().enumerate() {
        ws.write_string_with_format(3, c as u16, *h, &header)?;
    }

    let mut total_runs = 0u64;
    let mut total_fails = 0u64;
    for (i, cj) in crons.iter().enumerate() {
        let r = 4 + i as u32;
        ws.write_string_with_format(r, 0, &cj.name, &body)?;
        ws.write_number_with_format(r, 1, cj.total_runs as f64, &body)?;
        ws.write_number_with_format(r, 2, cj.total_success as f64, &body)?;
        ws.write_number_with_format(r, 3, cj.total_failures as f64, &body)?;
        ws.write_number_with_format(r, 4, cj.avg_duration_ms, &ms_body)?;
        ws.write_number_with_format(r, 5, cj.min_duration_ms as f64, &ms_body)?;
        ws.write_number_with_format(r, 6, cj.max_duration_ms as f64, &ms_body)?;
        ws.write_string_with_format(r, 7, if cj.last_status.is_empty() { "-" } else { &cj.last_status }, &body)?;
        total_runs += cj.total_runs;
        total_fails += cj.total_failures;
    }

    if !crons.is_empty() {
        let tr = 4 + crons.len() as u32;
        ws.write_string_with_format(tr, 0, "Total", &total)?;
        ws.write_number_with_format(tr, 1, total_runs as f64, &total)?;
        ws.write_number_with_format(tr, 3, total_fails as f64, &total)?;
        ws.autofilter(3, 0, tr - 1, 7)?;
    }

    ws.set_freeze_panes(4, 0)?;
    Ok(())
}

fn now_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let rem = secs % 86400;
    // 1970-01-01 was a Thursday; derive a simple yyyy-mm-dd hh:mm:ss (UTC).
    let (y, m, d) = civil_from_days(days as i64);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y,
        m,
        d,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

pub fn export_to_excel(summary: &LogAnalysisSummary, path: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let mut api_ws = workbook.add_worksheet();
    api_ws.set_name("API Endpoints")?;
    build_api_sheet(&mut api_ws, &summary.endpoints)?;

    if !summary.cron_jobs.is_empty() {
        let mut cron_ws = workbook.add_worksheet();
        cron_ws.set_name("Cron Jobs")?;
        build_cron_sheet(&mut cron_ws, &summary.cron_jobs)?;
    }

    workbook.save(path)?;
    Ok(())
}

pub fn export_to_csv(summary: &LogAnalysisSummary, path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "Method,Endpoint,Calls,ErrorCalls,ErrorRate(%),AvgDuration(ms),p50(ms),p95(ms),p99(ms)")?;

    for ep in &summary.endpoints {
        writeln!(
            writer,
            "\"{}\",\"{}\",{},{},{:.2},{:.2},{:.2},{:.2},{:.2}",
            ep.method.as_str(),
            ep.path,
            ep.total_calls,
            ep.error_calls,
            ep.error_rate(),
            ep.avg_duration_ms(),
            ep.p50_ms,
            ep.p95_ms,
            ep.p99_ms
        )?;
    }
    writer.flush()?;
    Ok(())
}

pub fn export_to_json(summary: &LogAnalysisSummary, path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, summary)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_summary() -> LogAnalysisSummary {
        let mut s = LogAnalysisSummary::default();
        s.matched_http_requests = 3;
        s.endpoints = vec![
            EndpointStats {
                path: "/api/users".into(),
                method: Method::Get,
                total_calls: 100,
                error_calls: 2,
                total_duration_ms: 5000.0,
                min_duration_ms: 10.0,
                max_duration_ms: 120.0,
                p50_ms: 40.0,
                p90_ms: 90.0,
                p95_ms: 100.0,
                p99_ms: 110.0,
                status_counts: Default::default(),
            },
            EndpointStats {
                path: "/api/orders".into(),
                method: Method::Post,
                total_calls: 50,
                error_calls: 0,
                total_duration_ms: 4000.0,
                min_duration_ms: 5.0,
                max_duration_ms: 200.0,
                p50_ms: 60.0,
                p90_ms: 150.0,
                p95_ms: 180.0,
                p99_ms: 195.0,
                status_counts: Default::default(),
            },
        ];
        s.cron_jobs = vec![CronStats {
            name: "daily-report".into(),
            total_runs: 30,
            total_success: 28,
            total_failures: 2,
            total_duration_ms: 9000.0,
            avg_duration_ms: 300.0,
            min_duration_ms: 250.0,
            max_duration_ms: 400.0,
            last_status: "completed".into(),
        }];
        s
    }

    #[test]
    fn export_to_excel_produces_valid_workbook() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("pm2_test_{}.xlsx", std::process::id()));
        export_to_excel(&sample_summary(), path.to_str().unwrap()).unwrap();

        let file = std::fs::File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Required xlsx container parts exist.
        for entry_name in [
            "[Content_Types].xml",
            "_rels/.rels",
            "xl/workbook.xml",
            "xl/worksheets/sheet1.xml",
            "xl/worksheets/sheet2.xml",
        ] {
            assert!(
                archive.by_name(entry_name).is_ok(),
                "missing container part: {}",
                entry_name
            );
        }

        let workbook_xml = {
            let mut s = String::new();
            use std::io::Read;
            archive.by_name("xl/workbook.xml").unwrap().read_to_string(&mut s).unwrap();
            s
        };
        assert!(workbook_xml.contains("API Endpoints"), "missing API sheet name");
        assert!(workbook_xml.contains("Cron Jobs"), "missing Cron sheet name");

        // The ms number format is registered in the workbook styles.
        let styles = {
            let mut s = String::new();
            use std::io::Read;
            archive.by_name("xl/styles.xml").unwrap().read_to_string(&mut s).unwrap();
            s
        };
        assert!(
            styles.contains("ms") && styles.contains("#,##0.0"),
            "missing ms number format in styles.xml"
        );

        // Strings are stored in the shared string table; text cells reference
        // entries there, so verify the data lives in the workbook.
        let shared = {
            let mut s = String::new();
            use std::io::Read;
            archive.by_name("xl/sharedStrings.xml").unwrap().read_to_string(&mut s).unwrap();
            s
        };
        assert!(shared.contains("/api/users"), "missing endpoint data");
        assert!(shared.contains("Total"), "missing totals row");
        assert!(shared.contains("PM2 Log Analyzer — API Endpoints"), "missing title");

        std::fs::remove_file(&path).ok();
    }
}
