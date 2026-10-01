use super::*;

pub(super) fn render_mongo_kpis(ui: &mut Ui, res: &MongoAggregationResult, dark: bool) {
    let summary = &res.summary;
    let is_high_collscan = summary.collscan_count > 0;
    let collscan_pct = if summary.slow_query_count > 0 {
        ((summary.collscan_count as f64 / summary.slow_query_count as f64) * 100.0).round() as u64
    } else {
        0
    };

    let peak = res.connections.peak_concurrent;
    let diag_events: u64 = res.errors.iter().map(|d| d.count).sum();
    let diag_types = res.errors.len();

    let total_width = ui.available_width();
    let gap = 12.0;
    let columns = if total_width >= LG_CONTENT_WIDTH {
        6
    } else if total_width >= SM_CONTENT_WIDTH {
        3
    } else {
        2
    };
    let card_w = ((total_width - gap * (columns - 1) as f32) / columns as f32).max(120.0);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        // 1. Slow Queries (Database icon, emerald)
        render_kpi_card(
            ui,
            card_w,
            "Slow Queries",
            "database",
            TailwindColors::EMERALD_600,
            &format_num(summary.slow_query_count),
            if dark {
                TailwindColors::SLATE_100
            } else {
                TailwindColors::SLATE_900
            },
            &format!(
                "{} patterns · {} colls",
                summary.unique_patterns, summary.unique_collections
            ),
            TailwindColors::SLATE_500,
            false,
            dark,
        );
        // 2. COLLSCANs (Flame icon, amber)
        let collscan_sub = if is_high_collscan {
            format!("{collscan_pct}% unindexed scans")
        } else {
            "Zero table scans".to_string()
        };
        render_kpi_card(
            ui,
            card_w,
            "COLLSCANs",
            "flame",
            TailwindColors::AMBER_600,
            &format_num(summary.collscan_count),
            if is_high_collscan {
                TailwindColors::AMBER_700
            } else {
                TailwindColors::SLATE_900
            },
            &collscan_sub,
            TailwindColors::AMBER_600,
            is_high_collscan,
            dark,
        );
        // 3. P95 Latency (Clock icon, blue)
        let p95_sub = format!(
            "Avg: {} · P99: {}",
            format_ms(summary.avg_duration_ms),
            format_ms(summary.p99_duration_ms as f64)
        );
        render_kpi_card(
            ui,
            card_w,
            "P95 Latency",
            "clock",
            TailwindColors::BLUE_600,
            &format_ms(summary.p95_duration_ms as f64),
            TailwindColors::BLUE_600,
            &p95_sub,
            TailwindColors::SLATE_500,
            false,
            dark,
        );
        // 4. Max Duration (Flame icon, rose)
        let max_sub = format!("P50: {}", format_ms(summary.p50_duration_ms as f64));
        render_kpi_card(
            ui,
            card_w,
            "Max Duration",
            "flame",
            TailwindColors::ROSE_600,
            &format_ms(summary.max_duration_ms as f64),
            TailwindColors::ROSE_600,
            &max_sub,
            TailwindColors::SLATE_500,
            false,
            dark,
        );
        // 5. Docs Examined (Network icon, emerald)
        let docs_sub = format!("{:.1}x scan ratio", summary.overall_scan_ratio);
        render_kpi_card(
            ui,
            card_w,
            "Docs Examined",
            "network",
            TailwindColors::EMERALD_600,
            &format_num(summary.total_docs_examined),
            if dark {
                TailwindColors::SLATE_100
            } else {
                TailwindColors::SLATE_900
            },
            &docs_sub,
            TailwindColors::SLATE_500,
            false,
            dark,
        );
        // 6. Diagnostics (Shield-alert icon, amber)
        let diag_val = if peak > 0 {
            format!("{peak} peak")
        } else {
            "Normal".to_string()
        };
        let diag_sub = if diag_events > 0 {
            format!("{diag_events} events ({diag_types} types)")
        } else {
            "No engine errors".to_string()
        };
        render_kpi_card(
            ui,
            card_w,
            "Diagnostics",
            "shield-alert",
            TailwindColors::AMBER_600,
            &diag_val,
            if dark {
                TailwindColors::SLATE_100
            } else {
                TailwindColors::SLATE_900
            },
            &diag_sub,
            TailwindColors::SLATE_500,
            false,
            dark,
        );
    });
}

fn render_kpi_card(
    ui: &mut Ui,
    width: f32,
    title: &str,
    icon_name: &str,
    icon_color: Color32,
    value: &str,
    val_color: Color32,
    subtext: &str,
    sub_color: Color32,
    highlight_amber: bool,
    dark: bool,
) {
    let bg_color = if highlight_amber {
        if dark {
            TailwindColors::AMBER_950
        } else {
            Color32::from_rgb(254, 252, 232)
        }
    } else if dark {
        TailwindColors::SLATE_900
    } else {
        Color32::WHITE
    };
    let border_color = if highlight_amber {
        if dark {
            TailwindColors::AMBER_700
        } else {
            Color32::from_rgb(252, 211, 77)
        }
    } else if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_200
    };

    Frame::NONE
        .fill(bg_color)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_width((width - 28.0).max(60.0));
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(title).size(11.5).strong().color(
                        if highlight_amber {
                            if dark {
                                TailwindColors::AMBER_400
                            } else {
                                TailwindColors::AMBER_700
                            }
                        } else if dark {
                            TailwindColors::SLATE_400
                        } else {
                            TailwindColors::SLATE_500
                        },
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(render_icon(icon_name, icon_color, 15.0));
                    });
                });

                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(value)
                        .size(18.0)
                        .strong()
                        .color(val_color),
                );

                ui.add_space(2.0);
                ui.label(egui::RichText::new(subtext).size(11.0).color(sub_color));
            });
        });
}
