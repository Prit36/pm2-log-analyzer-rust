use super::controls::paint_th;
use super::*;

// 3. User Activity tab
pub(super) fn render_user_activity_tab(
    ui: &mut Ui,
    res: &MongoAggregationResult,
    active_user_detail: &mut Option<MongoUserActivity>,
    user_search: &mut String,
    filters: &mut MongoFilters,
    active_view: &mut MongoActiveView,
    dark: bool,
) -> bool {
    let mut filter_changed = false;
    let users = &res.users;
    let connections = &res.connections;
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

    // 1. Top 4 KPI cards
    let named_users = users.iter().filter(|u| u.user_name != "system").count();
    let total_auth_success = if connections.auth_success > 0 {
        connections.auth_success
    } else {
        users.iter().map(|u| u.auth_success_count).sum()
    };
    let total_auth_fail = if connections.auth_failed > 0 {
        connections.auth_failed
    } else {
        users.iter().map(|u| u.auth_fail_count).sum()
    };
    let top_user = users
        .iter()
        .filter(|u| u.user_name != "system")
        .max_by_key(|u| u.slow_query_count);

    let total_w = content_width(ui);
    let gap = 12.0;
    let card_columns = if total_w >= LG_CONTENT_WIDTH { 4 } else { 2 };
    let card_w = (total_w - gap * (card_columns - 1) as f32) / card_columns as f32;
    let (fail_icon_col, fail_bg_col) = if total_auth_fail > 0 {
        (TailwindColors::ROSE_600, Color32::from_rgb(255, 241, 242))
    } else {
        (TailwindColors::SLATE_400, Color32::from_rgb(248, 250, 252))
    };
    let top_name = top_user
        .map(|u| u.user_name.clone())
        .unwrap_or_else(|| "None".to_string());
    let top_queries = top_user.map(|u| u.slow_query_count).unwrap_or(0);
    let cards = [
        (
            "users",
            TailwindColors::EMERALD_600,
            Color32::from_rgb(236, 253, 245),
            "Identified Users",
            format!("{named_users}"),
            Some(format!("({} total)", users.len())),
        ),
        (
            "user-check",
            TailwindColors::BLUE_600,
            Color32::from_rgb(239, 246, 255),
            "Authenticated Sessions",
            format_num(total_auth_success),
            None,
        ),
        (
            "shield-alert",
            fail_icon_col,
            fail_bg_col,
            "Auth / Security Fails",
            format_num(total_auth_fail),
            None,
        ),
        (
            "flame",
            TailwindColors::PURPLE_600,
            Color32::from_rgb(243, 232, 255),
            "Top Querying User",
            top_name,
            Some(format!("{} queries", format_num(top_queries))),
        ),
    ];

    ui.allocate_ui_with_layout(
        egui::vec2(total_w, 0.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_width(total_w);
            for row_start in (0..cards.len()).step_by(card_columns) {
                let row_end = (row_start + card_columns).min(cards.len());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for (icon, icon_col, icon_bg, label, value, sub) in &cards[row_start..row_end] {
                        render_user_kpi_card(
                            ui,
                            icon,
                            *icon_col,
                            *icon_bg,
                            label,
                            value,
                            sub.as_deref(),
                            card_w,
                            dark,
                        );
                    }
                });
            }
        },
    );

    ui.add_space(16.0);

    // 2. Master-Detail Split
    let has_detail = active_user_detail.is_some();
    let wide_layout = total_w >= LG_CONTENT_WIDTH;
    let available_split_width = total_w - 16.0;
    let left_w = if wide_layout {
        if has_detail {
            (available_split_width * 0.60).floor()
        } else {
            (available_split_width * 0.65).floor()
        }
    } else {
        total_w
    };
    let right_w = if wide_layout {
        total_w - left_w - 16.0
    } else {
        total_w
    };
    let detail_layout = if wide_layout {
        egui::Layout::left_to_right(egui::Align::Min)
    } else {
        egui::Layout::top_down(egui::Align::Min)
    };

    ui.allocate_ui_with_layout(egui::vec2(total_w, 0.0), detail_layout, |ui| {
        // Left Column: User Activity Tracking
        Frame::NONE
            .fill(bg_color)
            .stroke(Stroke::new(1.0, border_color))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::ZERO)
            .show(ui, |ui| {
                ui.set_width(left_w);
                ui.vertical(|ui| {
                    // Card header
                    Frame::NONE
                        .inner_margin(Margin::symmetric(14, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(render_icon("user", TailwindColors::EMERALD_600, 16.0));
                                ui.add_space(4.0);
                                ui.label(egui::RichText::new(format!("User Activity Tracking ({})", users.len())).size(13.0).strong().color(if dark { TailwindColors::SLATE_100 } else { TailwindColors::SLATE_900 }));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(user_search)
                                            .hint_text("Filter users, IPs, apps...")
                                            .desired_width(180.0)
                                            .margin(Margin::symmetric(8, 5))
                                    );
                                });
                            });
                        });

                    // User Table
                    let q_search = user_search.to_lowercase();
                    let filtered_users: Vec<&MongoUserActivity> = users.iter().filter(|u| {
                        if q_search.is_empty() {
                            true
                        } else {
                            u.user_name.to_lowercase().contains(&q_search)
                                || u.app_name.to_lowercase().contains(&q_search)
                                || u.auth_db.to_lowercase().contains(&q_search)
                                || u.client_ips.iter().any(|ip| ip.contains(&q_search))
                        }
                    }).collect();

                    let effective_height = (filtered_users.len() as f32 * 46.0 + 36.0).clamp(160.0, 480.0);

                    let compact_table = left_w < 700.0;
                    let (queries_width, scans_width, time_width, avg_width, security_width, actions_width) =
                        if compact_table {
                            (50.0, 58.0, 62.0, 74.0, 56.0, 56.0)
                        } else {
                            (60.0, 76.0, 76.0, 90.0, 72.0, 64.0)
                        };
                    let fixed_width = queries_width
                        + scans_width
                        + time_width
                        + avg_width
                        + security_width
                        + actions_width;
                    let first_min_width = if compact_table { 240.0 } else { 260.0 };
                    let table_min_width = left_w.max(fixed_width + first_min_width);
                    egui::ScrollArea::horizontal()
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.set_width(table_min_width);
                    TableBuilder::new(ui)
                        .striped(true)
                        .vscroll(true)
                        .min_scrolled_height(effective_height)
                        .max_scroll_height(effective_height)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::remainder().at_least(first_min_width)) // User & Client
                        .column(Column::exact(queries_width)) // Queries
                        .column(Column::exact(scans_width)) // COLLSCANs
                        .column(Column::exact(time_width)) // Total Time
                        .column(Column::exact(avg_width)) // Avg (P95)
                        .column(Column::exact(security_width)) // Security
                        .column(Column::exact(actions_width)) // Actions
                        .auto_shrink([false, true])
                    .header(30.0, |mut header| {
                        header.col(|ui| { paint_th(ui, dark); ui.label(egui::RichText::new("User & Client").size(11.0).strong().color(TailwindColors::SLATE_600)); });
                        header.col(|ui| { paint_th(ui, dark); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.add_space(4.0); ui.label(egui::RichText::new("Queries").size(11.0).strong().color(TailwindColors::SLATE_600)); }); });
                        header.col(|ui| { paint_th(ui, dark); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.add_space(4.0); ui.label(egui::RichText::new(if compact_table { "Scans" } else { "COLLSCANs" }).size(11.0).strong().color(TailwindColors::SLATE_600)); }); });
                        header.col(|ui| { paint_th(ui, dark); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.add_space(4.0); ui.label(egui::RichText::new(if compact_table { "Total" } else { "Total Time" }).size(11.0).strong().color(TailwindColors::SLATE_600)); }); });
                        header.col(|ui| { paint_th(ui, dark); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.add_space(4.0); ui.label(egui::RichText::new(if compact_table { "Avg/P95" } else { "Avg (P95)" }).size(11.0).strong().color(TailwindColors::SLATE_600)); }); });
                        header.col(|ui| { paint_th(ui, dark); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.add_space(4.0); ui.label(egui::RichText::new(if compact_table { "Auth" } else { "Security" }).size(11.0).strong().color(TailwindColors::SLATE_600)); }); });
                        header.col(|ui| { paint_th(ui, dark); ui.centered_and_justified(|ui| { ui.label(egui::RichText::new("Actions").size(11.0).strong().color(TailwindColors::SLATE_600)); }); });
                    })
                    .body(|mut body| {
                        for u in &filtered_users {
                            body.row(46.0, |mut row_ui| {
                                // 1. User & Client
                                row_ui.col(|ui| {
                                    let user_ui = ui.horizontal(|ui| {
                                        let initial = u.user_name.chars().next().unwrap_or('?').to_uppercase().to_string();
                                        Frame::NONE
                                            .fill(if u.user_name == "system" { TailwindColors::SLATE_200 } else { Color32::from_rgb(209, 250, 229) })
                                            .corner_radius(CornerRadius::same(12))
                                            .inner_margin(Margin::same(4))
                                            .show(ui, |ui| {
                                                ui.set_width(16.0);
                                                ui.centered_and_justified(|ui| {
                                                    ui.label(egui::RichText::new(initial).size(10.5).strong().color(if u.user_name == "system" { TailwindColors::SLATE_600 } else { TailwindColors::EMERALD_800 }));
                                                });
                                            });
                                        ui.add_space(4.0);
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new(&u.user_name).size(11.5).strong().color(if dark { TailwindColors::SLATE_100 } else { TailwindColors::SLATE_900 }));
                                                if !u.auth_db.is_empty() {
                                                    ui.label(egui::RichText::new(format!("@{}", u.auth_db)).size(9.5).color(TailwindColors::SLATE_400));
                                                }
                                            });
                                            if !u.client_ips.is_empty() {
                                                ui.label(egui::RichText::new(u.client_ips.join(", ")).size(9.5).monospace().color(TailwindColors::SLATE_400));
                                            }
                                        });
                                    });
                                    if user_ui.response.interact(egui::Sense::click()).clicked() {
                                        *active_user_detail = Some((*u).clone());
                                    }
                                });

                                // 2. Queries
                                row_ui.col(|ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new(format_num(u.slow_query_count)).size(11.0).strong().color(if dark { TailwindColors::SLATE_200 } else { TailwindColors::SLATE_800 }));
                                    });
                                });

                                // 3. COLLSCANs
                                row_ui.col(|ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(4.0);
                                        if u.collscan_count > 0 {
                                            Frame::NONE
                                                .fill(Color32::from_rgb(254, 243, 199))
                                                .corner_radius(CornerRadius::same(4))
                                                .inner_margin(Margin::symmetric(4, 2))
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.add(render_icon("flame", TailwindColors::AMBER_600, 10.0));
                                                        ui.label(egui::RichText::new(format_num(u.collscan_count)).size(10.0).strong().color(TailwindColors::AMBER_800));
                                                    });
                                                });
                                        } else {
                                            ui.label(egui::RichText::new("0").size(11.0).color(TailwindColors::SLATE_400));
                                        }
                                    });
                                });

                                // 4. Total Time
                                row_ui.col(|ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new(format_ms(u.total_duration_ms as f64)).size(11.0).monospace().color(if dark { TailwindColors::SLATE_300 } else { TailwindColors::SLATE_700 }));
                                    });
                                });

                                // 5. Avg (P95)
                                row_ui.col(|ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(4.0);
                                        let text = format!("{} ({})", format_ms(u.avg_duration_ms), format_ms(u.p95_duration_ms as f64));
                                        ui.label(egui::RichText::new(text).size(10.0).color(TailwindColors::SLATE_500));
                                    });
                                });

                                // 6. Security
                                row_ui.col(|ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(4.0);
                                        if u.auth_fail_count > 0 {
                                            Frame::NONE
                                                .fill(Color32::from_rgb(254, 242, 242))
                                                .corner_radius(CornerRadius::same(4))
                                                .inner_margin(Margin::symmetric(4, 2))
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.add(render_icon("alert-triangle", TailwindColors::ROSE_600, 10.0));
                                                        ui.label(egui::RichText::new(format!("{} fails", u.auth_fail_count)).size(9.5).strong().color(TailwindColors::ROSE_700));
                                                    });
                                                });
                                        } else if u.auth_success_count > 0 {
                                            Frame::NONE
                                                .fill(Color32::from_rgb(236, 253, 245))
                                                .corner_radius(CornerRadius::same(4))
                                                .inner_margin(Margin::symmetric(4, 2))
                                                .show(ui, |ui| {
                                                    ui.label(egui::RichText::new(format!("{} auth", u.auth_success_count)).size(9.5).color(TailwindColors::EMERALD_700));
                                                });
                                        } else {
                                            ui.label(egui::RichText::new("-").size(10.0).color(TailwindColors::SLATE_400));
                                        }
                                    });
                                });

                                // 7. Actions
                                row_ui.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        let is_filtered = filters.user_filter == u.user_name;
                                        let btn_text = if is_filtered { "Active" } else { "Filter →" };
                                        let btn = ui.add(
                                            egui::Button::new(egui::RichText::new(btn_text).size(10.5).strong())
                                                .fill(if is_filtered { TailwindColors::EMERALD_600 } else { if dark { TailwindColors::SLATE_800 } else { TailwindColors::SLATE_100 } })
                                                .corner_radius(CornerRadius::same(4))
                                        );
                                        if btn.clicked() {
                                            if is_filtered {
                                                filters.user_filter = "all".to_string();
                                            } else {
                                                filters.user_filter = u.user_name.clone();
                                            }
                                            filter_changed = true;
                                        }
                                    });
                                });
                            });
                        }
                    });
                        });
                });
            });

        ui.add_space(16.0);

        // Right Column: Inspector or Empty State
        if let Some(user_detail) = active_user_detail {
            render_user_inspector_card(ui, user_detail, filters, active_view, &mut filter_changed, right_w, dark);
        } else {
            render_user_inspector_empty(ui, right_w, dark);
        }
    });

    filter_changed
}

fn render_user_kpi_card(
    ui: &mut Ui,
    icon: &str,
    icon_col: Color32,
    icon_bg: Color32,
    label: &str,
    val: &str,
    sub: Option<&str>,
    width: f32,
    dark: bool,
) {
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
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width((width - 28.0).max(80.0));
            ui.horizontal(|ui| {
                Frame::NONE
                    .fill(if dark {
                        TailwindColors::SLATE_800
                    } else {
                        icon_bg
                    })
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.add(render_icon(icon, icon_col, 20.0));
                    });
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .size(11.0)
                            .color(TailwindColors::SLATE_500),
                    );
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(val).size(18.0).strong().color(if dark {
                            TailwindColors::SLATE_100
                        } else {
                            TailwindColors::SLATE_900
                        }));
                        if let Some(s) = sub {
                            ui.label(
                                egui::RichText::new(s)
                                    .size(11.0)
                                    .color(TailwindColors::SLATE_400),
                            );
                        }
                    });
                });
            });
        });
}

fn render_user_inspector_empty(ui: &mut Ui, width: f32, dark: bool) {
    let border_color = if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_200
    };
    Frame::NONE
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.5, border_color))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(32))
        .show(ui, |ui| {
            ui.set_width((width - 64.0).max(160.0));
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.add(render_icon("user", if dark { TailwindColors::SLATE_700 } else { TailwindColors::SLATE_300 }, 40.0));
                ui.add_space(12.0);
                ui.label(egui::RichText::new("Select a User to Inspect").size(14.0).strong().color(if dark { TailwindColors::SLATE_300 } else { TailwindColors::SLATE_700 }));
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Click on any user in the table to review their executed queries, affected collections, client applications, and security authorization logs.")
                        .size(11.0)
                        .color(TailwindColors::SLATE_400),
                );
                ui.add_space(40.0);
            });
        });
}

fn render_user_inspector_card(
    ui: &mut Ui,
    u: &MongoUserActivity,
    filters: &mut MongoFilters,
    active_view: &mut MongoActiveView,
    filter_changed: &mut bool,
    width: f32,
    dark: bool,
) {
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
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width((width - 32.0).max(180.0));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let initial = u
                        .user_name
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string();
                    Frame::NONE
                        .fill(TailwindColors::EMERALD_600)
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::same(6))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(initial)
                                    .size(14.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                        });
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&u.user_name).size(14.0).strong().color(
                            if dark {
                                TailwindColors::SLATE_100
                            } else {
                                TailwindColors::SLATE_900
                            },
                        ));
                        if !u.auth_db.is_empty() {
                            ui.label(
                                egui::RichText::new(format!("DB: {}", u.auth_db))
                                    .size(10.5)
                                    .color(TailwindColors::SLATE_400),
                            );
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Filter View").clicked() {
                            filters.user_filter = u.user_name.clone();
                            *filter_changed = true;
                        }
                    });
                });

                ui.add_space(14.0);

                ui.horizontal(|ui| {
                    let stat_w = (width - 48.0) / 4.0;
                    render_user_stat_box(
                        ui,
                        "DB Time",
                        &format_ms(u.total_duration_ms as f64),
                        stat_w,
                        dark,
                    );
                    ui.add_space(6.0);
                    render_user_stat_box(
                        ui,
                        "Queries",
                        &format_num(u.slow_query_count),
                        stat_w,
                        dark,
                    );
                    ui.add_space(6.0);
                    let unindexed_pct = if u.slow_query_count > 0 {
                        ((u.collscan_count as f64 / u.slow_query_count as f64) * 100.0).round()
                            as u64
                    } else {
                        0
                    };
                    render_user_stat_box(
                        ui,
                        "COLLSCANs",
                        &format!("{}%", unindexed_pct),
                        stat_w,
                        dark,
                    );
                    ui.add_space(6.0);
                    render_user_stat_box(
                        ui,
                        "P95",
                        &format_ms(u.p95_duration_ms as f64),
                        stat_w,
                        dark,
                    );
                });

                ui.add_space(14.0);

                ui.label(
                    egui::RichText::new("Top Affected Collections")
                        .size(11.5)
                        .strong()
                        .color(if dark {
                            TailwindColors::SLATE_300
                        } else {
                            TailwindColors::SLATE_700
                        }),
                );
                ui.add_space(4.0);
                for col in u.top_collections.iter().take(5) {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&col.ns)
                                .size(10.5)
                                .strong()
                                .color(if dark {
                                    TailwindColors::SLATE_200
                                } else {
                                    TailwindColors::SLATE_800
                                }),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("{} queries", col.count))
                                    .size(10.0)
                                    .color(TailwindColors::SLATE_400),
                            );
                        });
                    });
                    ui.separator();
                }

                ui.add_space(10.0);

                if ui
                    .button(
                        egui::RichText::new(format!(
                            "View {} queries in Slow Query Log →",
                            u.slow_query_count
                        ))
                        .size(11.5)
                        .strong()
                        .color(TailwindColors::EMERALD_600),
                    )
                    .clicked()
                {
                    filters.user_filter = u.user_name.clone();
                    *active_view = MongoActiveView::SlowQueries;
                    *filter_changed = true;
                }
            });
        });
}

pub(super) fn render_user_stat_box(ui: &mut Ui, label: &str, val: &str, w: f32, dark: bool) {
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
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(6, 6))
        .show(ui, |ui| {
            ui.set_width((w - 12.0).max(60.0));
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(9.5)
                        .color(TailwindColors::SLATE_400),
                );
                ui.label(egui::RichText::new(val).size(12.0).strong().color(if dark {
                    TailwindColors::SLATE_100
                } else {
                    TailwindColors::SLATE_900
                }));
            });
        });
}
