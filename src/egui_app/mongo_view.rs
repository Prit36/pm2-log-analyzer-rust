mod charts;
mod controls;
mod diagnostics;
mod kpis;
mod patterns;
mod slow_queries;
mod users;

use super::icons::render_icon;
use super::theme::{
    LG_CONTENT_WIDTH, SM_CONTENT_WIDTH, TailwindColors, font_medium, font_mono, font_semibold,
};
use crate::core::mongo_models::{
    MongoAggregationResult, MongoCollectionMetric, MongoFilters, MongoPlanFilter, MongoSlowQuery,
    MongoSortDirection, MongoSortField, MongoUserActivity,
};
use crate::store::mongo_store::{MongoActiveView, MongoChartMode, MongoDiagTab, MongoState};
use crate::utils::format::{format_bytes, format_date_time, format_ms, format_num};
use egui::{Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect, Stroke, Ui, Vec2};
use egui_extras::{Column, TableBuilder};

pub enum MongoViewAction {
    None,
    FiltersChanged,
    CopyJson(String),
}

pub(super) fn compact_width(ui: &Ui) -> bool {
    viewport_width(ui) < 1100.0
}

pub(super) fn content_width(ui: &Ui) -> f32 {
    (viewport_width(ui) - 32.0).min(1280.0).max(320.0)
}

fn viewport_width(ui: &Ui) -> f32 {
    ui.ctx()
        .input(|input| input.raw.screen_rect.map(|rect| rect.width()))
        .unwrap_or_else(|| ui.clip_rect().width())
}

pub fn render_mongo_view(
    ui: &mut Ui,
    state: &mut MongoState,
    selected_query: &mut Option<MongoSlowQuery>,
    dark: bool,
) -> MongoViewAction {
    let mut action = MongoViewAction::None;

    let res = match &state.result {
        Some(r) => r,
        None => return action,
    };

    // 1. KPI cards (6 responsive cards matching MongoKpiRow.tsx and reference screenshot)
    kpis::render_mongo_kpis(ui, res, dark);

    ui.add_space(10.0);

    // 2. Combined View Switcher Tabs & Filter Bar (matching MongoFilterBar.tsx)
    let filter_changed =
        controls::render_mongo_controls(ui, &mut state.active_view, &mut state.filters, res, dark);
    if filter_changed {
        action = MongoViewAction::FiltersChanged;
    }

    ui.add_space(10.0);

    // 3. Active tab view
    match state.active_view {
        MongoActiveView::Patterns => {
            if patterns::render_patterns_tab(ui, res, &mut state.filters, selected_query, dark) {
                action = MongoViewAction::FiltersChanged;
            }
        }
        MongoActiveView::SlowQueries => {
            if let Some(sq) = slow_queries::render_slow_queries_tab(ui, res, dark) {
                *selected_query = Some(sq);
            }
        }
        MongoActiveView::Charts => {
            charts::render_mongo_charts_tab(ui, res, &mut state.chart_mode, dark);
        }
        MongoActiveView::Users => {
            if users::render_user_activity_tab(
                ui,
                res,
                &mut state.active_user_detail,
                &mut state.user_search,
                &mut state.filters,
                &mut state.active_view,
                dark,
            ) {
                action = MongoViewAction::FiltersChanged;
            }
        }
        MongoActiveView::Diagnostics => {
            diagnostics::render_diagnostics_tab(ui, res, &mut state.diag_tab, dark);
        }
    }

    // Modal popup for selected slow query
    if let Some(sq) = selected_query.clone() {
        let (close, copy_json) = render_mongo_query_modal(ui.ctx(), &sq, dark);
        if copy_json {
            let command_str = serde_json::to_string_pretty(&sq.command)
                .unwrap_or_else(|_| sq.command.to_string());
            action = MongoViewAction::CopyJson(command_str);
        }
        if close {
            *selected_query = None;
        }
    }

    action
}

fn render_mongo_query_modal(ctx: &egui::Context, sq: &MongoSlowQuery, dark: bool) -> (bool, bool) {
    let mut close = false;
    let mut copy_json = false;
    let command_str =
        serde_json::to_string_pretty(&sq.command).unwrap_or_else(|_| sq.command.to_string());

    let screen_rect = ctx
        .input(|i| i.raw.screen_rect)
        .unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0)));
    // Dimmed modal backdrop
    egui::Area::new(egui::Id::new("mongo_modal_backdrop"))
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                screen_rect,
                CornerRadius::ZERO,
                Color32::from_black_alpha(160),
            );
        });

    egui::Window::new("##mongo_query_detail_modal")
        .title_bar(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .fixed_size(Vec2::new(760.0, 560.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            Frame::NONE
                .fill(if dark {
                    TailwindColors::SLATE_900
                } else {
                    Color32::WHITE
                })
                .stroke(Stroke::new(
                    1.0,
                    if dark {
                        TailwindColors::SLATE_800
                    } else {
                        TailwindColors::SLATE_200
                    },
                ))
                .corner_radius(CornerRadius::same(16))
                .inner_margin(Margin::same(0)),
        )
        .show(ctx, |ui| {
            // 1. Header
            Frame::NONE
                .fill(if dark {
                    TailwindColors::SLATE_950
                } else {
                    Color32::WHITE
                })
                .stroke(Stroke::new(
                    1.0,
                    if dark {
                        TailwindColors::SLATE_800
                    } else {
                        TailwindColors::SLATE_200
                    },
                ))
                .inner_margin(Margin::symmetric(24, 16))
                .corner_radius(CornerRadius {
                    nw: 16,
                    ne: 16,
                    sw: 0,
                    se: 0,
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        patterns::render_mongo_op_badge(ui, &sq.op, dark);
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(&sq.ns)
                                .size(15.0)
                                .strong()
                                .color(if dark {
                                    TailwindColors::SLATE_100
                                } else {
                                    TailwindColors::SLATE_900
                                }),
                        );
                        if sq.is_collscan {
                            ui.add_space(6.0);
                            Frame::NONE
                                .fill(if dark {
                                    TailwindColors::AMBER_950
                                } else {
                                    TailwindColors::AMBER_100
                                })
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(6, 2))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.add(render_icon(
                                            "flame",
                                            TailwindColors::AMBER_600,
                                            12.0,
                                        ));
                                        ui.add_space(2.0);
                                        ui.label(
                                            egui::RichText::new("COLLSCAN")
                                                .size(10.0)
                                                .strong()
                                                .color(TailwindColors::AMBER_800),
                                        );
                                    });
                                });
                        }

                        ui.add_space(6.0);
                        let dur_ms = sq.duration_ms as f64;
                        Frame::NONE
                            .fill(if dark {
                                TailwindColors::SLATE_800
                            } else {
                                TailwindColors::SLATE_100
                            })
                            .stroke(Stroke::new(
                                1.0,
                                if dark {
                                    TailwindColors::SLATE_700
                                } else {
                                    TailwindColors::SLATE_200
                                },
                            ))
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format_ms(dur_ms))
                                        .size(11.0)
                                        .monospace()
                                        .strong()
                                        .color(if dark {
                                            TailwindColors::SLATE_200
                                        } else {
                                            TailwindColors::SLATE_700
                                        }),
                                );
                            });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let resp = ui.add(render_icon("x", TailwindColors::SLATE_400, 16.0));
                            if resp.interact(egui::Sense::click()).clicked() {
                                close = true;
                            }
                        });
                    });
                });

            // 2. Scrollable content
            ui.add_space(16.0);
            Frame::NONE
                .inner_margin(Margin::symmetric(24, 0))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(460.0)
                        .show(ui, |ui| {
                            // Recommended Index Banner (if any)
                            if let Some(sugg) = &sq.index_suggestion {
                                if !sugg.is_empty() {
                                    Frame::NONE
                                        .fill(if dark {
                                            Color32::from_rgb(6, 40, 25)
                                        } else {
                                            Color32::from_rgb(236, 253, 245)
                                        })
                                        .stroke(Stroke::new(
                                            1.0,
                                            if dark {
                                                Color32::from_rgb(16, 120, 70)
                                            } else {
                                                Color32::from_rgb(110, 231, 183)
                                            },
                                        ))
                                        .corner_radius(CornerRadius::same(10))
                                        .inner_margin(Margin::symmetric(14, 10))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.add(render_icon(
                                                    "lightbulb",
                                                    TailwindColors::EMERALD_600,
                                                    18.0,
                                                ));
                                                ui.add_space(6.0);
                                                ui.vertical(|ui| {
                                                    ui.label(
                                                        egui::RichText::new("Recommended Index")
                                                            .font(font_semibold(11.0))
                                                            .color(if dark {
                                                                TailwindColors::EMERALD_300
                                                            } else {
                                                                TailwindColors::EMERALD_900
                                                            }),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(sugg)
                                                            .font(font_mono(11.0))
                                                            .color(if dark {
                                                                TailwindColors::EMERALD_400
                                                            } else {
                                                                TailwindColors::EMERALD_800
                                                            }),
                                                    );
                                                });
                                            });
                                        });
                                    ui.add_space(12.0);
                                }
                            }

                            // 4 Diagnostic metric cards
                            ui.horizontal(|ui| {
                                let card_w = (ui.available_width() - 36.0) / 4.0;
                                render_modal_metric_card(
                                    ui,
                                    "Execution Plan",
                                    &sq.plan_summary,
                                    None,
                                    card_w,
                                    dark,
                                );
                                ui.add_space(12.0);
                                let docs_str = format!(
                                    "{} / {}",
                                    format_num(sq.docs_examined),
                                    format_num(sq.nreturned)
                                );
                                let sub_str = format!("Ratio: {:.1}x", sq.scan_ratio);
                                render_modal_metric_card(
                                    ui,
                                    "Docs Examined / Ret",
                                    &docs_str,
                                    Some(&sub_str),
                                    card_w,
                                    dark,
                                );
                                ui.add_space(12.0);
                                render_modal_metric_card(
                                    ui,
                                    "Keys Examined",
                                    &format_num(sq.keys_examined),
                                    None,
                                    card_w,
                                    dark,
                                );
                                ui.add_space(12.0);
                                let yields_str = format!(
                                    "{} yields · {}",
                                    sq.num_yields,
                                    format_bytes(sq.reslen)
                                );
                                render_modal_metric_card(
                                    ui,
                                    "Yields & Payload",
                                    &yields_str,
                                    None,
                                    card_w,
                                    dark,
                                );
                            });

                            ui.add_space(16.0);

                            // Command JSON container
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Command Document")
                                        .font(font_semibold(12.0))
                                        .color(if dark {
                                            TailwindColors::SLATE_300
                                        } else {
                                            TailwindColors::SLATE_700
                                        }),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let btn = ui
                                            .horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 4.0;
                                                ui.add(render_icon(
                                                    "copy",
                                                    TailwindColors::SLATE_400,
                                                    12.0,
                                                ));
                                                ui.label(
                                                    egui::RichText::new("Copy JSON")
                                                        .font(font_medium(11.0))
                                                        .color(if dark {
                                                            TailwindColors::SLATE_400
                                                        } else {
                                                            TailwindColors::SLATE_600
                                                        }),
                                                )
                                            })
                                            .response;
                                        if btn.hovered() {
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        if btn.interact(egui::Sense::click()).clicked() {
                                            copy_json = true;
                                        }
                                    },
                                );
                            });

                            ui.add_space(6.0);
                            Frame::NONE
                                .fill(if dark {
                                    Color32::from_rgb(10, 15, 26)
                                } else {
                                    Color32::from_rgb(15, 23, 42)
                                })
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(&command_str)
                                            .font(font_mono(11.0))
                                            .color(Color32::from_rgb(226, 232, 240)),
                                    );
                                });
                        });
                });
        });

    (close, copy_json)
}

fn render_modal_metric_card(
    ui: &mut Ui,
    title: &str,
    value: &str,
    sub: Option<&str>,
    width: f32,
    dark: bool,
) {
    Frame::NONE
        .fill(if dark {
            TailwindColors::SLATE_800
        } else {
            TailwindColors::SLATE_50
        })
        .stroke(Stroke::new(
            1.0,
            if dark {
                TailwindColors::SLATE_700
            } else {
                TailwindColors::SLATE_200
            },
        ))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).size(10.0).color(if dark {
                    TailwindColors::SLATE_400
                } else {
                    TailwindColors::SLATE_500
                }));
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(value)
                        .size(12.0)
                        .strong()
                        .color(if dark {
                            TailwindColors::SLATE_100
                        } else {
                            TailwindColors::SLATE_900
                        }),
                );
                if let Some(s) = sub {
                    ui.label(
                        egui::RichText::new(s)
                            .size(10.0)
                            .color(TailwindColors::SLATE_400),
                    );
                }
            });
        });
}

// Exact 6 KPI cards matching MongoKpiRow.tsx and reference screenshot
