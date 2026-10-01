use super::controls::paint_th;
use super::*;

// 1. Query Patterns tab
pub(super) fn render_patterns_tab(
    ui: &mut Ui,
    res: &MongoAggregationResult,
    filters: &mut MongoFilters,
    selected: &mut Option<MongoSlowQuery>,
    dark: bool,
) -> bool {
    let mut sort_changed = false;
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

    Frame::NONE
        .fill(bg_color)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::ZERO)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let first_column_width = (content_width(ui) - 541.0).max(320.0);
            TableBuilder::new(ui)
                .id_salt("mongo_patterns_table")
                .striped(true)
                .vscroll(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(first_column_width)) // Query Pattern / Collection
                .column(Column::exact(46.0)) // Count
                .column(Column::exact(64.0)) // Total Time
                .column(Column::exact(54.0)) // Avg
                .column(Column::exact(54.0)) // P95
                .column(Column::exact(54.0)) // Max
                .column(Column::exact(68.0)) // Scan Ratio
                .column(Column::exact(165.0)) // Suggested Index
                .column(Column::exact(36.0)) // View
                .auto_shrink([false, true])
                .header(32.0, |mut header| {
                    header.col(|ui| {
                        paint_th(ui, dark);
                        render_sortable_col(
                            ui,
                            "Query Pattern / Collection",
                            MongoSortField::Collection,
                            filters,
                            &mut sort_changed,
                        );
                    });
                    header.col(|ui| {
                        paint_th(ui, dark);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            render_sortable_col(
                                ui,
                                "Count",
                                MongoSortField::Count,
                                filters,
                                &mut sort_changed,
                            );
                        });
                    });
                    header.col(|ui| {
                        paint_th(ui, dark);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            render_sortable_col(
                                ui,
                                "Total Time",
                                MongoSortField::TotalDurationMs,
                                filters,
                                &mut sort_changed,
                            );
                        });
                    });
                    header.col(|ui| {
                        paint_th(ui, dark);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            render_sortable_col(
                                ui,
                                "Avg",
                                MongoSortField::AvgDurationMs,
                                filters,
                                &mut sort_changed,
                            );
                        });
                    });
                    header.col(|ui| {
                        paint_th(ui, dark);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            render_sortable_col(
                                ui,
                                "P95",
                                MongoSortField::P95DurationMs,
                                filters,
                                &mut sort_changed,
                            );
                        });
                    });
                    header.col(|ui| {
                        paint_th(ui, dark);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            render_sortable_col(
                                ui,
                                "Max",
                                MongoSortField::MaxDurationMs,
                                filters,
                                &mut sort_changed,
                            );
                        });
                    });
                    header.col(|ui| {
                        paint_th(ui, dark);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            render_sortable_col(
                                ui,
                                "Scan Ratio",
                                MongoSortField::ScanRatio,
                                filters,
                                &mut sort_changed,
                            );
                        });
                    });
                    header.col(|ui| {
                        paint_th(ui, dark);
                        ui.label(
                            egui::RichText::new("Suggested Index (1-Click Copy)")
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
                    for pat in &res.patterns {
                        body.row(52.0, |mut row_ui| {
                            // Pattern name & command
                            row_ui.col(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        render_mongo_op_badge(ui, &pat.op, dark);
                                        let col_resp = ui.label(
                                            egui::RichText::new(&pat.collection)
                                                .size(12.0)
                                                .strong()
                                                .color(if dark {
                                                    TailwindColors::SLATE_100
                                                } else {
                                                    TailwindColors::SLATE_900
                                                }),
                                        );
                                        if col_resp.interact(egui::Sense::click()).clicked() {
                                            *selected = Some(pat.example_query.clone());
                                        }

                                        if pat.is_collscan {
                                            ui.add_space(2.0);
                                            Frame::NONE
                                                .fill(if dark {
                                                    TailwindColors::AMBER_950
                                                } else {
                                                    TailwindColors::AMBER_100
                                                })
                                                .stroke(Stroke::new(1.0, TailwindColors::AMBER_300))
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
                                                                .color(TailwindColors::AMBER_800),
                                                        );
                                                    });
                                                });
                                        }
                                    });

                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&pat.fingerprint)
                                                .size(11.0)
                                                .monospace()
                                                .color(TailwindColors::SLATE_500),
                                        )
                                        .truncate(),
                                    );
                                });
                            });

                            // Count
                            row_ui.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format_num(pat.count))
                                                .size(11.5)
                                                .monospace(),
                                        );
                                    },
                                );
                            });

                            // Total Time
                            row_ui.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{:.1}s",
                                                pat.total_duration_ms as f64 / 1000.0
                                            ))
                                            .size(11.5)
                                            .monospace()
                                            .strong()
                                            .color(
                                                if dark {
                                                    TailwindColors::SLATE_100
                                                } else {
                                                    TailwindColors::SLATE_900
                                                },
                                            ),
                                        );
                                    },
                                );
                            });

                            // Avg
                            row_ui.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format_ms(pat.avg_duration_ms))
                                                .size(11.5)
                                                .monospace()
                                                .color(TailwindColors::SLATE_600),
                                        );
                                    },
                                );
                            });

                            // P95 (Blue)
                            row_ui.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format_ms(
                                                pat.p95_duration_ms as f64,
                                            ))
                                            .size(11.5)
                                            .monospace()
                                            .strong()
                                            .color(TailwindColors::BLUE_600),
                                        );
                                    },
                                );
                            });

                            // Max (Rose)
                            row_ui.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format_ms(
                                                pat.max_duration_ms as f64,
                                            ))
                                            .size(11.5)
                                            .monospace()
                                            .strong()
                                            .color(TailwindColors::ROSE_600),
                                        );
                                    },
                                );
                            });

                            // Scan Ratio
                            row_ui.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let col = if pat.scan_ratio > 100.0 {
                                            TailwindColors::ROSE_600
                                        } else {
                                            TailwindColors::SLATE_600
                                        };
                                        let ratio_str = if pat.scan_ratio > 0.0 {
                                            format!("{:.1}x", pat.scan_ratio)
                                        } else {
                                            "0x".to_string()
                                        };
                                        ui.label(
                                            egui::RichText::new(ratio_str)
                                                .size(11.5)
                                                .monospace()
                                                .color(col),
                                        );
                                    },
                                );
                            });

                            // Index suggestion (with Copy)
                            row_ui.col(|ui| {
                                if !pat.index_suggestion.is_empty() {
                                    let idx_sugg = &pat.index_suggestion;
                                    let copy_frame = Frame::NONE
                                        .fill(if dark {
                                            TailwindColors::EMERALD_950
                                        } else {
                                            TailwindColors::EMERALD_50
                                        })
                                        .stroke(Stroke::new(1.0, TailwindColors::EMERALD_200))
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(Margin::symmetric(8, 3));

                                    let resp = copy_frame
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.add_sized(
                                                    [ui.available_width() - 16.0, 14.0],
                                                    egui::Label::new(
                                                        egui::RichText::new(idx_sugg)
                                                            .size(10.0)
                                                            .monospace()
                                                            .color(TailwindColors::EMERALD_800),
                                                    )
                                                    .truncate(),
                                                );
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.add(render_icon(
                                                            "copy",
                                                            TailwindColors::EMERALD_600,
                                                            11.0,
                                                        ));
                                                    },
                                                );
                                            });
                                        })
                                        .response;

                                    if resp.interact(egui::Sense::click()).clicked() {
                                        ui.copy_text(idx_sugg.clone());
                                    }
                                } else {
                                    ui.label(
                                        egui::RichText::new("Indexed / Covered")
                                            .size(10.0)
                                            .italics()
                                            .color(TailwindColors::SLATE_400),
                                    );
                                }
                            });

                            // Inspect icon
                            row_ui.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    let resp = ui.add(render_icon(
                                        "external-link",
                                        TailwindColors::SLATE_400,
                                        14.0,
                                    ));
                                    if resp.interact(egui::Sense::click()).clicked() {
                                        *selected = Some(pat.example_query.clone());
                                    }
                                });
                            });
                        });
                    }
                });
        });

    sort_changed
}

fn render_sortable_col(
    ui: &mut Ui,
    title: &str,
    field: MongoSortField,
    filters: &mut MongoFilters,
    sort_changed: &mut bool,
) {
    let is_active = filters.sort_field == field;
    let icon_name = if is_active {
        if filters.sort_direction == MongoSortDirection::Asc {
            "arrow-up"
        } else {
            "arrow-down"
        }
    } else {
        "arrow-up-down"
    };

    let text_col = TailwindColors::SLATE_600;
    let icon_col = if is_active {
        TailwindColors::EMERALD_600
    } else {
        TailwindColors::SLATE_400
    };

    let resp = ui
        .with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(11.0)
                    .strong()
                    .color(text_col),
            );
            ui.add_space(2.0);
            ui.add(render_icon(icon_name, icon_col, 10.0));
        })
        .response;

    if resp.interact(egui::Sense::click()).clicked() {
        if is_active {
            filters.sort_direction = if filters.sort_direction == MongoSortDirection::Asc {
                MongoSortDirection::Desc
            } else {
                MongoSortDirection::Asc
            };
        } else {
            filters.sort_field = field;
            filters.sort_direction = MongoSortDirection::Desc;
        }
        *sort_changed = true;
    }
}

pub(super) fn render_mongo_op_badge(ui: &mut Ui, op: &str, dark: bool) {
    let (bg, fg) = match op {
        "find" => (TailwindColors::SKY_50, TailwindColors::SKY_700),
        "aggregate" => (
            Color32::from_rgb(250, 245, 255),
            Color32::from_rgb(126, 34, 206),
        ),
        "distinct" => (TailwindColors::INDIGO_50, TailwindColors::INDIGO_600),
        "getMore" => (TailwindColors::TEAL_50, TailwindColors::TEAL_700),
        "update" | "findAndModify" => (TailwindColors::AMBER_50, TailwindColors::AMBER_700),
        "delete" => (TailwindColors::ROSE_50, TailwindColors::ROSE_700),
        "count" => (TailwindColors::EMERALD_50, TailwindColors::EMERALD_700),
        _ => (TailwindColors::SLATE_100, TailwindColors::SLATE_600),
    };

    Frame::NONE
        .fill(if dark { TailwindColors::SLATE_800 } else { bg })
        .corner_radius(CornerRadius::same(3))
        .inner_margin(Margin::symmetric(5, 1))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(op.to_uppercase())
                    .size(9.5)
                    .strong()
                    .color(fg),
            );
        });
}
