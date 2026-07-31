use crate::core::models::CronStats;
use egui::{Color32, RichText, ScrollArea, Ui, Vec2};

pub fn render_cron_table(ui: &mut Ui, cron_jobs: &[CronStats]) {
    if cron_jobs.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("No cron job events detected in log file.").size(16.0).color(Color32::GRAY));
        });
        return;
    }

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

        header_title(ui, "Cron Task Name", 340.0, false);
        header_title(ui, "Total Runs", 100.0, true);
        header_title(ui, "Successes", 100.0, true);
        header_title(ui, "Failures", 100.0, true);
        header_title(ui, "Avg Dur (ms)", 100.0, true);
        header_title(ui, "Last Status", 100.0, true);
    });

    ui.separator();

    let row_height = 24.0;
    ScrollArea::vertical().show_rows(ui, row_height, cron_jobs.len(), |ui, row_range| {
        for idx in row_range {
            let job = &cron_jobs[idx];
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                // Task Name
                cell_left(
                    ui,
                    340.0,
                    row_height,
                    RichText::new(&job.name).strong().color(Color32::WHITE),
                    Some(&job.name),
                );

                // Total Runs
                cell_right(
                    ui,
                    100.0,
                    row_height,
                    RichText::new(format!("{}", job.total_runs)).color(Color32::from_rgb(226, 232, 240)),
                );

                // Successes
                cell_right(
                    ui,
                    100.0,
                    row_height,
                    RichText::new(format!("{}", job.total_success)).color(Color32::from_rgb(34, 197, 94)),
                );

                // Failures
                let fail_color = if job.total_failures > 0 {
                    Color32::from_rgb(239, 68, 68)
                } else {
                    Color32::from_rgb(148, 163, 184)
                };
                cell_right(
                    ui,
                    100.0,
                    row_height,
                    RichText::new(format!("{}", job.total_failures)).color(fail_color),
                );

                // Avg Dur (ms)
                cell_right(
                    ui,
                    100.0,
                    row_height,
                    RichText::new(format!("{:.1}", job.avg_duration_ms)).color(Color32::from_rgb(203, 213, 225)),
                );

                // Last Status
                let status_color = match job.last_status.as_str() {
                    "success" => Color32::from_rgb(34, 197, 94),
                    "failed" => Color32::from_rgb(239, 68, 68),
                    _ => Color32::from_rgb(148, 163, 184),
                };
                cell_right(
                    ui,
                    100.0,
                    row_height,
                    RichText::new(&job.last_status).color(status_color),
                );
            });
        }
    });
}

fn header_title(ui: &mut Ui, title: &str, width: f32, align_right: bool) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, 24.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            let layout = if align_right {
                egui::Layout::right_to_left(egui::Align::Center)
            } else {
                egui::Layout::left_to_right(egui::Align::Center)
            };
            ui.with_layout(layout, |ui| {
                ui.label(RichText::new(title).strong().color(Color32::from_rgb(148, 163, 184)));
            });
        },
    );
}

fn cell_left(ui: &mut Ui, width: f32, height: f32, text: RichText, tooltip: Option<&str>) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            let label = egui::Label::new(text).truncate();
            let res = ui.add(label);
            if let Some(tip) = tooltip {
                res.on_hover_text(tip);
            }
        },
    );
}

fn cell_right(ui: &mut Ui, width: f32, height: f32, text: RichText) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(egui::Label::new(text).truncate());
            });
        },
    );
}
