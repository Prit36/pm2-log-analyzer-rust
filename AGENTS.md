# AGENTS.md

Operating instructions for coding agents working on PM2 Log Analyzer Native.

## Project Context
- Language & Edition: Rust 2024 (`edition = "2024"`)
- GUI Framework: `egui` 0.31 + `eframe` 0.31 + `egui_plot` 0.31
- Engine & Allocator:
  - Memory mapping (`memmap2`) with zero-copy byte slice parsing (`memchr`)
  - Multi-threaded parallel chunk execution (`rayon`)
  - Lock-free memory allocator (`mimalloc`) & fast hashing (`rapidhash`)
  - Compact columnar storage (`PackedEntry` 16-byte struct) & string interning (`path_norm.rs`)
  - Logarithmic relative-error histogram sketch (`RelHist`) for $O(1)$ memory quantile estimation (p50, p90, p95, p99)
- Architecture Breakdown:
  - `src/lib.rs` / `src/main.rs`: Application entry point and window orchestration
  - `src/core/`: Log parser (`log_parser.rs`), parallel aggregator (`aggregator.rs`), relative histogram (`relhist.rs`), path normalization (`path_norm.rs`), mmap handler (`mmap.rs`), models (`models.rs`)
  - `src/ui/`: `egui` components (KPI cards, endpoint table, cron table, latency charts, raw viewer, dark theme)
  - `src/utils/`: Data exporters (`exporter.rs`) for CSV/JSON reporting
  - `src/bin/bench.rs`: Benchmark engine binary
- Platform Target: Pure Native Windows `.exe` (`x86_64-pc-windows-msvc` / `x86_64-pc-windows-gnu`)

## Building, Testing, and Benchmarking
- Check compilation: `cargo check`
- Run unit tests: `cargo test`
- Run GUI app (debug): `cargo run`
- Run GUI app (release): `cargo run --release`
- Build release executable: `cargo build --release` (Produces `target/release/pm2-log-analyzer.exe`)
- Run benchmark engine: `cargo run --release --bin bench -- <path-to-log-file>` (Produces `target/release/bench.exe`)

