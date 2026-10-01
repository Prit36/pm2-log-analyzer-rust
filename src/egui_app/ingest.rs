use super::icons::Icon;
use super::theme::{
    LH_XS, TailwindColors, font_bold, font_medium, font_mono, font_regular, font_semibold,
};
use super::widgets::{HDR, PAD, SMALL, cancel_button, primary_button, secondary_button};
use crate::core::pm2::LoadedSource;
use crate::store::analysis_store::ParseProgress;
use crate::utils::format::format_bytes;
use egui::{Align, Color32, CornerRadius, Frame, Margin, Rect, Stroke, Ui, pos2, vec2};

pub enum IngestAction {
    None,
    BrowseFiles { append: bool },
    BrowseFolder { append: bool },
    TogglePaste,
    AnalyzePaste(String),
    CancelParse,
}

pub struct IngestProps<'a> {
    pub is_parsing: bool,
    pub progress: Option<&'a ParseProgress>,
    pub has_data: bool,
    pub loaded_files: &'a [LoadedSource],
    pub paste_open: bool,
    pub drag_over: bool,
    pub dark: bool,
    pub is_mongo: bool,
}

pub fn render_ingest(
    ui: &mut Ui,
    props: IngestProps<'_>,
    paste_buffer: &mut String,
) -> IngestAction {
    let mut action = IngestAction::None;
    let dark = props.dark;

    let border_color = if props.drag_over {
        TailwindColors::BLUE_500
    } else if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_200
    };

    let bg_color = if props.drag_over {
        if dark {
            TailwindColors::BLUE_950.gamma_multiply(0.5)
        } else {
            TailwindColors::BLUE_50.gamma_multiply(0.8)
        }
    } else if dark {
        TailwindColors::SLATE_900
    } else {
        Color32::WHITE
    };

    // 1. Parsing progress bar view
    if props.is_parsing {
        // Reference: cancel sits inside the loaded panel when files exist
        // (`px-2.5 py-1`) and inside the dropzone otherwise (`px-3 py-1.5`).
        let cancel_style = if props.has_data { SMALL } else { PAD };
        Frame::NONE
            .fill(bg_color)
            .stroke(Stroke::new(1.0, border_color))
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (stage, pct_val) = if let Some(p) = props.progress {
                        (p.stage.as_str(), p.percent)
                    } else {
                        ("Parsing log files...", 30)
                    };

                    ui.label(
                        egui::RichText::new(stage)
                            .font(font_medium(12.0))
                            .color(if dark {
                                TailwindColors::SLATE_200
                            } else {
                                TailwindColors::SLATE_800
                            }),
                    );

                    ui.label(
                        egui::RichText::new(format!("{pct_val}%"))
                            .font(font_bold(12.0))
                            .color(if dark {
                                TailwindColors::BLUE_400
                            } else {
                                TailwindColors::BLUE_600
                            }),
                    );

                    let bar_w = (ui.available_width() - 90.0).max(100.0);
                    let (bar_rect, _) =
                        ui.allocate_exact_size(vec2(bar_w, 8.0), egui::Sense::hover());
                    if ui.is_rect_visible(bar_rect) {
                        let bar_bg = if dark {
                            TailwindColors::SLATE_800
                        } else {
                            TailwindColors::SLATE_100
                        };
                        ui.painter()
                            .rect_filled(bar_rect, CornerRadius::same(4), bar_bg);
                        let fill_w = bar_rect.width() * (pct_val as f32 / 100.0).clamp(0.0, 1.0);
                        let fill_rect =
                            Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_rect.height()));
                        ui.painter().rect_filled(
                            fill_rect,
                            CornerRadius::same(4),
                            TailwindColors::BLUE_600,
                        );
                    }

                    if cancel_button(ui, "Cancel", dark, cancel_style).clicked() {
                        action = IngestAction::CancelParse;
                    }
                });
            });
        return action;
    }

    // 2. Mongo mode has-data view (matches MongoIngestPanel.tsx with dashed border)
    if props.is_mongo && props.has_data {
        let total_files = props.loaded_files.len();
        let frame = Frame::NONE
            .fill(bg_color)
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin::symmetric(20, 12));

        let resp = frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Loaded {} file{}",
                            total_files,
                            if total_files != 1 { "s" } else { "" }
                        ))
                        .font(font_medium(12.0))
                        .color(if dark {
                            TailwindColors::SLATE_400
                        } else {
                            TailwindColors::SLATE_600
                        }),
                    );

                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if secondary_button(ui, "Replace", Some("upload"), dark, HDR).clicked() {
                            action = IngestAction::BrowseFiles { append: false };
                        }
                        ui.add_space(6.0);
                        if secondary_button(ui, "Add More Files", Some("plus"), dark, HDR).clicked()
                        {
                            action = IngestAction::BrowseFiles { append: true };
                        }
                    });
                });
            })
            .response;

        paint_dashed_rect(ui.painter(), resp.rect, Stroke::new(1.5, border_color));
        return action;
    }

    // 3. Has data view (compact bar matching reference HasDataPanel)
    if props.has_data {
        Frame::NONE
            .fill(bg_color)
            .stroke(Stroke::new(1.0, border_color))
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    // Left: Upload icon and files list
                    ui.add(Icon::Upload.image(TailwindColors::SLATE_400, 16.0));

                    let total_files = props.loaded_files.len();
                    ui.label(
                        egui::RichText::new(format!(
                            "{total_files} file{}:",
                            if total_files > 1 { "s" } else { "" }
                        ))
                        .font(font_semibold(12.0))
                        .color(if dark {
                            TailwindColors::SLATE_300
                        } else {
                            TailwindColors::SLATE_700
                        }),
                    );

                    for file in props.loaded_files.iter().take(5) {
                        let pill_bg = if dark {
                            TailwindColors::SLATE_800
                        } else {
                            TailwindColors::SLATE_100
                        };
                        let pill_fg = if dark {
                            TailwindColors::SLATE_300
                        } else {
                            TailwindColors::SLATE_700
                        };

                        Frame::NONE
                            .fill(pill_bg)
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(8, 2))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let name_str = file.name();
                                    let short_name = if name_str.chars().count() > 22 {
                                        let mut s: String = name_str.chars().take(20).collect();
                                        s.push('…');
                                        s
                                    } else {
                                        name_str.to_string()
                                    };
                                    ui.label(
                                        egui::RichText::new(short_name)
                                            .font(font_mono(11.0))
                                            .color(pill_fg),
                                    );
                                    ui.label(
                                        egui::RichText::new(format_bytes(file.size()))
                                            .font(font_mono(10.0))
                                            .color(TailwindColors::SLATE_400),
                                    );
                                });
                            });
                    }

                    if total_files > 5 {
                        ui.label(
                            egui::RichText::new(format!("+{} more", total_files - 5))
                                .font(font_medium(11.0))
                                .color(TailwindColors::SLATE_400),
                        );
                    }

                    // Right: Action buttons matching reference HasDataPanel
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        // Paste button
                        if secondary_button(ui, "Paste", Some("clipboard-paste"), dark, SMALL)
                            .clicked()
                        {
                            action = IngestAction::TogglePaste;
                        }

                        ui.add_space(8.0);

                        // Choose folder button
                        if secondary_button(ui, "Add Folder", Some("folder-open"), dark, SMALL)
                            .clicked()
                        {
                            action = IngestAction::BrowseFolder { append: true };
                        }

                        ui.add_space(8.0);

                        // Replace button
                        if secondary_button(ui, "Replace", Some("refresh-cw"), dark, SMALL)
                            .clicked()
                        {
                            action = IngestAction::BrowseFiles { append: false };
                        }

                        ui.add_space(8.0);

                        // Add / Append button (primary blue)
                        if primary_button(ui, "Add / Append", Some("file-plus"), SMALL).clicked() {
                            action = IngestAction::BrowseFiles { append: true };
                        }
                    });
                });
            });
    } else {
        // 4. Empty State: Full dropzone matching reference EmptyPanel
        let frame = Frame::NONE
            .fill(bg_color)
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin::symmetric(24, 40));

        let resp = frame
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add(Icon::Upload.image(TailwindColors::SLATE_400, 32.0));
                    ui.add_space(12.0);

                    let title = if props.is_mongo {
                        "Drop MongoDB JSON logs or .zip archive"
                    } else {
                        "Drop PM2 / API logs or .zip archive"
                    };
                    ui.label(
                        egui::RichText::new(title)
                            .font(font_medium(14.0))
                            .line_height(Some(20.0))
                            .color(if dark {
                                TailwindColors::SLATE_200
                            } else {
                                TailwindColors::SLATE_800
                            }),
                    );
                    ui.add_space(4.0);

                    ui.label(
                        egui::RichText::new(
                            ".log / .log.1 / .gz / .zip (auto-classifies API & MongoDB logs)",
                        )
                        .font(font_regular(12.0))
                        .color(TailwindColors::SLATE_500),
                    );
                    ui.add_space(12.0);

                    // 3 buttons: Browse files, Choose folder, Paste logs
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 360.0).max(0.0) / 2.0);

                        if primary_button(ui, "Browse files", None, PAD).clicked() {
                            action = IngestAction::BrowseFiles { append: false };
                        }

                        ui.add_space(8.0);

                        if secondary_button(ui, "Choose folder", Some("folder-open"), dark, PAD)
                            .clicked()
                        {
                            action = IngestAction::BrowseFolder { append: false };
                        }

                        ui.add_space(8.0);

                        if secondary_button(ui, "Paste logs", Some("clipboard-paste"), dark, PAD)
                            .clicked()
                        {
                            action = IngestAction::TogglePaste;
                        }
                    });

                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(
                            "Example: 2026-07-24T00:00:10: GET /api/health 200 12.5 ms - 42",
                        )
                        .font(font_mono(11.0))
                        .color(TailwindColors::SLATE_400),
                    );
                });
            })
            .response;

        let _ = resp;
    }

    // 5. Paste drawer: attached under the ingest card like the reference's
    // border-t panel (square top corners, rounded bottom, no gap).
    if props.paste_open {
        Frame::NONE
            .fill(if dark {
                TailwindColors::SLATE_900
            } else {
                Color32::WHITE
            })
            .stroke(Stroke::new(1.0, border_color))
            .corner_radius(egui::CornerRadius {
                nw: 0,
                ne: 0,
                sw: 4,
                se: 4,
            })
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Paste log text")
                                .font(font_medium(12.0))
                                .line_height(Some(LH_XS))
                                .color(if dark {
                                    TailwindColors::SLATE_300
                                } else {
                                    TailwindColors::SLATE_600
                                }),
                        );

                        let bytes = paste_buffer.len();
                        let lines = paste_buffer.lines().count();
                        ui.label(
                            egui::RichText::new(format!(
                                "({lines} lines, {})",
                                format_bytes(bytes as u64)
                            ))
                            .font(font_mono(11.0))
                            .color(TailwindColors::SLATE_400),
                        );
                    });

                    ui.add_space(6.0);

                    // Reference textarea: rounded border, bg-slate-50 /
                    // dark:bg-slate-950, px-3 py-2.
                    let edit_bg = if dark {
                        TailwindColors::SLATE_950
                    } else {
                        TailwindColors::SLATE_50
                    };
                    let edit_border = if dark {
                        TailwindColors::SLATE_700
                    } else {
                        TailwindColors::SLATE_200
                    };
                    Frame::NONE
                        .fill(edit_bg)
                        .stroke(Stroke::new(1.0, edit_border))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(paste_buffer)
                                    .font(font_mono(11.0))
                                    .desired_rows(6)
                                    .desired_width(ui.available_width())
                                    .frame(egui::Frame::NONE)
                                    .hint_text("Paste PM2 stdout/stderr here…"),
                            );
                        });
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        action = IngestAction::TogglePaste;
                    }

                    ui.add_space(8.0);

                    // Right-aligned actions: Analyze (primary), Clear, Cancel.
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let can_analyze = !paste_buffer.trim().is_empty();

                        if can_analyze && primary_button(ui, "Analyze paste", None, PAD).clicked() {
                            action = IngestAction::AnalyzePaste(paste_buffer.clone());
                        }

                        if !paste_buffer.is_empty()
                            && secondary_button(ui, "Clear text", None, dark, PAD).clicked()
                        {
                            paste_buffer.clear();
                        }

                        if secondary_button(ui, "Cancel", None, dark, PAD).clicked() {
                            action = IngestAction::TogglePaste;
                        }
                    });
                });
            });
    }

    action
}

fn paint_dashed_rect(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    let dash = 6.0;
    let gap = 4.0;
    // top
    let mut x = rect.min.x + 8.0;
    while x < rect.max.x - 8.0 {
        let x2 = (x + dash).min(rect.max.x - 8.0);
        painter.line_segment([pos2(x, rect.min.y), pos2(x2, rect.min.y)], stroke);
        x += dash + gap;
    }
    // bottom
    let mut x = rect.min.x + 8.0;
    while x < rect.max.x - 8.0 {
        let x2 = (x + dash).min(rect.max.x - 8.0);
        painter.line_segment([pos2(x, rect.max.y), pos2(x2, rect.max.y)], stroke);
        x += dash + gap;
    }
    // left
    let mut y = rect.min.y + 8.0;
    while y < rect.max.y - 8.0 {
        let y2 = (y + dash).min(rect.max.y - 8.0);
        painter.line_segment([pos2(rect.min.x, y), pos2(rect.min.x, y2)], stroke);
        y += dash + gap;
    }
    // right
    let mut y = rect.min.y + 8.0;
    while y < rect.max.y - 8.0 {
        let y2 = (y + dash).min(rect.max.y - 8.0);
        painter.line_segment([pos2(rect.max.x, y), pos2(rect.max.x, y2)], stroke);
        y += dash + gap;
    }
}
