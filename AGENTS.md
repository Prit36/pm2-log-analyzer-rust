# AGENTS.md

Operating instructions for coding agents working on PM2 Log Analyzer Native.

## Project Context
- Language & Edition: Rust 2024 (`edition = "2024"`)
- GUI Framework: **Dioxus 0.6 desktop** (WebView2 + rsx! HTML) styled with Tailwind v4, matching
  the React/Wasm reference app in `../pm2-log-analyzer` 1:1. There is no egui code.
- Parity rule: `../pm2-log-analyzer` is the specification. The Wasm kernels there are pure Rust
  and are vendored verbatim into `src/kernels/`. Do not "improve" kernel code without changing
  the reference first.
- Architecture Breakdown:
  - `src/main.rs`: window + WebView2 data-dir hook (`PM2_ANALYZER_DATA_DIR`, used by tests)
  - `src/app.rs`: shell (`AppHeader` + mode switch + `main` + toast), `Pm2AppView`
  - `src/kernels/pm2/`: vendored pm2-core (`parse.rs`, `normalize.rs`, `relhist.rs`, `store.rs`)
  - `src/core/`: native driver (`pm2.rs`, mirrors `logParserWorker.ts`), result models
    (`models.rs`, mirrors `parser/types.ts`), JS-sketch coordinator math (`relhist_js.rs`)
  - `src/store/`: Zustand-equivalent `AnalysisStore` + `AppModeStore` (signals + free functions),
    localStorage persistence with the reference keys (`pm2-analyzer-filters`, `app-analyzer-mode`)
  - `src/ui/`: reference-parity components (header, ingest, kpi, filters, api_table, cron_table,
    charts, skipped, toast, mongo placeholder) + generated `icons.rs`
  - `src/utils/`: `format.rs` (JS-exact `toFixed`/locale behavior), `table_ops.rs` (client-side
    filter/sort/TSV), `persist.rs`, `cn.rs`, `chart_renderer.rs`, `export_spreadsheet.rs`
  - `src/bin/parity_runner.rs`: data-parity CLI (`--json`, `--dump-wires`, sample assertions)
  - `src/bin/export_runner.rs`: Excel export CLI (builds the app workbook from a log file)
  - `src/utils/chart_renderer.rs`: canvas->SVG->PNG port of the reference chart images (resvg)
  - `src/bin/bench.rs`: parse/reagg benchmark
- Platform Target: Pure Native Windows `.exe` (`x86_64-pc-windows-msvc` / `x86_64-pc-windows-gnu`)

## Building, Testing, and Benchmarking
- Check compilation: `cargo check`
- Run unit tests: `cargo test`
- Run GUI app (debug/release): `cargo run` / `cargo run --release`
- Build release executable: `cargo build --release`
- Rebuild Tailwind CSS after changing rsx classes: `npm run css:build` (then commit `assets/style.css`)
- Regenerate icons: `node scripts/gen-icons.mjs ../pm2-log-analyzer/node_modules/lucide-react/dist/esm`

## Parity verification (required before claiming UI/data parity)
- Data: `parity_runner --dump-wires` + `parity/coordinator.mts` (reference decoders/merge) +
  `parity/compare.mjs` — the native `AggregatedResult` must deep-equal the reference coordinator's.
- Visual/text: `visual-tests/run.mjs` (requires the reference `vite preview` on :5173). It
  restarts the native app per case with an isolated WebView2 data dir, compares region pixel
  similarity and normalized region text, and exits non-zero below the threshold.
- The webview cannot be reloaded (Dioxus IPC dies); always restart the app instead.
- `visual-tests/dump-chart.mjs` + `summary-chart.mjs` capture Recharts ground truth for chart work.
- Excel export: `visual-tests/export-ref.mjs` (Playwright download capture) +
  `export_runner` + `visual-tests/export-compare.mjs` (exceljs structural diff).
- Stage status, Mongo/archive inventory and wiring points: `PARITY_PLAN.md`.

## Learned constraints
- Dioxus signals are read with `signal()` only for locals; for struct fields use accessor methods
  (`store.filters()`) or `(store.field)()`. `set` needs a mutable binding (`mut store`).
- The visual gate is capped by text rasterization differences between WebView2 and Chromium;
  keep text content identical (checked by the harness) and pixels as close as practical.
