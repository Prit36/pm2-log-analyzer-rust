//! Excel export runner — builds the same workbook the app produces, for parity comparison
//! against the reference browser export.
//!
//! Usage: export_runner <log-file> <out.xlsx> [--charts-dir DIR]

use std::path::PathBuf;

use pm2_log_analyzer::core::models::{NormalizeMode, ParseOptions};
use pm2_log_analyzer::core::pm2::{parse_paths, JobControl};
use pm2_log_analyzer::store::AnalysisFilters;
use pm2_log_analyzer::utils::export_spreadsheet::{write_export, ExportInput};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: export_runner <log-file> <out.xlsx> [--charts-dir DIR]");
        std::process::exit(2);
    }
    let log_path = PathBuf::from(&args[0]);
    let out_path = PathBuf::from(&args[1]);
    let charts_dir = args
        .iter()
        .position(|a| a == "--charts-dir")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);

    let control = JobControl::new(0);
    let mut kernel = parse_paths(&[log_path.clone()], NormalizeMode::CollapseIds, &control)
        .expect("parse failed");
    let opts = ParseOptions::default();
    let result = kernel.reaggregate(&opts);

    let filters = AnalysisFilters::default();
    let source_label = log_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string());
    let input = ExportInput {
        api: &result.api,
        cron: &result.cron,
        hourly: &result.hourly_stats,
        daily: &result.daily_stats,
        summary: Some(&result.summary),
        filters: &filters,
        source_label: source_label.as_deref(),
    };
    write_export(&input, &out_path).expect("export failed");
    println!(
        "wrote {} ({} endpoints, {} cron jobs, {} hourly, {} daily)",
        out_path.display(),
        result.api.len(),
        result.cron.len(),
        result.hourly_stats.len(),
        result.daily_stats.len()
    );

    if let Some(dir) = charts_dir {
        use pm2_log_analyzer::utils::chart_renderer as cr;
        std::fs::create_dir_all(&dir).expect("charts dir");
        let charts: [(&str, String); 4] = [
            ("time_vs_latency", cr::render_time_vs_latency(&result.hourly_stats)),
            ("hourly_volume", cr::render_hourly_volume(&result.hourly_stats)),
            ("distribution", cr::render_distribution(&result.api)),
            ("daily_trend", cr::render_daily_trend(&result.daily_stats)),
        ];
        for (name, svg) in charts {
            std::fs::write(dir.join(format!("{name}.svg")), &svg).expect("svg write");
            if let Some(bytes) = cr::svg_to_png(&svg) {
                std::fs::write(dir.join(format!("{name}.png")), bytes).expect("png write");
            }
        }
    }
}
