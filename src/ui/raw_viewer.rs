use egui::{Color32, RichText, ScrollArea, Ui};

pub fn render_raw_viewer(ui: &mut Ui, samples: &[String]) {
    if samples.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("No unmatched or sample log lines recorded.").size(16.0).color(Color32::GRAY));
        });
        return;
    }

    ui.heading(RichText::new("Unmatched / Sample Raw Log Lines").color(Color32::WHITE));
    ui.label(RichText::new("Displaying sample log lines that did not match standard HTTP or Cron formats:").color(Color32::from_rgb(148, 163, 184)));
    ui.add_space(8.0);

    let row_height = 20.0;
    ScrollArea::both().show_rows(ui, row_height, samples.len(), |ui, row_range| {
        for idx in row_range {
            let line = &samples[idx];
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(RichText::new(line).monospace().size(12.0).color(Color32::from_rgb(226, 232, 240))));
            });
        }
    });
}
