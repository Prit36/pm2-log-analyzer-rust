use pm2_log_analyzer::core::aggregator::parse_log_buffer;
use pm2_log_analyzer::core::mmap::MmapReader;
use pm2_log_analyzer::core::models::{
    FilterOptions, Method, PathNormMode, StatusFamily,
};
use pm2_log_analyzer::ui::endpoint_table::{SortColumn, SortDirection, TableSortState};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let default_file = "c:/Users/My_Home/Desktop/projects/pm2-logs/pm2-log-analyzer/test_data/api-out-5gb.log";
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        default_file
    };

    println!("================================================================");
    println!("      PM2 LOG ANALYZER NATIVE RUST BENCHMARK SUITE");
    println!("================================================================");
    println!("Target File: {}", file_path);

    let path = PathBuf::from(file_path);
    if !path.exists() {
        eprintln!("Error: File not found at {}", file_path);
        std::process::exit(1);
    }

    // 1. Mmap File Open
    let t_mmap_start = Instant::now();
    let reader = MmapReader::open(&path).expect("Failed to mmap file");
    let mmap_ms = t_mmap_start.elapsed().as_millis();
    let file_size_bytes = reader.len() as u64;
    let file_size_mb = file_size_bytes as f64 / (1024.0 * 1024.0);
    let file_size_gb = file_size_mb / 1024.0;

    println!("[1/6] Mmap File Open: {} ms (Size: {:.2} GB / {:.2} MB)", mmap_ms, file_size_gb, file_size_mb);

    // 2. Parallel Parse Wall Time
    println!("\n[2/6] Parsing log buffer across {} Rayon threads...", rayon::current_num_threads());
    let t_parse_start = Instant::now();
    let engine = parse_log_buffer(reader.as_slice());
    let parse_elapsed = t_parse_start.elapsed();
    let parse_sec = parse_elapsed.as_secs_f64();
    let parse_ms = parse_elapsed.as_millis();
    let throughput_mbps = file_size_mb / parse_sec;

    println!("  - Parse Wall Time: {:.3} sec ({} ms)", parse_sec, parse_ms);
    println!("  - Throughput: {:.2} MB/s", throughput_mbps);
    println!("  - Total Lines Parsed: {}", engine.total_lines);
    println!("  - Matched HTTP Entries: {}", engine.entries.len());
    println!("  - Unique Paths Interned: {}", engine.path_off.len());
    println!("  - Cron Events: {}", engine.cron_events.len());
    println!("  - Unmatched Lines: {}", engine.unmatched_count);

    // 3. START TIMING IMMEDIATELY AS SOON AS PARSE FINISHES (UI PREPARATION & INITIAL SUMMARY AGGREGATION)
    let t_parse_finish = Instant::now();

    let default_filters = FilterOptions::default();
    let summary = engine.aggregate(file_size_bytes, parse_ms as u64, &default_filters);

    // Initial Endpoint Sort for Table Rendering
    let sort_state = TableSortState {
        column: SortColumn::Calls,
        direction: SortDirection::Descending,
    };
    let mut sorted_endpoints = summary.endpoints.clone();
    sorted_endpoints.sort_by(|a, b| b.total_calls.cmp(&a.total_calls));

    let ui_prep_elapsed = t_parse_finish.elapsed();
    let ui_prep_ms = ui_prep_elapsed.as_millis();
    let ui_prep_sec = ui_prep_elapsed.as_secs_f64();

    println!("\n[3/6] UI Preparation & Display Timing (Started IMMEDIATELY after parse finish):");
    println!("  - Time from Parse Finish -> First UI Frame Ready: {} ms ({:.3} s)", ui_prep_ms, ui_prep_sec);
    println!("  - Total Endpoints Prepared for Render: {}", sorted_endpoints.len());
    println!("  - Overall Latency Percentiles: p50={:.2}ms, p90={:.2}ms, p95={:.2}ms, p99={:.2}ms",
        summary.overall_p50_ms, summary.overall_p90_ms, summary.overall_p95_ms, summary.overall_p99_ms);
    println!("  - Error Count: {} ({:.2}%)", summary.overall_error_count, summary.overall_error_rate);

    // 4. Combined Total Load + Display Wall Time
    let total_wall_sec = parse_sec + ui_prep_sec;
    println!("\n[4/6] End-to-End Time (Parse + UI Initial Render):");
    println!("  - Parse Time: {:.3} s", parse_sec);
    println!("  - UI Display Prep Time: {:.3} s", ui_prep_sec);
    println!("  - Total Time to Render UI: {:.3} s", total_wall_sec);

    // 5. Interactive UI Re-aggregation Timings (Filter Toggles)
    println!("\n[5/6] Interactive UI Re-aggregation Timings (Filter Toggles):");

    let t_filter_search = Instant::now();
    let search_filters = FilterOptions {
        search_query: "auth".to_string(),
        ..FilterOptions::default()
    };
    let search_summary = engine.aggregate(file_size_bytes, parse_ms as u64, &search_filters);
    let search_ms = t_filter_search.elapsed().as_millis();
    println!("  - Filter by Search Query (\"auth\"): {} ms (Endpoints: {})", search_ms, search_summary.endpoints.len());

    let t_filter_method = Instant::now();
    let method_filters = FilterOptions {
        method: Some(Method::Post),
        ..FilterOptions::default()
    };
    let method_summary = engine.aggregate(file_size_bytes, parse_ms as u64, &method_filters);
    let method_ms = t_filter_method.elapsed().as_millis();
    println!("  - Filter by Method (POST): {} ms (Matched requests: {})", method_ms, method_summary.matched_http_requests);

    let t_filter_status = Instant::now();
    let status_filters = FilterOptions {
        status_family: StatusFamily::ClientError,
        ..FilterOptions::default()
    };
    let status_summary = engine.aggregate(file_size_bytes, parse_ms as u64, &status_filters);
    let status_ms = t_filter_status.elapsed().as_millis();
    println!("  - Filter by Status (4xx Errors Only): {} ms (Error requests: {})", status_ms, status_summary.matched_http_requests);

    let t_norm_raw = Instant::now();
    let raw_norm_filters = FilterOptions {
        path_norm_mode: PathNormMode::Raw,
        ..FilterOptions::default()
    };
    let raw_summary = engine.aggregate(file_size_bytes, parse_ms as u64, &raw_norm_filters);
    let raw_ms = t_norm_raw.elapsed().as_millis();
    println!("  - Switch Path Norm Mode (Raw Path): {} ms (Endpoints: {})", raw_ms, raw_summary.endpoints.len());

    // 6. UI Table Sorting Performance
    println!("\n[6/6] UI Table Sorting Timings across 8 Sort Columns:");
    let cols = [
        ("Path", SortColumn::Path),
        ("Method", SortColumn::Method),
        ("Calls", SortColumn::Calls),
        ("ErrorRate", SortColumn::ErrorRate),
        ("AvgDuration", SortColumn::AvgDuration),
        ("P50", SortColumn::P50),
        ("P95", SortColumn::P95),
        ("P99", SortColumn::P99),
    ];

    let mut endpoints_to_sort = summary.endpoints.clone();
    let t_sort_all = Instant::now();
    for (name, col) in &cols {
        let t_sort = Instant::now();
        let sort_state = TableSortState {
            column: *col,
            direction: SortDirection::Descending,
        };
        endpoints_to_sort.sort_by(|a, b| {
            let cmp = match sort_state.column {
                SortColumn::Path => a.path.cmp(&b.path),
                SortColumn::Method => a.method.as_str().cmp(b.method.as_str()),
                SortColumn::Calls => a.total_calls.cmp(&b.total_calls),
                SortColumn::ErrorRate => a.error_rate().total_cmp(&b.error_rate()),
                SortColumn::AvgDuration => a.avg_duration_ms().total_cmp(&b.avg_duration_ms()),
                SortColumn::P50 => a.p50_ms.total_cmp(&b.p50_ms),
                SortColumn::P95 => a.p95_ms.total_cmp(&b.p95_ms),
                SortColumn::P99 => a.p99_ms.total_cmp(&b.p99_ms),
            };
            cmp.reverse()
        });
        println!("  - Sort by {:<12}: {:.3} ms", name, t_sort.elapsed().as_secs_f64() * 1000.0);
    }
    let total_sort_ms = t_sort_all.elapsed().as_secs_f64() * 1000.0;
    println!("  - Total 8-Column Table Sort Overhead: {:.3} ms", total_sort_ms);

    println!("\n================================================================");
    println!("                     BENCHMARK SUMMARY");
    println!("================================================================");
    println!("Dataset: {:.2} GB log file ({})", file_size_gb, file_path);
    println!("1. Parse Wall Time:                      {:.3} s ({:.2} MB/s)", parse_sec, throughput_mbps);
    println!("2. Time from Parse Finish to UI Display: {:.3} s ({} ms)", ui_prep_sec, ui_prep_ms);
    println!("3. Total Time to Render UI:              {:.3} s", total_wall_sec);
    println!("4. Average Filter Toggle Response:       {:.2} ms", (search_ms + method_ms + status_ms + raw_ms) as f64 / 4.0);
    println!("================================================================");
}
