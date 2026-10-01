pub mod api_table;
pub mod charts;
pub mod cron_table;
pub mod filters;
pub mod header;
pub mod icons;
pub mod ingest;
pub mod kpi;
pub mod mongo_view;
pub mod skipped;
pub mod theme;
pub mod toast;
pub mod widgets;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use egui::{ScrollArea, Ui};

use crate::core::models::{AggregatedEndpoint, AggregatedResult, CronAggregated};
use crate::core::mongo::{MongoKernel, parse_paths as parse_mongo_sources};
use crate::core::mongo_models::{MongoAggregationResult, MongoSlowQuery};
use crate::core::pm2::{JobControl, LoadedSource, ParseError, Pm2Kernel, parse_sources};
use crate::store::analysis_store::{
    AnalysisFilters, AnalysisState, ChartLayout, ParseProgress, SortDirection, Theme, parse_options,
};
use crate::store::app_mode_store::{AppMode, persist_mode, restore_mode};
use crate::store::mongo_store::MongoState;
use crate::utils::export_mongo_spreadsheet;
use crate::utils::export_spreadsheet::{self, ExportInput};
use crate::utils::table_ops::{
    build_api_tsv, build_cron_tsv, filter_api_endpoints, sort_api_endpoints, sort_cron_jobs,
};

use self::api_table::{ApiTableAction, render_api_table};
use self::charts::{ChartAction, ChartProps, ChartViewMode, render_chart};
use self::cron_table::{CronTableAction, render_cron_table};
use self::filters::{FilterAction, render_filters};
use self::header::{HeaderAction, HeaderProps, render_header};
use self::ingest::{IngestAction, IngestProps, render_ingest};
use self::kpi::render_kpis;
use self::mongo_view::{MongoViewAction, render_mongo_view};
use self::skipped::{SkippedAction, render_skipped};
use self::theme::{LG_CONTENT_WIDTH, apply_theme, setup_fonts};
use self::toast::ToastManager;

enum BgEvent {
    ArchiveExtracted {
        result: Result<crate::core::archive::ExtractedArchive, String>,
        append: bool,
    },
    Pm2Done {
        kernel: Arc<Mutex<Pm2Kernel>>,
        result: AggregatedResult,
        files: Vec<LoadedSource>,
    },
    Pm2Error(String),
    MongoDone {
        kernel: Arc<MongoKernel>,
        result: MongoAggregationResult,
        files: Vec<LoadedSource>,
    },
    MongoError(String),
}

pub struct EguiApp {
    pub mode: AppMode,
    pub analysis: AnalysisState,
    pub mongo: MongoState,

    pm2_kernel: Option<Arc<Mutex<Pm2Kernel>>>,
    mongo_kernel: Option<Arc<MongoKernel>>,
    job_control: Option<Arc<JobControl>>,

    bg_rx: Receiver<BgEvent>,
    bg_tx: Sender<BgEvent>,

    pub api_rows: Vec<AggregatedEndpoint>,
    pub cron_rows: Vec<CronAggregated>,

    chart_mode: ChartViewMode,
    paste_buffer: String,
    paste_open: bool,
    skipped_open: bool,
    drag_over: bool,

    selected_mongo_query: Option<MongoSlowQuery>,
    toasts: ToastManager,

    /// Animated reveal when the palette changes (fastframe-theme).
    theme_transition: fastframe_theme::Transition,
    /// The palette actually applied to egui; a difference from
    /// `analysis.theme` starts a transition.
    applied_dark: bool,
}

impl EguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (bg_tx, bg_rx) = channel();

        let mut app = Self {
            mode: restore_mode().unwrap_or(AppMode::Pm2),
            analysis: AnalysisState::default(),
            mongo: MongoState::default(),
            pm2_kernel: None,
            mongo_kernel: None,
            job_control: None,
            bg_rx,
            bg_tx,
            api_rows: Vec::new(),
            cron_rows: Vec::new(),
            chart_mode: ChartViewMode::TimeOfDay,
            paste_buffer: String::new(),
            paste_open: false,
            skipped_open: false,
            drag_over: false,
            selected_mongo_query: None,
            toasts: ToastManager::default(),
            theme_transition: fastframe_theme::Transition::new(fastframe_theme::Reveal::Circle),
            applied_dark: false,
        };

        egui_extras::install_image_loaders(&cc.egui_ctx);
        icons::install_icons(&cc.egui_ctx);
        setup_fonts(&cc.egui_ctx);
        let dark = app.analysis.theme == Theme::Dark;
        apply_theme(&cc.egui_ctx, dark);
        app.applied_dark = dark;
        app
    }

    fn refresh_derived_rows(&mut self) {
        if let Some(res) = &self.analysis.result {
            let filtered = filter_api_endpoints(
                &res.api,
                &self.analysis.filters.methods,
                &self.analysis.filters.query,
            );
            self.api_rows = sort_api_endpoints(
                &filtered,
                self.analysis.filters.sort_key,
                self.analysis.filters.sort_dir,
            );

            self.cron_rows = sort_cron_jobs(
                &res.cron,
                self.analysis.filters.cron_sort_key,
                self.analysis.filters.cron_sort_dir,
            );
        } else {
            self.api_rows.clear();
            self.cron_rows.clear();
        }
    }

    fn handle_api_table_action(&mut self, ui: &mut Ui, action: ApiTableAction) {
        match action {
            ApiTableAction::CopyPath(path) => {
                ui.copy_text(path);
                self.toasts.show("Path copied to clipboard!");
            }
            ApiTableAction::CopyTsv => {
                ui.copy_text(build_api_tsv(&self.api_rows));
                self.toasts.show("API table TSV copied to clipboard!");
            }
            ApiTableAction::ChangeSort(key) => {
                if self.analysis.filters.sort_key == key {
                    self.analysis.filters.sort_dir = match self.analysis.filters.sort_dir {
                        SortDirection::Asc => SortDirection::Desc,
                        SortDirection::Desc => SortDirection::Asc,
                    };
                } else {
                    self.analysis.filters.sort_key = key;
                    self.analysis.filters.sort_dir = SortDirection::Desc;
                }
                self.refresh_derived_rows();
            }
            ApiTableAction::None => {}
        }
    }

    fn handle_cron_table_action(&mut self, ui: &mut Ui, action: CronTableAction) {
        match action {
            CronTableAction::CopyTsv => {
                ui.copy_text(build_cron_tsv(&self.cron_rows));
                self.toasts.show("Cron table TSV copied to clipboard!");
            }
            CronTableAction::ChangeSort(key) => {
                if self.analysis.filters.cron_sort_key == key {
                    self.analysis.filters.cron_sort_dir = match self.analysis.filters.cron_sort_dir
                    {
                        SortDirection::Asc => SortDirection::Desc,
                        SortDirection::Desc => SortDirection::Asc,
                    };
                } else {
                    self.analysis.filters.cron_sort_key = key;
                    self.analysis.filters.cron_sort_dir = SortDirection::Desc;
                }
                self.refresh_derived_rows();
            }
            CronTableAction::Reaggregate => self.reaggregate_pm2(),
            CronTableAction::None => {}
        }
    }

    fn trigger_pm2_parse(&mut self, sources: Vec<LoadedSource>, append: bool) {
        if sources.is_empty() {
            return;
        }

        let mut all_sources = if append {
            self.analysis.loaded_files.clone()
        } else {
            Vec::new()
        };
        all_sources.extend(sources);

        let total_size: u64 = all_sources.iter().map(|s| s.size()).sum();
        let control = Arc::new(JobControl::new(total_size));
        self.job_control = Some(control.clone());
        self.analysis.is_parsing = true;
        self.analysis.progress = Some(ParseProgress {
            stage: "Parsing PM2 logs...".to_string(),
            processed: 0,
            total: total_size,
            percent: 0,
        });

        let norm_mode = self.analysis.filters.normalize_mode;
        let tx = self.bg_tx.clone();
        let sources_clone = all_sources.clone();
        let opts = parse_options(&self.analysis.filters);

        std::thread::spawn(
            move || match parse_sources(&sources_clone, norm_mode, &control) {
                Ok(mut kernel) => {
                    let result = kernel.reaggregate(&opts);
                    let _ = tx.send(BgEvent::Pm2Done {
                        kernel: Arc::new(Mutex::new(kernel)),
                        result,
                        files: sources_clone,
                    });
                }
                Err(ParseError::Cancelled) => {}
                Err(ParseError::Io(e)) => {
                    let _ = tx.send(BgEvent::Pm2Error(e));
                }
            },
        );
    }

    fn trigger_mongo_parse(&mut self, sources: Vec<LoadedSource>, append: bool) {
        if sources.is_empty() {
            return;
        }

        let mut all_sources = if append {
            self.mongo.loaded_files.clone()
        } else {
            Vec::new()
        };
        all_sources.extend(sources);

        let total_size: u64 = all_sources.iter().map(|s| s.size()).sum();
        let control = Arc::new(JobControl::new(total_size));
        self.job_control = Some(control.clone());
        self.mongo.is_parsing = true;
        self.mongo.progress = Some(ParseProgress {
            stage: "Parsing MongoDB logs...".to_string(),
            processed: 0,
            total: total_size,
            percent: 0,
        });

        let tx = self.bg_tx.clone();
        let sources_clone = all_sources.clone();
        let filters = self.mongo.filters.clone();

        std::thread::spawn(
            move || match parse_mongo_sources(&sources_clone, &control) {
                Ok(kernel) => {
                    let result = kernel.reaggregate(&filters);
                    let _ = tx.send(BgEvent::MongoDone {
                        kernel: Arc::new(kernel),
                        result,
                        files: sources_clone,
                    });
                }
                Err(ParseError::Cancelled) => {}
                Err(ParseError::Io(e)) => {
                    let _ = tx.send(BgEvent::MongoError(e));
                }
            },
        );
    }

    fn reaggregate_pm2(&mut self) {
        let res = if let Some(kernel_mutex) = &self.pm2_kernel {
            if let Ok(mut kernel) = kernel_mutex.lock() {
                let opts = parse_options(&self.analysis.filters);
                Some(kernel.reaggregate(&opts))
            } else {
                None
            }
        } else {
            None
        };
        if let Some(res) = res {
            self.analysis.result = Some(res);
            self.refresh_derived_rows();
        }
    }

    fn reaggregate_mongo(&mut self) {
        if let Some(kernel) = &self.mongo_kernel {
            self.mongo.result = Some(kernel.reaggregate(&self.mongo.filters));
        }
    }

    fn handle_dropped_paths(&mut self, paths: Vec<PathBuf>, append: bool) {
        if paths.is_empty() {
            return;
        }

        let mut archives = Vec::new();
        let mut regular_files = Vec::new();
        for p in paths {
            if crate::core::archive::is_archive(&p) {
                archives.push(p);
            } else {
                regular_files.push(p);
            }
        }

        if !archives.is_empty() {
            self.analysis.is_parsing = true;
            self.analysis.progress = Some(ParseProgress {
                stage: "Extracting archive...".to_string(),
                processed: 0,
                total: 100,
                percent: 50,
            });
            let tx = self.bg_tx.clone();
            std::thread::spawn(move || {
                for arc_path in archives {
                    let res = crate::core::archive::extract_archive(&arc_path);
                    let _ = tx.send(BgEvent::ArchiveExtracted {
                        result: res,
                        append,
                    });
                }
            });
            if !regular_files.is_empty() {
                let sources: Vec<LoadedSource> =
                    regular_files.into_iter().map(LoadedSource::Path).collect();
                if self.mode == AppMode::Pm2 {
                    self.trigger_pm2_parse(sources, append);
                } else {
                    self.trigger_mongo_parse(sources, append);
                }
            }
            return;
        }

        let sources: Vec<LoadedSource> =
            regular_files.into_iter().map(LoadedSource::Path).collect();
        if self.mode == AppMode::Pm2 {
            self.trigger_pm2_parse(sources, append);
        } else {
            self.trigger_mongo_parse(sources, append);
        }
    }

    #[cfg(test)]
    pub fn new_headless() -> Self {
        let (bg_tx, bg_rx) = channel();
        Self {
            mode: AppMode::Pm2,
            analysis: AnalysisState::default(),
            mongo: MongoState::default(),
            pm2_kernel: None,
            mongo_kernel: None,
            job_control: None,
            bg_rx,
            bg_tx,
            api_rows: Vec::new(),
            cron_rows: Vec::new(),
            chart_mode: ChartViewMode::TimeOfDay,
            paste_buffer: String::new(),
            paste_open: false,
            skipped_open: false,
            drag_over: false,
            selected_mongo_query: None,
            toasts: ToastManager::default(),
            theme_transition: fastframe_theme::Transition::new(fastframe_theme::Reveal::Circle),
            applied_dark: false,
        }
    }
}

impl eframe::App for EguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if let Ok(theme_var) = std::env::var("PM2_THEME") {
            if theme_var.trim() == "dark" && self.analysis.theme != Theme::Dark {
                self.analysis.theme = Theme::Dark;
            }
        }

        // Theme change → animated reveal. Hold the old palette while the
        // screenshot for the transition is taken, then apply the new one.
        // Screenshot-capture runs skip the animation for deterministic QA.
        let wanted_dark = self.analysis.theme == Theme::Dark;
        let capture_mode = std::env::var_os("PM2_CAPTURE_SCREENSHOT").is_some()
            || std::env::var_os("PM2_CAPTURE_MONGO_SCREENSHOT").is_some();
        if wanted_dark != self.applied_dark {
            if capture_mode {
                apply_theme(&ctx, wanted_dark);
                self.applied_dark = wanted_dark;
            } else {
                self.theme_transition.begin(&ctx);
                if !self.theme_transition.holding(&ctx) {
                    apply_theme(&ctx, wanted_dark);
                    self.applied_dark = wanted_dark;
                }
            }
        }

        let dark = wanted_dark;

        if let Ok(target_path) = std::env::var("PM2_CAPTURE_SCREENSHOT") {
            static mut TRIGGERED: bool = false;
            static mut READY_FRAMES: u32 = 0;
            unsafe {
                if !TRIGGERED {
                    TRIGGERED = true;
                    let source = crate::core::pm2::LoadedSource::Path(PathBuf::from("smoke.log"));
                    self.trigger_pm2_parse(vec![source], false);
                } else if self.analysis.has_data && !self.analysis.is_parsing {
                    READY_FRAMES += 1;
                    if READY_FRAMES == 5 {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::default(),
                        ));
                    }
                    ctx.request_repaint();
                }
            }

            for event in &ctx.input(|i| i.raw.events.clone()) {
                if let egui::Event::Screenshot { image, .. } = event {
                    let w = image.size[0] as u32;
                    let h = image.size[1] as u32;
                    let pixels: Vec<u8> = image
                        .pixels
                        .iter()
                        .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
                        .collect();
                    if let Some(pixmap) = resvg::tiny_skia::PixmapRef::from_bytes(&pixels, w, h) {
                        let _ =
                            std::fs::write(&target_path, pixmap.encode_png().unwrap_or_default());
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        if let Ok(target_path) = std::env::var("PM2_CAPTURE_MONGO_SCREENSHOT") {
            static mut MONGO_TRIGGERED: bool = false;
            static mut MONGO_READY_FRAMES: u32 = 0;
            unsafe {
                if !MONGO_TRIGGERED {
                    MONGO_TRIGGERED = true;
                    self.mode = AppMode::Mongo;
                    let source = crate::core::pm2::LoadedSource::Path(PathBuf::from(
                        "../pm2-log-analyzer/mongodb_logs_sample/eSanad-mongod.log",
                    ));
                    self.trigger_mongo_parse(vec![source], false);
                } else if self.mongo.has_data && !self.mongo.is_parsing {
                    if MONGO_READY_FRAMES == 1 {
                        if let Ok(v) = std::env::var("PM2_CAPTURE_MONGO_VIEW") {
                            match v.as_str() {
                                "slow" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::SlowQueries
                                }
                                "users" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Users
                                }
                                "charts" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Charts
                                }
                                "plans" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Charts;
                                    self.mongo.chart_mode =
                                        crate::store::mongo_store::MongoChartMode::Plans;
                                }
                                "top-collections" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Charts;
                                    self.mongo.chart_mode =
                                        crate::store::mongo_store::MongoChartMode::TopCollections;
                                }
                                "diag" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Diagnostics
                                }
                                "connections" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Diagnostics;
                                    self.mongo.diag_tab =
                                        crate::store::mongo_store::MongoDiagTab::Connections;
                                }
                                "collections" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Diagnostics;
                                    self.mongo.diag_tab =
                                        crate::store::mongo_store::MongoDiagTab::Collections;
                                }
                                "checkpoints" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Diagnostics;
                                    self.mongo.diag_tab =
                                        crate::store::mongo_store::MongoDiagTab::Checkpoints;
                                }
                                "user-detail" => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Users;
                                    self.mongo.active_user_detail = self
                                        .mongo
                                        .result
                                        .as_ref()
                                        .and_then(|result| result.users.first().cloned());
                                }
                                _ => {
                                    self.mongo.active_view =
                                        crate::store::mongo_store::MongoActiveView::Patterns
                                }
                            }
                        }
                    }
                    MONGO_READY_FRAMES += 1;
                    if MONGO_READY_FRAMES == 5 {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::default(),
                        ));
                    }
                    ctx.request_repaint();
                }
            }

            for event in &ctx.input(|i| i.raw.events.clone()) {
                if let egui::Event::Screenshot { image, .. } = event {
                    let w = image.size[0] as u32;
                    let h = image.size[1] as u32;
                    let pixels: Vec<u8> = image
                        .pixels
                        .iter()
                        .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
                        .collect();
                    if let Some(pixmap) = resvg::tiny_skia::PixmapRef::from_bytes(&pixels, w, h) {
                        let _ =
                            std::fs::write(&target_path, pixmap.encode_png().unwrap_or_default());
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // 1. Process background events
        while let Ok(ev) = self.bg_rx.try_recv() {
            match ev {
                BgEvent::ArchiveExtracted { result, append } => {
                    self.analysis.is_parsing = false;
                    self.analysis.progress = None;
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
                                self.toasts
                                    .show("No valid API or MongoDB logs found in archive");
                            } else {
                                let has_pm2 = !pm2_files.is_empty();
                                let has_mongo = !mongo_files.is_empty();
                                let pm2_count = pm2_files.len();
                                let mongo_count = mongo_files.len();

                                if has_pm2 && has_mongo {
                                    self.toasts.show(format!(
                                        "Archive extracted in {}ms: {pm2_count} PM2 log(s), {mongo_count} Mongo log(s)",
                                        log_set.duration_ms
                                    ));
                                    self.trigger_pm2_parse(pm2_files, append);
                                    self.trigger_mongo_parse(mongo_files, append);
                                } else if has_pm2 {
                                    self.mode = AppMode::Pm2;
                                    let _ = persist_mode(AppMode::Pm2);
                                    self.toasts.show(format!(
                                        "Archive extracted in {}ms: {pm2_count} PM2 log(s)",
                                        log_set.duration_ms
                                    ));
                                    self.trigger_pm2_parse(pm2_files, append);
                                } else if has_mongo {
                                    self.mode = AppMode::Mongo;
                                    let _ = persist_mode(AppMode::Mongo);
                                    self.toasts.show(format!(
                                        "Archive extracted in {}ms: {mongo_count} Mongo log(s)",
                                        log_set.duration_ms
                                    ));
                                    self.trigger_mongo_parse(mongo_files, append);
                                }
                            }
                        }
                        Err(e) => {
                            self.toasts.show(format!("Failed to extract archive: {e}"));
                        }
                    }
                }
                BgEvent::Pm2Done {
                    kernel,
                    result,
                    files,
                } => {
                    self.analysis.is_parsing = false;
                    self.analysis.progress = None;
                    self.analysis.has_data = true;
                    self.analysis.file_name = files.first().map(|f| f.name());
                    self.analysis.file_size = Some(files.iter().map(|f| f.size()).sum());
                    self.analysis.file_names = files.iter().map(|f| f.name()).collect();
                    self.analysis.loaded_files = files;
                    self.analysis.result = Some(result);
                    self.pm2_kernel = Some(kernel);
                    self.refresh_derived_rows();
                    self.toasts.show("PM2 logs parsed successfully!");
                }
                BgEvent::Pm2Error(e) => {
                    self.analysis.is_parsing = false;
                    self.analysis.progress = None;
                    self.toasts.show(format!("Error parsing PM2 logs: {e}"));
                }
                BgEvent::MongoDone {
                    kernel,
                    result,
                    files,
                } => {
                    self.mongo.is_parsing = false;
                    self.mongo.progress = None;
                    self.mongo.has_data = true;
                    self.mongo.file_name = files.first().map(|f| f.name());
                    self.mongo.file_size = Some(files.iter().map(|f| f.size()).sum());
                    self.mongo.file_names = files.iter().map(|f| f.name()).collect();
                    self.mongo.loaded_files = files;
                    self.mongo.result = Some(result);
                    self.mongo_kernel = Some(kernel);
                    self.toasts.show("MongoDB logs parsed successfully!");
                }
                BgEvent::MongoError(e) => {
                    self.mongo.is_parsing = false;
                    self.mongo.progress = None;
                    self.toasts.show(format!("Error parsing MongoDB logs: {e}"));
                }
            }
        }

        // Keep repainting if jobs are running
        if self.analysis.is_parsing || self.mongo.is_parsing {
            if let Some(ctrl) = &self.job_control {
                let pct = ctrl.percent();
                if let Some(p) = &mut self.analysis.progress {
                    p.percent = pct;
                }
                if let Some(p) = &mut self.mongo.progress {
                    p.percent = pct;
                }
            }
            ctx.request_repaint_after(Duration::from_millis(60));
        }

        // 2. Drag & Drop handling
        let mut dropped_files = Vec::new();
        ctx.input(|i| {
            self.drag_over = !i.raw.hovered_files.is_empty();
            if !i.raw.dropped_files.is_empty() {
                for f in &i.raw.dropped_files {
                    let path = f.path();
                    if !path.as_os_str().is_empty() {
                        dropped_files.push(path.to_path_buf());
                    }
                }
            }
        });
        if !dropped_files.is_empty() {
            self.handle_dropped_paths(dropped_files, false);
        }

        // 3. Render Top Header
        let is_pm2 = self.mode == AppMode::Pm2;
        let pm2_dates = self.analysis.result.as_ref().map(|r| r.dates.as_slice());
        let mongo_dates = self.mongo.result.as_ref().map(|r| r.dates.as_slice());
        let mongo_stats = self.mongo.result.as_ref().map(|r| {
            (
                r.summary.total_lines as usize,
                r.summary.slow_query_count as usize,
                r.summary.overall_scan_ratio,
            )
        });

        let header_props = HeaderProps {
            mode: self.mode,
            dark,
            has_pm2_data: self.analysis.has_data,
            has_mongo_data: self.mongo.has_data,
            pm2_file_name: self.analysis.file_name.as_deref(),
            pm2_file_size: self.analysis.file_size,
            pm2_file_count: self.analysis.loaded_files.len(),
            pm2_dates,
            mongo_file_name: self.mongo.file_name.as_deref(),
            mongo_file_size: self.mongo.file_size,
            mongo_stats,
            mongo_dates,
            is_parsing: if is_pm2 {
                self.analysis.is_parsing
            } else {
                self.mongo.is_parsing
            },
        };

        // Frame::NONE keeps the default side/top frame's 2px inner margins and
        // 1px separator reservation out of the box: the header must be exactly
        // its reference height with the border painted by `render_header` itself.
        let header_action = egui::Panel::top("app_header")
            .frame(egui::Frame::NONE)
            .show_separator_line(false)
            .show(ui, |ui| render_header(ui, header_props))
            .inner;

        match header_action {
            HeaderAction::SwitchMode(m) => {
                self.mode = m;
                persist_mode(m);
            }
            HeaderAction::ToggleTheme => {
                self.analysis.theme = if dark { Theme::Light } else { Theme::Dark };
                // apply_theme runs at the top of the next frame, when the
                // transition begins holding the old palette.
            }
            HeaderAction::Export => {
                if is_pm2 {
                    if let Some(res) = &self.analysis.result {
                        let input = ExportInput {
                            api: &self.api_rows,
                            cron: &self.cron_rows,
                            hourly: &res.hourly_stats,
                            daily: &res.daily_stats,
                            summary: Some(&res.summary),
                            filters: &self.analysis.filters,
                            source_label: self.analysis.file_name.as_deref(),
                        };
                        match export_spreadsheet::build_workbook(&input) {
                            Ok(mut wb) => {
                                if let Some(save_path) = rfd::FileDialog::new()
                                    .set_file_name("pm2-log-analysis.xlsx")
                                    .add_filter("Excel Workbook", &["xlsx"])
                                    .save_file()
                                {
                                    if let Err(e) = wb.save(&save_path) {
                                        self.toasts.show(format!("Export failed: {e}"));
                                    } else {
                                        self.toasts.show("Report exported successfully!");
                                    }
                                }
                            }
                            Err(e) => {
                                self.toasts.show(format!("Export failed: {e}"));
                            }
                        }
                    }
                } else if let Some(res) = &self.mongo.result {
                    match export_mongo_spreadsheet::build_workbook(
                        res,
                        self.mongo.file_name.as_deref(),
                    ) {
                        Ok(mut wb) => {
                            if let Some(save_path) = rfd::FileDialog::new()
                                .set_file_name("mongo-log-analysis.xlsx")
                                .add_filter("Excel Workbook", &["xlsx"])
                                .save_file()
                            {
                                if let Err(e) = wb.save(&save_path) {
                                    self.toasts.show(format!("Export failed: {e}"));
                                } else {
                                    self.toasts.show("Report exported successfully!");
                                }
                            }
                        }
                        Err(e) => {
                            self.toasts.show(format!("Export failed: {e}"));
                        }
                    }
                }
            }
            HeaderAction::Clear => {
                if is_pm2 {
                    self.analysis = AnalysisState::default();
                    self.pm2_kernel = None;
                    self.api_rows.clear();
                    self.cron_rows.clear();
                    self.toasts.show("PM2 state cleared");
                } else {
                    self.mongo = MongoState::default();
                    self.mongo_kernel = None;
                    self.toasts.show("MongoDB state cleared");
                }
            }
            HeaderAction::None => {}
        }

        // 4. Central Panel with ScrollArea
        // Frame with no margins: the reference page padding (`px-4 py-4`) is
        // applied by the scroll content below — the default central frame adds
        // another 8px. The fill keeps the canvas background behind the cards.
        let page_bg = ui.style().visuals.panel_fill;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(page_bg))
            .show(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                let total_width = ui.available_width();
                let max_w = 1280.0;
                let inner_w = (total_width - 32.0).min(max_w).max(320.0);
                let side_pad = (total_width - inner_w) / 2.0;

                // The reference page is `px-4 py-4 gap-4` inside max-w-7xl.
                // Spacers that follow a widget must lose `item_spacing.y` (3px),
                // which egui adds before the next widget in the column.
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    ui.add_space(side_pad);
                    ui.vertical(|ui| {
                        ui.set_width(inner_w);

                if is_pm2 {
                    // PM2 Ingest
                    let ingest_props = IngestProps {
                        is_parsing: self.analysis.is_parsing,
                        progress: self.analysis.progress.as_ref(),
                        has_data: self.analysis.has_data,
                        loaded_files: &self.analysis.loaded_files,
                        paste_open: self.paste_open,
                        drag_over: self.drag_over,
                        dark,
                        is_mongo: false,
                    };

                    match render_ingest(ui, ingest_props, &mut self.paste_buffer) {
                        IngestAction::BrowseFiles { append } => {
                            if let Some(files) = rfd::FileDialog::new().pick_files() {
                                self.handle_dropped_paths(files, append);
                            }
                        }
                        IngestAction::BrowseFolder { append } => {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                if let Ok(entries) = std::fs::read_dir(folder) {
                                    let paths: Vec<PathBuf> =
                                        entries.filter_map(|e| e.ok().map(|d| d.path())).collect();
                                    self.handle_dropped_paths(paths, append);
                                }
                            }
                        }
                        IngestAction::TogglePaste => {
                            self.paste_open = !self.paste_open;
                        }
                        IngestAction::AnalyzePaste(text) => {
                            let source = LoadedSource::Memory {
                                name: "Pasted text".to_string(),
                                bytes: Arc::new(text.into_bytes()),
                            };
                            self.trigger_pm2_parse(vec![source], false);
                            self.paste_open = false;
                        }
                        IngestAction::CancelParse => {
                            if let Some(ctrl) = &self.job_control {
                                ctrl.cancel();
                            }
                            self.analysis.is_parsing = false;
                        }
                        IngestAction::None => {}
                    }

                    // PM2 Data Views
                    if let Some(result) = self.analysis.result.clone() {
                        ui.add_space(13.0);

                        // KPIs
                        let cron_jobs_count = result.cron_summary.jobs;
                        let has_cron_events = result.cron_summary.starts
                            + result.cron_summary.dones
                            + result.cron_summary.fails
                            > 0;
                        render_kpis(
                            ui,
                            &result.summary,
                            cron_jobs_count,
                            has_cron_events,
                            dark,
                        );

                        ui.add_space(13.0);

                        // Filters
                        let available_methods: Vec<String> = {
                            let mut m: Vec<String> =
                                result.api.iter().map(|a| a.method.as_str().to_string()).collect();
                            m.sort();
                            m.dedup();
                            m
                        };
                        match render_filters(
                            ui,
                            &mut self.analysis.filters,
                            &result.dates,
                            &available_methods,
                            dark,
                        ) {
                            FilterAction::Changed => {
                                self.refresh_derived_rows();
                            }
                            FilterAction::Reaggregate => {
                                self.reaggregate_pm2();
                            }
                            FilterAction::Reset => {
                                self.analysis.filters = AnalysisFilters::default();
                                self.reaggregate_pm2();
                                self.toasts.show("Filters reset to default");
                            }
                            FilterAction::None => {}
                        }

                        ui.add_space(13.0);

                        // At the React `lg` breakpoint the split table/chart
                        // layout becomes a single column. Keep that behavior
                        // when a saved Split preference is opened in a narrow
                        // native window.
                        let total_w = ui.available_width();
                        let chart_props = ChartProps {
                            rows: &self.api_rows,
                            hourly: &result.hourly_stats,
                            daily: &result.daily_stats,
                            layout: self.analysis.chart_layout,
                            date_filter: &self.analysis.filters.date_filter,
                            dark,
                        };

                        if self.analysis.chart_layout == ChartLayout::Wide {
                            if render_chart(ui, chart_props, &mut self.chart_mode)
                                == ChartAction::ToggleLayout
                            {
                                self.analysis.chart_layout = ChartLayout::Split;
                            }

                            ui.add_space(13.0);
                            let table_action = render_api_table(
                                ui,
                                &self.api_rows,
                                result.api.len(),
                                self.analysis.filters.top_n,
                                self.analysis.filters.sort_key,
                                self.analysis.filters.sort_dir,
                                420.0,
                                dark,
                            );
                            self.handle_api_table_action(ui, table_action);
                        } else if total_w < LG_CONTENT_WIDTH {
                            let table_action = render_api_table(
                                ui,
                                &self.api_rows,
                                result.api.len(),
                                self.analysis.filters.top_n,
                                self.analysis.filters.sort_key,
                                self.analysis.filters.sort_dir,
                                420.0,
                                dark,
                            );
                            self.handle_api_table_action(ui, table_action);

                            ui.add_space(13.0);
                            let chart_action = render_chart(
                                ui,
                                ChartProps {
                                    rows: &self.api_rows,
                                    hourly: &result.hourly_stats,
                                    daily: &result.daily_stats,
                                    layout: self.analysis.chart_layout,
                                    date_filter: &self.analysis.filters.date_filter,
                                    dark,
                                },
                                &mut self.chart_mode,
                            );
                            if chart_action == ChartAction::ToggleLayout {
                                self.analysis.chart_layout = ChartLayout::Wide;
                            }
                        } else {
                            // Split layout (3/5 API Table : 2/5 Chart), as in
                            // the reference app's `lg:grid-cols-5` layout.
                            let gap = 16.0;
                            let table_w = ((total_w - gap) * (3.0 / 5.0)).floor();
                            let chart_w = total_w - gap - table_w;

                            let mut table_action = ApiTableAction::None;
                            let mut chart_action = ChartAction::None;

                            ui.horizontal_top(|ui| {
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(table_w, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.set_width(table_w);
                                        table_action = render_api_table(
                                            ui,
                                            &self.api_rows,
                                            result.api.len(),
                                            self.analysis.filters.top_n,
                                            self.analysis.filters.sort_key,
                                            self.analysis.filters.sort_dir,
                                            420.0,
                                            dark,
                                        );
                                    },
                                );

                                ui.add_space(gap - ui.spacing().item_spacing.x);
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(chart_w, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        ui.set_width(chart_w);
                                        chart_action = render_chart(
                                            ui,
                                            chart_props,
                                            &mut self.chart_mode,
                                        );
                                    },
                                );
                            });

                            if chart_action == ChartAction::ToggleLayout {
                                self.analysis.chart_layout = ChartLayout::Wide;
                            }
                            self.handle_api_table_action(ui, table_action);
                        }

                        // Cron Table
                        if has_cron_events {
                            ui.add_space(13.0);
                            let action = render_cron_table(
                                ui,
                                &self.cron_rows,
                                &mut self.analysis.filters,
                                dark,
                            );
                            self.handle_cron_table_action(ui, action);
                        }

                        // Skipped lines
                        ui.add_space(13.0);
                        if let SkippedAction::CopySamples(text) = render_skipped(
                            ui,
                            result.summary.unmatched,
                            &result.unmatched_sample,
                            &mut self.skipped_open,
                            dark,
                        ) {
                            ui.copy_text(text);
                            self.toasts.show("Skipped sample lines copied to clipboard!");
                        }
                    }
                } else {
                    // Mongo Mode
                    let mongo_ingest_props = IngestProps {
                        is_parsing: self.mongo.is_parsing,
                        progress: self.mongo.progress.as_ref(),
                        has_data: self.mongo.has_data,
                        loaded_files: &self.mongo.loaded_files,
                        paste_open: self.paste_open,
                        drag_over: self.drag_over,
                        dark,
                        is_mongo: true,
                    };

                    match render_ingest(ui, mongo_ingest_props, &mut self.paste_buffer) {
                        IngestAction::BrowseFiles { append } => {
                            if let Some(files) = rfd::FileDialog::new().pick_files() {
                                self.handle_dropped_paths(files, append);
                            }
                        }
                        IngestAction::BrowseFolder { append } => {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                if let Ok(entries) = std::fs::read_dir(folder) {
                                    let paths: Vec<PathBuf> =
                                        entries.filter_map(|e| e.ok().map(|d| d.path())).collect();
                                    self.handle_dropped_paths(paths, append);
                                }
                            }
                        }
                        IngestAction::TogglePaste => {
                            self.paste_open = !self.paste_open;
                        }
                        IngestAction::AnalyzePaste(text) => {
                            let source = LoadedSource::Memory {
                                name: "Pasted text".to_string(),
                                bytes: Arc::new(text.into_bytes()),
                            };
                            self.trigger_mongo_parse(vec![source], false);
                            self.paste_open = false;
                        }
                        IngestAction::CancelParse => {
                            if let Some(ctrl) = &self.job_control {
                                ctrl.cancel();
                            }
                            self.mongo.is_parsing = false;
                        }
                        IngestAction::None => {}
                    }

                    if self.mongo.has_data {
                        ui.add_space(13.0);
                        match render_mongo_view(
                            ui,
                            &mut self.mongo,
                            &mut self.selected_mongo_query,
                            dark,
                        ) {
                            MongoViewAction::FiltersChanged => {
                                self.reaggregate_mongo();
                            }
                            MongoViewAction::CopyJson(json) => {
                                ui.copy_text(json);
                                self.toasts.show("Query JSON copied to clipboard!");
                            }
                            MongoViewAction::None => {}
                        }
                    }
                }

                ui.add_space(21.0);
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 360.0).max(0.0) / 2.0);
                        ui.add(super::egui_app::icons::render_icon(
                            "shield-alert",
                            theme::TailwindColors::SLATE_400,
                            12.0,
                        ));
                        ui.label(
                            egui::RichText::new(
                                "Parses locally in pure native Rust — logs never leave this machine",
                            )
                            .size(11.0)
                            .color(theme::TailwindColors::SLATE_400),
                        );
                    });
                });
                ui.add_space(37.0);
            });
            ui.add_space(side_pad);
        });
            });
        });

        // 5. Toast Notifications
        self.toasts.render(&ctx, dark);

        // 6. Theme-change reveal, painted last over the interface.
        self.theme_transition.paint(&ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::analysis_store::ApiSortKey;
    use egui::RawInput;

    fn create_test_ctx() -> egui::Context {
        let ctx = egui::Context::default();
        setup_fonts(&ctx);
        icons::install_icons(&ctx);
        ctx
    }

    fn run_test_frame<F: FnMut(&mut egui::Ui)>(ctx: &egui::Context, f: F) {
        let mut output = ctx.run_ui(RawInput::default(), f);
        output.textures_delta.clear();
    }

    #[test]
    fn test_api_table_empty_state() {
        let ctx = create_test_ctx();
        run_test_frame(&ctx, |ui| {
            let action = api_table::render_api_table(
                ui,
                &[],
                0,
                50,
                ApiSortKey::P95Ms,
                SortDirection::Desc,
                120.0,
                false,
            );
            assert!(matches!(action, api_table::ApiTableAction::None));
        });
    }

    #[test]
    fn test_cron_table_empty_state() {
        let ctx = create_test_ctx();
        let mut filters = AnalysisFilters::default();
        run_test_frame(&ctx, |ui| {
            let action = cron_table::render_cron_table(ui, &[], &mut filters, false);
            assert!(matches!(action, CronTableAction::None));
        });
    }

    #[test]
    fn test_toast_manager_lifecycle() {
        let mut toasts = ToastManager::default();
        toasts.show("Test message");

        let ctx = create_test_ctx();
        run_test_frame(&ctx, |_ui| {
            toasts.render(&ctx, false);
        });
    }

    #[test]
    fn test_skipped_action() {
        let ctx = create_test_ctx();
        let mut open = true;
        let samples = vec!["skipped line 1".to_string(), "skipped line 2".to_string()];
        run_test_frame(&ctx, |ui| {
            let action = skipped::render_skipped(ui, 2, &samples, &mut open, false);
            assert!(matches!(action, skipped::SkippedAction::None));
        });
    }
}
