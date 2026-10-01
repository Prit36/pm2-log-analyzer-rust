use super::theme::{TailwindColors, font_mono_semibold, font_semibold};
use crate::core::models::LogSummary;
use crate::utils::format::{format_ms, format_num};
use egui::{Color32, CornerRadius, Frame, Margin, Stroke, Ui, Vec2};

pub fn render_kpis(
    ui: &mut Ui,
    summary: &LogSummary,
    cron_jobs: u64,
    has_cron_events: bool,
    dark: bool,
) {
    let border_color = if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_200
    };
    let cell_bg = if dark {
        TailwindColors::SLATE_900
    } else {
        Color32::WHITE
    };

    struct KpiItem {
        label: &'static str,
        value: String,
        color: Color32,
    }

    let p95_color = if dark {
        TailwindColors::BLUE_400
    } else {
        TailwindColors::BLUE_600
    };

    let error_color = if summary.errors > 0 {
        if dark {
            TailwindColors::ROSE_400
        } else {
            TailwindColors::ROSE_600
        }
    } else if dark {
        TailwindColors::SLATE_100
    } else {
        TailwindColors::SLATE_900
    };

    let default_val_color = if dark {
        TailwindColors::SLATE_100
    } else {
        TailwindColors::SLATE_900
    };

    let mut items = vec![
        KpiItem {
            label: "REQUESTS",
            value: format_num(summary.matched),
            color: default_val_color,
        },
        KpiItem {
            label: "AVG",
            value: format_ms(summary.avg),
            color: default_val_color,
        },
        KpiItem {
            label: "P95",
            value: format_ms(summary.p95_ms),
            color: p95_color,
        },
        KpiItem {
            label: "ERRORS",
            value: format_num(summary.errors),
            color: error_color,
        },
        KpiItem {
            label: "SLOW ≥3S",
            value: format_num(summary.slow),
            color: default_val_color,
        },
    ];

    if has_cron_events {
        items.push(KpiItem {
            label: "CRON JOBS",
            value: format_num(cron_jobs),
            color: default_val_color,
        });
    }

    let pad_x = 12.0;
    let pad_y = 12.0;
    let col_count = items.len();
    let total_width = ui.available_width();
    let total_hairlines = (col_count as f32 - 1.0) * 1.0;
    let avail_for_cells = total_width - total_hairlines - 2.0;
    let cell_width = (avail_for_cells / col_count as f32).floor();
    let inner_cell_w = (cell_width - (pad_x * 2.0)).max(10.0);

    // Single unified card with gap-px divider hairlines (matches reference `grid gap-px rounded border border-slate-200 bg-slate-200 dark:border-slate-800 dark:bg-slate-800`)
    Frame::NONE
        .fill(border_color)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::ZERO)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0);
            ui.horizontal(|ui| {
                for (i, item) in items.iter().enumerate() {
                    let cell_corner = if i == 0 {
                        CornerRadius {
                            nw: 3,
                            ne: 0,
                            sw: 3,
                            se: 0,
                        }
                    } else if i == col_count - 1 {
                        CornerRadius {
                            nw: 0,
                            ne: 3,
                            sw: 0,
                            se: 3,
                        }
                    } else {
                        CornerRadius::ZERO
                    };
                    Frame::NONE
                        .fill(cell_bg)
                        .corner_radius(cell_corner)
                        .inner_margin(Margin::symmetric(pad_x as i8, pad_y as i8))
                        .show(ui, |ui| {
                            ui.set_width(inner_cell_w);
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(item.label)
                                        .font(font_semibold(10.0))
                                        .line_height(Some(super::theme::LH_10))
                                        .extra_letter_spacing(0.25)
                                        .color(if dark {
                                            TailwindColors::SLATE_400
                                        } else {
                                            TailwindColors::SLATE_500
                                        }),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(&item.value)
                                        .font(font_mono_semibold(18.0))
                                        .line_height(Some(28.0))
                                        .color(item.color),
                                );
                            });
                        });
                }
            });
        });
}
