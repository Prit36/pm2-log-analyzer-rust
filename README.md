# PM2 Log Analyzer Native

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Rust Edition](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![GUI Framework](https://img.shields.io/badge/gui-iced--0.14-blue.svg)](https://iced.rs)
[![Target OS](https://img.shields.io/badge/platform-Windows%20x64-0078D6.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A high-throughput, native Windows desktop application for analyzing large-scale PM2 process log files. Built in Rust 2024 using `iced` and parallel memory-mapped I/O, it parses multi-gigabyte log files at **~3.3 GB/s** throughput and reaches an interactive latency dashboard in **~1.7 seconds for a 5.22 GB log file (65 Million lines)** — about **3.5x faster than the Wasm/React reference** on the same machine, at ~0.8 GB RSS.

---

## Key Features

- **High-Throughput Parallel Engine**: Memory-mapped I/O (`memmap2`) with per-shard segment mapping, zero-copy `feed_slice` ingest, one shard per logical core (`rayon`) and lock-free thread allocation (`mimalloc`) yielding up to **3,287 MB/s throughput**.
- **Compact Columnar Storage**: Zero per-line heap allocations (`PackedEntry` 16-byte representation with path string interning).
- **Logarithmic Histogram Sketches**: Relative-error histogram sketches (`RelHist`, $\gamma \approx 1.0202$) for bounded $O(1)$ memory quantile estimation (p50, p90, p95, p99) without storing raw duration arrays.
- **Strict Format Parity**: Full support for PM2 Format A (`[TIMESTAMP] METHOD PATH STATUS DURATION ms - BYTES`), Format B (`DURATION ms METHOD PATH`), Cron events (`[cron]`), ISO timestamps, and inline ANSI escape sequences.
- **Sub-Frame Interactive Filtering**: Filter by search queries, HTTP methods, or HTTP status families with response times of **< 20 ms** on 65M line datasets.

---

## Performance Benchmarks

All benchmarks were conducted on **Windows 11 x64 (12-Thread CPU)** using the release build (`target/release/bench.exe`).

### 1. Large Dataset: 5.22 GB Log File (`api-out-5gb.log` / 65,083,800 Lines)

Same corpus and same aggregated numbers as the browser reference (20,315,200 matched /
33,123,400 unmatched / 5,416 endpoints / p95 2416.9 ms). The reference column is the
latest `bench:mongo`-family browser run recorded in `../pm2-log-analyzer/scripts/bench/history.json`
(`native-compare`, 2026-09-11); the native column is `target/release/bench.exe` on the same
machine (12-thread CPU, warm page cache, 12 shards).

| Metric | Wasm engine (browser + 4 workers) | Native Rust engine | Delta |
| :--- | :--- | :--- | :--- |
| **Parse wall time** | 5.82 s | **1.63 s** (1.63–1.72 s over 5 runs) | **3.6x faster** |
| **Throughput** | 919 MB/s | **3,287 MB/s** | **3.6x** |
| **Peak RSS** | 2,871 MB | **~805 MB** | **3.6x less** |
| **First re-aggregation** | 110.7 ms | **47 ms** | 2.4x |
| **Matched / unmatched / endpoints / p95** | 20,315,200 / 33,123,400 / 5,416 / 2416.9 ms | identical | **100% parity** |

### 2. Medium / small datasets

| File | Lines | Parse wall | Throughput |
| :--- | ---: | ---: | ---: |
| `api-out-500mb.log` | 6,508,380 | **214 ms** | **2,503 MB/s** |
| `api-out.log` | 650,838 | **26 ms** | **2,032 MB/s** |

Re-aggregation (client-side filter/sort changes) stays in the tens of milliseconds:
34 ms for the default collapse-ids view, 6–11 ms for status/min-ms/date filters, and ~1.2 s
for the raw-path (exact) view over 270k unique endpoints.

Benchmarks are reported by `cargo run --release --bin bench -- <file>`: parse wall covers
opening, full parse, summary/hourly/daily stats, and the pre-aggregation pass; no work is
excluded or deferred.

---

## Architecture & Engineering Design

```
+-----------------------------------------------------------------------------------+
|                                 Input File                                        |
|                          (memmap2 - 5.22 GB Log File)                             |
+--------------------------------─────────+-----------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
|                        Parallel SIMD & Byte Parser                                |
|             - Rayon parallel chunk execution (8 MB cache-aligned)                 |
|             - Lock-free memory allocations (mimalloc)                             |
|             - Zero-copy slice parsing & fast-path inline method matching          |
+--------------------------------─────────+-----------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
|                          Columnar Storage Engine                                  |
|             - 16-byte PackedEntry struct (path_id, duration, status, method)      |
|             - Unique path byte string interning                                   |
|             - Pre-computed active path normalization tables                       |
+--------------------------------─────────+-----------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
|                      Parallel Dense Array Aggregator                              |
|             - Direct array indexing: slot_idx = (norm_id << 3) | method            |
|             - Relative-error histogram sketch (RelHist, gamma ≈ 1.0202)            |
|             - Sub-20ms multi-threaded re-aggregation                              |
+--------------------------------─────────+-----------------------------------------+
                                          |
                                          v
+-----------------------------------------------------------------------------------+
|                           Native User Interface                                   |
|             - Native GPU UI (iced 0.14, wgpu)                                     |
|             - Interactive KPI cards, endpoint tables, & latency charts            |
+-----------------------------------------------------------------------------------+
```

### Memory Layout & Optimizations

1. **Zero-Allocation Log Processing**: During line parsing, path strings are interned once into a continuous byte array (`path_bytes: Vec<u8>`). Entries are stored as compact `PackedEntry` structs:
   ```rust
   #[repr(C, align(16))]
   pub struct PackedEntry {
       pub path_id: u32,
       pub duration: f32,
       pub status: u16,
       pub method: u8,
       pub _pad: u8,
   }
   ```
2. **$O(1)$ Bounded Quantile Estimation**: Percentile latencies (p50, p90, p95, p99) are computed using logarithmic bucket sketches (`RelHist`) with a pre-computed inverse log scale, avoiding sorting millions of floats in memory.
3. **Zero-Hash Aggregation**: Aggregation uses pre-computed normalized path mapping tables to index directly into dense contiguous vectors (`(norm_id << 3) | method`), bypassing HashMap hash calculations during log scans.

---

## Building and Installation

### Prerequisites

- Rust 2024 Edition compiler toolchain (`cargo`)
- Windows OS (x86_64)

### Building from Source

To compile the release executable:

```bash
cargo build --release
```

The compiled binary will be placed at:
```
target/release/pm2-log-analyzer.exe
```

### Running Benchmarks

To execute the benchmark suite against a target log file:

```bash
cargo run --release --bin bench -- "path/to/logfile.log"
```

### Running Unit Tests

To run all unit tests for line parsing, path normalization, and quantile sketches:

```bash
cargo test
```

---

## License

Distributed under the MIT License. See [LICENSE](LICENSE) for more information.
