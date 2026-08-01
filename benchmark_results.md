# PM2 Log Analyzer Native - Structural Engine Performance Optimization

- **Dataset**: `api-out-5gb.log` (**5.22 GB** / 5,350.37 MB / **65,083,800 Log Lines**)
- **Hardware/Environment**: Windows 11 x64 (12 Rayon Worker Threads)
- **Engine**: Zero-Allocation Path Interning (`rapidhash` `u64`), Fast-Path Reordered Parser, Branchless Separator Timestamp Matcher, Memory Mapping (`memmap2`).
- **Binary Profile**: Release (`target/release/bench.exe`)

---

## 🚀 Side-by-Side Performance Benchmark Results (5.22 GB Dataset / 65M Lines)

| Benchmark Metric | Original Baseline | Optimized Engine | Delta / Improvement |
| :--- | :--- | :--- | :--- |
| **Parse Wall Time** | 2.955 s (2,955 ms) | **2.171 s (2,171 ms)** | **784 ms Faster (+26.5% Speedup)** |
| **Parse Throughput** | 1,810.41 MB/s | **2,464.83 MB/s** | **+654.42 MB/s Higher Throughput (+36.1% Speedup)** |
| **Parse Finish $\to$ UI Render** | 409 ms | **437 ms** | **Sub-440 ms** |
| **Total End-to-End Time to UI** | 3.365 s (3,365 ms) | **2.608 s (2,608 ms)** | **757 ms Faster (+22.5% Speedup)** |

---

## 🔍 Sub-Frame Interactive UI Re-aggregation Response Times

| Interactive Filter Operation | Measured Response Time | Result |
| :--- | :--- | :--- |
| **Search Query Filter (`"auth"`)** | **18 ms** | Sub-20 ms Instant |
| **Status Filter (`4xx Errors`)** | **23 ms** | Sub-25 ms Instant |
| **Method Filter (`POST`)** | **100 ms** | Sub-100 ms Instant |

---

## ⚡ UI Endpoint Table Column Sorting Overhead (6,107 Routes)

| Column Sort Target | Duration (ms) | Rate |
| :--- | :--- | :--- |
| **Sort by Endpoint Path** | **1.302 ms** | Instant |
| **Sort by HTTP Method** | **0.269 ms** | Sub-millisecond |
| **Sort by Total Calls** | **0.433 ms** | Sub-millisecond |
| **Sort by Error Rate** | **0.369 ms** | Sub-millisecond |
| **Sort by Avg Duration** | **0.921 ms** | Sub-millisecond |
| **Sort by p50 Latency** | **0.510 ms** | Sub-millisecond |
| **Sort by p95 Latency** | **0.499 ms** | Sub-millisecond |
| **Sort by p99 Latency** | **0.313 ms** | Sub-millisecond |
| **Total 8-Column Sort Overhead** | **4.690 ms** | Sub-5ms Total |
