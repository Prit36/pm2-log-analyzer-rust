# PM2 Log Analyzer Native - Comprehensive Benchmark Suite Results

- **Environment**: Windows 11 x64 (12-Thread Rayon Worker Pool)
- **Engine**: Memory Mapping (`memmap2`), SIMD Byte Search (`memchr`), Multithreaded Processing (`rayon`), Rapid Hashing (`rapidhash`), Lock-Free Allocations (`mimalloc`)
- **Binary Profile**: Release (`target/release/bench.exe`)

---

## 📊 Comprehensive Multi-Dataset Comparison Table

| Benchmark Metric | Small (53.5 MB / 650K Lines) | Medium (561 MB / 6.5M Lines) | Large (5.22 GB / 65M Lines) |
| :--- | :--- | :--- | :--- |
| **Log File Path** | `api-out.log` | `api-out-500mb.log` | `api-out-5gb.log` |
| **File Size (MB / GB)** | 53.50 MB (0.05 GB) | 535.04 MB (0.52 GB) | 5,350.37 MB (5.22 GB) |
| **Total Lines Parsed** | **650,974** | **6,508,380** | **65,083,800** |
| **Matched HTTP Requests** | 371,845 | 3,718,450 | 37,184,500 |
| **Unique Paths Interned** | 32,850 | 505,195 | 505,195 |
| **Cron Events Detected** | 134 | 1,340 | 13,400 |
| **Unmatched Log Lines** | 278,859 | 2,788,590 | 27,885,900 |
| **Mmap File Open Time** | **0 ms** | **0 ms** | **0 ms** |
| **Parse Wall Time** | **41 ms (0.041 s)** | **598 ms (0.598 s)** | **2,955 ms (2.955 s)** |
| **Parse Throughput** | **1,296.13 MB/s** | **894.63 MB/s** | **1,810.41 MB/s** |
| **Parse Finish $\to$ UI Render** | **13 ms** | **76 ms** | **409 ms** |
| **End-to-End Time to UI** | **54 ms (0.054 s)** | **675 ms (0.675 s)** | **3,365 ms (3.365 s)** |

---

## 🔍 Interactive UI Filter & Re-aggregation Response Times

| Interactive Filter Operation | Small (53.5 MB) | Medium (561 MB) | Large (5.22 GB) |
| :--- | :--- | :--- | :--- |
| **Search Query (`"auth"`)** | **1 ms** | **3 ms** | **18 ms** |
| **HTTP Method Filter (`POST`)** | **3 ms** | **14 ms** | **95 ms** |
| **Status Filter (`4xx Errors`)** | **1 ms** | **5 ms** | **24 ms** |
| **Path Norm Switch (`Raw Path`)** | **51 ms** | **802 ms** | **2,537 ms** |
| **Avg Filter Response Time** | **14.00 ms** | **206.00 ms** | **668.50 ms** |

---

## ⚡ UI Endpoint Table Column Sorting Overhead (6,107 Routes)

| Column Sort Target | Sorting Duration (ms) | Sorting Rate |
| :--- | :--- | :--- |
| **Sort by Endpoint Path** | **1.113 ms** | Instant |
| **Sort by HTTP Method** | **0.223 ms** | Sub-millisecond |
| **Sort by Total Calls** | **0.266 ms** | Sub-millisecond |
| **Sort by Error Rate** | **0.287 ms** | Sub-millisecond |
| **Sort by Avg Duration** | **0.833 ms** | Sub-millisecond |
| **Sort by p50 Latency** | **0.517 ms** | Sub-millisecond |
| **Sort by p95 Latency** | **0.681 ms** | Sub-millisecond |
| **Sort by p99 Latency** | **0.477 ms** | Sub-millisecond |
| **Total 8-Column Sort Overhead** | **4.458 ms** | Sub-5ms Total |
