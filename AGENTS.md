# AGENTS.md

Operating instructions for coding agents working on PM2 Log Analyzer Native.

## Project Context
- Language & Edition: Rust 2024 (`edition = "2024"`)
- GUI Framework: **iced 0.14** (Elm architecture, wgpu/tiny-skia renderer) porting the
  React/Wasm reference app in `../pm2-log-analyzer` 1:1. There is no HTML/CSS/WebView2 and
  no Tailwind build: every reference Tailwind class maps to a function in `src/ui/style.rs`.
- Parity rule: `../pm2-log-analyzer` is the specification. The Wasm kernels there are pure Rust
  and are vendored verbatim into `src/kernels/`. Do not "improve" kernel code without changing
  the reference first. Data parity (`parity_runner`) is the hard gate; pixel parity against the
  browser no longer applies because iced renders natively (see Parity verification).
- Architecture Breakdown:
  - `src/main.rs`: iced application boot (window size/min size, theme, subscription, style)
  - `src/app.rs`: `App` state + `Message` enum + `update`/`view`/`subscription`; worker-thread
    `JobEvent` streaming; derived rows (`refresh_rows`) and chart cache (`sync_chart`)
  - `src/kernels/pm2/`: vendored pm2-core (`parse.rs`, `normalize.rs`, `relhist.rs`, `store.rs`)
  - `src/core/`: native driver (`pm2.rs`, mirrors `logParserWorker.ts`), result models
    (`models.rs`, mirrors `parser/types.ts`), JS-sketch coordinator math (`relhist_js.rs`)
  - `src/store/`: plain `AnalysisState` + `AppState` transitions (Zustand equivalent, no signals),
    `AppMode`; persistence keyed by the reference keys (`pm2-analyzer-filters`, `app-analyzer-mode`)
  - `src/ui/`: view modules (header, ingest, kpi, filters, api_table, cron_table, charts, skipped,
    toast, mongo placeholder), `style.rs` (palette + widget styles), `virtualize.rs`
    (fixed-row-height table windows), generated `icons.rs`
  - `src/utils/`: `format.rs` (JS-exact `toFixed`/locale behavior), `table_ops.rs` (client-side
    filter/sort/TSV), `persist.rs` (native JSON files), `chart_renderer.rs` (SVG->PNG for Excel),
    `export_spreadsheet.rs` (`ExportData` + workbook builder)
  - `src/bin/parity_runner.rs`: data-parity CLI (`--json`, `--dump-wires`, sample assertions)
  - `src/bin/export_runner.rs`: Excel export CLI (builds the app workbook from a log file)
  - `src/bin/bench.rs`: parse/reagg benchmark
- Platform Target: Pure Native Windows `.exe` (`x86_64-pc-windows-msvc` / `x86_64-pc-windows-gnu`)

## Building, Testing, and Benchmarking
- Check compilation: `cargo check` (must be warning-free)
- Run unit + headless UI tests: `cargo test`
- Run GUI app (debug/release): `cargo run` / `cargo run --release`
- Build release executable: `cargo build --release`
- Regenerate icons: `node scripts/gen-icons.mjs ../pm2-log-analyzer/node_modules/lucide-react/dist/esm`
- Render the UI headlessly for visual inspection:
  `cargo test --lib writes_ui_preview -- --ignored` (writes `target/ui-preview*-<renderer>.png`)

## Parity verification (required before claiming UI/data parity)
- Data: `parity_runner --dump-wires` + `parity/coordinator.mts` (reference decoders/merge) +
  `parity/compare.mjs` — the native `AggregatedResult` must deep-equal the reference coordinator's.
- UI: `cargo test` covers the headless iced view (`iced_test::simulator`: text presence and
  interaction->message wiring). `writes_ui_preview` renders light/split and dark/wide PNGs.
- Chart ground truth: `visual-tests/dump-chart.mjs` + `summary-chart.mjs` dump Recharts geometry
  from the reference app; the in-app chart renders that same SVG through iced's svg widget.
- Excel export: `visual-tests/export-ref.mjs` (Playwright download capture) +
  `export_runner` + `visual-tests/export-compare.mjs` (exceljs structural diff).
- Stage status, Mongo/archive inventory and wiring points: `PARITY_PLAN.md`.

## Learned constraints
- Mutate state only inside `App::update`; the tail of `update` refreshes derived state
  (`refresh_rows`, `sync_chart`) and the toast deadline. Never do work in `view`.
- Asynchronous work returns a `Task`: parse/reaggregate run on `std::thread` workers and stream
  `JobEvent`s into the runtime through `Task::run` with an unbounded channel. Cancel via
  `JobControl`, and ignore finished jobs whose `Arc<JobControl>` is not the current one.
- Tables are virtualized by `ui::virtualize::visible_range` + top/bottom spacer rows; never render
  every endpoint row — 6k+ rows must stay O(viewport).
- Icons are generated SVG documents with the stroke color baked in (see `ui/icons.rs`).
- Typography is bundled IBM Plex (`ui/fonts.rs`, loaded via `iced::application().font(...)` with
  `style::REGULAR` as the default face), so `font-medium` is a real weight. Truncated table cells
  use `utils/text_fit::truncate_mono` (exact advance from the bundled font — no shaping).
- Persistence is file-based JSON under `PM2_ANALYZER_DATA_DIR` (or `%APPDATA%/pm2-log-analyzer`),
  using the reference keys `pm2-analyzer-filters`, `mongo-analyzer-filters` and `app-analyzer-mode`.
- MongoDB mode shares the PM2 architecture: vendored kernels in `src/kernels/mongo`, a native
  driver in `src/core/mongo.rs`, `MongoState` in `src/store/mongo_store.rs`, and views under
  `src/ui/mongo/`. Filter changes re-run `kernel.reaggregate` synchronously (microseconds).

## 13. Project Learnings (pixel parity)

- Tailwind line heights are load-bearing: `text-xs` is 12px/16, `text-base` 16px/24, `text-lg`
  18px/28, arbitrary `text-[10px]`/`[11px]` use the font's ~1.5em normal. iced's default line box
  is ~1.3em, so every section came out short and the error accumulated down the page. Use
  `ui::lined` / `ui::lined_styled` (fixed-height containers) for the elements whose height drives a
  card — labels, KPI values, section titles — and `TextInput::line_height(Pixels(16.0))` for
  inputs (a bare `f32` is a *relative* factor in iced and blows the layout up).
- Container `height` is the total box: `.height(15).padding(2)` leaves 11px for the child. Put the
  fixed-height container *inside* the padded one.
- Chromium's overlay scrollbar takes no layout width; iced's scrollable reserves it, which shifted
  every right-hand column by ~10px. `Direction::Vertical(Scrollbar::new().width(0.0))` matches.
- Gate with `visual-tests/shot-ref.mjs` + `pixel-diff.mjs` (aligned diff via `offset2.mjs`).
  95.7% aligned similarity is the ceiling across renderers; judge the rest structurally.
