//! MongoDB analysis state — mirrors `src/store/mongoStore.ts`.
//!
//! Same shape as the PM2 store: plain data plus synchronous transitions, with
//! the filters/view pair persisted under the reference's `mongo-analyzer-filters`
//! key.

use serde::{Deserialize, Serialize};

use crate::core::mongo_models::{
    MongoAggregationResult, MongoFilters, MongoSlowQuery, MongoUserActivity,
};
use crate::core::pm2::LoadedSource;
use crate::store::analysis_store::{ParseProgress, SourceKind, TOAST_MS};
use crate::utils::persist;

pub const MONGO_FILTERS_KEY: &str = "mongo-analyzer-filters";
pub use crate::store::analysis_store::TOAST_MS as MONGO_TOAST_MS;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MongoChartMode {
    #[default]
    ThroughputLatency,
    Plans,
    TopCollections,
}

impl MongoChartMode {
    pub const ALL: [MongoChartMode; 3] = [
        MongoChartMode::ThroughputLatency,
        MongoChartMode::Plans,
        MongoChartMode::TopCollections,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MongoChartMode::ThroughputLatency => "Queries & P95",
            MongoChartMode::Plans => "COLLSCANs vs IXSCAN",
            MongoChartMode::TopCollections => "Top Slow Collections",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MongoActiveView {
    #[default]
    Patterns,
    SlowQueries,
    Charts,
    Diagnostics,
    Users,
}

impl MongoActiveView {
    pub const ALL: [MongoActiveView; 5] = [
        MongoActiveView::Patterns,
        MongoActiveView::SlowQueries,
        MongoActiveView::Users,
        MongoActiveView::Charts,
        MongoActiveView::Diagnostics,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MongoActiveView::Patterns => "Query Patterns",
            MongoActiveView::SlowQueries => "Slow Query Log",
            MongoActiveView::Charts => "Latency Charts",
            MongoActiveView::Diagnostics => "Diagnostics",
            MongoActiveView::Users => "User Activity",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct PersistedMongo {
    filters: MongoFilters,
    active_view: MongoActiveView,
}

/// `MongoDiagnosticsPanel`'s local tab state.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MongoDiagTab {
    #[default]
    Errors,
    Connections,
    Collections,
    Checkpoints,
}

#[derive(Clone)]
pub struct MongoState {
    pub source_kind: SourceKind,
    pub file_name: Option<String>,
    pub file_names: Vec<String>,
    pub file_size: Option<u64>,
    pub loaded_files: Vec<LoadedSource>,
    pub has_data: bool,
    pub result: Option<MongoAggregationResult>,
    pub is_parsing: bool,
    pub progress: Option<ParseProgress>,
    pub error: Option<String>,
    pub filters: MongoFilters,
    pub toast: Option<String>,
    pub toast_epoch: u64,
    pub paste_open: bool,
    pub active_slow_query: Option<MongoSlowQuery>,
    pub active_user_detail: Option<MongoUserActivity>,
    pub active_view: MongoActiveView,
    pub chart_mode: MongoChartMode,
    pub diag_tab: MongoDiagTab,
    /// Local `MongoUserActivityPanel` search box.
    pub user_search: String,
}

impl Default for MongoState {
    fn default() -> Self {
        Self {
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
            filters: MongoFilters::default(),
            toast: None,
            toast_epoch: 0,
            paste_open: false,
            active_slow_query: None,
            active_user_detail: None,
            active_view: MongoActiveView::default(),
            chart_mode: MongoChartMode::default(),
            diag_tab: MongoDiagTab::default(),
            user_search: String::new(),
        }
    }
}

impl MongoState {
    /// `setSourceFiles` — the header's file label and total size.
    pub fn set_source_files(&mut self, files: &[LoadedSource]) {
        self.source_kind = SourceKind::File;
        self.file_name = Some(display_name(files));
        self.file_names = files.iter().map(LoadedSource::name).collect();
        self.file_size = Some(files.iter().map(source_size).sum());
        self.paste_open = false;
    }

    /// `appendLoadedFiles` — dedupe by size, keep the previous selection.
    pub fn append_loaded_files(&mut self, files: Vec<LoadedSource>) -> bool {
        let existing: Vec<u64> = self.loaded_files.iter().map(source_size).collect();
        let mut unique = Vec::new();
        for file in files {
            let size = source_size(&file);
            if !existing.contains(&size) && !unique.iter().any(|f: &LoadedSource| source_size(f) == size)
            {
                unique.push(file);
            }
        }
        if unique.is_empty() {
            self.show_toast("All selected files are already loaded");
            return false;
        }
        self.loaded_files.extend(unique);
        let combined = self.loaded_files.clone();
        self.set_source_files(&combined);
        self.loaded_files = combined;
        true
    }

    pub fn set_loaded_files(&mut self, files: Vec<LoadedSource>) {
        self.loaded_files = files;
        let current = self.loaded_files.clone();
        self.set_source_files(&current);
        self.loaded_files = current;
    }

    pub fn set_source_paste(&mut self) {
        self.source_kind = SourceKind::Paste;
        self.file_name = None;
        self.file_names.clear();
        self.file_size = None;
        self.loaded_files.clear();
    }

    /// `setResult` — `hasData` keys off the parsed counters.
    pub fn set_result(&mut self, result: Option<MongoAggregationResult>) {
        self.has_data = result
            .as_ref()
            .is_some_and(|r| r.summary.slow_query_count > 0 || r.summary.total_lines > 0);
        self.result = result;
    }

    pub fn show_toast(&mut self, message: impl Into<String>) {
        self.toast_epoch += 1;
        self.toast = Some(message.into());
    }

    pub fn clear_toast(&mut self) {
        self.toast = None;
    }

    pub fn toast_ms(&self) -> u64 {
        TOAST_MS
    }

    pub fn persist(&self) {
        let payload = PersistedMongo {
            filters: self.filters.clone(),
            active_view: self.active_view,
        };
        if let Ok(json) = serde_json::to_string(&payload) {
            persist::set_item(MONGO_FILTERS_KEY, &json);
        }
    }

    pub fn restore_persisted(&mut self) {
        let Some(raw) = persist::load_item(MONGO_FILTERS_KEY) else {
            return;
        };
        if let Ok(payload) = serde_json::from_str::<PersistedMongo>(&raw) {
            self.filters = payload.filters;
            self.active_view = payload.active_view;
        }
    }

    pub fn reset_filters(&mut self) {
        self.filters = MongoFilters::default();
        self.active_slow_query = None;
        self.persist();
    }

    /// `clearAnalysis` — drops data, filters stay.
    pub fn clear(&mut self) {
        self.source_kind = SourceKind::None;
        self.file_name = None;
        self.file_names.clear();
        self.file_size = None;
        self.loaded_files.clear();
        self.has_data = false;
        self.result = None;
        self.progress = None;
        self.error = None;
        self.is_parsing = false;
        self.paste_open = false;
        self.active_slow_query = None;
        self.active_user_detail = None;
    }

    pub fn collections(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .result
            .as_ref()
            .map(|r| {
                r.collections
                    .iter()
                    .map(|c| c.collection.clone())
                    .filter(|name| !name.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names.dedup();
        names
    }
}

fn display_name(files: &[LoadedSource]) -> String {
    match files {
        [] => String::new(),
        [one] => one.name(),
        [first, ..] => format!("{} log files ({}, ...)", files.len(), first.name()),
    }
}

fn source_size(source: &LoadedSource) -> u64 {
    match source {
        LoadedSource::Path(path) => std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
        LoadedSource::Memory { bytes, .. } => bytes.len() as u64,
    }
}
