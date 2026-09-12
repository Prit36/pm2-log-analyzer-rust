//! App shell — port of `src/App.tsx` + `src/components/Pm2AppView.tsx`.
//!
//! iced is an Elm-style runtime: [`App`] holds every piece of state, [`Message`]
//! describes every interaction, and the parse/reaggregate jobs stream their
//! progress back through [`Task::run`] instead of through UI-thread polling.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::widget::{column, container, operation, row, scrollable, svg, text_editor};
use iced::{clipboard, keyboard, window, Element, Fill, Subscription, Task};

use crate::core::models::{
    AggregatedEndpoint, AggregatedResult, CronAggregated, NormalizeMode, ParseOptions, StatusFamily,
    EMPTY_RESULT,
};
use crate::core::mongo::{parse_paths as parse_mongo_paths, MongoKernel};
use crate::core::mongo_models::{
    MongoAggregationResult, MongoFilters, MongoPlanFilter, MongoSlowQuery, MongoSlowQuerySortField,
    MongoSortDirection, MongoSortField, MongoUserActivity,
};
use crate::core::pm2::{
    parse_sources, JobControl, LoadedSource, ParseError, Pm2Kernel,
};
use crate::ui::charts::ChartMode;
use crate::store::analysis_store::{
    parse_options, persist, restore_persisted, AnalysisFilters, AnalysisState, ApiSortKey,
    ChartLayout, CronSortKey, ParseProgress, SortDirection, SourceKind, Theme, PASTE_WARN_BYTES,
    TOAST_MS,
};
use crate::store::app_mode_store::{persist_mode, restore_mode, AppMode};
use crate::store::mongo_store::{MongoActiveView, MongoState};
use crate::ui::{
    api_table, charts, cron_table, filters, header, ingest, kpi, mongo, skipped, style, toast,
};
use crate::utils::export_spreadsheet::{
    self as export_spreadsheet, ExportData,
};
use crate::utils::export_mongo_spreadsheet;use crate::utils::format::{format_bytes, format_num};
use crate::utils::table_ops::{
    build_api_tsv, build_cron_tsv, filter_api_endpoints, sort_api_endpoints, sort_cron_jobs,
};

/// Text input focused by the `/` shortcut (see the reference `FilterBar`).
pub const SEARCH_INPUT_ID: &str = "filter-search";
/// Multi-line paste editor id, used to avoid stealing focus while typing.
pub const PASTE_EDITOR_ID: &str = "paste-logs";

const PROGRESS_TICK: Duration = Duration::from_millis(120);
/// A multi-file drop arrives as one `FileDropped` per file; wait for the batch
/// to settle so it is loaded exactly like the browser's single `drop` event.
const DROP_SETTLE: Duration = Duration::from_millis(120);

pub struct App {
    pub analysis: AnalysisState,
    pub mongo: MongoState,
    /// Parsed MongoDB kernel — reaggregation is microseconds, so filters run
    /// synchronously instead of round-tripping a worker.
    mongo_kernel: Option<Arc<MongoKernel>>,
    mongo_job: Option<Arc<JobControl>>,
    pub mongo_paste_text: text_editor::Content,
    pub mongo_scroll: f32,
    pub mongo_slow_scroll: f32,
    pub mongo_copied_index: Option<String>,
    mongo_toast_epoch: u64,
    pub mode: AppMode,
    pub window_width: f32,

    /// Filtered + sorted rows derived from `analysis.result` and `filters`.
    pub api_rows: Vec<AggregatedEndpoint>,
    pub cron_rows: Vec<CronAggregated>,
    rows_dirty: bool,
    rows_epoch: u64,
    result_epoch: u64,

    /// Chart SVG cache; regenerated only when its inputs change.
    pub chart: Option<svg::Handle>,
    chart_key: Option<ChartKey>,
    pub chart_mode: ChartMode,
    /// Category index under the cursor — drives the chart tooltip overlay.
    pub chart_hover: Option<usize>,
    /// MongoDB chart mode + hover (separate from the PM2 chart).
    pub mongo_chart_mode: crate::store::mongo_store::MongoChartMode,
    pub mongo_chart_hover: Option<usize>,

    pub drag_over: bool,
    dropped_files: Vec<LoadedSource>,
    pub pending_drop: Option<Vec<LoadedSource>>,
    pub paste_text: text_editor::Content,
    pub min_ms_input: String,
    pub top_n_input: String,
    pub cron_min_ms_input: String,
    pub skipped_open: bool,

    pub api_scroll: f32,
    pub cron_scroll: f32,
    /// Row/header under the cursor — the reference reveals copy and sort
    /// affordances on `group-hover`.
    pub api_hover: Option<usize>,
    pub api_header_hover: Option<ApiSortKey>,
    pub cron_header_hover: Option<CronSortKey>,

    reagg_epoch: u64,
    /// Bumped per toast so a superseded timer expires nothing.
    toast_epoch: u64,
    /// Bumped per drop event so only the last file of a batch commits.
    drop_epoch: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ChartKey {
    mode: ChartMode,
    dark: bool,
    width: u32,
    height: u32,
    rows: u64,
    data: u64,
}

#[derive(Clone)]
pub enum Message {
    // Ingest
    Browse { append: bool },
    Autoload(PathBuf),
    FileHovered(PathBuf),
    FileDropped(PathBuf),
    FilesHoveredLeft,
    CommitDropped(u64),
    PendingDropAppend,
    PendingDropReplace,
    PendingDropCancel,
    TogglePaste,
    PasteEdit(text_editor::Action),
    PasteAnalyze,
    CancelParse,
    ClearAll,
    ArchiveExtracted {
        result: Result<crate::core::archive::ExtractedArchive, String>,
        append: bool,
    },

    // Filters
    QueryChanged(String),
    NormalizeChanged(NormalizeMode),
    StatusChanged(StatusFamily),
    MinMsChanged(String),
    TopNChanged(String),
    ApiSortKeyChanged(ApiSortKey),
    ApiSortToggled(ApiSortKey),
    MethodChipToggled(Option<String>),
    DateFilterChanged(String),
    ResetFilters,
    CronQueryChanged(String),
    CronMinMsChanged(String),
    CronFailedOnly(bool),
    CronSortKeyChanged(CronSortKey),
    CronSortToggled(CronSortKey),

    // Tables
    ApiScrolled(scrollable::Viewport),
    CronScrolled(scrollable::Viewport),
    ApiRowHovered(Option<usize>),
    ApiHeaderHovered(Option<ApiSortKey>),
    CronHeaderHovered(Option<CronSortKey>),
    CopyApiTsv,
    CopyCronTsv,
    CopyPath(String),

    // Charts
    ChartMode(ChartMode),
    ChartHover(Option<usize>),
    ToggleChartLayout,

    // Chrome
    ToggleTheme,
    SetMode(AppMode),
    ToggleSkipped,
    ExportPm2,
    ExportMongo,
    ExportFinished(Result<bool, String>),
    Toast(String),
    ToastExpired(u64),

    // MongoDB
    MongoBrowse { append: bool },
    MongoPasteToggle,
    MongoPasteEdit(text_editor::Action),
    MongoPasteAnalyze,
    MongoCancel,
    MongoClear,
    MongoActiveView(MongoActiveView),
    MongoDiagTab(crate::store::mongo_store::MongoDiagTab),
    MongoUserSearchChanged(String),
    MongoSearchChanged(String),
    MongoPlanFilter(MongoPlanFilter),
    MongoOperationChanged(String),
    MongoCollectionChanged(String),
    MongoUserChanged(String),
    MongoMinDuration(u32),
    MongoScanRatioToggled,
    MongoResetFilters,
    MongoSortToggled(MongoSortField),
    MongoSlowSortToggled(MongoSlowQuerySortField),
    MongoOpenSlowQuery(Option<Box<MongoSlowQuery>>),
    MongoOpenUser(Option<Box<MongoUserActivity>>),
    MongoCopyIndex(String),
    MongoChartMode(crate::store::mongo_store::MongoChartMode),
    MongoChartHover(Option<usize>),
    MongoToast(String),
    MongoToastExpired(u64),
    MongoScrolled(scrollable::Viewport),
    MongoSlowScrolled(scrollable::Viewport),
    MongoJob(MongoJobEvent),

    // Async jobs
    Job(JobEvent),
    Reaggregated {
        epoch: u64,
        result: Box<AggregatedResult>,
    },

    // Keyboard
    WindowResized(f32),
    SlashPressed,
    SlashPasteProbe(bool),
    SlashSearchProbe(bool),
}

/// Events streamed from the parse worker thread.
#[derive(Clone)]
pub enum JobEvent {
    Progress {
        control: Arc<JobControl>,
        processed: u64,
        total: u64,
        percent: u32,
    },
    Finished {
        control: Arc<JobControl>,
        job: Box<FinishedJob>,
    },
}

/// Outcome of a parse + first reaggregation, ready to be installed in the app.
#[derive(Clone)]
pub struct FinishedJob {
    pub kernel: Option<Arc<Mutex<Pm2Kernel>>>,
    pub result: Option<AggregatedResult>,
    pub elapsed_ms: u64,
    pub source_count: usize,
    pub total: u64,
    pub error: Option<String>,
}

/// Events streamed from the MongoDB parse worker thread.
#[derive(Clone)]
pub enum MongoJobEvent {
    Progress {
        control: Arc<JobControl>,
        percent: u32,
    },
    Finished(Box<MongoFinishedJob>),
}

#[derive(Clone)]
pub struct MongoFinishedJob {
    pub control: Arc<JobControl>,
    pub kernel: Option<Arc<MongoKernel>>,
    pub result: Option<MongoAggregationResult>,
    pub elapsed_ms: u64,
    pub source_count: usize,
    pub error: Option<String>,
}

impl App {
    // ── Boot ────────────────────────────────────────────────────────────────

    pub fn boot() -> (Self, Task<Message>) {
        let mut analysis = AnalysisState::new();
        restore_persisted(&mut analysis);
        let mode = restore_mode().unwrap_or(AppMode::Pm2);
        let mut mongo = MongoState::default();
        mongo.restore_persisted();

        let mut app = Self {
            analysis,
            mongo,
            mongo_kernel: None,
            mongo_job: None,
            mongo_paste_text: text_editor::Content::new(),
            mongo_scroll: 0.0,
            mongo_slow_scroll: 0.0,
            mongo_copied_index: None,
            mongo_toast_epoch: 0,
            mode,
            window_width: 1440.0,
            api_rows: Vec::new(),
            cron_rows: Vec::new(),
            rows_dirty: true,
            rows_epoch: 0,
            result_epoch: 0,
            chart: None,
            chart_key: None,
            chart_mode: ChartMode::TimeOfDay,
            chart_hover: None,
            mongo_chart_mode: crate::store::mongo_store::MongoChartMode::default(),
            mongo_chart_hover: None,
            drag_over: false,
            dropped_files: Vec::new(),
            pending_drop: None,
            paste_text: text_editor::Content::new(),
            min_ms_input: String::new(),
            top_n_input: String::new(),
            cron_min_ms_input: String::new(),
            skipped_open: false,
            api_scroll: 0.0,
            cron_scroll: 0.0,
            api_hover: None,
            api_header_hover: None,
            cron_header_hover: None,
            reagg_epoch: 0,
            toast_epoch: 0,
            drop_epoch: 0,
        };
        app.sync_filter_inputs();
        app.refresh_rows();
        app.sync_chart();

        // Test hooks: load a log file at startup for either mode.
        let task = if let Ok(path) = std::env::var("PM2_ANALYZER_AUTOLOAD_MONGO") {
            app.mode = AppMode::Mongo;
            app.start_mongo_parse(vec![LoadedSource::Path(PathBuf::from(path))])
        } else if let Ok(path) = std::env::var("PM2_ANALYZER_AUTOLOAD") {
            Task::done(Message::Autoload(PathBuf::from(path)))
        } else {
            Task::none()
        };
        (app, task)
    }

    // ── Elm plumbing ────────────────────────────────────────────────────────

    pub fn title(&self) -> String {
        match self.mode {
            AppMode::Pm2 => "PM2 Log Analyzer".to_string(),
            AppMode::Mongo => "MongoDB Log Analyzer".to_string(),
        }
    }

    pub fn theme(&self) -> iced::Theme {
        if self.analysis.is_dark() {
            iced::Theme::Dark
        } else {
            iced::Theme::Light
        }
    }

    pub fn style(&self, theme: &iced::Theme) -> iced::theme::Style {
        style::app_style(theme)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::events().filter_map(|(_id, event)| match event {
                window::Event::Resized(size) => Some(Message::WindowResized(size.width)),
                window::Event::FileHovered(path) => Some(Message::FileHovered(path)),
                window::Event::FileDropped(path) => Some(Message::FileDropped(path)),
                window::Event::FilesHoveredLeft => Some(Message::FilesHoveredLeft),
                _ => None,
            }),
            keyboard::listen().filter_map(|event| match event {
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(character),
                    modifiers,
                    repeat: false,
                    ..
                } if character.as_str() == "/"
                    && !modifiers.control()
                    && !modifiers.alt()
                    && !modifiers.command() =>
                {
                    Some(Message::SlashPressed)
                }
                _ => None,
            }),
        ])
    }

    /// Toast notification sent to both PM2 and Mongo stores so whichever mode is
    /// active (or switched to) receives the alert.
    pub fn notify(&mut self, message: impl Into<String>) {
        let msg = message.into();
        self.analysis.show_toast(msg.clone());
        self.mongo.show_toast(msg);
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let task = match message {
            // ── Ingest ──────────────────────────────────────────────────────
            Message::Browse { append } => {
                if self.analysis.is_parsing {
                    Task::none()
                } else {
                    let files = ingest::pick_files();
                    if files.is_empty() {
                        Task::none()
                    } else {
                        let use_append = append
                            && self.analysis.has_data
                            && !self.analysis.loaded_files.is_empty();
                        self.route_upload(files, use_append)
                    }
                }
            }
            Message::Autoload(path) => {
                self.route_upload(vec![LoadedSource::Path(path)], false)
            }
            Message::FileHovered(_) => {
                self.drag_over = true;
                Task::none()
            }
            Message::FileDropped(path) => {
                if ingest::is_valid_file_path(&path) {
                    self.dropped_files.push(LoadedSource::Path(path));
                    // A batch arrives as one message per file; only the newest
                    // timer commits, so the whole drop loads in one pass.
                    self.drop_epoch += 1;
                    let epoch = self.drop_epoch;
                    Task::perform(
                        async move { tokio::time::sleep(DROP_SETTLE).await },
                        move |()| Message::CommitDropped(epoch),
                    )
                } else {
                    self.analysis.show_toast(
                        "Please upload log, text, or archive files (.log, .zip, .gz, .txt, etc.)",
                    );
                    Task::none()
                }
            }
            Message::FilesHoveredLeft => {
                self.drag_over = false;
                self.commit_drop()
            }
            Message::CommitDropped(epoch) => {
                if epoch == self.drop_epoch {
                    self.drag_over = false;
                    self.commit_drop()
                } else {
                    Task::none()
                }
            }
            Message::PendingDropAppend => match self.pending_drop.take() {
                Some(files) => self.route_upload(files, true),
                None => Task::none(),
            },
            Message::PendingDropReplace => match self.pending_drop.take() {
                Some(files) => self.route_upload(files, false),
                None => Task::none(),
            },
            Message::PendingDropCancel => {
                self.pending_drop = None;
                Task::none()
            }
            Message::TogglePaste => {
                self.analysis.paste_open = !self.analysis.paste_open;
                Task::none()
            }
            Message::PasteEdit(action) => {
                self.paste_text.perform(action);
                Task::none()
            }
            Message::PasteAnalyze => self.analyze_paste(),
            Message::CancelParse => {
                self.analysis.cancel();
                Task::none()
            }
            Message::ClearAll => {
                self.analysis.clear();
                self.pending_drop = None;
                self.dropped_files.clear();
                self.skipped_open = false;
                self.api_scroll = 0.0;
                self.cron_scroll = 0.0;
                self.mark_rows_dirty();
                Task::none()
            }
            Message::ArchiveExtracted { result, append } => {
                self.analysis.is_parsing = false;
                self.analysis.progress = None;
                self.mongo.is_parsing = false;
                self.mongo.progress = None;
                match result {
                    Ok(log_set) => {
                        let mut pm2_files = log_set.pm2_sources;
                        let mut mongo_files = log_set.mongo_sources;
                        for unk in log_set.unknown_sources {
                            if self.mode == AppMode::Mongo {
                                mongo_files.push(unk);
                            } else {
                                pm2_files.push(unk);
                            }
                        }
                        if pm2_files.is_empty() && mongo_files.is_empty() {
                            self.notify("No valid API or MongoDB logs found in archive");
                            return Task::none();
                        }

                        let has_pm2 = !pm2_files.is_empty();
                        let has_mongo = !mongo_files.is_empty();
                        let pm2_count = pm2_files.len();
                        let mongo_count = mongo_files.len();
                        let duration_ms = log_set.duration_ms;

                        let pm2_task = if has_pm2 {
                            let sources = if append {
                                self.analysis.append_loaded_files(pm2_files)
                            } else {
                                self.analysis.set_loaded_files(pm2_files)
                            };
                            if !sources.is_empty() {
                                self.start_parse(sources)
                            } else {
                                Task::none()
                            }
                        } else {
                            Task::none()
                        };

                        let mongo_task = if has_mongo {
                            let sources = if append {
                                if self.mongo.append_loaded_files(mongo_files) {
                                    self.mongo.loaded_files.clone()
                                } else {
                                    Vec::new()
                                }
                            } else {
                                self.mongo.set_loaded_files(mongo_files);
                                self.mongo.loaded_files.clone()
                            };
                            if !sources.is_empty() {
                                self.start_mongo_parse(sources)
                            } else {
                                Task::none()
                            }
                        } else {
                            Task::none()
                        };

                        if has_pm2 && has_mongo {
                            self.notify(format!(
                                "Extracted {pm2_count} API log(s) and {mongo_count} MongoDB log(s) in {duration_ms}ms! Both tabs populated."
                            ));
                        } else if has_mongo {
                            self.mode = AppMode::Mongo;
                            persist_mode(self.mode);
                            self.notify(format!(
                                "Extracted {mongo_count} MongoDB log(s) in {duration_ms}ms into MongoDB Analyzer"
                            ));
                        } else {
                            self.mode = AppMode::Pm2;
                            persist_mode(self.mode);
                            self.notify(format!(
                                "Extracted {pm2_count} API log(s) in {duration_ms}ms into PM2 Analyzer"
                            ));
                        }

                        Task::batch([pm2_task, mongo_task])
                    }
                    Err(err) => {
                        self.notify(format!("Extraction failed: {err}"));
                        Task::none()
                    }
                }
            }

            // ── Filters ─────────────────────────────────────────────────────
            Message::QueryChanged(value) => {
                let mut filters = self.analysis.filters.clone();
                filters.query = value;
                self.edit_filters(filters, false)
            }
            Message::NormalizeChanged(mode) => {
                let mut filters = self.analysis.filters.clone();
                filters.normalize_mode = mode;
                self.edit_filters(filters, true)
            }
            Message::StatusChanged(family) => {
                let mut filters = self.analysis.filters.clone();
                filters.status_family = family;
                self.edit_filters(filters, true)
            }
            Message::MinMsChanged(value) => {
                self.min_ms_input = value.clone();
                let mut filters = self.analysis.filters.clone();
                filters.min_ms = value.parse::<f64>().unwrap_or(0.0).max(0.0);
                self.edit_filters(filters, true)
            }
            Message::TopNChanged(value) => {
                self.top_n_input = value.clone();
                let mut filters = self.analysis.filters.clone();
                filters.top_n = value.parse::<usize>().unwrap_or(50).clamp(1, 500);
                self.edit_filters(filters, false)
            }
            Message::ApiSortKeyChanged(key) => {
                let mut filters = self.analysis.filters.clone();
                filters.sort_key = key;
                self.edit_filters(filters, false)
            }
            Message::ApiSortToggled(key) => {
                let mut filters = self.analysis.filters.clone();
                if filters.sort_key == key {
                    filters.sort_dir = if filters.sort_dir == SortDirection::Asc {
                        SortDirection::Desc
                    } else {
                        SortDirection::Asc
                    };
                } else {
                    filters.sort_key = key;
                    filters.sort_dir = SortDirection::Desc;
                }
                self.edit_filters(filters, false)
            }
            Message::MethodChipToggled(method) => {
                let mut filters = self.analysis.filters.clone();
                match method {
                    None => filters.methods.clear(),
                    Some(method) => {
                        let all_selected = filters.methods.is_empty();
                        if all_selected {
                            filters.methods = vec![method];
                        } else if let Some(index) =
                            filters.methods.iter().position(|m| *m == method)
                        {
                            filters.methods.remove(index);
                        } else {
                            filters.methods.push(method);
                        }
                    }
                }
                self.edit_filters(filters, false)
            }
            Message::DateFilterChanged(date) => {
                let mut filters = self.analysis.filters.clone();
                filters.date_filter = date;
                self.edit_filters(filters, true)
            }
            Message::ResetFilters => {
                self.analysis.filters = AnalysisFilters::default();
                self.sync_filter_inputs();
                persist(&self.analysis);
                self.rows_dirty = true;
                self.reaggregate().unwrap_or_else(Task::none)
            }
            Message::CronQueryChanged(value) => {
                let mut filters = self.analysis.filters.clone();
                filters.cron_query = value;
                self.edit_filters(filters, true)
            }
            Message::CronMinMsChanged(value) => {
                self.cron_min_ms_input = value.clone();
                let mut filters = self.analysis.filters.clone();
                filters.cron_min_ms = value.parse::<f64>().unwrap_or(0.0).max(0.0);
                self.edit_filters(filters, true)
            }
            Message::CronFailedOnly(value) => {
                let mut filters = self.analysis.filters.clone();
                filters.cron_show_failed_only = value;
                self.edit_filters(filters, true)
            }
            Message::CronSortKeyChanged(key) => {
                let mut filters = self.analysis.filters.clone();
                filters.cron_sort_key = key;
                filters.cron_sort_dir = if key == CronSortKey::Name {
                    SortDirection::Asc
                } else {
                    SortDirection::Desc
                };
                self.edit_filters(filters, false)
            }
            Message::CronSortToggled(key) => {
                let mut filters = self.analysis.filters.clone();
                if filters.cron_sort_key == key {
                    filters.cron_sort_dir = if filters.cron_sort_dir == SortDirection::Asc {
                        SortDirection::Desc
                    } else {
                        SortDirection::Asc
                    };
                } else {
                    filters.cron_sort_key = key;
                    filters.cron_sort_dir = if key == CronSortKey::Name {
                        SortDirection::Asc
                    } else {
                        SortDirection::Desc
                    };
                }
                self.edit_filters(filters, false)
            }

            // ── Tables / clipboard ──────────────────────────────────────────
            Message::ApiScrolled(viewport) => {
                self.api_scroll = viewport.absolute_offset().y;
                Task::none()
            }
            Message::CronScrolled(viewport) => {
                self.cron_scroll = viewport.absolute_offset().y;
                Task::none()
            }
            Message::ApiRowHovered(index) => {
                self.api_hover = index;
                Task::none()
            }
            Message::ApiHeaderHovered(key) => {
                self.api_header_hover = key;
                Task::none()
            }
            Message::CronHeaderHovered(key) => {
                self.cron_header_hover = key;
                Task::none()
            }
            Message::CopyApiTsv => {
                if self.api_rows.is_empty() {
                    Task::none()
                } else {
                    Task::batch([
                        clipboard::write(build_api_tsv(&self.api_rows)),
                        Task::done(Message::Toast(
                            "API table copied — paste into Excel".to_string(),
                        )),
                    ])
                }
            }
            Message::CopyCronTsv => {
                if self.cron_rows.is_empty() {
                    Task::none()
                } else {
                    Task::batch([
                        clipboard::write(build_cron_tsv(&self.cron_rows)),
                        Task::done(Message::Toast(
                            "Cron table copied — paste into Excel".to_string(),
                        )),
                    ])
                }
            }
            Message::CopyPath(path) => Task::batch([
                clipboard::write(path),
                Task::done(Message::Toast("Path copied".to_string())),
            ]),

            // ── Charts ──────────────────────────────────────────────────────
            Message::ChartMode(mode) => {
                self.chart_mode = mode;
                self.chart_hover = None;
                Task::none()
            }
            Message::ChartHover(index) => {
                self.chart_hover = index;
                Task::none()
            }
            Message::ToggleChartLayout => {
                self.analysis.chart_layout = if self.analysis.chart_layout == ChartLayout::Split {
                    ChartLayout::Wide
                } else {
                    ChartLayout::Split
                };
                persist(&self.analysis);
                Task::none()
            }

            // ── Chrome ──────────────────────────────────────────────────────
            Message::ToggleTheme => {
                self.analysis.theme = if self.analysis.is_dark() {
                    Theme::Light
                } else {
                    Theme::Dark
                };
                persist(&self.analysis);
                Task::none()
            }
            Message::SetMode(mode) => {
                self.mode = mode;
                persist_mode(mode);
                Task::none()
            }
            Message::ToggleSkipped => {
                self.skipped_open = !self.skipped_open;
                Task::none()
            }
            Message::ExportPm2 => self.export_pm2(),
            Message::ExportMongo => self.export_mongo(),
            Message::ExportFinished(result) => {
                match (&result, self.mode) {
                    (Ok(true), AppMode::Mongo) => self.mongo.show_toast(
                        "Excel downloaded — Patterns, Slow Queries, Collections, Diagnostics, Users",
                    ),
                    (Err(_), AppMode::Mongo) => self.mongo.show_toast("Excel export failed"),
                    (Ok(true), AppMode::Pm2) => self
                        .analysis
                        .show_toast("Excel downloaded — Visual Analytics + Data sheets"),
                    (Ok(false), AppMode::Pm2) => self
                        .analysis
                        .show_toast("Excel downloaded — Visual Analytics + API sheets"),
                    (Err(_), AppMode::Pm2) => self.analysis.show_toast("Excel export failed"),
                    (Ok(false), AppMode::Mongo) => self
                        .mongo
                        .show_toast("Excel downloaded — Visual Analytics + API sheets"),
                }
                Task::none()
            }
            Message::Toast(message) => {
                self.analysis.show_toast(message);
                Task::none()
            }
            Message::ToastExpired(epoch) => {
                if epoch == self.analysis.toast_epoch {
                    self.analysis.clear_toast();
                }
                Task::none()
            }

            // ── MongoDB ─────────────────────────────────────────────────────
            Message::MongoBrowse { append } => {
                if self.mongo.is_parsing {
                    Task::none()
                } else {
                    let files = ingest::pick_files();
                    if files.is_empty() {
                        Task::none()
                    } else {
                        let use_append = append
                            && self.mongo.has_data
                            && !self.mongo.loaded_files.is_empty();
                        self.route_upload(files, use_append)
                    }
                }
            }
            Message::MongoPasteToggle => {
                self.mongo.paste_open = !self.mongo.paste_open;
                Task::none()
            }
            Message::MongoPasteEdit(action) => {
                self.mongo_paste_text.perform(action);
                Task::none()
            }
            Message::MongoPasteAnalyze => {
                let text = self.mongo_paste_text.text().to_string();
                if text.trim().is_empty() {
                    Task::none()
                } else {
                    self.mongo.set_source_paste();
                    self.mongo.paste_open = false;
                    self.mongo_paste_text = text_editor::Content::new();
                    let sources = vec![LoadedSource::Memory {
                        name: "pasted-mongodb-logs".to_string(),
                        bytes: Arc::new(text.into_bytes()),
                    }];
                    self.start_mongo_parse(sources)
                }
            }
            Message::MongoCancel => {
                if let Some(control) = &self.mongo_job {
                    control.cancel();
                }
                self.mongo.is_parsing = false;
                Task::none()
            }
            Message::MongoClear => {
                self.mongo.clear();
                self.mongo_kernel = None;
                self.mongo_job = None;
                self.mongo_scroll = 0.0;
                self.mongo_slow_scroll = 0.0;
                Task::none()
            }
            Message::MongoActiveView(view) => {
                self.mongo.active_view = view;
                self.mongo.persist();
                Task::none()
            }
            Message::MongoDiagTab(tab) => {
                self.mongo.diag_tab = tab;
                Task::none()
            }
            Message::MongoUserSearchChanged(value) => {
                self.mongo.user_search = value;
                Task::none()
            }
            Message::MongoSearchChanged(value) => {
                self.mongo.filters.search_query = value;
                self.apply_mongo_filters()
            }
            Message::MongoPlanFilter(plan) => {
                self.mongo.filters.plan_filter = plan;
                self.apply_mongo_filters()
            }
            Message::MongoOperationChanged(operation) => {
                self.mongo.filters.operation = operation;
                self.apply_mongo_filters()
            }
            Message::MongoCollectionChanged(collection) => {
                self.mongo.filters.collection = collection;
                self.apply_mongo_filters()
            }
            Message::MongoUserChanged(user) => {
                self.mongo.filters.user_filter = user;
                self.apply_mongo_filters()
            }
            Message::MongoMinDuration(ms) => {
                self.mongo.filters.min_duration_ms = ms;
                self.apply_mongo_filters()
            }
            Message::MongoScanRatioToggled => {
                self.mongo.filters.high_scan_ratio_only = !self.mongo.filters.high_scan_ratio_only;
                self.apply_mongo_filters()
            }
            Message::MongoResetFilters => {
                self.mongo.reset_filters();
                self.apply_mongo_filters()
            }
            Message::MongoSortToggled(field) => {
                let filters = &mut self.mongo.filters;
                if filters.sort_field == field {
                    filters.sort_direction = match filters.sort_direction {
                        MongoSortDirection::Asc => MongoSortDirection::Desc,
                        MongoSortDirection::Desc => MongoSortDirection::Asc,
                    };
                } else {
                    filters.sort_field = field;
                    filters.sort_direction = MongoSortDirection::Desc;
                }
                self.mongo.persist();
                Task::none()
            }
            Message::MongoSlowSortToggled(field) => {
                let filters = &mut self.mongo.filters;
                if filters.slow_sort_field == field {
                    filters.slow_sort_direction = match filters.slow_sort_direction {
                        MongoSortDirection::Asc => MongoSortDirection::Desc,
                        MongoSortDirection::Desc => MongoSortDirection::Asc,
                    };
                } else {
                    filters.slow_sort_field = field;
                    filters.slow_sort_direction = MongoSortDirection::Desc;
                }
                self.mongo.persist();
                Task::none()
            }
            Message::MongoOpenSlowQuery(query) => {
                self.mongo.active_slow_query = query.map(|query| *query);
                Task::none()
            }
            Message::MongoOpenUser(user) => {
                let next = user.map(|user| *user);
                let same = match (&self.mongo.active_user_detail, &next) {
                    (Some(current), Some(incoming)) => current.user_name == incoming.user_name,
                    _ => false,
                };
                self.mongo.active_user_detail = if same { None } else { next };
                Task::none()
            }
            Message::MongoCopyIndex(suggestion) => {
                self.mongo_copied_index = Some(suggestion.clone());
                Task::batch([
                    clipboard::write(suggestion.clone()),
                    Task::done(Message::MongoToast(format!("Copied index: {suggestion}"))),
                ])
            }
            Message::MongoScrolled(viewport) => {
                self.mongo_scroll = viewport.absolute_offset().y;
                Task::none()
            }
            Message::MongoSlowScrolled(viewport) => {
                self.mongo_slow_scroll = viewport.absolute_offset().y;
                Task::none()
            }
            Message::MongoChartMode(mode) => {
                self.mongo_chart_mode = mode;
                self.mongo_chart_hover = None;
                Task::none()
            }
            Message::MongoChartHover(index) => {
                self.mongo_chart_hover = index;
                Task::none()
            }
            Message::MongoToast(message) => {
                self.mongo.show_toast(message);
                Task::none()
            }
            Message::MongoToastExpired(epoch) => {
                if epoch == self.mongo.toast_epoch {
                    self.mongo.clear_toast();
                    self.mongo_copied_index = None;
                }
                Task::none()
            }
            Message::MongoJob(MongoJobEvent::Progress { control, percent }) => {
                if self.mongo.is_parsing && self.is_current_mongo_job(&control) {
                    self.mongo.progress = Some(ParseProgress {
                        stage: "parsing".to_string(),
                        processed: 0,
                        total: control.total,
                        percent,
                    });
                }
                Task::none()
            }
            Message::MongoJob(MongoJobEvent::Finished(job)) => {
                if self.is_current_mongo_job(&job.control) {
                    self.finish_mongo_parse(*job);
                }
                Task::none()
            }

            // ── Async jobs ──────────────────────────────────────────────────
            Message::Job(JobEvent::Progress {
                control,
                processed,
                total,
                percent,
            }) => {
                if self.analysis.is_parsing && self.is_current_job(&control) {
                    self.analysis.progress = Some(ParseProgress {
                        stage: "parsing".to_string(),
                        processed,
                        total,
                        percent,
                    });
                }
                Task::none()
            }
            Message::Job(JobEvent::Finished { control, job }) => {
                if self.is_current_job(&control) {
                    self.finish_parse(*job);
                }
                Task::none()
            }
            Message::Reaggregated { epoch, result } => {
                if epoch == self.reagg_epoch {
                    self.install_result(*result);
                }
                Task::none()
            }
            // ── Window / keyboard ───────────────────────────────────────────
            Message::WindowResized(width) => {
                self.window_width = width;
                Task::none()
            }
            Message::SlashPressed => operation::is_focused(PASTE_EDITOR_ID)
                .map(Message::SlashPasteProbe),
            Message::SlashPasteProbe(paste_focused) => {
                if paste_focused {
                    Task::none()
                } else {
                    operation::is_focused(SEARCH_INPUT_ID).map(Message::SlashSearchProbe)
                }
            }
            Message::SlashSearchProbe(search_focused) => {
                if search_focused {
                    Task::none()
                } else {
                    operation::focus(SEARCH_INPUT_ID)
                }
            }
        };

        // Derived state: rows, chart and the toast expiry timers.
        self.refresh_rows();
        self.sync_chart();
        let mut task = task;
        if self.analysis.toast_epoch != self.toast_epoch {
            self.toast_epoch = self.analysis.toast_epoch;
            let epoch = self.toast_epoch;
            task = Task::batch([
                task,
                Task::perform(
                    async move { tokio::time::sleep(Duration::from_millis(TOAST_MS)).await },
                    move |()| Message::ToastExpired(epoch),
                ),
            ]);
        }
        if self.mongo.toast_epoch != self.mongo_toast_epoch {
            self.mongo_toast_epoch = self.mongo.toast_epoch;
            let epoch = self.mongo_toast_epoch;
            task = Task::batch([
                task,
                Task::perform(
                    async move { tokio::time::sleep(Duration::from_millis(TOAST_MS)).await },
                    move |()| Message::MongoToastExpired(epoch),
                ),
            ]);
        }

        task
    }

    // ── View ────────────────────────────────────────────────────────────────

    pub fn view(&self) -> Element<'_, Message> {
        let page = column![
            header::view(self),
            crate::ui::hrule(style::divider),
            scrollable(
                container(self.body())
                    .padding(16)
                    .max_width(1280.0)
                    .center_x(Fill)
            )
            // The reference's overlay scrollbar takes no layout width, so the
            // page content spans the full viewport here too.
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new().width(0.0).scroller_width(0.0),
            ))
            .height(Fill),
        ];

        let toast_message = match self.mode {
            AppMode::Pm2 => self.analysis.toast.as_ref(),
            AppMode::Mongo => self.mongo.toast.as_ref(),
        };
        match toast_message {
            Some(message) => iced::widget::stack![page, toast::view(message)].into(),
            None => page.into(),
        }
    }

    fn body(&self) -> Element<'_, Message> {
        container(match self.mode {
            AppMode::Pm2 => self.pm2_body(),
            AppMode::Mongo => mongo::view(self),
        })
        .width(Fill)
        .into()
    }

    fn pm2_body(&self) -> Element<'_, Message> {
        let wide = self.analysis.chart_layout == ChartLayout::Wide;
        let stacked = self.window_width < 1024.0;

        let mut content = column![
            ingest::view(self),
            kpi::view(self),
            filters::view(self)
        ]
        .spacing(16)
        .width(Fill);

        if wide {
            content = content.push(charts::view(self)).push(api_table::view(self));
        } else if stacked {
            content = content.push(api_table::view(self)).push(charts::view(self));
        } else {
            // Reference: `grid gap-4 lg:grid-cols-5` with `col-span-3` /
            // `col-span-2`, i.e. the gaps are carved out of the five equal
            // columns *before* the spans are computed.
            content = content.push(
                iced::widget::responsive(move |size| {
                    let column = ((size.width - 4.0 * 16.0) / 5.0).max(0.0);
                    let api_width = column * 3.0 + 2.0 * 16.0;
                    let chart_width = column * 2.0 + 16.0;
                    row![
                        container(api_table::view(self)).width(iced::Length::Fixed(api_width)),
                        container(charts::view(self)).width(iced::Length::Fixed(chart_width)),
                    ]
                    .spacing(16)
                    .align_y(iced::Top)
                    .into()
                }),
            );
        }

        if self.analysis.has_cron_events() {
            content = content.push(cron_table::view(self));
        }

        content = content
            .push(skipped::view(self))
            .push(Self::footer());

        content.into()
    }

    fn footer() -> Element<'static, Message> {
        container(
            iced::widget::text("Parses in your browser - logs never leave this machine")
                .size(11)
                .style(style::text_faint),
        )
        .width(Fill)
        .center_x(Fill)
        .padding(iced::Padding {
            top: 8.0,
            bottom: 24.0,
            left: 0.0,
            right: 0.0,
        })
        .into()
    }

    // ── State helpers ───────────────────────────────────────────────────────

    fn sync_filter_inputs(&mut self) {
        self.min_ms_input = format_input_number(self.analysis.filters.min_ms);
        self.top_n_input = self.analysis.filters.top_n.to_string();
        self.cron_min_ms_input = format_input_number(self.analysis.filters.cron_min_ms);
    }

    fn is_current_job(&self, control: &Arc<JobControl>) -> bool {
        self.analysis
            .job
            .as_ref()
            .is_some_and(|job| Arc::ptr_eq(job, control))
    }

    fn mark_rows_dirty(&mut self) {
        self.rows_dirty = true;
    }

    /// Recompute the filtered/sorted table rows the views and chart consume.
    fn refresh_rows(&mut self) {
        if !self.rows_dirty {
            return;
        }
        let filters = &self.analysis.filters;
        let (api, cron) = match &self.analysis.result {
            Some(result) => (result.api.as_slice(), result.cron.as_slice()),
            None => (&[][..], &[][..]),
        };
        let filtered = filter_api_endpoints(api, &filters.methods, &filters.query);
        let sorted = sort_api_endpoints(&filtered, filters.sort_key, filters.sort_dir);
        self.api_rows = sorted.into_iter().take(filters.top_n).collect();
        self.cron_rows = sort_cron_jobs(cron, filters.cron_sort_key, filters.cron_sort_dir);
        self.rows_dirty = false;
        self.rows_epoch += 1;
    }

    /// Regenerate the chart SVG only when mode/theme/size/rows/data change.
    fn sync_chart(&mut self) {
        let Some(result) = self.analysis.result.as_ref() else {
            self.chart = None;
            self.chart_key = None;
            return;
        };
        let is_dark = self.analysis.is_dark();
        let (width, height, _) = charts::chart_size(
            f64::from(self.window_width),
            self.analysis.chart_layout == ChartLayout::Wide,
        );
        let key = ChartKey {
            mode: self.chart_mode,
            dark: is_dark,
            width: width.round() as u32,
            height: height.round() as u32,
            rows: self.rows_epoch,
            data: self.result_epoch,
        };
        if self.chart_key == Some(key) {
            return;
        }
        let has_data =
            !self.api_rows.is_empty() || result.hourly_stats.iter().any(|h| h.count > 0);
        if !has_data {
            self.chart = None;
            self.chart_key = Some(key);
            return;
        }
        let document = charts::render_svg(
            self.chart_mode,
            width,
            height,
            &result.hourly_stats,
            &result.daily_stats,
            &self.api_rows,
            is_dark,
        );
        self.chart = Some(svg::Handle::from_memory(document.into_bytes()));
        self.chart_key = Some(key);
    }

    fn install_result(&mut self, result: AggregatedResult) {
        self.analysis.set_result(result);
        self.result_epoch += 1;
        self.mark_rows_dirty();
    }

    fn edit_filters(&mut self, filters: AnalysisFilters, refresh: bool) -> Task<Message> {
        self.analysis.filters = filters;
        persist(&self.analysis);
        self.mark_rows_dirty();
        if refresh {
            self.reaggregate().unwrap_or_else(Task::none)
        } else {
            Task::none()
        }
    }

    fn handle_upload(&mut self, files: Vec<LoadedSource>, append: bool) -> Task<Message> {
        self.route_upload(files, append)
    }

    /// Upload resolution supporting archive extraction (.zip, .gz) and log classification (PM2 vs Mongo).
    fn route_upload(&mut self, files: Vec<LoadedSource>, append: bool) -> Task<Message> {
        if files.is_empty() {
            return Task::none();
        }

        let has_archive = files.iter().any(|f| match f {
            LoadedSource::Path(p) => crate::core::archive::is_archive(p),
            LoadedSource::Memory { name, .. } => crate::core::archive::is_archive_name(name),
        });

        if has_archive {
            self.analysis.is_parsing = true;
            self.analysis.progress = Some(ParseProgress {
                stage: "reading".to_string(),
                processed: 10,
                total: 100,
                percent: 10,
            });
            self.mongo.is_parsing = true;
            self.mongo.progress = Some(ParseProgress {
                stage: "reading".to_string(),
                processed: 10,
                total: 100,
                percent: 10,
            });

            return Task::perform(
                async move {
                    let mut combined = crate::core::archive::ExtractedArchive::default();
                    let start = std::time::Instant::now();
                    for file in files {
                        match file {
                            LoadedSource::Path(path) => {
                                if crate::core::archive::is_archive(&path) {
                                    match crate::core::archive::extract_archive(&path) {
                                        Ok(mut ext) => {
                                            combined.pm2_sources.append(&mut ext.pm2_sources);
                                            combined.mongo_sources.append(&mut ext.mongo_sources);
                                            combined.unknown_sources.append(&mut ext.unknown_sources);
                                            combined.skipped.append(&mut ext.skipped);
                                        }
                                        Err(err) => return Err(err),
                                    }
                                } else {
                                    let name = path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().into_owned())
                                        .unwrap_or_default();
                                    let kind = crate::core::classify::classify_by_name(&name)
                                        .unwrap_or(crate::core::classify::LogKind::Unknown);
                                    let source = LoadedSource::Path(path);
                                    match kind {
                                        crate::core::classify::LogKind::Pm2 => combined.pm2_sources.push(source),
                                        crate::core::classify::LogKind::Mongo => combined.mongo_sources.push(source),
                                        crate::core::classify::LogKind::Unknown => combined.unknown_sources.push(source),
                                        crate::core::classify::LogKind::Skip => combined.skipped.push(name),
                                    }
                                }
                            }
                            LoadedSource::Memory { name, bytes } => {
                                if crate::core::archive::is_archive_name(&name) {
                                    if bytes.starts_with(&[0x1f, 0x8b]) {
                                        match crate::core::archive::decompress_gzip(&bytes) {
                                            Ok(decompressed) => {
                                                let clean = crate::core::archive::clean_log_name(&name);
                                                let kind = crate::core::classify::classify_log(&clean, &decompressed);
                                                let source = LoadedSource::Memory {
                                                    name: clean,
                                                    bytes: std::sync::Arc::new(decompressed),
                                                };
                                                match kind {
                                                    crate::core::classify::LogKind::Pm2 => combined.pm2_sources.push(source),
                                                    crate::core::classify::LogKind::Mongo => combined.mongo_sources.push(source),
                                                    crate::core::classify::LogKind::Unknown => combined.unknown_sources.push(source),
                                                    crate::core::classify::LogKind::Skip => combined.skipped.push(name),
                                                }
                                            }
                                            Err(err) => return Err(err),
                                        }
                                    }
                                } else {
                                    let kind = crate::core::classify::classify_log(&name, &bytes);
                                    let source = LoadedSource::Memory { name: name.clone(), bytes };
                                    match kind {
                                        crate::core::classify::LogKind::Pm2 => combined.pm2_sources.push(source),
                                        crate::core::classify::LogKind::Mongo => combined.mongo_sources.push(source),
                                        crate::core::classify::LogKind::Unknown => combined.unknown_sources.push(source),
                                        crate::core::classify::LogKind::Skip => combined.skipped.push(name),
                                    }
                                }
                            }
                        }
                    }
                    combined.duration_ms = start.elapsed().as_millis() as u64;
                    Ok(combined)
                },
                move |result| Message::ArchiveExtracted { result, append },
            );
        }

        // Non-archive files: classify directly and route
        let mut pm2_files = Vec::new();
        let mut mongo_files = Vec::new();

        for file in files {
            let name = file.name();
            let cat = crate::core::classify::classify_by_name(&name);
            match cat {
                Some(crate::core::classify::LogKind::Mongo) => mongo_files.push(file),
                Some(crate::core::classify::LogKind::Pm2) => pm2_files.push(file),
                Some(crate::core::classify::LogKind::Skip) => continue,
                _ => {
                    let kind = match &file {
                        LoadedSource::Path(p) => {
                            let mut buf = [0u8; 4096];
                            if let Ok(mut f) = std::fs::File::open(p) {
                                use std::io::Read;
                                let n = f.read(&mut buf).unwrap_or(0);
                                crate::core::classify::classify_log(&name, &buf[..n])
                            } else {
                                crate::core::classify::LogKind::Unknown
                            }
                        }
                        LoadedSource::Memory { bytes, .. } => {
                            crate::core::classify::classify_log(&name, bytes)
                        }
                    };
                    match kind {
                        crate::core::classify::LogKind::Mongo => mongo_files.push(file),
                        crate::core::classify::LogKind::Pm2 => pm2_files.push(file),
                        crate::core::classify::LogKind::Skip => continue,
                        crate::core::classify::LogKind::Unknown => {
                            if self.mode == AppMode::Mongo {
                                mongo_files.push(file);
                            } else {
                                pm2_files.push(file);
                            }
                        }
                    }
                }
            }
        }

        let pm2_len = pm2_files.len();
        let mongo_len = mongo_files.len();

        let pm2_task = if !pm2_files.is_empty() {
            let sources = if append {
                self.analysis.append_loaded_files(pm2_files)
            } else {
                self.analysis.set_loaded_files(pm2_files)
            };
            if !sources.is_empty() {
                self.start_parse(sources)
            } else {
                Task::none()
            }
        } else {
            Task::none()
        };

        let mongo_task = if !mongo_files.is_empty() {
            let sources = if append {
                if self.mongo.append_loaded_files(mongo_files) {
                    self.mongo.loaded_files.clone()
                } else {
                    Vec::new()
                }
            } else {
                self.mongo.set_loaded_files(mongo_files);
                self.mongo.loaded_files.clone()
            };
            if !sources.is_empty() {
                self.start_mongo_parse(sources)
            } else {
                Task::none()
            }
        } else {
            Task::none()
        };

        if pm2_len > 0 && mongo_len > 0 {
            self.notify(format!(
                "Classified {pm2_len} API log(s) and {mongo_len} MongoDB log(s). Both tabs populated."
            ));
        } else if mongo_len > 0 {
            self.mode = AppMode::Mongo;
            persist_mode(self.mode);
        } else if pm2_len > 0 {
            self.mode = AppMode::Pm2;
            persist_mode(self.mode);
        }

        Task::batch([pm2_task, mongo_task])
    }

    // ── MongoDB jobs ────────────────────────────────────────────────────────

    fn is_current_mongo_job(&self, control: &Arc<JobControl>) -> bool {
        self.mongo_job
            .as_ref()
            .is_some_and(|job| Arc::ptr_eq(job, control))
    }

    fn start_mongo_parse(&mut self, sources: Vec<LoadedSource>) -> Task<Message> {
        if sources.is_empty() {
            return Task::none();
        }
        let total: u64 = sources.iter().map(source_size).sum();
        let control = Arc::new(JobControl::new(total));
        self.mongo_job = Some(control.clone());
        self.mongo.is_parsing = true;
        self.mongo.progress = Some(ParseProgress {
            stage: "parsing".to_string(),
            processed: 0,
            total,
            percent: 0,
        });
        self.mongo.error = None;
        spawn_mongo_job(sources, control, self.mongo.filters.clone())
    }

    fn finish_mongo_parse(&mut self, job: MongoFinishedJob) {
        self.mongo.is_parsing = false;
        self.mongo.progress = None;
        self.mongo_job = None;
        match (job.kernel, job.result, job.error) {
            (Some(kernel), Some(result), _) => {
                let count = result.summary.slow_query_count;
                let collscans = result.summary.collscan_count;
                self.mongo_kernel = Some(kernel);
                self.mongo.set_result(Some(result));
                self.mongo.show_toast(if job.source_count > 1 {
                    format!(
                        "Parsed {} slow queries ({}) across {} files in {}ms",
                        count, collscans, job.source_count, job.elapsed_ms
                    )
                } else {
                    format!(
                        "Parsed {} slow queries ({}) in {}ms",
                        count, collscans, job.elapsed_ms
                    )
                });
            }
            (_, _, Some(error)) if error == "Cancelled" => {
                self.mongo.show_toast("Parsing cancelled");
            }
            (_, _, Some(error)) => {
                self.mongo.error = Some(error);
            }
            _ => {
                self.mongo.set_result(None);
            }
        }
    }

    /// Re-runs the kernel's filtered aggregation (microseconds) after a filter
    /// change, mirroring the reference's `reaggregateMongo()` round-trip.
    fn apply_mongo_filters(&mut self) -> Task<Message> {
        self.mongo.persist();
        if let Some(kernel) = &self.mongo_kernel {
            let result = kernel.reaggregate(&self.mongo.filters);
            self.mongo.set_result(Some(result));
        }
        Task::none()
    }

    fn commit_drop(&mut self) -> Task<Message> {
        let files = std::mem::take(&mut self.dropped_files);
        if files.is_empty() {
            return Task::none();
        }
        if self.mode == AppMode::Mongo {
            if self.mongo.is_parsing {
                return Task::none();
            }
            if self.mongo.has_data && !self.mongo.loaded_files.is_empty() {
                self.pending_drop = Some(files);
                Task::none()
            } else {
                self.route_upload(files, false)
            }
        } else if self.analysis.is_parsing {
            Task::none()
        } else if self.analysis.has_data && !self.analysis.loaded_files.is_empty() {
            self.pending_drop = Some(files);
            Task::none()
        } else {
            self.handle_upload(files, false)
        }
    }

    fn analyze_paste(&mut self) -> Task<Message> {
        let text = self.paste_text.text().trim().to_string();
        if text.is_empty() {
            self.analysis.show_toast("Paste some log lines first");
            return Task::none();
        }
        let bytes = text.len();
        if bytes > PASTE_WARN_BYTES {
            self.analysis.show_toast(format!(
                "Paste is {} — save as a .log file and upload instead (limit ~{})",
                format_bytes(bytes as u64),
                format_bytes(PASTE_WARN_BYTES as u64)
            ));
            return Task::none();
        }
        self.analysis.set_source_paste();
        let source = LoadedSource::Memory {
            name: "paste".to_string(),
            bytes: Arc::new(text.into_bytes()),
        };
        self.start_parse(vec![source])
    }

    fn start_parse(&mut self, sources: Vec<LoadedSource>) -> Task<Message> {
        let total: u64 = sources.iter().map(LoadedSource::size).sum();
        let control = Arc::new(JobControl::new(total));
        self.analysis.job = Some(control.clone());
        self.analysis.is_parsing = true;
        self.analysis.error = None;
        self.analysis.progress = Some(ParseProgress {
            stage: "parsing".to_string(),
            processed: 0,
            total,
            percent: 0,
        });

        let mode = self.analysis.filters.normalize_mode;
        let options = parse_options(&self.analysis.filters);
        spawn_parse_job(sources, mode, options, control)
    }

    fn reaggregate(&mut self) -> Option<Task<Message>> {
        if self.analysis.is_parsing {
            return None;
        }
        let Some(kernel) = self.analysis.kernel.clone() else {
            self.install_result(EMPTY_RESULT);
            return None;
        };
        let options = parse_options(&self.analysis.filters);
        self.reagg_epoch += 1;
        let epoch = self.reagg_epoch;
        Some(Task::perform(
            async move {
                let (sender, receiver) = iced::futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let mut guard = kernel.lock().expect("kernel lock");
                    let result = guard.reaggregate(&options);
                    let _ = sender.send(result);
                });
                receiver.await.unwrap_or(EMPTY_RESULT)
            },
            move |result| Message::Reaggregated {
                epoch,
                result: Box::new(result),
            },
        ))
    }

    fn finish_parse(&mut self, job: FinishedJob) {
        self.analysis.job = None;

        if let Some(error) = &job.error {
            self.analysis.is_parsing = false;
            // Cancellation is a user action, not an error.
            if error != "Cancelled" {
                self.analysis.error = Some(error.clone());
                self.analysis.show_toast(error.clone());
            }
            return;
        }

        let (Some(kernel), Some(result)) = (job.kernel, job.result) else {
            self.analysis.is_parsing = false;
            return;
        };
        let matched = result.summary.matched;
        self.analysis.kernel = Some(kernel);
        self.install_result(result);
        self.analysis.progress = Some(ParseProgress {
            stage: "complete".to_string(),
            processed: job.total,
            total: job.total,
            percent: 100,
        });
        self.analysis.is_parsing = false;

        if job.source_count > 1 {
            self.analysis.show_toast(format!(
                "Parsed {} requests across {} files in {}ms",
                format_num(matched),
                job.source_count,
                job.elapsed_ms
            ));
        } else {
            self.analysis.show_toast(format!(
                "Parsed {} requests in {}ms",
                format_num(matched),
                job.elapsed_ms
            ));
        }
    }

    fn export_mongo(&mut self) -> Task<Message> {
        let Some(result) = self.mongo.result.clone() else {
            self.mongo.show_toast("Nothing to export yet");
            return Task::none();
        };
        let source_label = self.mongo.file_name.clone();
        let Some(path) = export_mongo_spreadsheet::save_target() else {
            return Task::none();
        };
        Task::perform(
            async move {
                let (sender, receiver) = iced::futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let result = export_mongo_spreadsheet::write_export(
                        &result,
                        source_label.as_deref(),
                        &path,
                    );
                    let _ = sender.send(result);
                });
                match receiver.await {
                    Ok(Ok(())) => Ok(true),
                    Ok(Err(error)) => Err(error),
                    Err(_) => Err("Excel export failed".to_string()),
                }
            },
            Message::ExportFinished,
        )
    }

    fn export_pm2(&mut self) -> Task<Message> {
        let Some(data) = ExportData::collect(
            self.analysis.result.as_ref(),
            &self.analysis.filters,
            export_source_label(&self.analysis),
        ) else {
            self.analysis.show_toast("Nothing to export yet");
            return Task::none();
        };
        let had_cron = !data.cron.is_empty();
        let Some(path) = export_spreadsheet::save_target() else {
            return Task::none();
        };
        Task::perform(
            async move {
                let (sender, receiver) = iced::futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let result = data.write(&path);
                    let _ = sender.send(result);
                });
                match receiver.await {
                    Ok(Ok(())) => Ok(had_cron),
                    Ok(Err(error)) => Err(error),
                    Err(_) => Err("Excel export failed".to_string()),
                }
            },
            Message::ExportFinished,
        )
    }
}

/// Total bytes of a loaded source (progress + header label).
fn source_size(source: &LoadedSource) -> u64 {
    match source {
        LoadedSource::Path(path) => std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
        LoadedSource::Memory { bytes, .. } => bytes.len() as u64,
    }
}

/// Runs the MongoDB parse + first aggregation on a worker thread.
fn spawn_mongo_job(
    sources: Vec<LoadedSource>,
    control: Arc<JobControl>,
    filters: MongoFilters,
) -> Task<Message> {
    let (sender, receiver) = iced::futures::channel::mpsc::unbounded();
    let source_count = sources.len();

    std::thread::spawn(move || {
        let finished = Arc::new(AtomicBool::new(false));
        let monitor = {
            let finished = finished.clone();
            let sender = sender.clone();
            let control = control.clone();
            std::thread::spawn(move || {
                while !finished.load(Ordering::Relaxed) {
                    std::thread::sleep(PROGRESS_TICK);
                    if finished.load(Ordering::Relaxed) {
                        break;
                    }
                    let _ = sender.unbounded_send(MongoJobEvent::Progress {
                        control: control.clone(),
                        percent: control.percent(),
                    });
                }
            })
        };

        let started = Instant::now();
        let job = match parse_mongo_paths(&sources, &control) {
            Ok(kernel) => {
                let result = kernel.reaggregate(&filters);
                MongoFinishedJob {
                    control: control.clone(),
                    kernel: Some(Arc::new(kernel)),
                    result: Some(result),
                    elapsed_ms: started.elapsed().as_millis() as u64,
                    source_count,
                    error: None,
                }
            }
            Err(ParseError::Cancelled) => MongoFinishedJob {
                control: control.clone(),
                kernel: None,
                result: None,
                elapsed_ms: started.elapsed().as_millis() as u64,
                source_count,
                error: Some("Cancelled".to_string()),
            },
            Err(error) => MongoFinishedJob {
                control: control.clone(),
                kernel: None,
                result: None,
                elapsed_ms: started.elapsed().as_millis() as u64,
                source_count,
                error: Some(error.to_string()),
            },
        };

        finished.store(true, Ordering::Relaxed);
        let _ = monitor.join();
        let _ = sender.unbounded_send(MongoJobEvent::Finished(Box::new(job)));
    });

    Task::run(receiver, Message::MongoJob)
}

fn export_source_label(analysis: &AnalysisState) -> Option<String> {
    match analysis.source_kind {
        SourceKind::File => analysis.file_name.as_ref().map(|name| match analysis.file_size {
            Some(size) => format!("{name} ({})", format_bytes(size)),
            None => name.clone(),
        }),
        SourceKind::Paste => Some("Pasted text".to_string()),
        SourceKind::None => None,
    }
}

fn format_input_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// Runs `parse_sources` + the first reaggregation on a worker thread and streams
/// progress/telemetry into the Elm runtime through an unbounded channel.
fn spawn_parse_job(
    sources: Vec<LoadedSource>,
    mode: NormalizeMode,
    options: ParseOptions,
    control: Arc<JobControl>,
) -> Task<Message> {
    let (sender, receiver) = iced::futures::channel::mpsc::unbounded();
    let source_count = sources.len();
    let total = control.total;

    std::thread::spawn(move || {
        let finished = Arc::new(AtomicBool::new(false));
        let monitor = {
            let finished = finished.clone();
            let sender = sender.clone();
            let control = control.clone();
            std::thread::spawn(move || {
                while !finished.load(Ordering::Relaxed) {
                    std::thread::sleep(PROGRESS_TICK);
                    if finished.load(Ordering::Relaxed) {
                        break;
                    }
                    let _ = sender.unbounded_send(JobEvent::Progress {
                        control: control.clone(),
                        processed: control.processed.load(Ordering::Relaxed),
                        total: control.total,
                        percent: control.percent(),
                    });
                }
            })
        };

        let started = Instant::now();
        let parsed = parse_sources(&sources, mode, &control);
        let job = match parsed {
            Ok(mut kernel) => {
                let result = kernel.reaggregate(&options);
                FinishedJob {
                    kernel: Some(Arc::new(Mutex::new(kernel))),
                    result: Some(result),
                    elapsed_ms: started.elapsed().as_millis() as u64,
                    source_count,
                    total,
                    error: None,
                }
            }
            Err(ParseError::Cancelled) => FinishedJob {
                kernel: None,
                result: None,
                elapsed_ms: started.elapsed().as_millis() as u64,
                source_count,
                total,
                error: Some("Cancelled".to_string()),
            },
            Err(error) => FinishedJob {
                kernel: None,
                result: None,
                elapsed_ms: started.elapsed().as_millis() as u64,
                source_count,
                total,
                error: Some(error.to_string()),
            },
        };

        finished.store(true, Ordering::Relaxed);
        let _ = monitor.join();
        let _ = sender.unbounded_send(JobEvent::Finished {
            control,
            job: Box::new(job),
        });
    });

    Task::run(receiver, Message::Job)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pm2::parse_paths;

    fn smoke_log() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("smoke.log")
    }

    fn isolated_app() -> App {
        use std::sync::atomic::AtomicU64;
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let data_dir = std::env::temp_dir().join(format!(
            "pm2-log-analyzer-test-{}-{unique}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&data_dir);
        let _ = std::fs::create_dir_all(&data_dir);
        // SAFETY: single-threaded test setup before any app state is created.
        unsafe {
            std::env::set_var("PM2_ANALYZER_DATA_DIR", &data_dir);
            std::env::remove_var("PM2_ANALYZER_AUTOLOAD");
        }
        App::boot().0
    }

    /// Headless simulator with the app's bundled fonts at a chosen viewport.
    fn test_simulator(app: &App, width: f32, height: f32) -> iced_test::Simulator<'_, Message> {
        let settings = iced::Settings {
            default_font: style::REGULAR,
            fonts: crate::ui::fonts::FILES
                .iter()
                .map(|face| std::borrow::Cow::Borrowed(*face))
                .collect(),
            ..iced::Settings::default()
        };
        iced_test::Simulator::with_size(
            settings,
            iced::Size::new(width, height),
            container(app.view())
                .width(Fill)
                .height(Fill)
                .style(style::canvas),
        )
    }

    /// Parses `smoke.log` through the native driver and installs the result,
    /// mirroring what the worker thread reports through [`JobEvent`].
    fn app_with_smoke_log() -> (App, u64) {
        let mut app = isolated_app();
        let total = std::fs::metadata(smoke_log()).expect("smoke.log").len();
        let control = JobControl::new(total);
        let mut kernel =
            parse_paths(&[smoke_log()], NormalizeMode::CollapseIds, &control).expect("parse");
        let result = kernel.reaggregate(&parse_options(&app.analysis.filters));
        let matched = result.summary.matched;
        assert!(matched > 0, "smoke.log must produce HTTP hits");
        // Mirror the upload path so the header/ingest show the loaded file.
        app.analysis.set_loaded_files(vec![LoadedSource::Path(smoke_log())]);
        app.finish_parse(FinishedJob {
            kernel: Some(Arc::new(Mutex::new(kernel))),
            result: Some(result),
            elapsed_ms: 1,
            source_count: 1,
            total,
            error: None,
        });
        app.refresh_rows();
        app.sync_chart();
        (app, matched)
    }

    #[test]
    fn headless_view_renders_parsed_log() {
        let (app, matched) = app_with_smoke_log();

        assert!(app.analysis.has_data);
        assert_eq!(app.analysis.summary().unwrap().matched, matched);
        assert!(!app.api_rows.is_empty(), "endpoint rows must be derived");
        assert!(app.chart.is_some(), "chart SVG must be generated");

        let expected_path = app.api_rows[0].path.clone();
        // The simulated window is wide enough for the full path (no ellipsis).
        let mut ui = test_simulator(&app, 1600.0, 1000.0);
        ui.find("PM2 Log Analyzer").expect("header title");
        ui.find("REQUESTS").expect("kpi label");
        ui.find("SLOW API ENDPOINTS").expect("api table title");
        ui.find("API VISUAL ANALYTICS").expect("chart title");
        ui.find(expected_path.as_str()).expect("first endpoint row");
    }

    #[test]
    fn interactions_produce_expected_state() {
        let (mut app, _) = app_with_smoke_log();

        let messages: Vec<Message> = {
            let mut ui = test_simulator(&app, 1024.0, 768.0);
            ui.click("Wide View").expect("layout toggle button");
            ui.click("Hourly Volume").expect("chart tab");
            ui.into_messages().collect()
        };
        for message in messages {
            let _ = app.update(message);
        }

        assert_eq!(app.analysis.chart_layout, ChartLayout::Wide);
        assert_eq!(app.chart_mode, ChartMode::Throughput);
        // Wide layout regenerates the chart at the wider geometry.
        assert!(app.chart.is_some());

        // Toggling the theme flips the window theme and invalidates the chart.
        let _ = app.update(Message::ToggleTheme);
        assert!(app.analysis.is_dark());
        assert!(app.chart.is_some());

        // Clearing wipes every derived row and the chart cache.
        let _ = app.update(Message::ClearAll);
        assert!(!app.analysis.has_data);
        assert!(app.api_rows.is_empty());
        assert!(app.chart.is_none());
    }

    /// Renders the full UI to `target/ui-preview-<renderer>.png` for manual
    /// inspection. Ignored by default: snapshot pixels are machine-dependent.
    #[test]
    #[ignore = "writes a PNG preview under target/ for manual inspection"]
    fn writes_ui_preview() {
        let (mut app, _) = app_with_smoke_log();
        // The default simulator viewport is 1024x768; keep the chart geometry in sync.
        app.window_width = 1024.0;
        // The reference capture lets the toast auto-hide, so nothing overlaps
        // the layout in either shot.
        app.analysis.toast = None;
        app.sync_chart();

        // `matches_image` only writes when the path is missing; drop any stale
        // renders (whose names carry the renderer suffix) first.
        if let Ok(entries) = std::fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"))
        {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("ui-preview") && name.ends_with(".png") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }

        {
            let mut ui = test_simulator(&app, 1024.0, 768.0);
            let snapshot = ui.snapshot(&app.theme()).expect("render snapshot");
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/ui-preview.png");
            snapshot.matches_image(&path).expect("write preview");
        }

        // Dark theme + wide layout exercises the other style/layout branch.
        let _ = app.update(Message::ToggleTheme);
        let _ = app.update(Message::ToggleChartLayout);
        {
            let mut ui = test_simulator(&app, 1024.0, 768.0);
            let snapshot = ui.snapshot(&app.theme()).expect("render dark snapshot");
            let path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/ui-preview-dark.png");
            snapshot.matches_image(&path).expect("write dark preview");
        }

        // Hovering a bucket shows the tooltip card over the chart.
        let _ = app.update(Message::ToggleTheme);
        let _ = app.update(Message::ChartHover(Some(12)));
        {
            let mut ui = test_simulator(&app, 1024.0, 768.0);
            let snapshot = ui.snapshot(&app.theme()).expect("render tooltip snapshot");
            let path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/ui-preview-tooltip.png");
            snapshot.matches_image(&path).expect("write tooltip preview");
        }
    }

    /// Dumps the layout bounds of well-known texts so the reference DOM dump
    /// (`visual-tests/dump-dom.mjs`) can be compared numerically.
    #[test]
    #[ignore = "diagnostic: prints text layout boxes"]
    fn dump_ui_geometry() {
        let (app, _) = app_with_smoke_log();
        let mut ui = test_simulator(&app, 1024.0, 768.0);
        let labels = [
            "PM2 Log Analyzer",
            "smoke.log · 2.6 KB",
            "24/07/2026",
            "PM2 Logs",
            "MongoDB Logs",
            "Export",
            "Clear",
            "1 file:",
            "smoke.log",
            "2.6 KB",
            "Add / Append",
            "Replace",
            "Paste",
            "REQUESTS",
            "21",
            "AVG",
            "245ms",
            "P95",
            "2.28s",
            "ERRORS",
            "3",
            "SLOW ≥3S",
            "0",
            "CRON JOBS",
            "SEARCH",
            "NORMALIZE",
            "STATUS",
            "MIN MS",
            "SORT",
            "TOP N",
            "Reset all filters",
            "METHODS",
            "All",
            "DELETE",
            "GET",
            "HEAD",
            "PATCH",
            "POST",
            "PUT",
            "SLOW API ENDPOINTS",
            "Copy TSV",
            "Endpoint",
            "Count",
            "Avg",
            "p95",
            "p99",
            "Max",
            "Errors",
            "API VISUAL ANALYTICS",
            "Wide View",
            "Time vs Latency",
            "Hourly Volume",
            "Distribution",
            "Top Slowest",
            "Avg Latency",
            "P95 Latency",
            "P99 Latency",
            "Parses in your browser - logs never leave this machine",
            "/api/admin/motor/quotegenerate/generateQuotes",
        ];
        println!("label\tx\ty\tw\th");
        for label in labels {
            match ui.find(label) {
                Ok(target) => {
                    let bounds = target.bounds();
                    println!(
                        "{label}\t{:.2}\t{:.2}\t{:.2}\t{:.2}",
                        bounds.x, bounds.y, bounds.width, bounds.height
                    );
                }
                Err(_) => println!("{label}\tMISSING"),
            }
        }
    }

    /// Dumps the generated chart SVG for geometry inspection.
    #[test]
    #[ignore = "diagnostic: writes target/native-chart.svg"]
    fn dump_chart_svg() {
        let (app, _) = app_with_smoke_log();
        let result = app.analysis.result.as_ref().expect("result");
        let (w, h, _) = charts::chart_size(f64::from(app.window_width), false);
        let svg = charts::render_svg(
            app.chart_mode,
            w,
            h,
            &result.hourly_stats,
            &result.daily_stats,
            &app.api_rows,
            false,
        );
        std::fs::write(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/native-chart.svg"),
            svg,
        )
        .expect("write chart svg");
    }

    /// Dumps the Mongo view's layout bounds for numeric comparison against
    /// `target/dom-mongo.json` (`dump-dom.mjs --mongo`).
    #[test]
    #[ignore = "diagnostic: prints Mongo text layout boxes"]
    fn dump_mongo_geometry() {
        let Some(app) = app_with_mongo_log() else {
            return;
        };
        let mut app = app;
        app.window_width = 1024.0;
        let mut ui = test_simulator(&app, 1024.0, 768.0);
        let labels = [
            "MongoDB Log Analyzer",
            "01/09/2026",
            "eSanad-mongod.log · 2.6 MB",
            "Loaded 1 file",
            "Add More Files",
            "Replace",
            "Slow Queries",
            "841",
            "COLLSCANs",
            "291",
            "P95 Latency",
            "5.83s",
            "Max Duration",
            "13.35s",
            "Docs Examined",
            "37,203,175",
            "Diagnostics",
            "109 peak",
            "Query Patterns",
            "Slow Query Log",
            "User Activity",
            "Latency Charts",
            "Search collection, plan, IP...",
            "Plan:",
            "All Plans",
            "COLLSCAN Only (291)",
            "IXSCAN",
            "Op:",
            "All Operations",
            "Collection:",
            "All Collections (27)",
            "User:",
            "All Users (2)",
            "Duration:",
            "All",
            ">100ms",
            "Scan Ratio >100x",
            "Reset all filters",
            "Query Pattern / Collection",
            "Count",
            "Total Time",
            "Suggested Index (1-Click Copy)",
            "View",
            "FIND",
            "AGGREGATE",
            "health_quotes",
            "251.7s",
            "6.24s",
            "0x",
            "COLLSCAN",
        ];
        println!("label\tx\ty\tw\th");
        for label in labels {
            match ui.find(label) {
                Ok(target) => {
                    let bounds = target.bounds();
                    println!(
                        "{label}\t{:.2}\t{:.2}\t{:.2}\t{:.2}",
                        bounds.x, bounds.y, bounds.width, bounds.height
                    );
                }
                Err(_) => println!("{label}\tMISSING"),
            }
        }
    }

    /// Dumps the Mongo slow-query view's layout for the reference comparison.
    #[test]
    #[ignore = "diagnostic: prints Mongo slow-query layout boxes"]
    fn dump_mongo_slow_geometry() {
        let Some(app) = app_with_mongo_log() else {
            return;
        };
        let mut app = app;
        app.window_width = 1024.0;
        let _ = app.update(Message::MongoActiveView(crate::store::mongo_store::MongoActiveView::SlowQueries));
        let mut ui = test_simulator(&app, 1024.0, 768.0);
        let labels = [
            "Time",
            "Duration",
            "User",
            "Collection & Plan",
            "Docs\nScanned",
            "Keys\nScanned",
            "Returned",
            "Scan\nRatio",
            "Client IP",
            "View",
            "13.35s",
            "eSanad",
            "conn14370",
            "AGGREGATE",
            "IXSCAN { createdAt: 1 }",
            "532,451",
            "53245.1",
            "20.233.24.214:34386",
            "11:04:10",
        ];
        println!("label\tx\ty\tw\th");
        for label in labels {
            match ui.find(label) {
                Ok(target) => {
                    let bounds = target.bounds();
                    println!(
                        "{label}\t{:.2}\t{:.2}\t{:.2}\t{:.2}",
                        bounds.x, bounds.y, bounds.width, bounds.height
                    );
                }
                Err(_) => println!("{label}\tMISSING"),
            }
        }
    }

    #[test]
    fn chart_hover_shows_a_tooltip() {
        let (mut app, _) = app_with_smoke_log();
        assert!(app.chart_hover.is_none(), "no tooltip before hovering");

        let _ = app.update(Message::ChartHover(Some(2)));
        assert_eq!(app.chart_hover, Some(2));
        let hour = app
            .analysis
            .result
            .as_ref()
            .expect("result")
            .hourly_stats
            .get(2)
            .map(|bucket| bucket.hour)
            .unwrap_or(0);
        let expected = format!("Time: {hour:02}:00");
        {
            let mut ui = test_simulator(&app, 1600.0, 1000.0);
            ui.find(expected.as_str()).expect("tooltip label");
            ui.find("P95 Latency").expect("tooltip series");
        }

        let _ = app.update(Message::ChartHover(None));
        let mut ui = test_simulator(&app, 1600.0, 1000.0);
        assert!(
            ui.find(expected.as_str()).is_err(),
            "tooltip disappears when the pointer leaves"
        );
    }

    #[test]
    fn viewer_resized_regenerates_chart() {
        let (mut app, _) = app_with_smoke_log();
        let before = app.chart.as_ref().map(svg::Handle::id);
        let _ = app.update(Message::WindowResized(700.0));
        let after = app.chart.as_ref().map(svg::Handle::id);
        assert!(before.is_some() && after.is_some());
        assert_ne!(before, after, "chart must re-render at the new width");
    }

    // ── MongoDB ────────────────────────────────────────────────────────────

    /// Reference sample log shipped next to the React app (352 MB sibling
    /// `methaq-mongod.log` is only used for manual perf runs).
    fn mongo_sample() -> Option<PathBuf> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../pm2-log-analyzer/mongodb_logs_sample/eSanad-mongod.log");
        path.exists().then_some(path)
    }

    fn app_with_mongo_log() -> Option<App> {
        let path = mongo_sample()?;
        let mut app = isolated_app();
        app.mode = AppMode::Mongo;
        let size = std::fs::metadata(&path).ok()?.len();
        let control = Arc::new(JobControl::new(size));
        let kernel = parse_mongo_paths(&[LoadedSource::Path(path.clone())], &control).ok()?;
        let result = kernel.reaggregate(&app.mongo.filters);
        assert!(result.summary.slow_query_count > 0, "sample has slow queries");
        app.mongo_kernel = Some(Arc::new(kernel));
        app.mongo.set_result(Some(result));
        // Keep the real path so the header and "Loaded N files" line match.
        app.mongo.set_loaded_files(vec![LoadedSource::Path(path)]);
        Some(app)
    }

    #[test]
    fn mongo_view_renders_kpis_and_patterns() {
        let Some(app) = app_with_mongo_log() else {
            return;
        };
        let expected_ns = app.mongo.result.as_ref().expect("result").patterns[0]
            .collection
            .clone();

        let mut ui = test_simulator(&app, 1600.0, 1000.0);
        ui.find("MongoDB Log Analyzer").expect("mongo header");
        ui.find("Slow Queries").expect("kpi label");
        ui.find("Plan:").expect("filter bar");
        ui.find("Query Patterns").expect("patterns tab");
        ui.find("Slow Query Log").expect("slow query tab");
        ui.find("User Activity").expect("users tab");
        ui.find("Latency Charts").expect("charts tab");
        ui.find("Diagnostics").expect("diagnostics tab");
        ui.find("Query Pattern / Collection")
            .expect("pattern table header");
        ui.find(expected_ns.as_str()).expect("first pattern row");
    }

    #[test]
    fn mongo_filters_reaggregate_and_switch_views() {
        let Some(mut app) = app_with_mongo_log() else {
            return;
        };
        let before = app
            .mongo
            .result
            .as_ref()
            .map(|result| result.slow_queries.len())
            .unwrap_or(0);

        // Duration preset flows through the kernel: ≥5s is a strict subset.
        let _ = app.update(Message::MongoMinDuration(5000));
        let after = app
            .mongo
            .result
            .as_ref()
            .map(|result| result.slow_queries.len())
            .unwrap_or(0);
        assert!(after < before, "duration filter must cut the query set");
        assert!(app
            .mongo
            .result
            .as_ref()
            .expect("result")
            .slow_queries
            .iter()
            .all(|query| query.duration_ms >= 5000));

        // COLLSCAN-only plan filter must keep only unindexed scans.
        let _ = app.update(Message::MongoMinDuration(0));
        let _ = app.update(Message::MongoPlanFilter(MongoPlanFilter::CollscanOnly));
        assert!(app
            .mongo
            .result
            .as_ref()
            .expect("result")
            .slow_queries
            .iter()
            .all(|query| query.is_collscan));

        // Reset restores the unfiltered set.
        let _ = app.update(Message::MongoResetFilters);
        assert_eq!(
            app.mongo.result.as_ref().map(|r| r.slow_queries.len()),
            Some(before)
        );

        // Each view renders with the parsed data (no placeholders left).
        for view in MongoActiveView::ALL {
            let _ = app.update(Message::MongoActiveView(view));
            let mut ui = test_simulator(&app, 1600.0, 1200.0);
            match view {
                MongoActiveView::Patterns => ui.find("Query Pattern / Collection").expect("patterns"),
                MongoActiveView::SlowQueries => ui.find("Slow Query Log").expect("queries"),
                MongoActiveView::Charts => {
                    ui.find("Database Performance Over Time").expect("chart");
                    ui.find("Queries & P95").expect("chart mode tab")
                }
                MongoActiveView::Diagnostics => {
                    ui.find("Errors & Warnings (3)").expect("diagnostics")
                }
                MongoActiveView::Users => ui.find("User Activity").expect("users"),
            };
        }
    }

    #[test]
    #[ignore = "diagnostic: writes target/mongo-view-*.png at 1024x768"]
    fn writes_mongo_view_previews() {
        let Some(mut app) = app_with_mongo_log() else {
            return;
        };
        app.window_width = 1024.0;
        app.mongo.toast = None;
        let views = [
            (MongoActiveView::SlowQueries, "slow"),
            (MongoActiveView::Charts, "charts"),
            (MongoActiveView::Users, "users"),
            (MongoActiveView::Diagnostics, "diag"),
        ];
        for (view, name) in views {
            let _ = app.update(Message::MongoActiveView(view));
            let mut ui = test_simulator(&app, 1024.0, 768.0);
            let snapshot = ui.snapshot(&app.theme()).expect("render snapshot");
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("target/mongo-view-{name}.png"));
            snapshot.matches_image(&path).expect("write preview");
        }
    }

    #[test]
    #[ignore = "writes a PNG preview under target/ for manual inspection"]
    fn writes_mongo_preview() {
        let Some(mut app) = app_with_mongo_log() else {
            return;
        };
        app.window_width = 1024.0;
        {
            let mut ui = test_simulator(&app, 1024.0, 768.0);
            let snapshot = ui.snapshot(&app.theme()).expect("render snapshot");
            snapshot
                .matches_image(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/mongo-preview.png"),
                )
                .expect("write preview");
        }

        let _ = app.update(Message::ToggleTheme);
        let _ = app.update(Message::MongoActiveView(MongoActiveView::Diagnostics));
        {
            let mut ui = test_simulator(&app, 1280.0, 900.0);
            let snapshot = ui.snapshot(&app.theme()).expect("render dark snapshot");
            snapshot
                .matches_image(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("target/mongo-preview-dark.png"),
                )
                .expect("write dark preview");
        }

        // Charts view with a hovered bucket (tooltip) in light mode.
        let _ = app.update(Message::ToggleTheme);
        let _ = app.update(Message::MongoActiveView(MongoActiveView::Charts));
        let _ = app.update(Message::MongoChartHover(Some(6)));
        {
            let mut ui = test_simulator(&app, 1280.0, 900.0);
            let snapshot = ui.snapshot(&app.theme()).expect("render chart snapshot");
            snapshot
                .matches_image(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("target/mongo-preview-charts.png"),
                )
                .expect("write chart preview");
        }

        // Top Slow Collections mode.
        let _ = app.update(Message::MongoChartMode(
            crate::store::mongo_store::MongoChartMode::TopCollections,
        ));
        let _ = app.update(Message::MongoChartHover(Some(2)));
        {
            let mut ui = test_simulator(&app, 1280.0, 900.0);
            let snapshot = ui.snapshot(&app.theme()).expect("render collections snapshot");
            snapshot
                .matches_image(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("target/mongo-preview-collections.png"),
                )
                .expect("write collections preview");
        }
    }

    #[test]
    fn archive_extracted_populates_both_tabs_when_mixed() {
        let mut app = isolated_app();
        let archive = crate::core::archive::ExtractedArchive {
            pm2_sources: vec![LoadedSource::Memory {
                name: "api-out.log".to_string(),
                bytes: Arc::new(b"2026-07-24T00:00:01: GET /api/users 200 12.0 ms - 45\n".to_vec()),
            }],
            mongo_sources: vec![LoadedSource::Memory {
                name: "mongod.log".to_string(),
                bytes: Arc::new(br#"{"t":{"$date":"2026-07-24T12:00:00Z"},"msg":"slow"}"#.to_vec()),
            }],
            unknown_sources: Vec::new(),
            skipped: Vec::new(),
            duration_ms: 42,
        };

        let _ = app.update(Message::ArchiveExtracted {
            result: Ok(archive),
            append: false,
        });

        assert_eq!(app.analysis.loaded_files.len(), 1);
        assert_eq!(app.mongo.loaded_files.len(), 1);
        let toast = app.analysis.toast.expect("toast message");
        assert!(toast.contains("Both tabs populated"), "toast was: {toast}");
    }

    #[test]
    fn archive_extracted_switches_to_mongo_mode() {
        let mut app = isolated_app();
        app.mode = AppMode::Pm2;

        let archive = crate::core::archive::ExtractedArchive {
            pm2_sources: Vec::new(),
            mongo_sources: vec![LoadedSource::Memory {
                name: "mongod.log".to_string(),
                bytes: Arc::new(br#"{"t":{"$date":"2026-07-24T12:00:00Z"},"msg":"slow"}"#.to_vec()),
            }],
            unknown_sources: Vec::new(),
            skipped: Vec::new(),
            duration_ms: 15,
        };

        let _ = app.update(Message::ArchiveExtracted {
            result: Ok(archive),
            append: false,
        });

        assert_eq!(app.mode, AppMode::Mongo);
        assert_eq!(app.mongo.loaded_files.len(), 1);
        let toast = app.mongo.toast.expect("mongo toast message");
        assert!(toast.contains("into MongoDB Analyzer"), "toast was: {toast}");
    }

    #[test]
    fn archive_extracted_switches_to_pm2_mode() {
        let mut app = isolated_app();
        app.mode = AppMode::Mongo;

        let archive = crate::core::archive::ExtractedArchive {
            pm2_sources: vec![LoadedSource::Memory {
                name: "api-out.log".to_string(),
                bytes: Arc::new(b"2026-07-24T00:00:01: GET /api/test 200 5.0 ms - 10\n".to_vec()),
            }],
            mongo_sources: Vec::new(),
            unknown_sources: Vec::new(),
            skipped: Vec::new(),
            duration_ms: 10,
        };

        let _ = app.update(Message::ArchiveExtracted {
            result: Ok(archive),
            append: false,
        });

        assert_eq!(app.mode, AppMode::Pm2);
        assert_eq!(app.analysis.loaded_files.len(), 1);
        let toast = app.analysis.toast.expect("pm2 toast message");
        assert!(toast.contains("into PM2 Analyzer"), "toast was: {toast}");
    }
}
