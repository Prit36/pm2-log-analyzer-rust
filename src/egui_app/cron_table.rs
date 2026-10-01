use super::icons::render_icon;
use super::theme::{
    TailwindColors, font_medium, font_mono, font_mono_semibold, font_regular, font_semibold,
};
use crate::core::models::CronAggregated;
use crate::store::analysis_store::{AnalysisFilters, CronSortKey, SortDirection};
use crate::utils::format::{format_ms, format_num};
use egui::{Align, Color32, CornerRadius, Frame, Margin, Rect, Stroke, Ui, pos2, vec2};
use egui_extras::{Column, TableBuilder};

pub enum CronTableAction {
    None,
    CopyTsv,
    ChangeSort(CronSortKey),
    Reaggregate,
}

pub fn render_cron_table(
    ui: &mut Ui,
    rows: &[CronAggregated],
    filters: &mut AnalysisFilters,
    dark: bool,
) -> CronTableAction {
    let mut action = CronTableAction::None;

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
            ui.vertical(|ui| {
                // Header bar matching CronTable.tsx
                let header_h = 34.0;
                let (h_rect, _) = ui.allocate_exact_size(
                    vec2(ui.available_width(), header_h),
                    egui::Sense::hover(),
                );
                if ui.is_rect_visible(h_rect) {
                    ui.painter()
                        .rect_filled(h_rect, CornerRadius::ZERO, bg_color);
                    ui.painter().line_segment(
                        [
                            pos2(h_rect.min.x, h_rect.max.y),
                            pos2(h_rect.max.x, h_rect.max.y),
                        ],
                        Stroke::new(1.0, border_color),
                    );

                    let title_pos = pos2(h_rect.min.x + 14.0, h_rect.center().y - 7.0);
                    let title_galley = ui.painter().layout_no_wrap(
                        "CRON JOBS".to_string(),
                        font_semibold(12.0),
                        if dark {
                            TailwindColors::SLATE_300
                        } else {
                            TailwindColors::SLATE_600
                        },
                    );
                    ui.painter().galley(title_pos, title_galley, Color32::WHITE);
                }

                // Controls inside header
                let mut controls_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_max(
                            pos2(h_rect.min.x + 110.0, h_rect.min.y + 2.0),
                            pos2(h_rect.max.x - 12.0, h_rect.max.y - 2.0),
                        ))
                        .layout(egui::Layout::left_to_right(Align::Center)),
                );

                // Search input
                let search_resp = controls_ui.add_sized(
                    [160.0, 30.0],
                    egui::TextEdit::singleline(&mut filters.cron_query)
                        .hint_text("Filter jobs…")
                        .font(font_medium(12.0)),
                );
                if search_resp.changed() {
                    action = CronTableAction::Reaggregate;
                }

                controls_ui.add_space(8.0);

                // Min ms input
                let mut min_ms_str = if filters.cron_min_ms > 0.0 {
                    filters.cron_min_ms.to_string()
                } else {
                    String::new()
                };
                let resp = controls_ui.add_sized(
                    [80.0, 30.0],
                    egui::TextEdit::singleline(&mut min_ms_str)
                        .hint_text("Min ms")
                        .font(font_medium(12.0)),
                );
                if resp.changed() {
                    filters.cron_min_ms = min_ms_str.parse::<f64>().unwrap_or(0.0);
                    action = CronTableAction::Reaggregate;
                }

                controls_ui.add_space(6.0);

                // Failures only checkbox
                let cb_resp = controls_ui.checkbox(
                    &mut filters.cron_show_failed_only,
                    egui::RichText::new("Failures only")
                        .font(font_medium(11.0))
                        .color(if dark {
                            TailwindColors::SLATE_400
                        } else {
                            TailwindColors::SLATE_600
                        }),
                );
                if cb_resp.changed() {
                    action = CronTableAction::Reaggregate;
                }

                // Right controls: Copy TSV
                controls_ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let btn = ui
                        .horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.add(render_icon("copy", TailwindColors::SLATE_400, 12.0));
                            ui.label(
                                egui::RichText::new("Copy TSV")
                                    .font(font_medium(11.0))
                                    .color(if dark {
                                        TailwindColors::SLATE_400
                                    } else {
                                        TailwindColors::SLATE_500
                                    }),
                            )
                        })
                        .response;
                    if btn.interact(egui::Sense::click()).clicked() {
                        action = CronTableAction::CopyTsv;
                    }
                    if btn.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                });

                // Empty state or Table
                if rows.is_empty() {
                    ui.add_space(32.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("No cron jobs match filters.")
                                .font(font_regular(14.0))
                                .line_height(Some(20.0))
                                .color(if dark {
                                    TailwindColors::SLATE_500
                                } else {
                                    TailwindColors::SLATE_400
                                }),
                        );
                    });
                    ui.add_space(32.0);
                } else {
                    let table_height = (rows.len() as f32 * 36.0 + 34.0).clamp(120.0, 360.0);

                    TableBuilder::new(ui)
                        .id_salt("cron_table")
                        .striped(false)
                        .vscroll(rows.len() > 8)
                        .max_scroll_height(table_height)
                        .cell_layout(egui::Layout::left_to_right(Align::Center))
                        .column(Column::remainder().at_least(160.0)) // Job Name
                        .column(Column::exact(56.0)) // Runs
                        .column(Column::exact(56.0)) // Starts
                        .column(Column::exact(56.0)) // Fails
                        .column(Column::exact(64.0)) // Avg
                        .column(Column::exact(64.0)) // p95
                        .column(Column::exact(64.0)) // p99
                        .column(Column::exact(64.0)) // Max
                        .column(Column::exact(64.0)) // Last Duration
                        .header(28.0, |mut header| {
                            header.col(|ui| {
                                if render_cron_sort_header(
                                    ui,
                                    "Job",
                                    CronSortKey::Name,
                                    filters.cron_sort_key,
                                    filters.cron_sort_dir,
                                    false,
                                    dark,
                                ) {
                                    action = CronTableAction::ChangeSort(CronSortKey::Name);
                                }
                            });
                            header.col(|ui| {
                                if render_cron_sort_header(
                                    ui,
                                    "Runs",
                                    CronSortKey::Runs,
                                    filters.cron_sort_key,
                                    filters.cron_sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = CronTableAction::ChangeSort(CronSortKey::Runs);
                                }
                            });
                            header.col(|ui| {
                                if render_cron_sort_header(
                                    ui,
                                    "Starts",
                                    CronSortKey::Starts,
                                    filters.cron_sort_key,
                                    filters.cron_sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = CronTableAction::ChangeSort(CronSortKey::Starts);
                                }
                            });
                            header.col(|ui| {
                                if render_cron_sort_header(
                                    ui,
                                    "Fails",
                                    CronSortKey::Fails,
                                    filters.cron_sort_key,
                                    filters.cron_sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = CronTableAction::ChangeSort(CronSortKey::Fails);
                                }
                            });
                            header.col(|ui| {
                                if render_cron_sort_header(
                                    ui,
                                    "Avg",
                                    CronSortKey::AvgMs,
                                    filters.cron_sort_key,
                                    filters.cron_sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = CronTableAction::ChangeSort(CronSortKey::AvgMs);
                                }
                            });
                            header.col(|ui| {
                                if render_cron_sort_header(
                                    ui,
                                    "p95",
                                    CronSortKey::P95Ms,
                                    filters.cron_sort_key,
                                    filters.cron_sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = CronTableAction::ChangeSort(CronSortKey::P95Ms);
                                }
                            });
                            header.col(|ui| {
                                if render_cron_sort_header(
                                    ui,
                                    "p99",
                                    CronSortKey::P99Ms,
                                    filters.cron_sort_key,
                                    filters.cron_sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = CronTableAction::ChangeSort(CronSortKey::P99Ms);
                                }
                            });
                            header.col(|ui| {
                                if render_cron_sort_header(
                                    ui,
                                    "Max",
                                    CronSortKey::MaxMs,
                                    filters.cron_sort_key,
                                    filters.cron_sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = CronTableAction::ChangeSort(CronSortKey::MaxMs);
                                }
                            });
                            header.col(|ui| {
                                if render_cron_sort_header(
                                    ui,
                                    "Last",
                                    CronSortKey::LastDurationMs,
                                    filters.cron_sort_key,
                                    filters.cron_sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action =
                                        CronTableAction::ChangeSort(CronSortKey::LastDurationMs);
                                }
                            });
                        })
                        .body(|body| {
                            body.rows(36.0, rows.len(), |mut row_ui| {
                                let idx = row_ui.index();
                                let row = &rows[idx];

                                let row_bg = if idx % 2 == 0 {
                                    if dark {
                                        TailwindColors::SLATE_900
                                    } else {
                                        Color32::WHITE
                                    }
                                } else {
                                    if dark {
                                        Color32::from_rgba_premultiplied(15, 23, 42, 100)
                                    } else {
                                        Color32::from_rgba_premultiplied(248, 250, 252, 180)
                                    }
                                };

                                let row_border = if dark {
                                    TailwindColors::SLATE_800
                                } else {
                                    TailwindColors::SLATE_100
                                };

                                // 1. Job Name
                                row_ui.col(|ui| {
                                    let rect = ui.max_rect();
                                    ui.painter().rect_filled(rect, CornerRadius::ZERO, row_bg);
                                    ui.painter().line_segment(
                                        [
                                            pos2(rect.min.x, rect.max.y),
                                            pos2(rect.max.x, rect.max.y),
                                        ],
                                        Stroke::new(1.0, row_border),
                                    );
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new(&row.name).font(font_mono(12.0)).color(
                                            if dark {
                                                TailwindColors::SLATE_200
                                            } else {
                                                TailwindColors::SLATE_800
                                            },
                                        ),
                                    );
                                });

                                // 2. Runs
                                row_ui.col(|ui| {
                                    let rect = ui.max_rect();
                                    ui.painter().rect_filled(rect, CornerRadius::ZERO, row_bg);
                                    ui.painter().line_segment(
                                        [
                                            pos2(rect.min.x, rect.max.y),
                                            pos2(rect.max.x, rect.max.y),
                                        ],
                                        Stroke::new(1.0, row_border),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new(format_num(row.runs))
                                                    .font(font_mono(12.0))
                                                    .color(if dark {
                                                        TailwindColors::SLATE_300
                                                    } else {
                                                        TailwindColors::SLATE_700
                                                    }),
                                            );
                                        },
                                    );
                                });

                                // 3. Starts
                                row_ui.col(|ui| {
                                    let rect = ui.max_rect();
                                    ui.painter().rect_filled(rect, CornerRadius::ZERO, row_bg);
                                    ui.painter().line_segment(
                                        [
                                            pos2(rect.min.x, rect.max.y),
                                            pos2(rect.max.x, rect.max.y),
                                        ],
                                        Stroke::new(1.0, row_border),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new(format_num(row.starts))
                                                    .font(font_mono(12.0))
                                                    .color(if dark {
                                                        TailwindColors::SLATE_500
                                                    } else {
                                                        TailwindColors::SLATE_400
                                                    }),
                                            );
                                        },
                                    );
                                });

                                // 4. Fails (Bold rose if > 0)
                                row_ui.col(|ui| {
                                    let rect = ui.max_rect();
                                    ui.painter().rect_filled(rect, CornerRadius::ZERO, row_bg);
                                    ui.painter().line_segment(
                                        [
                                            pos2(rect.min.x, rect.max.y),
                                            pos2(rect.max.x, rect.max.y),
                                        ],
                                        Stroke::new(1.0, row_border),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.add_space(6.0);
                                            let col = if row.fails > 0 {
                                                if dark {
                                                    TailwindColors::ROSE_400
                                                } else {
                                                    TailwindColors::ROSE_600
                                                }
                                            } else {
                                                if dark {
                                                    TailwindColors::SLATE_600
                                                } else {
                                                    TailwindColors::SLATE_400
                                                }
                                            };
                                            let font = if row.fails > 0 {
                                                font_mono_semibold(11.0)
                                            } else {
                                                font_mono(11.0)
                                            };
                                            ui.label(
                                                egui::RichText::new(format_num(row.fails))
                                                    .font(font)
                                                    .color(col),
                                            );
                                        },
                                    );
                                });

                                // 5. Avg
                                row_ui.col(|ui| {
                                    let rect = ui.max_rect();
                                    ui.painter().rect_filled(rect, CornerRadius::ZERO, row_bg);
                                    ui.painter().line_segment(
                                        [
                                            pos2(rect.min.x, rect.max.y),
                                            pos2(rect.max.x, rect.max.y),
                                        ],
                                        Stroke::new(1.0, row_border),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new(format_ms(row.avg_ms))
                                                    .font(font_mono(12.0))
                                                    .color(if dark {
                                                        TailwindColors::SLATE_300
                                                    } else {
                                                        TailwindColors::SLATE_700
                                                    }),
                                            );
                                        },
                                    );
                                });

                                // 6. p95 (Bold Blue)
                                row_ui.col(|ui| {
                                    let rect = ui.max_rect();
                                    ui.painter().rect_filled(rect, CornerRadius::ZERO, row_bg);
                                    ui.painter().line_segment(
                                        [
                                            pos2(rect.min.x, rect.max.y),
                                            pos2(rect.max.x, rect.max.y),
                                        ],
                                        Stroke::new(1.0, row_border),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new(format_ms(row.p95_ms))
                                                    .font(font_mono_semibold(12.0))
                                                    .color(if dark {
                                                        TailwindColors::BLUE_400
                                                    } else {
                                                        TailwindColors::BLUE_600
                                                    }),
                                            );
                                        },
                                    );
                                });

                                // 7. p99
                                row_ui.col(|ui| {
                                    let rect = ui.max_rect();
                                    ui.painter().rect_filled(rect, CornerRadius::ZERO, row_bg);
                                    ui.painter().line_segment(
                                        [
                                            pos2(rect.min.x, rect.max.y),
                                            pos2(rect.max.x, rect.max.y),
                                        ],
                                        Stroke::new(1.0, row_border),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new(format_ms(row.p99_ms))
                                                    .font(font_mono(12.0))
                                                    .color(if dark {
                                                        TailwindColors::SLATE_300
                                                    } else {
                                                        TailwindColors::SLATE_700
                                                    }),
                                            );
                                        },
                                    );
                                });

                                // 8. Max (Bold Amber)
                                row_ui.col(|ui| {
                                    let rect = ui.max_rect();
                                    ui.painter().rect_filled(rect, CornerRadius::ZERO, row_bg);
                                    ui.painter().line_segment(
                                        [
                                            pos2(rect.min.x, rect.max.y),
                                            pos2(rect.max.x, rect.max.y),
                                        ],
                                        Stroke::new(1.0, row_border),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new(format_ms(row.max_ms))
                                                    .font(font_mono_semibold(12.0))
                                                    .color(if dark {
                                                        TailwindColors::AMBER_400
                                                    } else {
                                                        TailwindColors::AMBER_700
                                                    }),
                                            );
                                        },
                                    );
                                });

                                // 9. Last Duration
                                row_ui.col(|ui| {
                                    let rect = ui.max_rect();
                                    ui.painter().rect_filled(rect, CornerRadius::ZERO, row_bg);
                                    ui.painter().line_segment(
                                        [
                                            pos2(rect.min.x, rect.max.y),
                                            pos2(rect.max.x, rect.max.y),
                                        ],
                                        Stroke::new(1.0, row_border),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.add_space(6.0);
                                            let txt = if let Some(last) = row.last_duration_ms {
                                                format_ms(last)
                                            } else {
                                                "-".to_string()
                                            };
                                            ui.label(
                                                egui::RichText::new(txt)
                                                    .font(font_mono(12.0))
                                                    .color(if dark {
                                                        TailwindColors::SLATE_500
                                                    } else {
                                                        TailwindColors::SLATE_400
                                                    }),
                                            );
                                        },
                                    );
                                });
                            });
                        });
                }
            });
        });

    action
}

fn render_cron_sort_header(
    ui: &mut Ui,
    title: &str,
    col_key: CronSortKey,
    current_key: CronSortKey,
    current_dir: SortDirection,
    right_align: bool,
    dark: bool,
) -> bool {
    let header_bg = if dark {
        TailwindColors::SLATE_950
    } else {
        TailwindColors::SLATE_50
    };
    let border_color = if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_200
    };
    let rect = ui.max_rect();
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, header_bg);
    ui.painter().line_segment(
        [pos2(rect.min.x, rect.max.y), pos2(rect.max.x, rect.max.y)],
        Stroke::new(1.0, border_color),
    );

    let is_active = col_key == current_key;
    let layout = if right_align {
        egui::Layout::right_to_left(Align::Center)
    } else {
        egui::Layout::left_to_right(Align::Center)
    };

    let mut clicked = false;

    ui.with_layout(layout, |ui| {
        let text_color = if is_active {
            if dark {
                TailwindColors::BLUE_400
            } else {
                TailwindColors::BLUE_600
            }
        } else {
            if dark {
                TailwindColors::SLATE_400
            } else {
                TailwindColors::SLATE_500
            }
        };

        let label = if is_active {
            let arrow = match current_dir {
                SortDirection::Asc => " ↑",
                SortDirection::Desc => " ↓",
            };
            format!("{title}{arrow}")
        } else {
            title.to_string()
        };

        if right_align {
            ui.add_space(6.0);
        } else {
            ui.add_space(8.0);
        }

        let resp = ui.add(
            egui::Label::new(
                egui::RichText::new(label)
                    .font(font_semibold(10.0))
                    .extra_letter_spacing(0.25)
                    .color(text_color),
            )
            .sense(egui::Sense::click()),
        );

        if resp.clicked() {
            clicked = true;
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });

    clicked
}
