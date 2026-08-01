# PM2 Log Analyzer Native - Performance History Log

This document records the exact, unedited raw terminal outputs from benchmark execution runs (`target/release/bench.exe`). Each entry is appended verbatim without modifying a single character.

---

## 🕒 Benchmark Run 1: 2026-08-01 15:32:23 IST

```text
================================================================
      PM2 LOG ANALYZER NATIVE RUST BENCHMARK SUITE
================================================================
Target File: c:/Users/My_Home/Desktop/projects/pm2-logs/pm2-log-analyzer/test_data/api-out-5gb.log
[1/6] Mmap File Open: 0 ms (Size: 5.22 GB / 5350.37 MB)

[2/6] Parsing log buffer across 12 Rayon threads...
  - Parse Wall Time: 2.370 sec (2370 ms)
  - Throughput: 2257.17 MB/s
  - Total Lines Parsed: 65083800
  - Matched HTTP Entries: 37184500
  - Unique Paths Interned: 505195
  - Cron Events: 13400
  - Unmatched Lines: 27885900

[3/6] UI Preparation & Display Timing (Started IMMEDIATELY after parse finish):
  - Time from Parse Finish -> First UI Frame Ready: 2 ms (0.002 s)
  - Total Endpoints Prepared for Render: 6107
  - Overall Latency Percentiles: p50=2.12ms, p90=257.29ms, p95=1153.13ms, p99=6570.16ms
  - Error Count: 1342300 (3.61%)

[4/6] End-to-End Time (Parse + UI Initial Render):
  - Parse Time: 2.370 s
  - UI Display Prep Time: 0.002 s
  - Total Time to Render UI: 2.373 s

[5/6] Interactive UI Re-aggregation Timings (Filter Toggles):
  - Filter by Search Query ("auth"): 19 ms (Endpoints: 6)
  - Filter by Method (POST): 85 ms (Matched requests: 8690600)
  - Filter by Status (4xx Errors Only): 20 ms (Error requests: 1335800)
  - Switch Path Norm Mode (Raw Path): 1286 ms (Endpoints: 519948)

[6/6] UI Table Sorting Timings across 8 Sort Columns:
  - Sort by Path        : 1.130 ms
  - Sort by Method      : 0.216 ms
  - Sort by Calls       : 0.294 ms
  - Sort by ErrorRate   : 0.322 ms
  - Sort by AvgDuration : 0.844 ms
  - Sort by P50         : 0.510 ms
  - Sort by P95         : 0.530 ms
  - Sort by P99         : 0.358 ms
  - Total 8-Column Table Sort Overhead: 4.261 ms

================================================================
                     BENCHMARK SUMMARY
================================================================
Dataset: 5.22 GB log file (c:/Users/My_Home/Desktop/projects/pm2-logs/pm2-log-analyzer/test_data/api-out-5gb.log)
1. Parse Wall Time:                      2.370 s (2257.17 MB/s)
2. Time from Parse Finish to UI Display: 0.002 s (2 ms)
3. Total Time to Render UI:              2.373 s
4. Average Filter Toggle Response:       352.50 ms
================================================================
```

---

## 🕒 Benchmark Run 2: 2026-08-01 15:38:00 IST (Parse Wall Time & UI Display Optimization Fix)

```text
================================================================
      PM2 LOG ANALYZER NATIVE RUST BENCHMARK SUITE
================================================================
Target File: c:/Users/My_Home/Desktop/projects/pm2-logs/pm2-log-analyzer/test_data/api-out-5gb.log
[1/6] Mmap File Open: 0 ms (Size: 5.22 GB / 5350.37 MB)

[2/6] Parsing log buffer across 12 Rayon threads...
  - Parse Wall Time: 2.100 sec (2099 ms)
  - Throughput: 2548.38 MB/s
  - Total Lines Parsed: 65083800
  - Matched HTTP Entries: 37184500
  - Unique Paths Interned: 505195
  - Cron Events: 13400
  - Unmatched Lines: 27885900

[3/6] UI Preparation & Display Timing (Started IMMEDIATELY after parse finish):
  - Time from Parse Finish -> First UI Frame Ready: 71 ms (0.072 s)
  - Total Endpoints Prepared for Render: 6107
  - Overall Latency Percentiles: p50=2.44ms, p90=273.20ms, p95=1200.19ms, p99=6702.89ms
  - Error Count: 1342300 (3.61%)

[4/6] End-to-End Time (Parse + UI Initial Render):
  - Parse Time: 2.100 s
  - UI Display Prep Time: 0.072 s
  - Total Time to Render UI: 2.171 s

[5/6] Interactive UI Re-aggregation Timings (Filter Toggles):
  - Filter by Search Query ("auth"): 28 ms (Endpoints: 6)
  - Filter by Method (POST): 76 ms (Matched requests: 8690600)
  - Filter by Status (4xx Errors Only): 32 ms (Error requests: 1335800)
  - Switch Path Norm Mode (Raw Path): 1009 ms (Endpoints: 519948)

[6/6] UI Table Sorting Timings across 8 Sort Columns:
  - Sort by Path        : 1.137 ms
  - Sort by Method      : 0.220 ms
  - Sort by Calls       : 0.304 ms
  - Sort by ErrorRate   : 0.325 ms
  - Sort by AvgDuration : 1.139 ms
  - Sort by P50         : 0.334 ms
  - Sort by P95         : 0.147 ms
  - Sort by P99         : 0.144 ms
  - Total 8-Column Table Sort Overhead: 3.812 ms

================================================================
                     BENCHMARK SUMMARY
================================================================
Dataset: 5.22 GB log file (c:/Users/My_Home/Desktop/projects/pm2-logs/pm2-log-analyzer/test_data/api-out-5gb.log)
1. Parse Wall Time:                      2.100 s (2548.38 MB/s)
2. Time from Parse Finish to UI Display: 0.072 s (71 ms)
3. Total Time to Render UI:              2.171 s
4. Average Filter Toggle Response:       286.25 ms
================================================================
```
