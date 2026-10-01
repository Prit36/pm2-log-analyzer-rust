use super::icons::render_icon;
use super::theme::{
    TailwindColors, font_medium, font_mono, font_mono_semibold, font_regular, font_semibold,
};
use super::widgets::method_badge;
use crate::core::models::AggregatedEndpoint;
use crate::store::analysis_store::{ApiSortKey, SortDirection};
use crate::utils::format::{format_ms, format_num};
use egui::{Align, Color32, CornerRadius, Frame, Margin, Rect, Stroke, Ui, Vec2, pos2, vec2};
use egui_extras::{Column, TableBuilder};

pub enum ApiTableAction {
    None,
    CopyPath(String),
    CopyTsv,
    ChangeSort(ApiSortKey),
}

pub fn render_api_table(
    ui: &mut Ui,
    rows: &[AggregatedEndpoint],
    _total_count: usize,
    top_n: usize,
    sort_key: ApiSortKey,
    sort_dir: SortDirection,
    _table_height: f32,
    dark: bool,
) -> ApiTableAction {
    let mut action = ApiTableAction::None;

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
                // Header bar: "SLOW API ENDPOINTS" + "Copy TSV"
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

                    let title_pos = pos2(h_rect.min.x + 12.0, h_rect.center().y - 7.0);
                    let title_galley = ui.painter().layout_no_wrap(
                        "SLOW API ENDPOINTS".to_string(),
                        font_semibold(12.0),
                        if dark {
                            TailwindColors::SLATE_300
                        } else {
                            TailwindColors::SLATE_600
                        },
                    );
                    ui.painter().galley(title_pos, title_galley, Color32::WHITE);

                    // Copy TSV button on right
                    let tsv_rect = Rect::from_min_size(
                        pos2(h_rect.max.x - 86.0, h_rect.center().y - 12.0),
                        vec2(76.0, 24.0),
                    );
                    let tsv_resp =
                        ui.interact(tsv_rect, ui.id().with("api_copy_tsv"), egui::Sense::click());
                    if ui.is_rect_visible(tsv_rect) {
                        let tsv_col = if tsv_resp.hovered() {
                            if dark {
                                TailwindColors::SLATE_200
                            } else {
                                TailwindColors::SLATE_800
                            }
                        } else {
                            TailwindColors::SLATE_400
                        };
                        let icon_rect = Rect::from_center_size(
                            pos2(tsv_rect.left() + 10.0, tsv_rect.center().y),
                            Vec2::splat(12.0),
                        );
                        render_icon("copy", tsv_col, 12.0).paint_at(ui, icon_rect);

                        let tsv_galley = ui.painter().layout_no_wrap(
                            "Copy TSV".to_string(),
                            font_medium(11.0),
                            tsv_col,
                        );
                        ui.painter().galley(
                            pos2(
                                tsv_rect.left() + 20.0,
                                tsv_rect.center().y - tsv_galley.size().y / 2.0,
                            ),
                            tsv_galley,
                            tsv_col,
                        );
                    }
                    if tsv_resp.clicked() {
                        action = ApiTableAction::CopyTsv;
                    }
                    if tsv_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }

                // Table with built-in virtualization matching React: Math.min(420, Math.max(120, rows.length * 32 + 36))
                let display_rows: Vec<&AggregatedEndpoint> = rows.iter().take(top_n).collect();
                let num_rows = display_rows.len();

                if num_rows == 0 {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("No matching endpoints")
                                .font(font_regular(14.0))
                                .line_height(Some(20.0))
                                .color(if dark {
                                    TailwindColors::SLATE_500
                                } else {
                                    TailwindColors::SLATE_400
                                }),
                        );
                    });
                    ui.add_space(40.0);
                } else {
                    let effective_height = (num_rows as f32 * 36.0 + 34.0).clamp(120.0, 420.0);

                    TableBuilder::new(ui)
                        .id_salt("api_table")
                        .striped(false)
                        .vscroll(true)
                        .min_scrolled_height(effective_height)
                        .max_scroll_height(effective_height)
                        .cell_layout(egui::Layout::left_to_right(Align::Center))
                        .column(Column::remainder().at_least(180.0)) // Endpoint
                        .column(Column::exact(56.0)) // Count
                        .column(Column::exact(58.0)) // Avg
                        .column(Column::exact(58.0)) // p95
                        .column(Column::exact(58.0)) // p99
                        .column(Column::exact(58.0)) // Max
                        .column(Column::exact(56.0)) // Errors
                        .header(28.0, |mut header| {
                            header.col(|ui| {
                                if render_sort_header(
                                    ui,
                                    "Endpoint",
                                    ApiSortKey::Path,
                                    sort_key,
                                    sort_dir,
                                    false,
                                    dark,
                                ) {
                                    action = ApiTableAction::ChangeSort(ApiSortKey::Path);
                                }
                            });
                            header.col(|ui| {
                                if render_sort_header(
                                    ui,
                                    "Count",
                                    ApiSortKey::Count,
                                    sort_key,
                                    sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = ApiTableAction::ChangeSort(ApiSortKey::Count);
                                }
                            });
                            header.col(|ui| {
                                if render_sort_header(
                                    ui,
                                    "Avg",
                                    ApiSortKey::AvgMs,
                                    sort_key,
                                    sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = ApiTableAction::ChangeSort(ApiSortKey::AvgMs);
                                }
                            });
                            header.col(|ui| {
                                if render_sort_header(
                                    ui,
                                    "p95",
                                    ApiSortKey::P95Ms,
                                    sort_key,
                                    sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = ApiTableAction::ChangeSort(ApiSortKey::P95Ms);
                                }
                            });
                            header.col(|ui| {
                                if render_sort_header(
                                    ui,
                                    "p99",
                                    ApiSortKey::P99Ms,
                                    sort_key,
                                    sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = ApiTableAction::ChangeSort(ApiSortKey::P99Ms);
                                }
                            });
                            header.col(|ui| {
                                if render_sort_header(
                                    ui,
                                    "Max",
                                    ApiSortKey::MaxMs,
                                    sort_key,
                                    sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = ApiTableAction::ChangeSort(ApiSortKey::MaxMs);
                                }
                            });
                            header.col(|ui| {
                                if render_sort_header(
                                    ui,
                                    "Errors",
                                    ApiSortKey::ErrorCount,
                                    sort_key,
                                    sort_dir,
                                    true,
                                    dark,
                                ) {
                                    action = ApiTableAction::ChangeSort(ApiSortKey::ErrorCount);
                                }
                            });
                        })
                        .body(|body| {
                            body.rows(36.0, num_rows, |mut row_ui| {
                                let idx = row_ui.index();
                                let row = display_rows[idx];

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

                                // 1. Endpoint (Method badge + Path)
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

                                    // Reference rows are a `group`: hovering
                                    // anywhere lights the path and reveals the
                                    // copy icon; only the path itself is a button.
                                    let row_hovered = ui
                                        .input(|i| i.pointer.hover_pos())
                                        .map(|p| p.y >= rect.min.y && p.y < rect.max.y)
                                        .unwrap_or(false);
                                    let path_hovered = ui.rect_contains_pointer(rect);
                                    if path_hovered {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }

                                    ui.horizontal(|ui| {
                                        ui.add_space(12.0);
                                        method_badge(ui, row.method.as_str(), dark);
                                        ui.add_space(8.0);

                                        let path_col = if row_hovered {
                                            if dark {
                                                TailwindColors::BLUE_400
                                            } else {
                                                TailwindColors::BLUE_600
                                            }
                                        } else if dark {
                                            TailwindColors::SLATE_200
                                        } else {
                                            TailwindColors::SLATE_800
                                        };

                                        let path_str = if row.path.chars().count() > 46 {
                                            let mut s: String = row.path.chars().take(44).collect();
                                            s.push('…');
                                            s
                                        } else {
                                            row.path.clone()
                                        };

                                        let path_resp = ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(&path_str)
                                                    .font(font_mono(11.0))
                                                    .color(path_col),
                                            )
                                            .truncate()
                                            .sense(egui::Sense::click()),
                                        );

                                        if row_hovered {
                                            ui.add(render_icon(
                                                "copy",
                                                TailwindColors::SLATE_400,
                                                12.0,
                                            ));
                                        }

                                        if path_resp.clicked()
                                            || (path_hovered
                                                && ui.input(|i| {
                                                    i.pointer.primary_clicked()
                                                        && rect.contains(
                                                            i.pointer
                                                                .interact_pos()
                                                                .unwrap_or_default(),
                                                        )
                                                }))
                                        {
                                            action = ApiTableAction::CopyPath(row.path.clone());
                                        }
                                    });
                                });

                                // 2. Count
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
                                                egui::RichText::new(format_num(row.count))
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

                                // 3. Avg
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

                                // 4. p95 (Bold Blue)
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

                                // 5. p99
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

                                // 6. Max (Bold Amber)
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

                                // 7. Errors (Bold Rose if > 0)
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
                                            ui.add_space(10.0);
                                            let col = if row.error_count > 0 {
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
                                            let font = if row.error_count > 0 {
                                                font_mono_semibold(12.0)
                                            } else {
                                                font_mono(12.0)
                                            };
                                            ui.label(
                                                egui::RichText::new(format_num(row.error_count))
                                                    .font(font)
                                                    .color(col),
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

fn render_sort_header(
    ui: &mut Ui,
    title: &str,
    col_key: ApiSortKey,
    current_key: ApiSortKey,
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
            if col_key == ApiSortKey::ErrorCount {
                ui.add_space(10.0);
            } else {
                ui.add_space(6.0);
            }
        } else {
            ui.add_space(4.0);
        }

        let resp = ui.add(
            egui::Label::new(
                egui::RichText::new(label)
                    .font(font_semibold(11.0))
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
