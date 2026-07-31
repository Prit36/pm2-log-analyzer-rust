# PM2 Log Analyzer Native

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Rust Edition](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![GUI Framework](https://img.shields.io/badge/gui-egui--0.31-blue.svg)](https://github.com/emilk/egui)
[![Target OS](https://img.shields.io/badge/platform-Windows%20x64-0078D6.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A high-throughput, native Windows desktop application for analyzing large-scale PM2 process log files. Built in Rust 2024 using `egui` and parallel memory-mapped I/O, it parses multi-gigabyte log files at **1.76 GB/s** throughput and renders interactive latency analytics dashboards in **< 3.5 seconds for a 5.22 GB log file (65 Million lines)**.

---

## Key Features

- **High-Throughput Parallel Engine**: Memory-mapped I/O (`memmap2`) combined with multi-threaded chunk parsing (`rayon`) and lock-free thread allocation (`mimalloc`) yielding up to **1,767 MB/s throughput**.
- **Compact Columnar Storage**: Zero per-line heap allocations (`PackedEntry` 16-byte representation with path string interning).
- **Logarithmic Histogram Sketches**: Relative-error histogram sketches (`RelHist`, $\gamma \approx 1.0202$) for bounded $O(1)$ memory quantile estimation (p50, p90, p95, p99) without storing raw duration arrays.
- **Strict Format Parity**: Full support for PM2 Format A (`[TIMESTAMP] METHOD PATH STATUS DURATION ms - BYTES`), Format B (`DURATION ms METHOD PATH`), Cron events (`[cron]`), ISO timestamps, and inline ANSI escape sequences.
- **Sub-Frame Interactive Filtering**: Filter by search queries, HTTP methods, or HTTP status families with response times of **< 20 ms** on 65M line datasets.

---

## Performance Benchmarks

All benchmarks were conducted on **Windows 11 x64 (12-Thread CPU)** using the release build (`target/release/bench.exe`).

### 1. Large Dataset: 5.22 GB Log File (`api-out-5gb.log` / 65,083,800 Lines)

| Benchmark Metric | WASM Engine (Browser + Workers) | Native Rust Engine (Phase 1) | Native Rust Engine (Final Release) | Delta vs. WASM |
| :--- | :--- | :--- | :--- | :--- |
| **Parse Wall Time** | 7.85 s – 8.07 s | 3.48 s | **3.03 s** | **2.60x Faster** |
| **Parse Throughput** | ~670 MB/s | 1,540 MB/s | **1,767 MB/s** | **+163% Throughput** |
| **Parse Finish $\to$ UI Render** | ~900 ms | 578 ms | **396 ms** | **2.27x Faster** |
| **End-to-End Time to Interactive UI** | ~8.95 s | 3.84 s | **3.42 s** | **2.61x Faster** |
| **Total Lines Parsed** | 65,083,800 | 65,083,800 | **65,083,800** | **100% Parity** |
| **Matched HTTP Requests** | 37,184,500 | 37,184,500 | **37,184,500** | **100% Parity** |
| **Unmatched Log Lines** | 27,885,900 | 27,885,900 | **27,885,900** | **100% Parity** |
| **Unique Endpoints (`:id`)** | 6,107 | 6,107 | **6,107** | **100% Parity** |
| **Overall p95 Latency** | 1,159.1 ms | 1,153.1 ms | **1,153.1 ms** | **100% Parity** |
| **Search Filter Query ("auth")** | ~400 ms | 3,025 ms | **19 ms** | **Sub-20 ms** |
| **Status Filter Query (4xx)** | ~300 ms | 332 ms | **21 ms** | **Sub-25 ms** |
| **Table Column Sort Overhead** | UI Re-render | 0.14 ms – 1.14 ms | **0.20 ms – 1.12 ms** | **Sub-millisecond** |

---

### 2. Medium Dataset: 500 MB Log File (`api-out-500mb.log` / 6,508,380 Lines)

| Benchmark Metric | Native Rust Engine (Final Release) | Details |
| :--- | :--- | :--- |
| **Parse Wall Time** | **0.620 s (619 ms)** | 863.45 MB/s throughput |
| **Time from Parse Finish $\to$ UI Render** | **64 ms** | Pre-computed path normalization |
| **Total End-to-End Load Time** | **0.684 s (684 ms)** | Complete dashboard initialization |
| **Search Filter Query ("auth")** | **4 ms** | Real-time sub-frame filtering |
| **Status Filter Query (4xx)** | **5 ms** | Real-time error rate filtering |

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
|             - Immediate-mode UI (egui 0.31)                                       |
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
