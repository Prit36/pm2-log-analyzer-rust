use crate::core::aggregator::{parse_log_buffer, Engine};
use crate::core::mmap::MmapReader;
use crate::core::models::{EndpointStats, FilterOptions, LogAnalysisSummary, Method, PathNormMode, StatusFamily};
use crate::ui::charts::render_charts;
use crate::ui::cron_table::render_cron_table;
use crate::ui::endpoint_table::{render_endpoint_table, SortColumn, SortDirection, TableSortState};
use crate::ui::kpi_cards::render_kpi_dashboard;
use crate::ui::raw_viewer::render_raw_viewer;
use crate::utils::exporter::{export_to_csv, export_to_json};

use egui::{Align, Color32, Layout, RichText};
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Endpoints,
    CronJobs,
    Charts,
    RawLogs,
}

enum ParseMessage {
    Complete {
        engine: Arc<Engine>,
        summary: LogAnalysisSummary,
        file_size: u64,
        elapsed_ms: u64,
    },
    Error(String),
}

pub struct Pm2App {
    pub current_file: Option<PathBuf>,
    pub is_parsing: bool,
    pub parse_progress: f32,
    pub status_msg: String,

    pub engine: Option<Arc<Engine>>,
    pub file_size: u64,
    pub parse_duration_ms: u64,
    pub summary: LogAnalysisSummary,
    pub sorted_endpoints: Vec<EndpointStats>,

    pub filters: FilterOptions,
    pub sort_state: TableSortState,
    pub active_tab: ActiveTab,

    tx: Sender<ParseMessage>,
    rx: Receiver<ParseMessage>,
}

impl Default for Pm2App {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            current_file: None,
            is_parsing: false,
            parse_progress: 0.0,
            status_msg: "Ready. Drop a PM2 log file or click Open File.".to_string(),
            engine: None,
            file_size: 0,
            parse_duration_ms: 0,
            summary: LogAnalysisSummary::default(),
            sorted_endpoints: Vec::new(),
            filters: FilterOptions::default(),
            sort_state: TableSortState::default(),
            active_tab: ActiveTab::Endpoints,
            tx,
            rx,
        }
    }
}

impl Pm2App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::ui::theme::apply_ops_theme(&cc.egui_ctx);
        Self::default()
    }

    pub fn load_file(&mut self, path: PathBuf, ctx: egui::Context) {
        self.is_parsing = true;
        self.parse_progress = 0.1;
        self.status_msg = format!("Opening file {}...", path.display());
        self.current_file = Some(path.clone());

        let tx = self.tx.clone();
        let filters = self.filters.clone();
        thread::spawn(move || {
            let start = Instant::now();
            match MmapReader::open(&path) {
                Ok(reader) => {
                    let file_size = reader.len() as u64;
                    let engine = parse_log_buffer(reader.as_slice());
                    let elapsed_ms = start.elapsed().as_millis() as u64;
                    let summary = engine.aggregate(file_size, elapsed_ms, &filters);
                    let engine = Arc::new(engine);

                    let _ = tx.send(ParseMessage::Complete {
                        engine,
                        summary,
                        file_size,
                        elapsed_ms,
                    });
                    ctx.request_repaint();
                }
                Err(e) => {
                    let _ = tx.send(ParseMessage::Error(format!("Failed to open file: {}", e)));
                    ctx.request_repaint();
                }
            }
        });
    }

    pub fn update_sorted_endpoints(&mut self) {
        let mut sorted = self.summary.endpoints.clone();
        let col = self.sort_state.column;
        let dir = self.sort_state.direction;

        sorted.sort_by(|a, b| {
            let cmp = match col {
                SortColumn::Path => a.path.cmp(&b.path),
                SortColumn::Method => a.method.as_str().cmp(b.method.as_str()),
                SortColumn::Calls => a.total_calls.cmp(&b.total_calls),
                SortColumn::ErrorRate => a.error_rate().total_cmp(&b.error_rate()),
                SortColumn::AvgDuration => a.avg_duration_ms().total_cmp(&b.avg_duration_ms()),
                SortColumn::P50 => a.p50_ms.total_cmp(&b.p50_ms),
                SortColumn::P95 => a.p95_ms.total_cmp(&b.p95_ms),
                SortColumn::P99 => a.p99_ms.total_cmp(&b.p99_ms),
            };
            match dir {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });

        self.sorted_endpoints = sorted;
    }

    pub fn recompute_summary(&mut self) {
        if let Some(ref engine) = self.engine {
            self.summary = engine.aggregate(self.file_size, self.parse_duration_ms, &self.filters);
            self.update_sorted_endpoints();
        }
    }

    fn check_async_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                ParseMessage::Complete {
                    engine,
                    summary,
                    file_size,
                    elapsed_ms,
                } => {
                    self.is_parsing = false;
                    self.file_size = file_size;
                    self.parse_duration_ms = elapsed_ms;
                    self.status_msg = format!(
                        "Parsed {} lines in {} ms",
                        engine.total_lines, elapsed_ms
                    );
                    self.engine = Some(engine);
                    self.summary = summary;
                    self.update_sorted_endpoints();
                }
                ParseMessage::Error(err) => {
                    self.is_parsing = false;
                    self.status_msg = err;
                }
            }
        }
    }

}

impl eframe::App for Pm2App {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        std::process::exit(0);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_async_messages();

        if self.is_parsing {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        // Handle File Drag & Drop
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                if let Some(path) = i.raw.dropped_files[0].path.clone() {
                    self.load_file(path, ctx.clone());
                }
            }
        });

        // Top Control Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("PM2 Log Analyzer Native").strong().size(18.0).color(Color32::WHITE));
                ui.add_space(20.0);

                if ui.button("📂 Open Log File").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("Log files", &["log", "txt"]).pick_file() {
                        self.load_file(path, ctx.clone());
                    }
                }

                if self.summary.matched_http_requests > 0 {
                    if ui.button("💾 Export CSV").clicked() {
                        if let Some(path) = FileDialog::new().set_file_name("pm2_analysis.csv").save_file() {
                            if let Err(e) = export_to_csv(&self.summary, path.to_str().unwrap_or("")) {
                                self.status_msg = format!("Export failed: {}", e);
                            } else {
                                self.status_msg = format!("Exported CSV to {}", path.display());
                            }
                        }
                    }

                    if ui.button("💾 Export JSON").clicked() {
                        if let Some(path) = FileDialog::new().set_file_name("pm2_analysis.json").save_file() {
                            if let Err(e) = export_to_json(&self.summary, path.to_str().unwrap_or("")) {
                                self.status_msg = format!("Export failed: {}", e);
                            } else {
                                self.status_msg = format!("Exported JSON to {}", path.display());
                            }
                        }
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.is_parsing {
                        ui.spinner();
                        ui.label(RichText::new(&self.status_msg).color(Color32::from_rgb(245, 158, 11)));
                    } else {
                        ui.label(RichText::new(&self.status_msg).color(Color32::from_rgb(148, 163, 184)));
                    }
                });
            });
            ui.add_space(6.0);
        });

        // Bottom Status Bar
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(ref path) = self.current_file {
                    let display_str = format!("File: {}", path.display());
                    let label = egui::Label::new(RichText::new(&display_str).size(11.0).color(Color32::from_rgb(148, 163, 184)))
                        .truncate();
                    ui.add(label).on_hover_text(path.to_string_lossy());
                } else {
                    ui.label(RichText::new("No file loaded").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                }
            });
        });

        // Main Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.summary.total_lines_parsed == 0 && !self.is_parsing {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading(RichText::new("Drop a PM2 log file here").size(24.0).color(Color32::WHITE));
                    ui.add_space(10.0);
                    ui.label(RichText::new("Supports multi-gigabyte log files with instant memory-mapped multi-threaded parsing.").color(Color32::from_rgb(148, 163, 184)));
                    ui.add_space(20.0);
                    if ui.button("Select File from Disk").clicked() {
                        if let Some(path) = FileDialog::new().add_filter("Log files", &["log", "txt"]).pick_file() {
                            self.load_file(path, ctx.clone());
                        }
                    }
                });
                return;
            }

            // Render Top KPI Cards
            render_kpi_dashboard(ui, &self.summary);
            ui.add_space(16.0);

            // Filter Bar
            let mut filter_changed = false;
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Search:").strong().color(Color32::WHITE));
                    if ui.add(egui::TextEdit::singleline(&mut self.filters.search_query).desired_width(220.0).hint_text("Filter path...")).changed() {
                        filter_changed = true;
                    }

                    ui.separator();
                    ui.label(RichText::new("Method:").strong().color(Color32::WHITE));
                    let mut method_opt = self.filters.method.map(|m| m.as_str()).unwrap_or("ALL");
                    egui::ComboBox::from_id_salt("method_combo")
                        .selected_text(method_opt)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut method_opt, "ALL", "ALL").clicked() {
                                self.filters.method = None;
                                filter_changed = true;
                            }
                            for m in &[Method::Get, Method::Post, Method::Put, Method::Patch, Method::Delete, Method::Options] {
                                if ui.selectable_value(&mut method_opt, m.as_str(), m.as_str()).clicked() {
                                    self.filters.method = Some(*m);
                                    filter_changed = true;
                                }
                            }
                        });

                    ui.separator();
                    ui.label(RichText::new("Status:").strong().color(Color32::WHITE));
                    if ui.radio_value(&mut self.filters.status_family, StatusFamily::All, "All").changed()
                        || ui.radio_value(&mut self.filters.status_family, StatusFamily::Success, "2xx").changed()
                        || ui.radio_value(&mut self.filters.status_family, StatusFamily::ClientError, "4xx").changed()
                        || ui.radio_value(&mut self.filters.status_family, StatusFamily::ServerError, "5xx").changed()
                        || ui.radio_value(&mut self.filters.status_family, StatusFamily::ErrorOnly, "Errors Only").changed()
                    {
                        filter_changed = true;
                    }

                    ui.separator();
                    ui.label(RichText::new("Path Norm:").strong().color(Color32::WHITE));
                    let norm_text = match self.filters.path_norm_mode {
                        PathNormMode::Raw => "Raw Path",
                        PathNormMode::StripQuery => "Strip Query",
                        PathNormMode::CollapseIds => "Collapse IDs (:id)",
                    };
                    egui::ComboBox::from_id_salt("norm_combo")
                        .selected_text(norm_text)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut self.filters.path_norm_mode, PathNormMode::CollapseIds, "Collapse IDs (:id)").clicked()
                                || ui.selectable_value(&mut self.filters.path_norm_mode, PathNormMode::StripQuery, "Strip Query").clicked()
                                || ui.selectable_value(&mut self.filters.path_norm_mode, PathNormMode::Raw, "Raw Path").clicked()
                            {
                                filter_changed = true;
                            }
                        });
                });
            });

            if filter_changed {
                self.recompute_summary();
            }

            ui.add_space(12.0);

            // Tab navigation
            ui.horizontal(|ui| {
                if ui.selectable_label(self.active_tab == ActiveTab::Endpoints, "📊 API Endpoints").clicked() {
                    self.active_tab = ActiveTab::Endpoints;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::CronJobs, "⏱ Cron Jobs").clicked() {
                    self.active_tab = ActiveTab::CronJobs;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::Charts, "📈 Latency Charts").clicked() {
                    self.active_tab = ActiveTab::Charts;
                }
                if ui.selectable_label(self.active_tab == ActiveTab::RawLogs, "🔍 Raw Log Samples").clicked() {
                    self.active_tab = ActiveTab::RawLogs;
                }
            });

            ui.separator();

            // Render Active Tab content
            match self.active_tab {
                ActiveTab::Endpoints => {
                    if render_endpoint_table(ui, &self.sorted_endpoints, &mut self.sort_state) {
                        self.update_sorted_endpoints();
                    }
                }
                ActiveTab::CronJobs => render_cron_table(ui, &self.summary.cron_jobs),
                ActiveTab::Charts => render_charts(ui, &self.summary),
                ActiveTab::RawLogs => render_raw_viewer(ui, &self.summary.unmatched_samples),
            }
        });
    }
}
