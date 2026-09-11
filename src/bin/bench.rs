//! Native engine benchmark: parse a log file and report wall time / throughput.
//!
//! Usage: `cargo run --release --bin bench -- <path-to-log-file>`

use std::path::PathBuf;
use std::time::Instant;

use pm2_log_analyzer::core::models::{NormalizeMode, ParseOptions};
use pm2_log_analyzer::core::pm2::{parse_paths, JobControl};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = args.first() else {
        eprintln!("usage: bench <path-to-log-file>");
        std::process::exit(2);
    };
    let path = PathBuf::from(path);
    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    let control = JobControl::new(size);
    let t0 = Instant::now();
    let mut kernel = match parse_paths(&[path], NormalizeMode::CollapseIds, &control) {
        Ok(k) => k,
        Err(err) => {
            eprintln!("parse failed: {err}");
            std::process::exit(1);
        }
    };
    let parse_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let t1 = Instant::now();
    let result = kernel.reaggregate(&ParseOptions::default());
    let reagg_ms = t1.elapsed().as_secs_f64() * 1000.0;

    println!("file          : {}", kernel.file_size);
    println!("shards        : {}", kernel.shard_count());
    println!("parse wall    : {parse_ms:.1} ms");
    if parse_ms > 0.0 {
        println!(
            "throughput    : {:.0} MB/s",
            (kernel.file_size as f64 / 1_048_576.0) / (parse_ms / 1000.0)
        );
    }
    println!("first reagg   : {reagg_ms:.1} ms");
    println!("matched hits  : {}", result.summary.matched);
    println!("unmatched     : {}", result.summary.unmatched);
    println!("endpoints     : {}", result.api.len());
    println!("cron jobs     : {}", result.cron.len());
    println!("p95           : {:.1} ms", result.summary.p95_ms);
}
