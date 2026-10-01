use super::theme::{LH_XS, TailwindColors, font_medium, font_mono, font_regular};
use crate::utils::format::format_num;
use egui::{Color32, CornerRadius, Frame, Margin, Stroke, Ui};

pub enum SkippedAction {
    None,
    CopySamples(String),
}

pub fn render_skipped(
    ui: &mut Ui,
    skipped_count: u64,
    samples: &[String],
    open: &mut bool,
    dark: bool,
) -> SkippedAction {
    let mut action = SkippedAction::None;
    if skipped_count == 0 {
        return action;
    }

    let border_color = if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_200
    };
    let bg_color = if dark {
        TailwindColors::SLATE_900
    } else {
        Color32::WHITE
    };
    let hover_bg = if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_50
    };

    // Reference: `rounded border`, summary `px-3 py-2 ... hover:bg-slate-50`
    // (full-row hover), body behind `border-t px-3 py-2`.
    Frame::NONE
        .fill(bg_color)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::ZERO)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Summary row: full-bleed hover, 12px side padding, 8px rows.
                let row_h = 34.0;
                let total_w = ui.available_width();
                let (row_rect, row_resp) =
                    ui.allocate_exact_size(egui::vec2(total_w, row_h), egui::Sense::click());

                if ui.is_rect_visible(row_rect) {
                    if row_resp.hovered() {
                        ui.painter()
                            .rect_filled(row_rect, CornerRadius::ZERO, hover_bg);
                    }

                    let mut row_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(row_rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    row_ui.spacing_mut().item_spacing.x = 6.0;
                    row_ui.add_space(12.0);

                    let chevron = if *open {
                        super::icons::render_icon("chevron-down", TailwindColors::SLATE_400, 14.0)
                    } else {
                        super::icons::render_icon("chevron-right", TailwindColors::SLATE_400, 14.0)
                    };
                    row_ui.add(chevron);

                    row_ui.label(
                        egui::RichText::new(format!("{} lines skipped", format_num(skipped_count)))
                            .font(font_medium(12.0))
                            .line_height(Some(LH_XS))
                            .color(if dark {
                                TailwindColors::SLATE_300
                            } else {
                                TailwindColors::SLATE_600
                            }),
                    );
                    row_ui.label(
                        egui::RichText::new("(non-HTTP / unmatched)")
                            .font(font_regular(12.0))
                            .line_height(Some(LH_XS))
                            .color(if dark {
                                TailwindColors::SLATE_500
                            } else {
                                TailwindColors::SLATE_400
                            }),
                    );

                    if *open && !samples.is_empty() {
                        row_ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.add_space(12.0);
                                let btn = ui
                                    .horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        ui.add(super::icons::render_icon(
                                            "copy",
                                            TailwindColors::SLATE_400,
                                            12.0,
                                        ));
                                        ui.label(
                                            egui::RichText::new("Copy Samples")
                                                .font(font_medium(11.0))
                                                .color(if dark {
                                                    TailwindColors::SLATE_400
                                                } else {
                                                    TailwindColors::SLATE_500
                                                }),
                                        )
                                    })
                                    .response;
                                if btn.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if btn.interact(egui::Sense::click()).clicked() {
                                    action = SkippedAction::CopySamples(samples.join("\n"));
                                }
                            },
                        );
                    }
                }

                if row_resp.clicked() {
                    *open = !*open;
                }
                if row_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if *open {
                    // border-t px-3 py-2 body.
                    ui.painter().line_segment(
                        [row_rect.left_bottom(), row_rect.right_bottom()],
                        Stroke::new(
                            1.0,
                            if dark {
                                TailwindColors::SLATE_800
                            } else {
                                TailwindColors::SLATE_100
                            },
                        ),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        ui.vertical(|ui| {
                            egui::ScrollArea::vertical()
                                .max_height(192.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.y = 4.0;
                                    for line in samples.iter() {
                                        ui.label(
                                            egui::RichText::new(line).font(font_mono(11.0)).color(
                                                if dark {
                                                    TailwindColors::SLATE_400
                                                } else {
                                                    TailwindColors::SLATE_600
                                                },
                                            ),
                                        );
                                    }
                                });

                            if skipped_count > samples.len() as u64 {
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Showing {} of {} samples",
                                        samples.len(),
                                        format_num(skipped_count)
                                    ))
                                    .font(font_medium(11.0))
                                    .color(if dark {
                                        TailwindColors::SLATE_500
                                    } else {
                                        TailwindColors::SLATE_400
                                    }),
                                );
                            }
                        });
                    });
                }
            });
        });

    action
}
