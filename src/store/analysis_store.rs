//! PM2 analysis state — mirrors `src/store/analysisStore.ts` (Zustand) and the
//! module-level actions in `src/hooks/useParserWorker.ts`.
//!
//! Plain data plus synchronous transitions; asynchronous orchestration (parse
//! jobs, reaggregation) lives in [`crate::app`].

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::core::models::{AggregatedResult, NormalizeMode, ParseOptions, StatusFamily};
use crate::core::pm2::{JobControl, LoadedSource, Pm2Kernel};
use crate::utils::persist;

pub const PASTE_WARN_BYTES: usize = 8 * 1024 * 1024;
pub const DEFAULT_TOP_N: usize = 50;
pub const TOAST_MS: u64 = 3200;

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

/// All analysis state shared by the UI, matching the Zustand reference store.
pub struct AnalysisState {
    pub theme: Theme,
    pub chart_layout: ChartLayout,
    pub source_kind: SourceKind,
    pub file_name: Option<String>,
    pub file_names: Vec<String>,
    pub file_size: Option<u64>,
    pub loaded_files: Vec<LoadedSource>,
    pub has_data: bool,
    pub result: Option<AggregatedResult>,
    pub is_parsing: bool,
    pub progress: Option<ParseProgress>,
    pub error: Option<String>,
    pub filters: AnalysisFilters,
    pub toast: Option<String>,
    pub toast_epoch: u64,
    pub paste_open: bool,
    pub kernel: Option<Arc<Mutex<Pm2Kernel>>>,
    pub job: Option<Arc<JobControl>>,
}

impl AnalysisState {
    pub fn new() -> Self {
        Self {
            theme: Theme::Light,
            chart_layout: ChartLayout::Split,
            source_kind: SourceKind::None,
            file_name: None,
            file_names: Vec::new(),
            file_size: None,
            loaded_files: Vec::new(),
            has_data: false,
            result: None,
            is_parsing: false,
            progress: None,
            error: None,
            filters: AnalysisFilters::default(),
            toast: None,
            toast_epoch: 0,
            paste_open: false,
            kernel: None,
            job: None,
        }
    }

    pub fn is_dark(&self) -> bool {
        self.theme == Theme::Dark
    }

    pub fn summary(&self) -> Option<crate::core::models::LogSummary> {
        self.result.as_ref().map(|r| r.summary.clone())
    }

    pub fn has_cron_events(&self) -> bool {
        self.result
            .as_ref()
            .map(|r| {
                let c = &r.cron_summary;
                c.starts + c.dones + c.fails > 0
            })
            .unwrap_or(false)
    }

    pub fn show_toast(&mut self, message: impl Into<String>) {
        self.toast_epoch += 1;
        self.toast = Some(message.into());
    }

    pub fn clear_toast(&mut self) {
        self.toast = None;
    }

    pub fn set_result(&mut self, result: AggregatedResult) {
        let cron = result.cron_summary.clone();
        self.has_data = result.summary.matched > 0
            || cron.starts + cron.dones + cron.fails > 0
            || result.unmatched_count > 0;
        self.result = Some(result);
    }

    pub fn clear(&mut self) {
        if let Some(job) = &self.job {
            job.cancel();
        }
        self.source_kind = SourceKind::None;
        self.file_name = None;
        self.file_names = Vec::new();
        self.file_size = None;
        self.loaded_files = Vec::new();
        self.has_data = false;
        self.result = None;
        self.progress = None;
        self.error = None;
        self.is_parsing = false;
        self.paste_open = false;
        self.kernel = None;
        self.job = None;
    }

    /// Cancel the running parse; a late finish is ignored by job identity.
    pub fn cancel(&mut self) {
        if let Some(job) = &self.job {
            job.cancel();
        }
        self.is_parsing = false;
    }

    /// Replace the loaded file set (`setLoadedFiles` parity).
    pub fn set_loaded_files(&mut self, files: Vec<LoadedSource>) -> Vec<LoadedSource> {
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
            self.show_toast(format!(
                "Skipped {skipped} duplicate file{} (identical file size)",
                if skipped > 1 { "s" } else { "" }
            ));
        }
        self.source_kind = SourceKind::File;
        self.file_name = applied_display_name(&unique);
        self.file_names = unique.iter().map(|f| f.name()).collect();
        self.file_size = Some(total_size(&unique));
        self.loaded_files = unique.clone();
        self.paste_open = false;
        unique
    }

    /// Append to the loaded file set (`appendLoadedFiles` parity).
    pub fn append_loaded_files(&mut self, files: Vec<LoadedSource>) -> Vec<LoadedSource> {
        let total_incoming = files.len();
        let mut seen: Vec<u64> = self.loaded_files.iter().map(|f| f.size()).collect();
        let mut unique_new: Vec<LoadedSource> = Vec::new();
        for file in files {
            let size = file.size();
            if !seen.contains(&size) {
                seen.push(size);
                unique_new.push(file);
            }
        }
        if unique_new.is_empty() {
            self.show_toast("All selected files are already loaded (identical file size)");
            return self.loaded_files.clone();
        }
        let skipped = total_incoming - unique_new.len();
        if skipped > 0 {
            self.show_toast(format!(
                "Skipped {skipped} duplicate file{} (already loaded with same size)",
                if skipped > 1 { "s" } else { "" }
            ));
        }
        self.loaded_files.extend(unique_new);
        self.source_kind = SourceKind::File;
        self.file_name = applied_display_name(&self.loaded_files);
        self.file_names = self.loaded_files.iter().map(|f| f.name()).collect();
        self.file_size = Some(total_size(&self.loaded_files));
        self.paste_open = false;
        self.loaded_files.clone()
    }

    pub fn set_source_paste(&mut self) {
        self.source_kind = SourceKind::Paste;
        self.file_name = None;
        self.file_names = Vec::new();
        self.file_size = None;
        self.loaded_files = Vec::new();
    }
}

impl Default for AnalysisState {
    fn default() -> Self {
        Self::new()
    }
}

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

// ── Persistence ─────────────────────────────────────────────────────────────

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

pub fn persist(state: &AnalysisState) {
    let json = persisted_json(&state.filters, state.theme, state.chart_layout);
    persist::set_item("pm2-analyzer-filters", &json);
}

/// Load persisted filters/theme/layout, restoring them into `state`.
pub fn restore_persisted(state: &mut AnalysisState) {
    let Some(raw) = persist::load_item("pm2-analyzer-filters") else {
        return;
    };
    if let Ok(envelope) = serde_json::from_str::<PersistedEnvelope>(&raw) {
        state.filters = envelope.state.filters;
        state.theme = envelope.state.theme;
        state.chart_layout = envelope.state.chart_layout;
    }
}

// ── Filters ─────────────────────────────────────────────────────────────────

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
