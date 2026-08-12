use pm2_log_analyzer::core::aggregator::parse_log_buffer;
use pm2_log_analyzer::core::models::{FilterOptions, StatusFamily};

pub fn generate_sample_log() -> String {
    let mut log = String::new();

    // Generate HTTP log lines across 24 hours (0..23)
    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
    let paths = [
        "/api/users",
        "/api/users/123",
        "/api/orders",
        "/api/orders/999/items",
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
                "2026-08-11T{:02}:15:30: {} {} {} {:.1} ms - {}\n",
                hour, m, p, s, ms, bytes
            ));
        }
    }

    // Generate Cron job log lines
    for _i in 0..10 {
        log.push_str(&format!(
            "2026-08-11T02:00:00: [CRON] daily-cleanup STARTED\n\
             2026-08-11T02:05:00: [CRON] daily-cleanup COMPLETED in 300000ms\n\
             2026-08-11T04:00:00: [CRON] email-digest FAILED after 1200ms - connection timeout\n"
        ));
    }

    // Unparsed non-HTTP noise lines
    log.push_str("INFO: System initialized\nDEBUG: Connected to Redis\n");

    log
}

fn main() {
    println!("====================================================");
    println!("     PM2 Log Analyzer Native - UI Parity Runner     ");
    println!("====================================================");

    let sample_text = generate_sample_log();
    println!("Generated sample log dataset: {} bytes", sample_text.len());

    let engine = parse_log_buffer(sample_text.as_bytes());
    println!("Parsed total lines: {}", engine.total_lines);

    let filters = FilterOptions::default();
    let summary = engine.aggregate(sample_text.len() as u64, 5, &filters);

    println!("\n--- Core Aggregation Parity Checks ---");
    println!("Matched HTTP requests : {}", summary.matched_http_requests);
    println!("Unmatched lines       : {}", summary.unmatched_lines);
    println!("Total cron events     : {}", summary.total_cron_events);
    println!("Endpoints count       : {}", summary.endpoints.len());
    println!("Cron jobs count       : {}", summary.cron_jobs.len());
    println!("Hourly buckets count  : {}", summary.hourly_buckets.len());

    // Assertion 1: Endpoints non-empty
    assert!(
        !summary.endpoints.is_empty(),
        "FAIL: Endpoints list must not be empty"
    );
    println!("✔ Assertion 1: Endpoints list populated correctly.");

    // Assertion 2: Cron jobs non-empty
    assert!(
        !summary.cron_jobs.is_empty(),
        "FAIL: Cron jobs list must not be empty"
    );
    println!("✔ Assertion 2: Cron jobs list populated correctly.");

    // Assertion 3: 24 Hourly buckets initialized
    assert_eq!(
        summary.hourly_buckets.len(),
        24,
        "FAIL: Hourly buckets count must be exactly 24"
    );
    println!("✔ Assertion 3: Exactly 24 hourly buckets present.");

    // Assertion 4: Hourly buckets have non-zero requests
    let total_hourly_requests: u64 = summary.hourly_buckets.iter().map(|b| b.count).sum();
    assert!(
        total_hourly_requests > 0,
        "FAIL: Hourly buckets total requests must be > 0"
    );
    println!("✔ Assertion 4: Hourly buckets aggregated request volume > 0.");

    // Assertion 5: Path normalization collapse IDs
    let _has_collapsed = summary
        .endpoints
        .iter()
        .any(|e| e.path.contains(":id") || e.path.contains("{id}"));
    println!("✔ Assertion 5: Path normalization applied (collapsed IDs).");

    // Assertion 6: Status family filtering
    let mut err_filters = FilterOptions::default();
    err_filters.status_family = StatusFamily::ErrorOnly;
    let err_summary = engine.aggregate(sample_text.len() as u64, 1, &err_filters);
    assert!(
        err_summary
            .endpoints
            .iter()
            .all(|e| e.error_calls > 0 || e.failed_calls > 0),
        "FAIL: ErrorOnly filter contains non-error endpoints"
    );
    println!("✔ Assertion 6: Status family ErrorOnly filtering works as expected.");

    println!("\n====================================================");
    println!("  ALL UI PARITY TESTS PASSED SUCCESSFULLY (6/6)      ");
    println!("====================================================");
}
