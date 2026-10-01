use super::theme::{LH_10, TailwindColors, font_regular, font_semibold};
use super::widgets::{
    filter_chip, method_chip, number_field, reset_filters_button, search_input, select_dropdown,
};
use crate::core::models::{NormalizeMode, StatusFamily};
use crate::store::analysis_store::{AnalysisFilters, ApiSortKey, DEFAULT_TOP_N};
use crate::utils::format::format_date;
use egui::{Align, Color32, CornerRadius, Frame, Margin, Stroke, Ui, vec2};

pub enum FilterAction {
    None,
    Changed,
    Reaggregate,
    Reset,
}

fn field_label(ui: &mut Ui, label: &str, dark: bool) {
    ui.label(
        egui::RichText::new(label)
            .font(font_semibold(10.0))
            .line_height(Some(LH_10))
            .extra_letter_spacing(0.25)
            .color(if dark {
                TailwindColors::SLATE_400
            } else {
                TailwindColors::SLATE_500
            }),
    );
    ui.add_space(4.0);
}

pub fn render_filters(
    ui: &mut Ui,
    filters: &mut AnalysisFilters,
    dates: &[String],
    available_methods: &[String],
    dark: bool,
) -> FilterAction {
    let mut action = FilterAction::None;

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

    // Calculate active filter count matching reference countActiveAnalysisFilters
    let mut active_count = 0;
    if !filters.query.is_empty() {
        active_count += 1;
    }
    if filters.normalize_mode != NormalizeMode::CollapseIds {
        active_count += 1;
    }
    if filters.status_family != StatusFamily::All {
        active_count += 1;
    }
    if filters.min_ms > 0.0 {
        active_count += 1;
    }
    if filters.sort_key != ApiSortKey::P95Ms {
        active_count += 1;
    }
    if filters.top_n != DEFAULT_TOP_N {
        active_count += 1;
    }
    if filters.date_filter != "all" {
        active_count += 1;
    }
    if !filters.methods.is_empty() {
        active_count += 1;
    }

    Frame::NONE
        .fill(bg_color)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(12, 12))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let compact = ui.available_width() < 1100.0;
                // Top row: Search, Normalize, Status, Min ms, Sort, Top N, Reset
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    // 1. Search (flex-1 matching reference className="min-w-[14rem] flex-1")
                    ui.vertical(|ui| {
                        field_label(ui, "SEARCH", dark);
                        let prev_q = filters.query.clone();
                        let search_w = (ui.available_width() - 560.0).clamp(240.0, 420.0);
                        search_input(
                            ui,
                            "pm2_search",
                            &mut filters.query,
                            "Filter endpoints… (/)",
                            search_w,
                            dark,
                        );
                        if filters.query != prev_q {
                            action = FilterAction::Changed;
                        }
                    });

                    // 2. Normalize
                    ui.vertical(|ui| {
                        field_label(ui, "NORMALIZE", dark);
                        let norm_options = [
                            (NormalizeMode::CollapseIds, "Collapse IDs"),
                            (NormalizeMode::StripQuery, "Strip query"),
                            (NormalizeMode::Exact, "Exact path"),
                        ];
                        if let Some(new_norm) = select_dropdown(
                            ui,
                            "norm_dropdown",
                            &filters.normalize_mode,
                            &norm_options,
                            dark,
                        ) {
                            if new_norm != filters.normalize_mode {
                                filters.normalize_mode = new_norm;
                                action = FilterAction::Reaggregate;
                            }
                        }
                    });

                    // 3. Status
                    ui.vertical(|ui| {
                        field_label(ui, "STATUS", dark);
                        let status_options = [
                            (StatusFamily::All, "All"),
                            (StatusFamily::X2xx, "2xx"),
                            (StatusFamily::X3xx, "3xx"),
                            (StatusFamily::X4xx, "4xx"),
                            (StatusFamily::X5xx, "5xx"),
                        ];
                        if let Some(new_status) = select_dropdown(
                            ui,
                            "status_dropdown",
                            &filters.status_family,
                            &status_options,
                            dark,
                        ) {
                            if new_status != filters.status_family {
                                filters.status_family = new_status;
                                action = FilterAction::Reaggregate;
                            }
                        }
                    });

                    // 4. Min ms
                    ui.vertical(|ui| {
                        field_label(ui, "MIN MS", dark);
                        let mut min_ms_val = filters.min_ms as usize;
                        if number_field(ui, "min_ms_field", &mut min_ms_val, 80.0, dark) {
                            filters.min_ms = min_ms_val as f64;
                            action = FilterAction::Reaggregate;
                        }
                    });

                    // 5. Sort
                    ui.vertical(|ui| {
                        field_label(ui, "SORT", dark);
                        let sort_options = [
                            (ApiSortKey::P95Ms, "p95"),
                            (ApiSortKey::P99Ms, "p99"),
                            (ApiSortKey::AvgMs, "avg"),
                            (ApiSortKey::MaxMs, "max"),
                            (ApiSortKey::Count, "count"),
                            (ApiSortKey::ErrorCount, "errors"),
                            (ApiSortKey::Path, "endpoint"),
                        ];
                        if let Some(new_sort) = select_dropdown(
                            ui,
                            "sort_dropdown",
                            &filters.sort_key,
                            &sort_options,
                            dark,
                        ) {
                            if new_sort != filters.sort_key {
                                filters.sort_key = new_sort;
                                action = FilterAction::Changed;
                            }
                        }
                    });

                    // 6. Top N
                    ui.vertical(|ui| {
                        field_label(ui, "TOP N", dark);
                        let mut top_n_val = filters.top_n;
                        if number_field(ui, "top_n_field", &mut top_n_val, 80.0, dark) {
                            filters.top_n = top_n_val.clamp(1, 500);
                            action = FilterAction::Changed;
                        }
                    });

                    // On a wide row, keep reset aligned with the other controls.
                    // The right-to-left child claims the entire wrapped row at
                    // narrower widths, so move it to its own aligned row there.
                    if !compact && ui.available_width() > 150.0 {
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            if reset_filters_button(ui, active_count, dark).clicked() {
                                action = FilterAction::Reset;
                            }
                        });
                    }
                });

                if compact {
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if reset_filters_button(ui, active_count, dark).clicked() {
                            action = FilterAction::Reset;
                        }
                    });
                }

                // Secondary row: Day chips and Methods chips
                let has_days = dates.len() > 1;
                let has_methods = !available_methods.is_empty();

                if has_days || has_methods {
                    ui.add_space(10.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        // Day filter chips
                        if has_days {
                            ui.label(
                                egui::RichText::new("DAY")
                                    .font(font_semibold(10.0))
                                    .line_height(Some(LH_10))
                                    .extra_letter_spacing(0.25)
                                    .color(if dark {
                                        TailwindColors::SLATE_400
                                    } else {
                                        TailwindColors::SLATE_500
                                    }),
                            );
                            ui.add_space(4.0);

                            if filter_chip(
                                ui,
                                &format!("All Days ({})", dates.len()),
                                filters.date_filter == "all",
                                dark,
                            )
                            .clicked()
                            {
                                if filters.date_filter != "all" {
                                    filters.date_filter = "all".to_string();
                                    action = FilterAction::Reaggregate;
                                }
                            }

                            for d in dates {
                                let is_active = filters.date_filter == *d;
                                if filter_chip(ui, &format_date(Some(d)), is_active, dark).clicked()
                                {
                                    if !is_active {
                                        filters.date_filter = d.clone();
                                        action = FilterAction::Reaggregate;
                                    }
                                }
                            }
                        }

                        // Reference: `h-4 w-px bg-slate-200 dark:bg-slate-700`
                        // divider between the day and method groups.
                        if has_days && has_methods {
                            ui.add_space(6.0);
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(1.0, 16.0), egui::Sense::hover());
                            ui.painter().rect_filled(
                                rect,
                                CornerRadius::ZERO,
                                if dark {
                                    TailwindColors::SLATE_700
                                } else {
                                    TailwindColors::SLATE_200
                                },
                            );
                            ui.add_space(6.0);
                        }

                        // Method filter chips
                        if has_methods {
                            ui.label(
                                egui::RichText::new("METHODS")
                                    .font(font_semibold(10.0))
                                    .line_height(Some(LH_10))
                                    .extra_letter_spacing(0.25)
                                    .color(if dark {
                                        TailwindColors::SLATE_400
                                    } else {
                                        TailwindColors::SLATE_500
                                    }),
                            );
                            ui.add_space(4.0);

                            let all_selected = filters.methods.is_empty();
                            if method_chip(ui, "All", all_selected, dark).clicked() {
                                if !all_selected {
                                    filters.methods.clear();
                                    action = FilterAction::Changed;
                                }
                            }

                            for m in available_methods {
                                let is_sel = filters.methods.contains(m);
                                if method_chip(ui, m, is_sel, dark).clicked() {
                                    if is_sel {
                                        filters.methods.retain(|x| x != m);
                                    } else {
                                        filters.methods.push(m.clone());
                                    }
                                    action = FilterAction::Changed;
                                }
                            }

                            if !all_selected {
                                let resp = ui.add(
                                    egui::Label::new(
                                        egui::RichText::new("Reset")
                                            .font(font_regular(11.0))
                                            .color(if dark {
                                                TailwindColors::SLATE_400
                                            } else {
                                                TailwindColors::SLATE_500
                                            }),
                                    )
                                    .sense(egui::Sense::click()),
                                );
                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if resp.clicked() {
                                    filters.methods.clear();
                                    action = FilterAction::Changed;
                                }
                            }
                        }
                    });
                }
            });
        });

    action
}
