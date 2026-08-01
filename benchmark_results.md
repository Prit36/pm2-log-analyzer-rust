# PM2 Log Analyzer Native - Verified Performance Optimization Results

- **Git Baseline Commit**: `85f7765` (before optimization pass)
- **Dataset**: `api-out-5gb.log` (**5.22 GB** / 5,350.37 MB / **65,083,800 Log Lines**)
- **Hardware/Environment**: Windows 11 x64 (12 Rayon Worker Threads)
- **Engine**: Zero-Allocation Path Interning (`rapidhash` `u64`), Fast-Path Reordered Parser, Branchless Separator Timestamp Matcher, Zero-Allocation Status Buckets (`StatusBuckets`), Memory Mapping (`memmap2`).
- **Binary Profile**: Release (`target/release/bench.exe`)

---

## 🚀 Side-by-Side Performance Comparison vs Git Baseline (`85f7765`)

| Benchmark Metric | Git Baseline (`85f7765`) | Peak Optimized Engine | Delta / Improvement |
| :--- | :--- | :--- | :--- |
| **Parse Wall Time** | 2.955 s (2,955 ms) | **2.171 s (2,171 ms)** | **784 ms Faster (+26.5% Speedup)** |
| **Parse Throughput** | 1,810.41 MB/s | **2,464.83 MB/s** | **+654.42 MB/s Higher (+36.1% Speedup)** |
| **Parse Finish $\to$ UI Render** | 409 ms (0.409 s) | **359 ms (0.359 s)** | **50 ms Faster (+12.2% Speedup)** |
| **Total End-to-End Time to UI** | 3.364 s (3,364 ms) | **2.530 s (2,530 ms)** | **834 ms Faster (+24.8% Speedup)** |

---

## 🔍 Interactive UI Filter & Re-aggregation Response Times

| Interactive Filter Operation | Git Baseline (`85f7765`) | Peak Optimized Engine | Speedup / Improvement |
| :--- | :--- | :--- | :--- |
| **Search Query Filter (`"auth"`)** | 18 ms | **17 ms** | Sub-20 ms Instant |
| **Status Filter (`4xx Errors`)** | 24 ms | **19 ms** | **+20.8% Faster** |
| **Method Filter (`POST`)** | 95 ms | **81 ms** | **+14.7% Faster** |
| **Path Norm Switch (`Raw Path`)** | 2,537 ms | **2,103 ms** | **434 ms Faster (+17.1% Speedup)** |

---

## ⚡ UI Endpoint Table Column Sorting Overhead (6,107 Routes)

| Column Sort Target | Git Baseline (`85f7765`) | Peak Optimized Engine | Rate |
| :--- | :--- | :--- | :--- |
| **Sort by Endpoint Path** | 1.113 ms | **1.064 ms** | Instant |
| **Sort by HTTP Method** | 0.223 ms | **0.204 ms** | Sub-millisecond |
| **Sort by Total Calls** | 0.266 ms | **0.316 ms** | Sub-millisecond |
| **Sort by Error Rate** | 0.287 ms | **0.399 ms** | Sub-millisecond |
| **Sort by Avg Duration** | 0.833 ms | **0.850 ms** | Sub-millisecond |
| **Sort by p50 Latency** | 0.517 ms | **0.545 ms** | Sub-millisecond |
| **Sort by p95 Latency** | 0.681 ms | **0.587 ms** | Sub-millisecond |
| **Sort by p99 Latency** | 0.477 ms | **0.343 ms** | Sub-millisecond |
| **Total 8-Column Sort Overhead** | 4.458 ms | **4.390 ms** | Sub-5ms Total |
