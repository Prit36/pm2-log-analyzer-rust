use crate::core::models::EndpointStats;
use crate::ui::theme::get_method_color;
use egui::{Color32, RichText, ScrollArea, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Path,
    Method,
    Calls,
    ErrorRate,
    AvgDuration,
    P50,
    P95,
    P99,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone)]
pub struct TableSortState {
    pub column: SortColumn,
    pub direction: SortDirection,
}

impl Default for TableSortState {
    fn default() -> Self {
        Self {
            column: SortColumn::Calls,
            direction: SortDirection::Descending,
        }
    }
}

pub fn render_endpoint_table(
    ui: &mut Ui,
    endpoints: &[EndpointStats],
    sort_state: &mut TableSortState,
) -> bool {
    if endpoints.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("No matching endpoints found").size(16.0).color(Color32::GRAY));
        });
        return false;
    }

    let mut sort_changed = false;

    // Render Header
    ui.horizontal(|ui| {
        sort_changed |= header_cell(ui, "Method", 70.0, SortColumn::Method, sort_state);
        sort_changed |= header_cell(ui, "Endpoint Path", 320.0, SortColumn::Path, sort_state);
        sort_changed |= header_cell(ui, "Total Calls", 100.0, SortColumn::Calls, sort_state);
        sort_changed |= header_cell(ui, "Error Rate", 90.0, SortColumn::ErrorRate, sort_state);
        sort_changed |= header_cell(ui, "Avg (ms)", 80.0, SortColumn::AvgDuration, sort_state);
        sort_changed |= header_cell(ui, "p50 (ms)", 80.0, SortColumn::P50, sort_state);
        sort_changed |= header_cell(ui, "p95 (ms)", 80.0, SortColumn::P95, sort_state);
        sort_changed |= header_cell(ui, "p99 (ms)", 80.0, SortColumn::P99, sort_state);
    });

    ui.separator();

    // Virtualized List
    let row_height = 24.0;
    let total_rows = endpoints.len();

    ScrollArea::vertical().show_rows(ui, row_height, total_rows, |ui, row_range| {
        for idx in row_range {
            let ep = &endpoints[idx];

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                // Method badge
                let method_color = get_method_color(ep.method.as_str());
                ui.add_sized(
                    Vec2::new(70.0, row_height),
                    egui::Label::new(RichText::new(ep.method.as_str()).strong().color(method_color)),
                );

                // Path
                ui.add_sized(
                    Vec2::new(320.0, row_height),
                    egui::Label::new(RichText::new(&ep.path).color(Color32::WHITE)),
                );

                // Calls
                ui.add_sized(
                    Vec2::new(100.0, row_height),
                    egui::Label::new(RichText::new(format!("{}", ep.total_calls)).color(Color32::from_rgb(226, 232, 240))),
                );

                // Error Rate
                let err_rate = ep.error_rate();
                let err_color = if err_rate > 5.0 {
                    Color32::from_rgb(239, 68, 68)
                } else if err_rate > 0.0 {
                    Color32::from_rgb(245, 158, 11)
                } else {
                    Color32::from_rgb(34, 197, 94)
                };
                ui.add_sized(
                    Vec2::new(90.0, row_height),
                    egui::Label::new(RichText::new(format!("{:.1}%", err_rate)).color(err_color)),
                );

                // Avg
                ui.add_sized(
                    Vec2::new(80.0, row_height),
                    egui::Label::new(RichText::new(format!("{:.1}", ep.avg_duration_ms())).color(Color32::from_rgb(203, 213, 225))),
                );

                // p50
                ui.add_sized(
                    Vec2::new(80.0, row_height),
                    egui::Label::new(RichText::new(format!("{:.1}", ep.p50_ms)).color(Color32::from_rgb(203, 213, 225))),
                );

                // p95
                ui.add_sized(
                    Vec2::new(80.0, row_height),
                    egui::Label::new(RichText::new(format!("{:.1}", ep.p95_ms)).color(Color32::from_rgb(245, 158, 11))),
                );

                // p99
                ui.add_sized(
                    Vec2::new(80.0, row_height),
                    egui::Label::new(RichText::new(format!("{:.1}", ep.p99_ms)).color(Color32::from_rgb(239, 68, 68))),
                );
            });
        }
    });

    sort_changed
}

fn header_cell(
    ui: &mut Ui,
    title: &str,
    width: f32,
    col: SortColumn,
    sort_state: &mut TableSortState,
) -> bool {
    let active = sort_state.column == col;
    let arrow = if active {
        match sort_state.direction {
            SortDirection::Ascending => " ▲",
            SortDirection::Descending => " ▼",
        }
    } else {
        ""
    };

    let text = format!("{}{}", title, arrow);
    let label = RichText::new(text).strong().color(if active { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(148, 163, 184) });

    if ui.add_sized(Vec2::new(width, 26.0), egui::Button::new(label).frame(false)).clicked() {
        if sort_state.column == col {
            sort_state.direction = match sort_state.direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            sort_state.column = col;
            sort_state.direction = SortDirection::Descending;
        }
        true
    } else {
        false
    }
}
