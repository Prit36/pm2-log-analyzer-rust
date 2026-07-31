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
        ui.add_sized(Vec2::new(260.0, 24.0), egui::Label::new(RichText::new("Cron Task Name").strong().color(Color32::from_rgb(148, 163, 184))));
        ui.add_sized(Vec2::new(100.0, 24.0), egui::Label::new(RichText::new("Total Runs").strong().color(Color32::from_rgb(148, 163, 184))));
        ui.add_sized(Vec2::new(100.0, 24.0), egui::Label::new(RichText::new("Successes").strong().color(Color32::from_rgb(148, 163, 184))));
        ui.add_sized(Vec2::new(100.0, 24.0), egui::Label::new(RichText::new("Failures").strong().color(Color32::from_rgb(148, 163, 184))));
        ui.add_sized(Vec2::new(100.0, 24.0), egui::Label::new(RichText::new("Avg Dur (ms)").strong().color(Color32::from_rgb(148, 163, 184))));
        ui.add_sized(Vec2::new(100.0, 24.0), egui::Label::new(RichText::new("Last Status").strong().color(Color32::from_rgb(148, 163, 184))));
    });

    ui.separator();

    let row_height = 24.0;
    ScrollArea::vertical().show_rows(ui, row_height, cron_jobs.len(), |ui, row_range| {
        for idx in row_range {
            let job = &cron_jobs[idx];
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

                ui.add_sized(Vec2::new(260.0, row_height), egui::Label::new(RichText::new(&job.name).strong().color(Color32::WHITE)));
                ui.add_sized(Vec2::new(100.0, row_height), egui::Label::new(RichText::new(format!("{}", job.total_runs)).color(Color32::from_rgb(226, 232, 240))));
                ui.add_sized(Vec2::new(100.0, row_height), egui::Label::new(RichText::new(format!("{}", job.total_success)).color(Color32::from_rgb(34, 197, 94))));
                ui.add_sized(Vec2::new(100.0, row_height), egui::Label::new(RichText::new(format!("{}", job.total_failures)).color(
                    if job.total_failures > 0 { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(148, 163, 184) }
                )));
                ui.add_sized(Vec2::new(100.0, row_height), egui::Label::new(RichText::new(format!("{:.1}", job.avg_duration_ms)).color(Color32::from_rgb(203, 213, 225))));

                let status_color = match job.last_status.as_str() {
                    "success" => Color32::from_rgb(34, 197, 94),
                    "failed" => Color32::from_rgb(239, 68, 68),
                    _ => Color32::from_rgb(148, 163, 184),
                };
                ui.add_sized(Vec2::new(100.0, row_height), egui::Label::new(RichText::new(&job.last_status).color(status_color)));
            });
        }
    });
}
