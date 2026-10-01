use super::*;

// 4. Charts tab
pub(super) fn render_mongo_charts_tab(
    ui: &mut Ui,
    res: &MongoAggregationResult,
    chart_mode: &mut MongoChartMode,
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
            ui.set_width((content_width(ui) - 32.0).max(320.0));
            ui.vertical(|ui| {
                let compact = compact_width(ui);
                ui.horizontal_wrapped(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Database Performance Over Time")
                                .size(14.0)
                                .strong()
                                .color(if dark {
                                    TailwindColors::SLATE_100
                                } else {
                                    TailwindColors::SLATE_900
                                }),
                        );
                        ui.label(
                            egui::RichText::new(
                                "Latency percentiles, collection scans, and top resource consumers",
                            )
                            .size(11.0)
                            .color(TailwindColors::SLATE_500),
                        );
                    });

                    if compact {
                        render_chart_mode_switcher(ui, chart_mode, dark);
                    } else {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            render_chart_mode_switcher(ui, chart_mode, dark);
                        });
                    }
                });

                ui.add_space(14.0);

                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new((content_width(ui) - 32.0).max(320.0), 340.0),
                    egui::Sense::hover(),
                );
                let painter = ui.painter_at(rect);

                match *chart_mode {
                    MongoChartMode::ThroughputLatency => {
                        let buckets = &res.time_buckets;
                        if buckets.is_empty() {
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "No time bucket data available",
                                FontId::proportional(12.0),
                                TailwindColors::SLATE_400,
                            );
                            return;
                        }

                        let max_q_raw = buckets
                            .iter()
                            .map(|b| b.query_count)
                            .max()
                            .unwrap_or(1)
                            .max(10) as f32;
                        let max_lat_raw = buckets
                            .iter()
                            .map(|b| b.p95_duration_ms)
                            .max()
                            .unwrap_or(100)
                            .max(1000) as f32;

                        let max_q = ((max_q_raw / 40.0).ceil() * 40.0).max(40.0);
                        let max_lat = ((max_lat_raw / 2000.0).ceil() * 2000.0).max(2000.0);

                        let plot_left = rect.min.x + 40.0;
                        let plot_right = rect.max.x - 65.0;
                        let plot_top = rect.min.y + 10.0;
                        let plot_bottom = rect.max.y - 30.0;
                        let plot_w = plot_right - plot_left;
                        let plot_h = plot_bottom - plot_top;

                        for step in 0..=4 {
                            let y = plot_bottom - (step as f32 / 4.0) * plot_h;
                            // Horizontal dotted grid line matching Recharts
                            let mut gx = plot_left;
                            while gx < plot_right {
                                let gx2 = (gx + 3.0).min(plot_right);
                                painter.line_segment(
                                    [Pos2::new(gx, y), Pos2::new(gx2, y)],
                                    Stroke::new(
                                        1.0,
                                        if dark {
                                            TailwindColors::SLATE_800
                                        } else {
                                            TailwindColors::SLATE_200
                                        },
                                    ),
                                );
                                gx += 7.0;
                            }

                            let q_val = (max_q * step as f32 / 4.0).round() as u64;
                            painter.text(
                                Pos2::new(plot_left - 8.0, y),
                                egui::Align2::RIGHT_CENTER,
                                format_num(q_val),
                                FontId::monospace(10.0),
                                TailwindColors::SLATE_400,
                            );

                            let lat_val = (max_lat * step as f32 / 4.0).round() as u64;
                            painter.text(
                                Pos2::new(plot_right + 8.0, y),
                                egui::Align2::LEFT_CENTER,
                                format!("{}ms", lat_val),
                                FontId::monospace(10.0),
                                TailwindColors::BLUE_600,
                            );
                        }

                        let n = buckets.len();
                        let slot_w = plot_w / n as f32;
                        let bar_w = (slot_w * 0.55).clamp(8.0, 36.0);

                        let mut lat_points = Vec::with_capacity(n);

                        for (i, b) in buckets.iter().enumerate() {
                            let cx = plot_left + (i as f32 + 0.5) * slot_w;
                            let bar_h = (b.query_count as f32 / max_q) * plot_h;
                            let bar_rect = Rect::from_min_max(
                                Pos2::new(cx - bar_w / 2.0, plot_bottom - bar_h),
                                Pos2::new(cx + bar_w / 2.0, plot_bottom),
                            );
                            painter.rect_filled(
                                bar_rect,
                                CornerRadius {
                                    nw: 4,
                                    ne: 4,
                                    sw: 0,
                                    se: 0,
                                },
                                Color32::from_rgb(52, 211, 153),
                            );

                            let lat_y = plot_bottom - (b.p95_duration_ms as f32 / max_lat) * plot_h;
                            let pt = Pos2::new(cx, lat_y);
                            lat_points.push(pt);

                            let lbl = if !b.hour_label.is_empty() {
                                &b.hour_label
                            } else {
                                &b.time_key
                            };
                            painter.text(
                                Pos2::new(cx, plot_bottom + 14.0),
                                egui::Align2::CENTER_CENTER,
                                lbl,
                                FontId::monospace(9.5),
                                TailwindColors::SLATE_500,
                            );
                        }

                        if lat_points.len() >= 2 {
                            for w in lat_points.windows(2) {
                                let p0 = w[0];
                                let p1 = w[1];
                                let dx = (p1.x - p0.x) * 0.4;
                                let c1 = Pos2::new(p0.x + dx, p0.y);
                                let c2 = Pos2::new(p1.x - dx, p1.y);
                                painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                                    [p0, c1, c2, p1],
                                    false,
                                    Color32::TRANSPARENT,
                                    Stroke::new(2.5, TailwindColors::BLUE_600),
                                ));
                            }
                        }
                    }
                    MongoChartMode::Plans => {
                        let buckets = &res.time_buckets;
                        let max_q = buckets
                            .iter()
                            .map(|b| b.query_count)
                            .max()
                            .unwrap_or(1)
                            .max(10) as f32;
                        let plot_left = rect.min.x + 40.0;
                        let plot_right = rect.max.x - 20.0;
                        let plot_top = rect.min.y + 10.0;
                        let plot_bottom = rect.max.y - 30.0;
                        let plot_w = plot_right - plot_left;
                        let plot_h = plot_bottom - plot_top;

                        let n = buckets.len();
                        let slot_w = plot_w / n.max(1) as f32;
                        let bar_w = (slot_w * 0.55).clamp(8.0, 36.0);

                        for (i, b) in buckets.iter().enumerate() {
                            let cx = plot_left + (i as f32 + 0.5) * slot_w;
                            let coll_h = (b.collscan_count as f32 / max_q) * plot_h;
                            let ix_cnt = b.query_count.saturating_sub(b.collscan_count);
                            let ix_h = (ix_cnt as f32 / max_q) * plot_h;

                            let ix_rect = Rect::from_min_max(
                                Pos2::new(cx - bar_w / 2.0, plot_bottom - ix_h),
                                Pos2::new(cx + bar_w / 2.0, plot_bottom),
                            );
                            painter.rect_filled(
                                ix_rect,
                                CornerRadius::ZERO,
                                Color32::from_rgb(16, 185, 129),
                            );

                            let coll_rect = Rect::from_min_max(
                                Pos2::new(cx - bar_w / 2.0, plot_bottom - ix_h - coll_h),
                                Pos2::new(cx + bar_w / 2.0, plot_bottom - ix_h),
                            );
                            painter.rect_filled(
                                coll_rect,
                                CornerRadius::same(2),
                                Color32::from_rgb(245, 158, 11),
                            );

                            let lbl = if !b.hour_label.is_empty() {
                                &b.hour_label
                            } else {
                                &b.time_key
                            };
                            painter.text(
                                Pos2::new(cx, plot_bottom + 14.0),
                                egui::Align2::CENTER_CENTER,
                                lbl,
                                FontId::monospace(9.5),
                                TailwindColors::SLATE_500,
                            );
                        }
                    }
                    MongoChartMode::TopCollections => {
                        let mut colls = res.collections.clone();
                        colls.sort_by(|a, b| b.total_duration_ms.cmp(&a.total_duration_ms));
                        let top_colls: Vec<&MongoCollectionMetric> = colls.iter().take(8).collect();
                        let max_dur = top_colls
                            .first()
                            .map(|c| c.total_duration_ms)
                            .unwrap_or(1)
                            .max(1) as f32;

                        let row_h = 32.0;
                        let start_y = rect.min.y + 15.0;

                        for (i, c) in top_colls.iter().enumerate() {
                            let y = start_y + i as f32 * (row_h + 6.0);
                            painter.text(
                                Pos2::new(rect.min.x + 10.0, y + 10.0),
                                egui::Align2::LEFT_CENTER,
                                &c.collection,
                                FontId::proportional(11.5),
                                if dark {
                                    TailwindColors::SLATE_200
                                } else {
                                    TailwindColors::SLATE_800
                                },
                            );

                            let bar_start = rect.min.x + 160.0;
                            let bar_max_w = rect.width() - 260.0;
                            let bar_w = (c.total_duration_ms as f32 / max_dur) * bar_max_w;

                            let b_rect = Rect::from_min_max(
                                Pos2::new(bar_start, y + 2.0),
                                Pos2::new(bar_start + bar_w.max(4.0), y + 20.0),
                            );
                            painter.rect_filled(
                                b_rect,
                                CornerRadius::same(3),
                                Color32::from_rgb(99, 102, 241),
                            );

                            painter.text(
                                Pos2::new(bar_start + bar_w + 8.0, y + 10.0),
                                egui::Align2::LEFT_CENTER,
                                format!(
                                    "{:.1}s ({} q)",
                                    c.total_duration_ms as f64 / 1000.0,
                                    format_num(c.query_count)
                                ),
                                FontId::monospace(10.5),
                                TailwindColors::SLATE_500,
                            );
                        }
                    }
                }
            });
        });
}

fn render_chart_mode_switcher(ui: &mut Ui, chart_mode: &mut MongoChartMode, dark: bool) {
    Frame::NONE
        .fill(if dark {
            TailwindColors::SLATE_800
        } else {
            TailwindColors::SLATE_100
        })
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for mode in [
                    MongoChartMode::ThroughputLatency,
                    MongoChartMode::Plans,
                    MongoChartMode::TopCollections,
                ] {
                    let is_active = *chart_mode == mode;
                    let btn_fg = if is_active {
                        if dark {
                            TailwindColors::EMERALD_400
                        } else {
                            TailwindColors::EMERALD_700
                        }
                    } else {
                        TailwindColors::SLATE_500
                    };
                    let btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(mode.label())
                                .size(11.0)
                                .strong()
                                .color(btn_fg),
                        )
                        .fill(if is_active {
                            if dark {
                                TailwindColors::SLATE_900
                            } else {
                                Color32::WHITE
                            }
                        } else {
                            Color32::TRANSPARENT
                        })
                        .corner_radius(CornerRadius::same(4)),
                    );
                    if btn.clicked() {
                        *chart_mode = mode;
                    }
                }
            });
        });
}
