use super::*;

// Combined View Switcher Tabs & Filter Bar matching MongoFilterBar.tsx
pub(super) fn render_mongo_controls(
    ui: &mut Ui,
    active_view: &mut MongoActiveView,
    filters: &mut MongoFilters,
    res: &MongoAggregationResult,
    dark: bool,
) -> bool {
    let mut changed = false;
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

    let collscan_count = res.summary.collscan_count;
    let diag_events: u64 = res.errors.iter().map(|d| d.count).sum();

    Frame::NONE
        .fill(bg_color)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width((content_width(ui) - 28.0).max(320.0));
            ui.vertical(|ui| {
                let compact = compact_width(ui);
                // Row 1: View switcher tabs
                let mode_bg = if dark {
                    TailwindColors::SLATE_950
                } else {
                    TailwindColors::SLATE_100
                };
                Frame::NONE
                    .fill(mode_bg)
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(3))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            render_tab_button(
                                ui,
                                active_view,
                                MongoActiveView::Patterns,
                                "database",
                                "Query Patterns",
                                Some(res.patterns.len()),
                                dark,
                            );
                            render_tab_button(
                                ui,
                                active_view,
                                MongoActiveView::SlowQueries,
                                "file-text",
                                "Slow Query Log",
                                Some(res.summary.slow_query_count as usize),
                                dark,
                            );
                            render_tab_button(
                                ui,
                                active_view,
                                MongoActiveView::Users,
                                "user",
                                "User Activity",
                                if res.users.is_empty() {
                                    None
                                } else {
                                    Some(res.users.len())
                                },
                                dark,
                            );
                            render_tab_button(
                                ui,
                                active_view,
                                MongoActiveView::Charts,
                                "chart-column",
                                "Latency Charts",
                                None,
                                dark,
                            );
                            render_tab_button(
                                ui,
                                active_view,
                                MongoActiveView::Diagnostics,
                                "shield-alert",
                                "Diagnostics",
                                if diag_events > 0 {
                                    Some(diag_events as usize)
                                } else {
                                    None
                                },
                                dark,
                            );
                        });
                    });

                ui.add_space(10.0);

                // Row 2: Search input across full width inside rounded card border matching MongoFilterBar.tsx
                let search_bg = if dark {
                    TailwindColors::SLATE_950
                } else {
                    TailwindColors::SLATE_50
                };
                let search_border = if dark {
                    TailwindColors::SLATE_800
                } else {
                    TailwindColors::SLATE_200
                };
                Frame::NONE
                    .fill(search_bg)
                    .stroke(Stroke::new(1.0, search_border))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(render_icon("search", TailwindColors::SLATE_400, 14.0));
                            ui.add_space(4.0);
                            let search_res = ui.add_sized(
                                [ui.available_width() - 8.0, 20.0],
                                egui::TextEdit::singleline(&mut filters.search_query)
                                    .hint_text("Search collection, plan, IP...")
                                    .frame(Frame::NONE),
                            );
                            if search_res.changed() {
                                changed = true;
                            }
                        });
                    });

                ui.add_space(10.0);

                // Row 3: Plan chips, Op dropdown, Collection dropdown, User dropdown
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("Plan:")
                            .size(11.0)
                            .strong()
                            .color(TailwindColors::SLATE_400),
                    );

                    // All Plans chip
                    let is_all = filters.plan_filter == MongoPlanFilter::All;
                    if render_filter_chip(ui, "All Plans", is_all, dark) {
                        filters.plan_filter = MongoPlanFilter::All;
                        changed = true;
                    }

                    // COLLSCAN Only chip (with flame and amber color)
                    let is_coll = filters.plan_filter == MongoPlanFilter::CollscanOnly;
                    let coll_label = format!("COLLSCAN Only ({collscan_count})");
                    if render_amber_chip(ui, &coll_label, is_coll, dark) {
                        filters.plan_filter = if is_coll {
                            MongoPlanFilter::All
                        } else {
                            MongoPlanFilter::CollscanOnly
                        };
                        changed = true;
                    }

                    // IXSCAN chip
                    let is_ix = filters.plan_filter == MongoPlanFilter::IxscanOnly;
                    if render_filter_chip(ui, "IXSCAN", is_ix, dark) {
                        filters.plan_filter = if is_ix {
                            MongoPlanFilter::All
                        } else {
                            MongoPlanFilter::IxscanOnly
                        };
                        changed = true;
                    }

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("|").color(TailwindColors::SLATE_200));
                    ui.add_space(8.0);

                    // Op dropdown
                    ui.label(
                        egui::RichText::new("Op:")
                            .size(11.0)
                            .strong()
                            .color(TailwindColors::SLATE_400),
                    );
                    let op_label = if filters.operation == "all" {
                        "All Operations".to_string()
                    } else {
                        filters.operation.clone()
                    };
                    egui::ComboBox::from_id_salt("mongo_op_dropdown")
                        .selected_text(op_label)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_value(
                                    &mut filters.operation,
                                    "all".to_string(),
                                    "All Operations",
                                )
                                .clicked()
                            {
                                changed = true;
                            }
                            for op in &res.operations {
                                if ui
                                    .selectable_value(&mut filters.operation, op.clone(), op)
                                    .clicked()
                                {
                                    changed = true;
                                }
                            }
                        });

                    ui.add_space(8.0);

                    // Collection dropdown
                    if !res.collections.is_empty() {
                        ui.label(
                            egui::RichText::new("Collection:")
                                .size(11.0)
                                .strong()
                                .color(TailwindColors::SLATE_400),
                        );
                        let coll_label = if filters.collection == "all" {
                            format!("All Collections ({})", res.collections.len())
                        } else {
                            res.collections
                                .iter()
                                .find(|c| c.ns == filters.collection)
                                .map(|c| c.collection.clone())
                                .unwrap_or_else(|| filters.collection.clone())
                        };
                        egui::ComboBox::from_id_salt("mongo_coll_dropdown")
                            .selected_text(coll_label)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_value(
                                        &mut filters.collection,
                                        "all".to_string(),
                                        format!("All Collections ({})", res.collections.len()),
                                    )
                                    .clicked()
                                {
                                    changed = true;
                                }
                                for c in &res.collections {
                                    let label = format!("{} ({})", c.collection, c.query_count);
                                    if ui
                                        .selectable_value(
                                            &mut filters.collection,
                                            c.ns.clone(),
                                            label,
                                        )
                                        .clicked()
                                    {
                                        changed = true;
                                    }
                                }
                            });
                    }

                    ui.add_space(8.0);

                    // User dropdown
                    if !res.user_names.is_empty() {
                        ui.label(
                            egui::RichText::new("User:")
                                .size(11.0)
                                .strong()
                                .color(TailwindColors::SLATE_400),
                        );
                        let user_label = if filters.user_filter == "all" {
                            format!("All Users ({})", res.user_names.len())
                        } else {
                            filters.user_filter.clone()
                        };
                        egui::ComboBox::from_id_salt("mongo_user_dropdown")
                            .selected_text(user_label)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_value(
                                        &mut filters.user_filter,
                                        "all".to_string(),
                                        format!("All Users ({})", res.user_names.len()),
                                    )
                                    .clicked()
                                {
                                    changed = true;
                                }
                                for u in &res.user_names {
                                    if ui
                                        .selectable_value(
                                            &mut filters.user_filter,
                                            u.clone(),
                                            u.clone(),
                                        )
                                        .clicked()
                                    {
                                        changed = true;
                                    }
                                }
                            });
                    }
                });

                ui.add_space(10.0);

                // Row 4: Duration presets, Scan Ratio toggle, Reset button
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("Duration:")
                            .size(11.0)
                            .strong()
                            .color(TailwindColors::SLATE_400),
                    );

                    let presets: [(&str, u32); 5] = [
                        ("All", 0),
                        (">100ms", 100),
                        (">500ms", 500),
                        (">1s", 1000),
                        (">5s", 5000),
                    ];
                    for (lbl, ms) in presets {
                        let is_active = filters.min_duration_ms == ms;
                        if render_filter_chip(ui, lbl, is_active, dark) {
                            filters.min_duration_ms = ms;
                            changed = true;
                        }
                    }

                    ui.add_space(8.0);

                    // Scan Ratio >100x button
                    let is_high_scan = filters.high_scan_ratio_only;
                    let scan_bg = if is_high_scan {
                        if dark {
                            TailwindColors::ROSE_950
                        } else {
                            TailwindColors::ROSE_50
                        }
                    } else if dark {
                        TailwindColors::SLATE_800
                    } else {
                        Color32::WHITE
                    };
                    let scan_border = if is_high_scan {
                        TailwindColors::ROSE_300
                    } else if dark {
                        TailwindColors::SLATE_700
                    } else {
                        TailwindColors::SLATE_200
                    };
                    let scan_fg = if is_high_scan {
                        TailwindColors::ROSE_700
                    } else if dark {
                        TailwindColors::SLATE_300
                    } else {
                        TailwindColors::SLATE_600
                    };

                    let scan_frame = Frame::NONE
                        .fill(scan_bg)
                        .stroke(Stroke::new(1.0, scan_border))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(8, 3));

                    let scan_resp = scan_frame
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(render_icon("filter", scan_fg, 11.0));
                                ui.add_space(2.0);
                                ui.label(
                                    egui::RichText::new("Scan Ratio >100x")
                                        .size(11.0)
                                        .strong()
                                        .color(scan_fg),
                                );
                            });
                        })
                        .response;

                    if scan_resp.interact(egui::Sense::click()).clicked() {
                        filters.high_scan_ratio_only = !filters.high_scan_ratio_only;
                        changed = true;
                    }

                    // A right-to-left child consumes a wrapped row on narrow
                    // windows; keep the reset action with the controls only
                    // when there is enough room.
                    if !compact {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if render_mongo_reset_button(ui, filters, dark) {
                                changed = true;
                            }
                        });
                    }
                });

                if compact {
                    ui.allocate_ui_with_layout(
                        egui::vec2((content_width(ui) - 28.0).max(0.0), 20.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if render_mongo_reset_button(ui, filters, dark) {
                                changed = true;
                            }
                        },
                    );
                }
            });
        });

    changed
}

fn render_mongo_reset_button(ui: &mut Ui, filters: &mut MongoFilters, dark: bool) -> bool {
    let active_count = filters.active_count();
    let reset_resp = ui
        .horizontal(|ui| {
            if active_count > 0 {
                Frame::NONE
                    .fill(if dark {
                        TailwindColors::ROSE_950
                    } else {
                        TailwindColors::ROSE_100
                    })
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(4, 1))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("{active_count}"))
                                .size(10.0)
                                .strong()
                                .color(TailwindColors::ROSE_700),
                        );
                    });
                ui.add_space(2.0);
            }
            ui.label(egui::RichText::new("Reset all filters").size(11.0).color(
                if active_count > 0 {
                    TailwindColors::ROSE_600
                } else {
                    TailwindColors::SLATE_400
                },
            ));
            ui.add_space(2.0);
            ui.add(render_icon(
                "rotate-ccw",
                if active_count > 0 {
                    TailwindColors::ROSE_600
                } else {
                    TailwindColors::SLATE_400
                },
                12.0,
            ));
        })
        .response;

    if active_count > 0 && reset_resp.interact(egui::Sense::click()).clicked() {
        *filters = MongoFilters::default();
        true
    } else {
        false
    }
}

fn render_tab_button(
    ui: &mut Ui,
    current: &mut MongoActiveView,
    target: MongoActiveView,
    icon_name: &str,
    label: &str,
    count: Option<usize>,
    dark: bool,
) {
    let is_active = *current == target;
    let bg = if is_active {
        if dark {
            TailwindColors::SLATE_900
        } else {
            Color32::WHITE
        }
    } else {
        Color32::TRANSPARENT
    };
    let fg = if is_active {
        TailwindColors::EMERALD_700
    } else if dark {
        TailwindColors::SLATE_400
    } else {
        TailwindColors::SLATE_600
    };

    let frame = Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(10, 6));

    let resp = frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(render_icon(icon_name, fg, 13.0));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(label).size(12.0).strong().color(fg));

                if let Some(c) = count {
                    ui.add_space(3.0);
                    Frame::NONE
                        .fill(if is_active {
                            if dark {
                                TailwindColors::EMERALD_950
                            } else {
                                TailwindColors::EMERALD_100
                            }
                        } else {
                            if dark {
                                TailwindColors::SLATE_800
                            } else {
                                TailwindColors::SLATE_200
                            }
                        })
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::symmetric(5, 1))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format_num(c as u64))
                                    .size(10.0)
                                    .strong()
                                    .color(if is_active {
                                        TailwindColors::EMERALD_800
                                    } else {
                                        TailwindColors::SLATE_700
                                    }),
                            );
                        });
                }
            });
        })
        .response;

    if resp.interact(egui::Sense::click()).clicked() {
        *current = target;
    }
}

fn render_filter_chip(ui: &mut Ui, label: &str, active: bool, dark: bool) -> bool {
    let bg = if active {
        TailwindColors::SLATE_900
    } else if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_100
    };
    let fg = if active {
        Color32::WHITE
    } else if dark {
        TailwindColors::SLATE_300
    } else {
        TailwindColors::SLATE_700
    };

    let frame = Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(8, 3));

    let resp = frame
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).size(11.0).strong().color(fg));
        })
        .response;

    resp.interact(egui::Sense::click()).clicked()
}

fn render_amber_chip(ui: &mut Ui, label: &str, active: bool, dark: bool) -> bool {
    let bg = if active {
        TailwindColors::AMBER_600
    } else if dark {
        TailwindColors::AMBER_950
    } else {
        TailwindColors::AMBER_50
    };
    let fg = if active {
        Color32::WHITE
    } else {
        TailwindColors::AMBER_700
    };
    let border = if active {
        TailwindColors::AMBER_600
    } else {
        TailwindColors::AMBER_300
    };

    let frame = Frame::NONE
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(8, 3));

    let resp = frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(render_icon(
                    "flame",
                    if active {
                        Color32::WHITE
                    } else {
                        TailwindColors::AMBER_500
                    },
                    11.0,
                ));
                ui.add_space(2.0);
                ui.label(egui::RichText::new(label).size(11.0).strong().color(fg));
            });
        })
        .response;

    resp.interact(egui::Sense::click()).clicked()
}

pub(super) fn paint_th(ui: &Ui, dark: bool) {
    let th_bg = if dark {
        TailwindColors::SLATE_950
    } else {
        TailwindColors::SLATE_50
    };
    let th_border = if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_200
    };
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, CornerRadius::ZERO, th_bg);
    ui.painter().line_segment(
        [
            egui::pos2(rect.min.x, rect.max.y),
            egui::pos2(rect.max.x, rect.max.y),
        ],
        Stroke::new(1.0, th_border),
    );
}
