//! PM2 analysis store — mirrors `src/store/analysisStore.ts` (Zustand) and the
//! module-level actions in `src/hooks/useParserWorker.ts`.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::models::{AggregatedResult, NormalizeMode, ParseOptions, StatusFamily};
use crate::core::pm2::{
    parse_sources, LoadedSource, JobControl, ParseError, Pm2Kernel,
};
use crate::utils::persist;
use crate::utils::format::format_num;

pub const PASTE_WARN_BYTES: usize = 8 * 1024 * 1024;
pub const DEFAULT_TOP_N: usize = 50;
const TOAST_MS: u64 = 3200;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartLayout {
    Split,
    Wide,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    None,
    File,
    Paste,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ApiSortKey {
    #[serde(rename = "p95Ms")]
    P95Ms,
    #[serde(rename = "p99Ms")]
    P99Ms,
    #[serde(rename = "avgMs")]
    AvgMs,
    #[serde(rename = "maxMs")]
    MaxMs,
    #[serde(rename = "count")]
    Count,
    #[serde(rename = "errorCount")]
    ErrorCount,
    #[serde(rename = "path")]
    Path,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CronSortKey {
    #[serde(rename = "p95Ms")]
    P95Ms,
    #[serde(rename = "p99Ms")]
    P99Ms,
    #[serde(rename = "avgMs")]
    AvgMs,
    #[serde(rename = "maxMs")]
    MaxMs,
    #[serde(rename = "runs")]
    Runs,
    #[serde(rename = "fails")]
    Fails,
    #[serde(rename = "starts")]
    Starts,
    #[serde(rename = "lastDurationMs")]
    LastDurationMs,
    #[serde(rename = "name")]
    Name,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisFilters {
    pub normalize_mode: NormalizeMode,
    pub status_family: StatusFamily,
    pub min_ms: f64,
    pub methods: Vec<String>,
    pub query: String,
    pub sort_key: ApiSortKey,
    pub sort_dir: SortDirection,
    pub top_n: usize,
    pub cron_query: String,
    pub cron_min_ms: f64,
    pub cron_show_failed_only: bool,
    pub cron_sort_key: CronSortKey,
    pub cron_sort_dir: SortDirection,
    pub date_filter: String,
}

impl Default for AnalysisFilters {
    fn default() -> Self {
        Self {
            normalize_mode: NormalizeMode::CollapseIds,
            status_family: StatusFamily::All,
            min_ms: 0.0,
            methods: Vec::new(),
            query: String::new(),
            sort_key: ApiSortKey::P95Ms,
            sort_dir: SortDirection::Desc,
            top_n: DEFAULT_TOP_N,
            cron_query: String::new(),
            cron_min_ms: 0.0,
            cron_show_failed_only: false,
            cron_sort_key: CronSortKey::P95Ms,
            cron_sort_dir: SortDirection::Desc,
            date_filter: "all".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ParseProgress {
    pub stage: String,
    pub processed: u64,
    pub total: u64,
    pub percent: u32,
}

#[derive(Serialize, Deserialize)]
struct PersistedState {
    filters: AnalysisFilters,
    theme: Theme,
    #[serde(rename = "chartLayout")]
    chart_layout: ChartLayout,
}

#[derive(Serialize, Deserialize)]
struct PersistedEnvelope {
    state: PersistedState,
    version: u32,
}

#[derive(Clone, Copy)]
pub struct AnalysisStore {
    pub theme: Signal<Theme>,
    pub chart_layout: Signal<ChartLayout>,
    pub source_kind: Signal<SourceKind>,
    pub file_name: Signal<Option<String>>,
    pub file_names: Signal<Vec<String>>,
    pub file_size: Signal<Option<u64>>,
    pub loaded_files: Signal<Vec<LoadedSource>>,
    pub has_data: Signal<bool>,
    pub result: Signal<Option<AggregatedResult>>,
    pub is_parsing: Signal<bool>,
    pub progress: Signal<Option<ParseProgress>>,
    pub error: Signal<Option<String>>,
    pub filters: Signal<AnalysisFilters>,
    pub toast: Signal<Option<String>>,
    pub paste_open: Signal<bool>,
    pub kernel: Signal<Option<Arc<Mutex<Pm2Kernel>>>>,
    pub job: Signal<Option<Arc<JobControl>>>,
    pub toast_epoch: Signal<u64>,
    pub reagg_epoch: Signal<u64>,
    pub window_width: Signal<f64>,
}

impl AnalysisStore {
    pub fn new() -> Self {
        Self {
            theme: Signal::new(Theme::Light),
            chart_layout: Signal::new(ChartLayout::Split),
            source_kind: Signal::new(SourceKind::None),
            file_name: Signal::new(None),
            file_names: Signal::new(Vec::new()),
            file_size: Signal::new(None),
            loaded_files: Signal::new(Vec::new()),
            has_data: Signal::new(false),
            result: Signal::new(None),
            is_parsing: Signal::new(false),
            progress: Signal::new(None),
            error: Signal::new(None),
            filters: Signal::new(AnalysisFilters::default()),
            toast: Signal::new(None),
            paste_open: Signal::new(false),
            kernel: Signal::new(None),
            job: Signal::new(None),
            toast_epoch: Signal::new(0),
            reagg_epoch: Signal::new(0),
            window_width: Signal::new(1440.0),
        }
    }

    pub fn is_dark(&self) -> bool {
        (self.theme)() == Theme::Dark
    }

    pub fn theme(&self) -> Theme {
        (self.theme)()
    }

    pub fn chart_layout(&self) -> ChartLayout {
        (self.chart_layout)()
    }

    pub fn source_kind(&self) -> SourceKind {
        (self.source_kind)()
    }

    pub fn file_name(&self) -> Option<String> {
        (self.file_name)()
    }

    pub fn file_names(&self) -> Vec<String> {
        (self.file_names)()
    }

    pub fn file_size(&self) -> Option<u64> {
        (self.file_size)()
    }

    pub fn loaded_files(&self) -> Vec<LoadedSource> {
        (self.loaded_files)()
    }

    pub fn has_data(&self) -> bool {
        (self.has_data)()
    }

    pub fn result(&self) -> Option<AggregatedResult> {
        (self.result)()
    }

    pub fn is_parsing(&self) -> bool {
        (self.is_parsing)()
    }

    pub fn progress(&self) -> Option<ParseProgress> {
        (self.progress)()
    }

    pub fn error(&self) -> Option<String> {
        (self.error)()
    }

    pub fn filters(&self) -> AnalysisFilters {
        (self.filters)()
    }

    pub fn toast(&self) -> Option<String> {
        (self.toast)()
    }

    pub fn paste_open(&self) -> bool {
        (self.paste_open)()
    }

    pub fn kernel(&self) -> Option<Arc<Mutex<Pm2Kernel>>> {
        (self.kernel)()
    }

    pub fn job(&self) -> Option<Arc<JobControl>> {
        (self.job)()
    }

    pub fn toast_epoch(&self) -> u64 {
        (self.toast_epoch)()
    }

    pub fn reagg_epoch(&self) -> u64 {
        (self.reagg_epoch)()
    }

    pub fn window_width(&self) -> f64 {
        (self.window_width)()
    }

    pub fn summary(&self) -> Option<crate::core::models::LogSummary> {
        self.result().map(|r| r.summary)
    }

    pub fn has_cron_events(&self) -> bool {
        self.result()
            .map(|r| {
                let c = r.cron_summary;
                c.starts + c.dones + c.fails > 0
            })
            .unwrap_or(false)
    }
}

impl Default for AnalysisStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_analysis_store() -> AnalysisStore {
    use_context::<AnalysisStore>()
}

// ── Persistence ─────────────────────────────────────────────────────────────

pub fn persisted_json(filters: &AnalysisFilters, theme: Theme, chart_layout: ChartLayout) -> String {
    let envelope = PersistedEnvelope {
        state: PersistedState {
            filters: filters.clone(),
            theme,
            chart_layout,
        },
        version: 0,
    };
    serde_json::to_string(&envelope).unwrap_or_default()
}

pub fn persist(store: AnalysisStore) {
    let json = persisted_json(&store.filters(), store.theme(), store.chart_layout());
    persist::set_item("pm2-analyzer-filters", &json);
}

/// Track window size for chart layout math (single eval listener; no polling).
pub async fn install_window_size(mut store: AnalysisStore) {
    let mut eval = document::eval(
        "dioxus.send(window.innerWidth);
         window.addEventListener('resize', () => dioxus.send(window.innerWidth));",
    );
    while let Ok(width) = eval.recv::<f64>().await {
        store.window_width.set(width);
    }
}

/// Load persisted filters/theme/layout at startup (async, once).
pub async fn restore_persisted(mut store: AnalysisStore) {
    if let Some(raw) = persist::load_item("pm2-analyzer-filters").await {
        if let Ok(envelope) = serde_json::from_str::<PersistedEnvelope>(&raw) {
            store.filters.set(envelope.state.filters);
            store.theme.set(envelope.state.theme);
            store.chart_layout.set(envelope.state.chart_layout);
        }
    }
    persist::set_html_dark(store.theme() == Theme::Dark);
}

// ── Theme / layout ──────────────────────────────────────────────────────────

pub fn toggle_theme(mut store: AnalysisStore) {
    let next = if store.theme() == Theme::Dark {
        Theme::Light
    } else {
        Theme::Dark
    };
    store.theme.set(next);
    persist::set_html_dark(next == Theme::Dark);
    persist(store);
}

pub fn toggle_chart_layout(mut store: AnalysisStore) {
    let next = if store.chart_layout() == ChartLayout::Split {
        ChartLayout::Wide
    } else {
        ChartLayout::Split
    };
    store.chart_layout.set(next);
    persist(store);
}

// ── Toast ───────────────────────────────────────────────────────────────────

pub fn show_toast(mut store: AnalysisStore, message: impl Into<String>) {
    let message = message.into();
    let epoch = store.toast_epoch() + 1;
    store.toast_epoch.set(epoch);
    store.toast.set(Some(message));
    spawn(async move {
        tokio::time::sleep(Duration::from_millis(TOAST_MS)).await;
        if store.toast_epoch() == epoch {
            store.toast.set(None);
        }
    });
}

// ── Filters ─────────────────────────────────────────────────────────────────

pub fn set_filters(mut store: AnalysisStore, filters: AnalysisFilters) {
    store.filters.set(filters);
    persist(store);
}

pub fn reset_filters(mut store: AnalysisStore) {
    store.filters.set(AnalysisFilters::default());
    persist(store);
    reaggregate(store);
}

pub fn count_active_analysis_filters(filters: &AnalysisFilters) -> usize {
    let mut count = 0;
    if !filters.query.trim().is_empty() {
        count += 1;
    }
    if filters.normalize_mode != NormalizeMode::CollapseIds {
        count += 1;
    }
    if filters.status_family != StatusFamily::All {
        count += 1;
    }
    if filters.min_ms > 0.0 {
        count += 1;
    }
    if !filters.methods.is_empty() {
        count += 1;
    }
    if filters.date_filter != "all" {
        count += 1;
    }
    if filters.top_n != DEFAULT_TOP_N {
        count += 1;
    }
    if filters.sort_key != ApiSortKey::P95Ms || filters.sort_dir != SortDirection::Desc {
        count += 1;
    }
    if !filters.cron_query.trim().is_empty() {
        count += 1;
    }
    if filters.cron_min_ms > 0.0 {
        count += 1;
    }
    if filters.cron_show_failed_only {
        count += 1;
    }
    count
}

/// `workerParseOptions(filters)` — the subset forwarded to the kernel.
pub fn parse_options(filters: &AnalysisFilters) -> ParseOptions {
    ParseOptions {
        normalize_mode: filters.normalize_mode,
        method_filter: None,
        status_family: filters.status_family,
        min_ms: filters.min_ms,
        cron_query: filters.cron_query.clone(),
        cron_min_ms: filters.cron_min_ms,
        cron_show_failed_only: filters.cron_show_failed_only,
        date_filter: if filters.date_filter == "all" {
            None
        } else {
            Some(filters.date_filter.clone())
        },
    }
}

// ── Result plumbing ─────────────────────────────────────────────────────────

fn set_result(mut store: AnalysisStore, result: AggregatedResult) {
    let cron = result.cron_summary.clone();
    let has_data = result.summary.matched > 0
        || cron.starts + cron.dones + cron.fails > 0
        || result.unmatched_count > 0;
    store.result.set(Some(result));
    store.has_data.set(has_data);
}

pub fn clear_analysis(mut store: AnalysisStore) {
    store.source_kind.set(SourceKind::None);
    store.file_name.set(None);
    store.file_names.set(Vec::new());
    store.file_size.set(None);
    store.loaded_files.set(Vec::new());
    store.has_data.set(false);
    store.result.set(None);
    store.progress.set(None);
    store.error.set(None);
    store.is_parsing.set(false);
    store.paste_open.set(false);
    store.kernel.set(None);
    store.job.set(None);
}

pub fn clear(store: AnalysisStore) {
    if let Some(job) = store.job() {
        job.cancel();
    }
    clear_analysis(store);
}

pub fn cancel(mut store: AnalysisStore) {
    if let Some(job) = store.job() {
        job.cancel();
    }
    store.is_parsing.set(false);
}

// ── Source selection (setLoadedFiles / appendLoadedFiles parity) ────────────

fn total_size(files: &[LoadedSource]) -> u64 {
    files.iter().map(|f| f.size()).sum()
}

fn applied_display_name(files: &[LoadedSource]) -> Option<String> {
    match files.len() {
        0 => None,
        1 => Some(files[0].name()),
        n => Some(format!("{n} log files ({}, ...)", files[0].name())),
    }
}

pub fn set_loaded_files(mut store: AnalysisStore, files: Vec<LoadedSource>) -> Vec<LoadedSource> {
    let total = files.len();
    let mut seen: Vec<u64> = Vec::new();
    let mut unique: Vec<LoadedSource> = Vec::new();
    for file in files {
        let size = file.size();
        if !seen.contains(&size) {
            seen.push(size);
            unique.push(file);
        }
    }
    let skipped = total - unique.len();
    if skipped > 0 {
        show_toast(
            store,
            format!(
                "Skipped {skipped} duplicate file{} (identical file size)",
                if skipped > 1 { "s" } else { "" }
            ),
        );
    }
    store.source_kind.set(SourceKind::File);
    store.file_name.set(applied_display_name(&unique));
    store.file_names.set(unique.iter().map(|f| f.name()).collect());
    store.file_size.set(Some(total_size(&unique)));
    store.loaded_files.set(unique.clone());
    store.paste_open.set(false);
    unique
}

pub fn append_loaded_files(mut store: AnalysisStore, files: Vec<LoadedSource>) -> Vec<LoadedSource> {
    let total_incoming = files.len();
    let existing = store.loaded_files();
    let mut seen: Vec<u64> = existing.iter().map(|f| f.size()).collect();
    let mut unique_new: Vec<LoadedSource> = Vec::new();
    for file in files {
        let size = file.size();
        if !seen.contains(&size) {
            seen.push(size);
            unique_new.push(file);
        }
    }
    if unique_new.is_empty() {
        show_toast(
            store,
            "All selected files are already loaded (identical file size)",
        );
        return existing;
    }
    let skipped = total_incoming - unique_new.len();
    if skipped > 0 {
        show_toast(
            store,
            format!(
                "Skipped {skipped} duplicate file{} (already loaded with same size)",
                if skipped > 1 { "s" } else { "" }
            ),
        );
    }
    let mut combined = existing;
    combined.extend(unique_new);
    store.source_kind.set(SourceKind::File);
    store.file_name.set(applied_display_name(&combined));
    store.file_names.set(combined.iter().map(|f| f.name()).collect());
    store.file_size.set(Some(total_size(&combined)));
    store.loaded_files.set(combined.clone());
    store.paste_open.set(false);
    combined
}

pub fn set_source_paste(mut store: AnalysisStore) {
    store.source_kind.set(SourceKind::Paste);
    store.file_name.set(None);
    store.file_names.set(Vec::new());
    store.file_size.set(None);
    store.loaded_files.set(Vec::new());
}

// ── Parse / reaggregate ─────────────────────────────────────────────────────

pub fn handle_log_files_upload(store: AnalysisStore, files: Vec<LoadedSource>, append: bool) {
    if files.is_empty() {
        return;
    }
    let sources = if append {
        append_loaded_files(store, files)
    } else {
        set_loaded_files(store, files)
    };
    if sources.is_empty() {
        return;
    }
    run_parse(store, sources);
}

pub fn parse_text(store: AnalysisStore, text: String) {
    let bytes = text.into_bytes();
    let source = LoadedSource::Memory {
        name: "paste".to_string(),
        bytes: Arc::new(bytes),
    };
    set_source_paste(store);
    run_parse(store, vec![source]);
}

fn run_parse(mut store: AnalysisStore, sources: Vec<LoadedSource>) {
    let total = total_size(&sources);
    let control = Arc::new(JobControl::new(total));
    store.job.set(Some(control.clone()));
    store.is_parsing.set(true);
    store.error.set(None);
    store.progress.set(Some(ParseProgress {
        stage: "parsing".to_string(),
        processed: 0,
        total,
        percent: 0,
    }));

    let mode = store.filters().normalize_mode;
    let opts = parse_options(&store.filters());
    let source_count = sources.len();

    spawn(async move {
        let control_bg = control.clone();
        let mut handle = tokio::task::spawn_blocking(move || {
            parse_sources(&sources, mode, &control_bg)
        });
        let started = std::time::Instant::now();
        loop {
            tokio::select! {
                res = &mut handle => {
                    if control.cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                        return;
                    }
                    match res {
                        Ok(Ok(mut kernel)) => {
                            let elapsed = started.elapsed().as_millis() as u64;
                            let summary_result = kernel.reaggregate(&opts);
                            let matched = summary_result.summary.matched;
                            store.kernel.set(Some(Arc::new(Mutex::new(kernel))));
                            set_result(store, summary_result);
                            store.progress.set(Some(ParseProgress {
                                stage: "complete".to_string(),
                                processed: total,
                                total,
                                percent: 100,
                            }));
                            store.is_parsing.set(false);
                            if source_count > 1 {
                                show_toast(
                                    store,
                                    format!(
                                        "Parsed {} requests across {} files in {}ms",
                                        format_num(matched),
                                        source_count,
                                        elapsed
                                    ),
                                );
                            } else {
                                show_toast(
                                    store,
                                    format!(
                                        "Parsed {} requests in {}ms",
                                        format_num(matched),
                                        elapsed
                                    ),
                                );
                            }
                        }
                        Ok(Err(err)) => {
                            finish_parse_error(store, err);
                        }
                        Err(join_err) => {
                            finish_parse_error(store, ParseError::Io(join_err.to_string()));
                        }
                    }
                    return;
                }
                _ = tokio::time::sleep(Duration::from_millis(120)) => {
                    store.progress.set(Some(ParseProgress {
                        stage: "parsing".to_string(),
                        processed: control.processed.load(std::sync::atomic::Ordering::Relaxed),
                        total,
                        percent: control.percent(),
                    }));
                }
            }
        }
    });
}

fn finish_parse_error(mut store: AnalysisStore, err: ParseError) {
    if matches!(err, ParseError::Cancelled) {
        return;
    }
    let message = err.to_string();
    store.error.set(Some(message.clone()));
    store.is_parsing.set(false);
    show_toast(store, message);
}

pub fn reaggregate(mut store: AnalysisStore) {
    if store.is_parsing() {
        return;
    }
    let Some(kernel) = store.kernel() else {
        set_result(store, crate::core::models::EMPTY_RESULT);
        return;
    };
    let opts = parse_options(&store.filters());
    let epoch = store.reagg_epoch() + 1;
    store.reagg_epoch.set(epoch);
    spawn(async move {
        let res = tokio::task::spawn_blocking(move || {
            let mut guard = kernel.lock().expect("kernel lock");
            guard.reaggregate(&opts)
        })
        .await;
        if let Ok(result) = res {
            if store.reagg_epoch() == epoch {
                set_result(store, result);
            }
        }
    });
}
