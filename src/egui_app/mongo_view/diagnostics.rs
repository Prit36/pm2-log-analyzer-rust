use super::controls::paint_th;
use super::users::render_user_stat_box;
use super::*;

// 5. Diagnostics tab
pub(super) fn render_diagnostics_tab(
    ui: &mut Ui,
    res: &MongoAggregationResult,
    diag_tab: &mut MongoDiagTab,
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
                // Sub-tab switcher bar matching ref-mongo-diag.png
                ui.horizontal_wrapped(|ui| {
                    let tabs = [
                        (
                            MongoDiagTab::Errors,
                            "alert-triangle",
                            format!("Errors & Warnings ({})", res.errors.len()),
                        ),
                        (
                            MongoDiagTab::Connections,
                            "server",
                            "Connection Pool & Drivers".to_string(),
                        ),
                        (
                            MongoDiagTab::Collections,
                            "database",
                            format!("Collections Summary ({})", res.collections.len()),
                        ),
                        (
                            MongoDiagTab::Checkpoints,
                            "clock",
                            format!("Checkpoints ({})", res.checkpoints.len()),
                        ),
                    ];

                    for (tab, icon, label) in tabs {
                        let is_active = *diag_tab == tab;
                        let (pill_bg, pill_fg, stroke_col) = if is_active {
                            match tab {
                                MongoDiagTab::Errors => (
                                    if dark {
                                        Color32::from_rgb(60, 45, 10)
                                    } else {
                                        Color32::from_rgb(254, 243, 199)
                                    },
                                    if dark {
                                        TailwindColors::AMBER_300
                                    } else {
                                        TailwindColors::AMBER_800
                                    },
                                    if dark {
                                        TailwindColors::AMBER_700
                                    } else {
                                        TailwindColors::AMBER_300
                                    },
                                ),
                                MongoDiagTab::Connections => (
                                    if dark {
                                        Color32::from_rgb(10, 45, 30)
                                    } else {
                                        Color32::from_rgb(209, 250, 229)
                                    },
                                    if dark {
                                        TailwindColors::EMERALD_300
                                    } else {
                                        TailwindColors::EMERALD_800
                                    },
                                    if dark {
                                        TailwindColors::EMERALD_700
                                    } else {
                                        TailwindColors::EMERALD_300
                                    },
                                ),
                                MongoDiagTab::Collections => (
                                    if dark {
                                        Color32::from_rgb(35, 15, 55)
                                    } else {
                                        Color32::from_rgb(243, 232, 255)
                                    },
                                    if dark {
                                        TailwindColors::PURPLE_200
                                    } else {
                                        TailwindColors::PURPLE_800
                                    },
                                    if dark {
                                        TailwindColors::PURPLE_700
                                    } else {
                                        TailwindColors::PURPLE_200
                                    },
                                ),
                                MongoDiagTab::Checkpoints => (
                                    if dark {
                                        Color32::from_rgb(15, 30, 60)
                                    } else {
                                        Color32::from_rgb(239, 246, 255)
                                    },
                                    if dark {
                                        TailwindColors::BLUE_300
                                    } else {
                                        TailwindColors::BLUE_800
                                    },
                                    if dark {
                                        TailwindColors::BLUE_700
                                    } else {
                                        TailwindColors::BLUE_300
                                    },
                                ),
                            }
                        } else {
                            (
                                Color32::TRANSPARENT,
                                TailwindColors::SLATE_500,
                                Color32::TRANSPARENT,
                            )
                        };

                        Frame::NONE
                            .fill(pill_bg)
                            .stroke(Stroke::new(1.0, stroke_col))
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                let resp = ui
                                    .horizontal(|ui| {
                                        ui.add(render_icon(icon, pill_fg, 13.0));
                                        ui.add_space(4.0);
                                        ui.label(
                                            egui::RichText::new(&label)
                                                .size(11.5)
                                                .strong()
                                                .color(pill_fg),
                                        );
                                    })
                                    .response;
                                if resp.interact(egui::Sense::click()).clicked() {
                                    *diag_tab = tab;
                                }
                            });
                        ui.add_space(6.0);
                    }
                });

                ui.add_space(16.0);

                // Sub-tab content
                match *diag_tab {
                    MongoDiagTab::Errors => {
                        if res.errors.is_empty() {
                            ui.label(
                                egui::RichText::new("No engine warnings or errors recorded.")
                                    .color(TailwindColors::SLATE_400),
                            );
                        } else {
                            for e in &res.errors {
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
                                    .inner_margin(Margin::symmetric(14, 12))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let (badge_bg, badge_fg) =
                                                if e.severity == "E" || e.severity == "F" {
                                                    (
                                                        Color32::from_rgb(254, 242, 242),
                                                        TailwindColors::ROSE_700,
                                                    )
                                                } else {
                                                    (
                                                        Color32::from_rgb(254, 243, 199),
                                                        TailwindColors::AMBER_800,
                                                    )
                                                };
                                            Frame::NONE
                                                .fill(badge_bg)
                                                .corner_radius(CornerRadius::same(4))
                                                .inner_margin(Margin::symmetric(6, 4))
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        egui::RichText::new(&e.severity)
                                                            .size(11.0)
                                                            .strong()
                                                            .color(badge_fg),
                                                    );
                                                });
                                            ui.add_space(8.0);
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&e.msg)
                                                        .size(12.0)
                                                        .strong()
                                                        .color(if dark {
                                                            TailwindColors::SLATE_100
                                                        } else {
                                                            TailwindColors::SLATE_800
                                                        }),
                                                );
                                                ui.add_space(2.0);
                                                let id_str =
                                                    e.id.map(|id| format!(" · ID: {id}"))
                                                        .unwrap_or_default();
                                                let dt_str = format_date_time(Some(&e.timestamp));
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "Component: {}{id_str} · {dt_str}",
                                                        e.component
                                                    ))
                                                    .size(10.5)
                                                    .color(TailwindColors::SLATE_500),
                                                );
                                            });

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    Frame::NONE
                                                        .fill(if dark {
                                                            TailwindColors::SLATE_700
                                                        } else {
                                                            TailwindColors::SLATE_200
                                                        })
                                                        .corner_radius(CornerRadius::same(10))
                                                        .inner_margin(Margin::symmetric(8, 2))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                egui::RichText::new(format!(
                                                                    "{}x",
                                                                    e.count
                                                                ))
                                                                .size(11.0)
                                                                .strong()
                                                                .color(if dark {
                                                                    TailwindColors::SLATE_200
                                                                } else {
                                                                    TailwindColors::SLATE_700
                                                                }),
                                                            );
                                                        });
                                                },
                                            );
                                        });
                                    });
                                ui.add_space(8.0);
                            }
                        }
                    }
                    MongoDiagTab::Connections => {
                        let conn = &res.connections;
                        ui.horizontal(|ui| {
                            let card_w = (ui.available_width() - 36.0) / 4.0;
                            render_user_stat_box(
                                ui,
                                "Accepted Connections",
                                &format_num(conn.accepted),
                                card_w,
                                dark,
                            );
                            ui.add_space(12.0);
                            render_user_stat_box(
                                ui,
                                "Closed Connections",
                                &format_num(conn.ended),
                                card_w,
                                dark,
                            );
                            ui.add_space(12.0);
                            render_user_stat_box(
                                ui,
                                "Peak Concurrent",
                                &format_num(conn.peak_concurrent),
                                card_w,
                                dark,
                            );
                            ui.add_space(12.0);
                            render_user_stat_box(
                                ui,
                                "Auth Fails",
                                &format_num(conn.auth_failed),
                                card_w,
                                dark,
                            );
                        });

                        ui.add_space(16.0);
                        ui.label(
                            egui::RichText::new("Connected Client Drivers")
                                .size(12.0)
                                .strong()
                                .color(if dark {
                                    TailwindColors::SLATE_100
                                } else {
                                    TailwindColors::SLATE_800
                                }),
                        );
                        ui.add_space(4.0);
                        for drv in &conn.drivers {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} {}",
                                        drv.driver_name, drv.driver_version
                                    ))
                                    .size(11.0)
                                    .strong(),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "({} {})",
                                        drv.platform, drv.os_name
                                    ))
                                    .size(10.5)
                                    .color(TailwindColors::SLATE_400),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} connections",
                                                drv.count
                                            ))
                                            .size(11.0)
                                            .monospace(),
                                        );
                                    },
                                );
                            });
                            ui.separator();
                        }
                    }
                    MongoDiagTab::Collections => {
                        let colls = &res.collections;
                        let effective_h = (colls.len() as f32 * 32.0 + 36.0).clamp(160.0, 480.0);
                        TableBuilder::new(ui)
                            .striped(true)
                            .vscroll(true)
                            .min_scrolled_height(effective_h)
                            .max_scroll_height(effective_h)
                            .column(Column::remainder().at_least(180.0))
                            .column(Column::exact(80.0))
                            .column(Column::exact(80.0))
                            .column(Column::exact(80.0))
                            .column(Column::exact(80.0))
                            .column(Column::exact(80.0))
                            .header(28.0, |mut h| {
                                h.col(|ui| {
                                    paint_th(ui, dark);
                                    ui.label(egui::RichText::new("Collection").size(11.0).strong());
                                });
                                h.col(|ui| {
                                    paint_th(ui, dark);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new("Queries").size(11.0).strong(),
                                            );
                                        },
                                    );
                                });
                                h.col(|ui| {
                                    paint_th(ui, dark);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new("Total Time")
                                                    .size(11.0)
                                                    .strong(),
                                            );
                                        },
                                    );
                                });
                                h.col(|ui| {
                                    paint_th(ui, dark);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new("P95").size(11.0).strong(),
                                            );
                                        },
                                    );
                                });
                                h.col(|ui| {
                                    paint_th(ui, dark);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new("COLLSCANs")
                                                    .size(11.0)
                                                    .strong(),
                                            );
                                        },
                                    );
                                });
                                h.col(|ui| {
                                    paint_th(ui, dark);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new("Scan Ratio")
                                                    .size(11.0)
                                                    .strong(),
                                            );
                                        },
                                    );
                                });
                            })
                            .body(|mut b| {
                                for c in colls {
                                    b.row(30.0, |mut r| {
                                        r.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(&c.collection)
                                                    .size(11.0)
                                                    .strong(),
                                            );
                                        });
                                        r.col(|ui| {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        egui::RichText::new(format_num(
                                                            c.query_count,
                                                        ))
                                                        .size(11.0)
                                                        .monospace(),
                                                    );
                                                },
                                            );
                                        });
                                        r.col(|ui| {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        egui::RichText::new(format_ms(
                                                            c.total_duration_ms as f64,
                                                        ))
                                                        .size(11.0)
                                                        .monospace(),
                                                    );
                                                },
                                            );
                                        });
                                        r.col(|ui| {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        egui::RichText::new(format_ms(
                                                            c.p95_duration_ms as f64,
                                                        ))
                                                        .size(11.0)
                                                        .monospace(),
                                                    );
                                                },
                                            );
                                        });
                                        r.col(|ui| {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        egui::RichText::new(format_num(
                                                            c.collscan_count,
                                                        ))
                                                        .size(11.0)
                                                        .monospace()
                                                        .color(if c.collscan_count > 0 {
                                                            TailwindColors::AMBER_600
                                                        } else {
                                                            TailwindColors::SLATE_400
                                                        }),
                                                    );
                                                },
                                            );
                                        });
                                        r.col(|ui| {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "{:.1}x",
                                                            c.scan_ratio
                                                        ))
                                                        .size(11.0)
                                                        .monospace()
                                                        .color(if c.scan_ratio > 100.0 {
                                                            TailwindColors::ROSE_600
                                                        } else {
                                                            TailwindColors::SLATE_600
                                                        }),
                                                    );
                                                },
                                            );
                                        });
                                    });
                                }
                            });
                    }
                    MongoDiagTab::Checkpoints => {
                        let checks = &res.checkpoints;
                        let effective_h = (checks.len() as f32 * 32.0 + 36.0).clamp(160.0, 480.0);
                        TableBuilder::new(ui)
                            .striped(true)
                            .vscroll(true)
                            .min_scrolled_height(effective_h)
                            .max_scroll_height(effective_h)
                            .column(Column::exact(220.0))
                            .column(Column::exact(100.0))
                            .column(Column::remainder().at_least(200.0))
                            .header(28.0, |mut h| {
                                h.col(|ui| {
                                    paint_th(ui, dark);
                                    ui.label(egui::RichText::new("Timestamp").size(11.0).strong());
                                });
                                h.col(|ui| {
                                    paint_th(ui, dark);
                                    ui.label(
                                        egui::RichText::new("Bytes Written").size(11.0).strong(),
                                    );
                                });
                                h.col(|ui| {
                                    paint_th(ui, dark);
                                    ui.label(egui::RichText::new("Message").size(11.0).strong());
                                });
                            })
                            .body(|mut b| {
                                for cp in checks {
                                    b.row(30.0, |mut r| {
                                        r.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(&cp.timestamp)
                                                    .size(10.5)
                                                    .monospace(),
                                            );
                                        });
                                        r.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(
                                                    cp.bytes_written
                                                        .map(format_bytes)
                                                        .unwrap_or_else(|| "-".to_string()),
                                                )
                                                .size(11.0)
                                                .monospace(),
                                            );
                                        });
                                        r.col(|ui| {
                                            ui.label(
                                                egui::RichText::new(&cp.msg)
                                                    .size(10.5)
                                                    .color(TailwindColors::SLATE_500),
                                            );
                                        });
                                    });
                                }
                            });
                    }
                }
            });
        });
}
