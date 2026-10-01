use super::controls::paint_th;
use super::patterns::render_mongo_op_badge;
use super::*;

// 2. Slow Queries tab
pub(super) fn render_slow_queries_tab(
    ui: &mut Ui,
    res: &MongoAggregationResult,
    dark: bool,
) -> Option<MongoSlowQuery> {
    let mut selected = None;
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

    let effective_height = (res.slow_queries.len() as f32 * 42.0 + 36.0).clamp(180.0, 520.0);
    let compact = compact_width(ui);

    Frame::NONE
        .fill(bg_color)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::ZERO)
        .show(ui, |ui| {
            let table_width = content_width(ui);
            let (
                time_width,
                duration_width,
                user_width,
                docs_width,
                keys_width,
                returned_width,
                ratio_width,
                ip_width,
                view_width,
            ) = if compact {
                (60.0, 62.0, 82.0, 72.0, 72.0, 58.0, 70.0, 92.0, 32.0)
            } else {
                (66.0, 68.0, 95.0, 86.0, 86.0, 66.0, 78.0, 120.0, 36.0)
            };
            let collection_width = table_width
                - time_width
                - duration_width
                - user_width
                - docs_width
                - keys_width
                - returned_width
                - ratio_width
                - ip_width
                - view_width;
            egui::ScrollArea::horizontal()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.set_width(table_width);
                    ui.spacing_mut().item_spacing.x = 0.0;
                    TableBuilder::new(ui)
                        .striped(true)
                        .vscroll(true)
                        .min_scrolled_height(effective_height)
                        .max_scroll_height(effective_height)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::exact(time_width)) // 1. Time
                        .column(Column::exact(duration_width)) // 2. Duration
                        .column(Column::exact(user_width)) // 3. User & Context
                        .column(Column::exact(collection_width)) // 4. Collection & Plan
                        .column(Column::exact(docs_width)) // 5. Docs Scanned
                        .column(Column::exact(keys_width)) // 6. Keys Scanned
                        .column(Column::exact(returned_width)) // 7. Returned
                        .column(Column::exact(ratio_width)) // 8. Scan Ratio
                        .column(Column::exact(ip_width)) // 9. Client IP
                        .column(Column::exact(view_width)) // 10. View
                        .auto_shrink([true, true])
                        .header(32.0, |mut header| {
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.label(
                                    egui::RichText::new("Time ↑↓")
                                        .size(11.0)
                                        .strong()
                                        .color(TailwindColors::SLATE_600),
                                );
                            });
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.label(
                                    egui::RichText::new("Duration ↓")
                                        .size(11.0)
                                        .strong()
                                        .color(TailwindColors::SLATE_600),
                                );
                            });
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.label(
                                    egui::RichText::new("User")
                                        .size(11.0)
                                        .strong()
                                        .color(TailwindColors::SLATE_600),
                                );
                            });
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.label(
                                    egui::RichText::new("Collection & Plan ↑↓")
                                        .size(11.0)
                                        .strong()
                                        .color(TailwindColors::SLATE_600),
                                );
                            });
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(2.0);
                                        ui.label(
                                            egui::RichText::new(if compact {
                                                "Docs ↑↓"
                                            } else {
                                                "Docs Scanned ↑↓"
                                            })
                                            .size(10.0)
                                            .strong()
                                            .color(TailwindColors::SLATE_600),
                                        );
                                    },
                                );
                            });
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(2.0);
                                        ui.label(
                                            egui::RichText::new(if compact {
                                                "Keys ↑↓"
                                            } else {
                                                "Keys Scanned ↑↓"
                                            })
                                            .size(10.0)
                                            .strong()
                                            .color(TailwindColors::SLATE_600),
                                        );
                                    },
                                );
                            });
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(2.0);
                                        ui.label(
                                            egui::RichText::new(if compact {
                                                "Ret. ↑↓"
                                            } else {
                                                "Returned ↑↓"
                                            })
                                            .size(10.0)
                                            .strong()
                                            .color(TailwindColors::SLATE_600),
                                        );
                                    },
                                );
                            });
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(2.0);
                                        ui.label(
                                            egui::RichText::new(if compact {
                                                "Ratio ↑↓"
                                            } else {
                                                "Scan Ratio ↑↓"
                                            })
                                            .size(10.0)
                                            .strong()
                                            .color(TailwindColors::SLATE_600),
                                        );
                                    },
                                );
                            });
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.label(
                                    egui::RichText::new(if compact { "IP" } else { "Client IP" })
                                        .size(11.0)
                                        .strong()
                                        .color(TailwindColors::SLATE_600),
                                );
                            });
                            header.col(|ui| {
                                paint_th(ui, dark);
                                ui.centered_and_justified(|ui| {
                                    ui.label(
                                        egui::RichText::new("View")
                                            .size(11.0)
                                            .strong()
                                            .color(TailwindColors::SLATE_600),
                                    );
                                });
                            });
                        })
                        .body(|mut body| {
                            for q in res.slow_queries.iter().take(300) {
                                body.row(42.0, |mut row_ui| {
                                    // 1. Time
                                    row_ui.col(|ui| {
                                        let time_str = if q.timestamp.len() >= 19 {
                                            &q.timestamp[11..19]
                                        } else {
                                            &q.timestamp
                                        };
                                        ui.label(
                                            egui::RichText::new(time_str)
                                                .size(11.0)
                                                .monospace()
                                                .color(TailwindColors::SLATE_500),
                                        );
                                    });

                                    // 2. Duration badge
                                    row_ui.col(|ui| {
                                        let (dur_bg, dur_fg) = if q.duration_ms >= 5000 {
                                            (
                                                Color32::from_rgb(243, 232, 255),
                                                Color32::from_rgb(107, 33, 168),
                                            )
                                        } else if q.duration_ms >= 1000 {
                                            (TailwindColors::ROSE_50, TailwindColors::ROSE_700)
                                        } else if q.duration_ms >= 500 {
                                            (TailwindColors::AMBER_50, TailwindColors::AMBER_700)
                                        } else {
                                            (TailwindColors::SLATE_100, TailwindColors::SLATE_700)
                                        };

                                        Frame::NONE
                                            .fill(dur_bg)
                                            .stroke(Stroke::new(
                                                1.0,
                                                if q.duration_ms >= 5000 {
                                                    Color32::from_rgb(216, 180, 254)
                                                } else if q.duration_ms >= 1000 {
                                                    TailwindColors::ROSE_200
                                                } else if q.duration_ms >= 500 {
                                                    TailwindColors::AMBER_200
                                                } else {
                                                    TailwindColors::SLATE_200
                                                },
                                            ))
                                            .corner_radius(CornerRadius::same(4))
                                            .inner_margin(Margin::symmetric(6, 2))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(format_ms(
                                                        q.duration_ms as f64,
                                                    ))
                                                    .size(11.0)
                                                    .monospace()
                                                    .strong()
                                                    .color(dur_fg),
                                                );
                                            });
                                    });

                                    // 3. User & Context
                                    row_ui.col(|ui| {
                                        ui.vertical(|ui| {
                                            let user = q.user.as_deref().unwrap_or("system");
                                            let is_system = user == "system";
                                            let user_col = if is_system {
                                                TailwindColors::SLATE_500
                                            } else if dark {
                                                TailwindColors::EMERALD_400
                                            } else {
                                                TailwindColors::EMERALD_700
                                            };
                                            ui.label(
                                                egui::RichText::new(user)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(user_col),
                                            );
                                            if !q.ctx.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(&q.ctx)
                                                        .size(9.0)
                                                        .monospace()
                                                        .color(TailwindColors::SLATE_400),
                                                );
                                            }
                                        });
                                    });

                                    // 4. Collection & Plan
                                    row_ui.col(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(&q.collection)
                                                    .size(11.5)
                                                    .strong()
                                                    .color(if dark {
                                                        TailwindColors::SLATE_100
                                                    } else {
                                                        TailwindColors::SLATE_900
                                                    }),
                                            );
                                            render_mongo_op_badge(ui, &q.op, dark);
                                            if q.is_collscan {
                                                Frame::NONE
                                                    .fill(if dark {
                                                        TailwindColors::AMBER_950
                                                    } else {
                                                        TailwindColors::AMBER_100
                                                    })
                                                    .stroke(Stroke::new(
                                                        1.0,
                                                        TailwindColors::AMBER_300,
                                                    ))
                                                    .corner_radius(CornerRadius::same(3))
                                                    .inner_margin(Margin::symmetric(4, 1))
                                                    .show(ui, |ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.add(render_icon(
                                                                "flame",
                                                                TailwindColors::AMBER_600,
                                                                10.0,
                                                            ));
                                                            ui.add_space(2.0);
                                                            ui.label(
                                                                egui::RichText::new("COLLSCAN")
                                                                    .size(9.0)
                                                                    .strong()
                                                                    .color(
                                                                        TailwindColors::AMBER_800,
                                                                    ),
                                                            );
                                                        });
                                                    });
                                            } else {
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
                                                            Color32::from_rgb(167, 243, 208)
                                                        },
                                                    ))
                                                    .corner_radius(CornerRadius::same(3))
                                                    .inner_margin(Margin::symmetric(4, 1))
                                                    .show(ui, |ui| {
                                                        ui.label(
                                                            egui::RichText::new(&q.plan_summary)
                                                                .size(9.5)
                                                                .monospace()
                                                                .color(if dark {
                                                                    TailwindColors::EMERALD_300
                                                                } else {
                                                                    TailwindColors::EMERALD_800
                                                                }),
                                                        );
                                                    });
                                            }
                                        });
                                    });

                                    // 5. Docs Scanned
                                    row_ui.col(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.add_space(6.0);
                                                ui.label(
                                                    egui::RichText::new(format_num(
                                                        q.docs_examined,
                                                    ))
                                                    .size(11.0)
                                                    .monospace()
                                                    .color(if dark {
                                                        TailwindColors::SLATE_300
                                                    } else {
                                                        TailwindColors::SLATE_700
                                                    }),
                                                );
                                            },
                                        );
                                    });

                                    // 6. Keys Scanned
                                    row_ui.col(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.add_space(6.0);
                                                ui.label(
                                                    egui::RichText::new(format_num(
                                                        q.keys_examined,
                                                    ))
                                                    .size(11.0)
                                                    .monospace()
                                                    .color(TailwindColors::SLATE_400),
                                                );
                                            },
                                        );
                                    });

                                    // 7. Returned
                                    row_ui.col(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.add_space(6.0);
                                                ui.label(
                                                    egui::RichText::new(format_num(q.nreturned))
                                                        .size(11.0)
                                                        .monospace()
                                                        .color(TailwindColors::SLATE_400),
                                                );
                                            },
                                        );
                                    });

                                    // 8. Scan Ratio
                                    row_ui.col(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.add_space(6.0);
                                                let col = if q.scan_ratio >= 1000.0 {
                                                    TailwindColors::ROSE_600
                                                } else if q.scan_ratio >= 100.0 {
                                                    TailwindColors::AMBER_600
                                                } else {
                                                    TailwindColors::SLATE_600
                                                };
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{:.1}x",
                                                        q.scan_ratio
                                                    ))
                                                    .size(11.0)
                                                    .monospace()
                                                    .strong()
                                                    .color(col),
                                                );
                                            },
                                        );
                                    });

                                    // 9. Client IP
                                    row_ui.col(|ui| {
                                        let remote = q.remote.as_deref().unwrap_or("unknown");
                                        ui.label(
                                            egui::RichText::new(remote)
                                                .size(10.5)
                                                .monospace()
                                                .color(TailwindColors::SLATE_500),
                                        );
                                    });

                                    // 10. View
                                    row_ui.col(|ui| {
                                        ui.centered_and_justified(|ui| {
                                            let resp = ui.add(render_icon(
                                                "external-link",
                                                TailwindColors::SLATE_400,
                                                14.0,
                                            ));
                                            if resp.interact(egui::Sense::click()).clicked() {
                                                selected = Some(q.clone());
                                            }
                                        });
                                    });
                                });
                            }
                        });
                });
        });

    selected
}
