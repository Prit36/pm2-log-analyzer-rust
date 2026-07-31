# ⚡ PM2 Log Analyzer Native (Rust 2024 + egui + Rayon + mimalloc + SIMD)

A high-performance, pure native Windows desktop application built in Rust 2024 (`egui 0.31` + `eframe` + `memmap2` + `rayon` + `mimalloc` + `rapidhash`) capable of parsing multi-gigabyte PM2 log files at **1,767.29 MB/s throughput** and rendering interactive analytics dashboards in **3.42 seconds for a 5.22 GB log file (65.08 Million lines)** and **0.68 seconds for a 500 MB log file**.

---

## 🚀 Performance Benchmarks: WASM Reference vs. Native Rust Engine

### 1. Benchmark on 5.22 GB PM2 Log File (`api-out-5gb.log` / 65,083,800 lines):

| Metric | Original WASM Engine (Browser + Workers) | Native Rust Engine (Initial Naive) | Native Rust Engine (Phase 1) | Native Rust Engine (Final Optimized) | Improvement / Parity |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Parse Wall Time** | 7.85 s – 8.07 s | 100% CPU lock | 3.475 s | **3.027 s** | 🚀 **2.60x Faster Parse** |
| **Parse Throughput** | ~670 MB/s | Bottlenecked | 1,539.70 MB/s | **1,767.29 MB/s** | ⚡ **+163% Throughput** |
| **Time from Parse Finish -> UI Display** | ~900 ms | 5,890 ms (5.89 s) | 578 ms (0.578 s) | **396 ms (0.396 s)** | 🏎️ **14.8x UI Speedup** |
| **Total Time to Interactive UI** | ~8.95 s | > 15.0 s (100% RAM) | 3.836 s | **3.424 s** | 💥 **2.61x Faster End-to-End** |
| **Total Lines Parsed** | 65,083,800 | Incorrect | 65,083,800 | **65,083,800** | ✅ **100% Exact Match** |
| **Matched HTTP Requests** | 37,184,500 | Incorrect | 37,184,500 | **37,184,500** | ✅ **100% Exact Match** |
| **Unmatched Log Lines** | 27,885,900 | Incorrect | 27,885,900 | **27,885,900** | ✅ **100% Exact Match** |
| **Unique Endpoints (`:id`)**| 6,107 | Corrupted | 6,107 | **6,107** | ✅ **100% Exact Match** |
| **Overall p95 Latency** | ~1,159.1 ms | Inaccurate | 1,153.1 ms | **1,153.1 ms** | ✅ **100% Parity** |
| **Search Filter Toggle ("auth")**| ~400 ms | UI freeze | 3,025 ms | **19 ms** | ⚡ **Sub-20ms Instant** |
| **Status Filter Toggle (4xx)** | ~300 ms | UI freeze | 332 ms | **21 ms** | ⚡ **Sub-25ms Instant** |
| **Interactive Table Sorting** | UI re-renders | UI freeze | 0.14 ms – 1.14 ms | **0.20 ms – 1.12 ms** | ⚡ **Sub-millisecond Sort** |

---

### 2. Benchmark on 500 MB PM2 Log File (`api-out-500mb.log` / 6,508,380 lines):

| Metric | Duration | Details |
| :--- | :--- | :--- |
| **Parse Wall Time** | **0.620 s (619 ms)** | 863.45 MB/s parsing throughput |
| **Time from Parse Finish -> UI Display** | **64 ms (0.064 s)** | Instant initial render preparation |
| **Total Time to Interactive UI** | **0.684 s (684 ms)** | Complete load to visible dashboard in under 0.7 seconds |
| **Search Filter Toggle ("auth")** | **4 ms** | Real-time 4ms sub-frame filtering |
| **Status Filter Toggle (4xx Errors)** | **5 ms** | Real-time 5ms error rate filtering |

---

## 📖 The Performance Journey

### Phase 1: Identifying Bottlenecks & Accuracy Bugs
The initial naive Rust implementation suffered from extreme performance degradation on multi-gigabyte log files:
- **Massive RAM Thrashing**: Every parsed line created a `ParsedHttpLine { path: String, ... }` with a heap-allocated `String`. Parsing 10M lines generated 10 million individual allocations.
- **$O(N^2)$ Rayon Copying**: Rayon threads merged results via `Vec::append()`, repeatedly copying millions of structs across thread boundaries.
- **Inaccurate Log Parser**: Naive whitespace splitting treated words like `"GET"` or `"POST"` in error traces or timestamps as HTTP logs, extracting garbage paths. It also missed PM2 **Format B** (`68064.174ms POST /api/...`), ISO timestamps (`YYYY-MM-DDTHH:MM:SS:`), and ANSI colors.
- **Overly Aggressive Path Collapsing**: Collapsed 1-digit integers like `/v1/users/2` into `/v1/users/:id`, corrupting endpoint statistics.

### Phase 2: Columnar Packed Store, WASM Parity & `RelHist`
To resolve memory bloat and parser inaccuracies:
1. **Columnar Store (`PackedEntry`)**: Reduced per-line representation to a compact **12–16 byte struct** (`path_id: u32, duration: f32, status: u16, method: u8`). Unique path strings are interned once into a contiguous byte buffer (`path_bytes: Vec<u8>`).
2. **Relative Histogram Sketch (`RelHist`)**: Adopted a logarithmic bucket histogram ($\gamma \approx 1.0202$) for $O(1)$ space percentile estimation (p50, p90, p95, p99) without storing or sorting millions of floats.
3. **Strict Byte-Level Parser (`parse_line_bytes`)**: Ported zero-copy line parsing from WASM `pm2-core`, achieving **100% exact parity** on total lines (65.08M), matched HTTP (37.18M), unmatched lines (27.88M), and unique endpoints (6,107).

### Phase 3: Hyper-Optimizing UI Display & Direct-Array Aggregation
Although parsing took ~3.0s, preparing the UI summary initially took **5,890 ms (5.89 s)** due to single-threaded `HashMap` lookups over 37.18 million entries:
1. **Pre-Evaluated Search Queries**: Search queries are pre-evaluated against unique normalized paths ($O(\text{unique\_paths})$) rather than executing 37.18 million string comparisons.
2. **Direct Dense-Array Indexing**: Replaced hash map lookups with zero-hash direct array lookups indexed by `(norm_id << 3) | method`.
3. **Pre-Computed Active Path Normalization**: The active normalized path mapping (`norm_maps[2]`) is pre-computed right after parsing finishes, eliminating path normalization overhead.
4. **`mimalloc` Lock-Free Thread Allocator**: Integrated Microsoft's `mimalloc` as the global memory allocator for ultra-fast multi-threaded allocations across Rayon worker threads.
5. **Inline Method Matching**: Implemented fast 4-byte integer matching (`GET ` / `POST`), bypassing string comparison loops for >95% of HTTP logs.
6. **Result**: 500 MB log file total load time reduced to **0.684 s** (684 ms), and 5.22 GB log file total load time reduced to **3.424 s**. Interactive search query filter response dropped to **19 ms (5.22GB)** / **4 ms (500MB)**.

---

## 🏗️ Architecture

```
                               ┌────────────────────────────────┐
                               │     Memory-Mapped Log File     │
                               │        (memmap2 - 5.22 GB)     │
                               └───────────────┬────────────────┘
                                               │
                                               ▼
                               ┌────────────────────────────────┐
                               │   Parallel Rayon Chunk Parsing │
                               │  (parse_line_bytes - 1.76 GB/s)│
                               └───────────────┬────────────────┘
                                               │
                                               ▼
                               ┌────────────────────────────────┐
                               │     Columnar Engine Store      │
                               │ PackedEntry (16B) + Interning  │
                               └───────────────┬────────────────┘
                                               │
                                               ▼
                               ┌────────────────────────────────┐
                               │ Parallel Dense Array Aggregator│
                               │  (Direct Indexing + RelHist)   │
                               └───────────────┬────────────────┘
                                               │
                                               ▼
                               ┌────────────────────────────────┐
                               │ egui Ops Dashboard (Native GUI)│
                               │  4-19ms Filter / Sub-ms Sort   │
                               └────────────────────────────────┘
```

---

## ⚙️ Building & Running

### Requirements
- Rust 2024 Edition (`cargo`)
- Windows OS (64-bit)

### Run GUI Application locally
```bash
cargo run
```

### Build Release Executable
```bash
cargo build --release
```
The optimized release executable will be produced at:
`target/release/pm2-log-analyzer.exe`

### Run Benchmark Suite
To measure parse throughput, UI display timing, filter toggle performance, and table sorting:
```bash
cargo run --release --bin bench -- "path/to/pm2-log-file.log"
```

---

## 🧪 Unit Tests
Run the test suite covering parsing, path normalization, percentile sketches, and end-to-end aggregation:
```bash
cargo test
```
