use super::icons::Icon;
use super::theme::{LH_BASE, LH_XS, TailwindColors, font_medium, font_mono, font_semibold};
use super::widgets::{HDR, SegmentedItem, icon_button, secondary_button, segmented_control};
use crate::store::app_mode_store::AppMode;
use crate::utils::format::{format_bytes, format_date};
use egui::{Align, Color32, CornerRadius, Margin, Rect, Stroke, Ui, pos2, vec2};

pub struct HeaderProps<'a> {
    pub mode: AppMode,
    pub dark: bool,
    pub has_pm2_data: bool,
    pub has_mongo_data: bool,
    pub pm2_file_name: Option<&'a str>,
    pub pm2_file_size: Option<u64>,
    pub pm2_file_count: usize,
    pub pm2_dates: Option<&'a [String]>,
    pub mongo_file_name: Option<&'a str>,
    pub mongo_file_size: Option<u64>,
    pub mongo_stats: Option<(usize, usize, f64)>,
    pub mongo_dates: Option<&'a [String]>,
    pub is_parsing: bool,
}

pub enum HeaderAction {
    None,
    SwitchMode(AppMode),
    ToggleTheme,
    Export,
    Clear,
}

pub fn render_header(ui: &mut Ui, props: HeaderProps<'_>) -> HeaderAction {
    let mut action = HeaderAction::None;
    let dark = props.dark;
    let is_pm2 = props.mode == AppMode::Pm2;

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

    let total_width = ui.available_width();
    let max_w = 1280.0;
    let inner_w = (total_width - 32.0).min(max_w).max(320.0);
    let side_margin = ((total_width - inner_w) / 2.0).max(16.0);

    // Reference: `px-4 py-2.5` around two text rows (title 24 + `mt-0.5`
    // + subtitle 16 = 42) → 10 + 42 + 10 = 62px of content box plus the 1px
    // `border-b` = 63px total. The border occupies the last pixel row.
    let header_h = 63.0;
    let (header_rect, _) =
        ui.allocate_exact_size(vec2(total_width, header_h), egui::Sense::hover());

    // Header background across the full window width, then the 1px bottom
    // border on the final row (a line at y-0.5 rounds onto row y-1).
    ui.painter()
        .rect_filled(header_rect, CornerRadius::ZERO, bg_color);
    ui.painter().line_segment(
        [
            pos2(header_rect.min.x, header_rect.max.y - 0.5),
            pos2(header_rect.max.x, header_rect.max.y - 0.5),
        ],
        Stroke::new(1.0, border_color),
    );

    let content_rect = Rect::from_min_size(
        pos2(header_rect.min.x + side_margin, header_rect.min.y + 10.0),
        vec2(inner_w, 42.0),
    );

    // 1. Center: Segmented App Switcher
    let switcher_w = 244.0;
    let center_x = content_rect.center().x;
    let switcher_rect = Rect::from_center_size(
        pos2(center_x, content_rect.center().y),
        vec2(switcher_w, 36.0),
    );

    let items = [
        SegmentedItem {
            label: "PM2 Logs",
            icon: Some("file-text"),
            badge: None,
            has_dot: !is_pm2 && props.has_pm2_data,
            dot_color: TailwindColors::BLUE_500,
            active_color: Some(TailwindColors::BLUE_600),
        },
        SegmentedItem {
            label: "MongoDB Logs",
            icon: Some("database"),
            badge: None,
            has_dot: is_pm2 && props.has_mongo_data,
            dot_color: TailwindColors::EMERALD_500,
            active_color: Some(TailwindColors::EMERALD_600),
        },
    ];

    let mut center_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(switcher_rect)
            .layout(egui::Layout::left_to_right(Align::Center)),
    );
    let selected = if is_pm2 { 0 } else { 1 };
    if let Some(new_sel) = segmented_control(&mut center_ui, &items, selected, dark) {
        if new_sel == 0 && !is_pm2 {
            action = HeaderAction::SwitchMode(AppMode::Pm2);
        } else if new_sel == 1 && is_pm2 {
            action = HeaderAction::SwitchMode(AppMode::Mongo);
        }
    }

    // 2. Right Actions: Theme Toggle, Export, Clear
    let right_rect = Rect::from_min_max(
        pos2(switcher_rect.max.x + 8.0, content_rect.min.y),
        content_rect.max,
    );
    let mut right_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(right_rect)
            .layout(egui::Layout::right_to_left(Align::Center)),
    );

    let can_clear = if is_pm2 {
        props.has_pm2_data || props.pm2_file_name.is_some()
    } else {
        props.has_mongo_data || props.mongo_file_name.is_some()
    };

    let can_export = if is_pm2 {
        props.has_pm2_data && !props.is_parsing
    } else {
        props.has_mongo_data && !props.is_parsing
    };

    // Clear button
    if can_clear {
        if secondary_button(&mut right_ui, "Clear", Some("eraser"), dark, HDR).clicked() {
            action = HeaderAction::Clear;
        }
        right_ui.add_space(8.0);
    }

    // Export button
    if can_export {
        if secondary_button(&mut right_ui, "Export", Some("download"), dark, HDR).clicked() {
            action = HeaderAction::Export;
        }
        right_ui.add_space(8.0);
    }

    // Theme toggle button
    let theme_icon = if dark { "sun" } else { "moon" };
    let theme_tip = if dark {
        "Switch to light mode"
    } else {
        "Switch to dark mode"
    };
    if icon_button(&mut right_ui, theme_icon, theme_tip, dark).clicked() {
        action = HeaderAction::ToggleTheme;
    }

    // 3. Left Side: App Branding & File Info
    let left_rect = Rect::from_min_max(
        content_rect.min,
        pos2(switcher_rect.min.x - 8.0, content_rect.max.y),
    );
    let mut left_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(left_rect)
            .layout(egui::Layout::left_to_right(Align::Center)),
    );

    let brand_icon = if is_pm2 {
        Icon::FileText
    } else {
        Icon::Database
    };
    let icon_col = if is_pm2 {
        if dark {
            TailwindColors::BLUE_400
        } else {
            TailwindColors::BLUE_600
        }
    } else {
        if dark {
            TailwindColors::EMERALD_400
        } else {
            TailwindColors::EMERALD_600
        }
    };

    left_ui.add(brand_icon.image(icon_col, 20.0));
    left_ui.add_space(8.0);

    left_ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.horizontal(|ui| {
            let title_text = if is_pm2 {
                "PM2 Log Analyzer"
            } else {
                "MongoDB Log Analyzer"
            };
            ui.label(
                egui::RichText::new(title_text)
                    .font(font_semibold(16.0))
                    .line_height(Some(LH_BASE))
                    .color(if dark {
                        TailwindColors::SLATE_100
                    } else {
                        TailwindColors::SLATE_900
                    }),
            );

            // Date badge
            let dates_opt = if is_pm2 {
                props.pm2_dates
            } else {
                props.mongo_dates
            };
            if let Some(dates) = dates_opt {
                if !dates.is_empty() {
                    let text = if dates.len() > 1 {
                        format!(
                            "{} → {} ({} days)",
                            format_date(Some(&dates[0])),
                            format_date(Some(&dates[dates.len() - 1])),
                            dates.len()
                        )
                    } else {
                        format_date(Some(&dates[0]))
                    };

                    let (badge_bg, badge_fg) = if is_pm2 {
                        if dark {
                            (
                                TailwindColors::BLUE_950.gamma_multiply(0.6),
                                TailwindColors::BLUE_300,
                            )
                        } else {
                            (TailwindColors::BLUE_50, TailwindColors::BLUE_700)
                        }
                    } else {
                        if dark {
                            (
                                TailwindColors::EMERALD_950.gamma_multiply(0.6),
                                TailwindColors::EMERALD_300,
                            )
                        } else {
                            (TailwindColors::EMERALD_50, TailwindColors::EMERALD_700)
                        }
                    };

                    egui::Frame::NONE
                        .fill(badge_bg)
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(8, 2))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(text)
                                    .font(font_medium(11.0))
                                    .color(badge_fg),
                            );
                        });
                }
            }
        });

        // Subtitle / Source info
        let subtitle = if is_pm2 {
            if let Some(name) = props.pm2_file_name {
                if let Some(sz) = props.pm2_file_size {
                    if props.pm2_file_count > 1 {
                        format!(
                            "{} (+{} more) · {}",
                            name,
                            props.pm2_file_count - 1,
                            format_bytes(sz)
                        )
                    } else {
                        format!("{} · {}", name, format_bytes(sz))
                    }
                } else {
                    name.to_string()
                }
            } else {
                "API latency & cron insight".to_string()
            }
        } else {
            if let Some(name) = props.mongo_file_name {
                if let Some(sz) = props.mongo_file_size {
                    format!("{} · {}", name, format_bytes(sz))
                } else {
                    name.to_string()
                }
            } else {
                "Slow queries, COLLSCAN detection & smart index advisor".to_string()
            }
        };

        ui.label(
            egui::RichText::new(subtitle)
                .font(font_mono(12.0))
                .line_height(Some(LH_XS))
                .color(if dark {
                    TailwindColors::SLATE_400
                } else {
                    TailwindColors::SLATE_500
                }),
        );
    });

    action
}
