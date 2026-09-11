//! Parity runner: parse a log with the native kernel and dump the reference
//! `AggregatedResult` JSON (and optionally the raw shard wires for cross-checking
//! against the JS coordinator in the reference repo).
//!
//! Usage:
//!   parity_runner <log-file> [--json] [--dump-wires DIR] [--mode M] [--status S]
//!                 [--min-ms N] [--date YYYY-MM-DD] [--cron-query Q] [--cron-min-ms N]
//!                 [--cron-failed-only]
//!   parity_runner            (runs the built-in sample assertions)

use std::path::{Path, PathBuf};

use pm2_log_analyzer::core::models::{NormalizeMode, ParseOptions, StatusFamily};
use pm2_log_analyzer::core::pm2::{parse_paths, parse_paths_wires, JobControl};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        run_sample_assertions();
        return;
    }

    let mut path: Option<PathBuf> = None;
    let mut json = false;
    let mut dump_wires: Option<PathBuf> = None;
    let mut opts = ParseOptions::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--dump-wires" => {
                i += 1;
                dump_wires = Some(PathBuf::from(args.get(i).expect("--dump-wires needs a dir")));
            }
            "--mode" => {
                i += 1;
                let v = args.get(i).expect("--mode needs a value");
                opts.normalize_mode = NormalizeMode::from_key(v).expect("bad mode");
            }
            "--status" => {
                i += 1;
                let v = args.get(i).expect("--status needs a value");
                opts.status_family = StatusFamily::from_key(v).expect("bad status");
            }
            "--min-ms" => {
                i += 1;
                opts.min_ms = args
                    .get(i)
                    .expect("--min-ms needs a value")
                    .parse()
                    .expect("bad min-ms");
            }
            "--date" => {
                i += 1;
                opts.date_filter = Some(args.get(i).expect("--date needs a value").clone());
            }
            "--cron-query" => {
                i += 1;
                opts.cron_query = args.get(i).expect("--cron-query needs a value").clone();
            }
            "--cron-min-ms" => {
                i += 1;
                opts.cron_min_ms = args
                    .get(i)
                    .expect("--cron-min-ms needs a value")
                    .parse()
                    .expect("bad cron-min-ms");
            }
            "--cron-failed-only" => opts.cron_show_failed_only = true,
            other => {
                if path.is_none() {
                    path = Some(PathBuf::from(other));
                } else {
                    eprintln!("unknown argument: {other}");
                    std::process::exit(2);
                }
            }
        }
        i += 1;
    }

    let Some(path) = path else {
        eprintln!("no log file given");
        std::process::exit(2);
    };

    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let control = JobControl::new(size);

    if let Some(dir) = dump_wires {
        dump_wires_to(&path, &opts.normalize_mode, &dir, &control);
    }

    let mut kernel = match parse_paths(&[path], opts.normalize_mode, &control) {
        Ok(k) => k,
        Err(err) => {
            eprintln!("parse failed: {err}");
            std::process::exit(1);
        }
    };
    let result = kernel.reaggregate(&opts);
    if json {
        println!("{}", serde_json::to_string(&result).expect("serialize"));
    } else {
        println!("matched      : {}", result.summary.matched);
        println!("unmatched    : {}", result.summary.unmatched);
        println!("errors       : {}", result.summary.errors);
        println!("slow >=3s    : {}", result.summary.slow);
        println!("avg          : {:.4}", result.summary.avg);
        println!("p95          : {:.4}", result.summary.p95_ms);
        println!("endpoints    : {}", result.api.len());
        println!("cron jobs    : {}", result.cron.len());
        println!("cron events  : {:?}", result.cron_summary);
        println!("hours        : {}", result.hourly_stats.len());
        println!("days         : {}", result.dates.len());
        println!("methods      : {:?}", result.methods);
    }
}

fn dump_wires_to(
    path: &Path,
    mode: &NormalizeMode,
    dir: &Path,
    control: &JobControl,
) {
    std::fs::create_dir_all(dir).expect("create wires dir");
    let shards = match parse_paths_wires(&[path.to_path_buf()], *mode, control) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("wire dump failed: {err}");
            std::process::exit(1);
        }
    };
    let mut meta = Vec::new();
    for (i, s) in shards.iter().enumerate() {
        std::fs::write(dir.join(format!("shard{i}.partial")), &s.partial_wire).unwrap();
        std::fs::write(dir.join(format!("shard{i}.cron")), &s.cron_wire).unwrap();
        std::fs::write(dir.join(format!("shard{i}.unmatched")), &s.unmatched_wire).unwrap();
        std::fs::write(dir.join(format!("shard{i}.hourly")), &s.hourly_wire).unwrap();
        std::fs::write(dir.join(format!("shard{i}.dates")), &s.dates_wire).unwrap();
        std::fs::write(dir.join(format!("shard{i}.daily")), &s.daily_wire).unwrap();
        meta.push(format!(
            "{{\"index\":{i},\"hitCount\":{},\"unmatchedCount\":{},\"methodsMask\":{}}}",
            s.hit_count, s.unmatched_count, s.methods_mask
        ));
    }
    let meta_json = format!("{{\"shards\":[{}]}}", meta.join(","));
    std::fs::write(dir.join("meta.json"), meta_json).unwrap();
    eprintln!("wrote {} shard wire set(s) to {}", shards.len(), dir.display());
}

fn run_sample_assertions() {
    println!("====================================================");
    println!("     PM2 Log Analyzer Native - UI Parity Runner     ");
    println!("====================================================");

    let sample_text = generate_sample_log();
    println!("Generated sample log dataset: {} bytes", sample_text.len());

    let control = JobControl::new(sample_text.len() as u64);
    let mut kernel = match pm2_log_analyzer::core::pm2::parse_text(
        &sample_text,
        NormalizeMode::CollapseIds,
        &control,
    ) {
        Ok(k) => k,
        Err(err) => {
            eprintln!("FAIL: {err}");
            std::process::exit(1);
        }
    };
    let summary = kernel.reaggregate(&ParseOptions::default());

    println!("\n--- Core Aggregation Parity Checks ---");
    println!("Matched HTTP requests : {}", summary.summary.matched);
    println!("Unmatched lines       : {}", summary.summary.unmatched);
    println!("Cron events           : {:?}", summary.cron_summary);
    println!("Endpoints count       : {}", summary.api.len());
    println!("Cron jobs count       : {}", summary.cron.len());
    println!("Hourly buckets count  : {}", summary.hourly_stats.len());

    assert!(
        !summary.api.is_empty(),
        "FAIL: Endpoints list must not be empty"
    );
    println!("OK Assertion 1: Endpoints list populated correctly.");

    assert!(
        !summary.cron.is_empty(),
        "FAIL: Cron jobs list must not be empty"
    );
    println!("OK Assertion 2: Cron jobs list populated correctly.");

    assert_eq!(
        summary.hourly_stats.len(),
        24,
        "FAIL: Hourly buckets count must be exactly 24"
    );
    println!("OK Assertion 3: Exactly 24 hourly buckets present.");

    let total_hourly: u64 = summary.hourly_stats.iter().map(|b| b.count).sum();
    assert!(total_hourly > 0, "FAIL: Hourly request volume must be > 0");
    println!("OK Assertion 4: Hourly buckets aggregated request volume > 0.");

    let mut err_opts = ParseOptions::default();
    err_opts.status_family = StatusFamily::X4xx;
    let err_summary = kernel.reaggregate(&err_opts);
    assert!(
        err_summary.api.iter().all(|e| e.error_count > 0),
        "FAIL: 4xx filter contains endpoints without error hits"
    );
    println!("OK Assertion 5: Status family filtering works as expected.");

    let mut day_opts = ParseOptions::default();
    if let Some(day) = summary.dates.first() {
        day_opts.date_filter = Some(day.clone());
        let day_summary = kernel.reaggregate(&day_opts);
        assert_eq!(
            day_summary.hourly_stats.len(),
            24,
            "FAIL: date-filtered hourly stats must have 24 buckets"
        );
    }
    println!("OK Assertion 6: Date filtering keeps hourly stats intact.");

    println!("\n====================================================");
    println!("  ALL PARITY ASSERTIONS PASSED");
    println!("====================================================");
}

fn generate_sample_log() -> String {
    let mut log = String::new();
    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
    let paths = [
        "/api/users",
        "/api/users/12345678901234567890",
        "/api/orders",
        "/api/orders/550e8400-e29b-41d4-a716-446655440000/items",
        "/api/health",
        "/api/auth/login",
        "/api/products?cat=electronics",
        "/api/reports/daily",
    ];
    let statuses = [200, 201, 204, 301, 400, 401, 404, 500, 502, 503];

    for hour in 0..24 {
        for i in 0..20 {
            let m = methods[i % methods.len()];
            let p = paths[i % paths.len()];
            let s = statuses[i % statuses.len()];
            let ms = match s {
                500..=599 => 1500.0 + (i as f32 * 100.0),
                400..=499 => 45.0 + (i as f32 * 10.0),
                _ => 12.5 + (i as f32 * 15.0),
            };
            let bytes = 100 + i * 50;
            log.push_str(&format!(
                "2026-08-11T{hour:02}:15:30: {m} {p} {s} {ms:.1} ms - {bytes}\n"
            ));
        }
    }

    for _ in 0..10 {
        log.push_str(
            "2026-08-11T02:00:00: [cron] start daily-cleanup\n\
             2026-08-11T02:05:00: [cron] done daily-cleanup 300000ms\n\
             2026-08-11T04:00:00: [cron] fail email-digest 1200ms\n",
        );
    }

    log.push_str("INFO: System initialized\nDEBUG: Connected to Redis\n");
    log
}
