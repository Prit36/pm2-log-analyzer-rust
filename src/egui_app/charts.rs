use super::icons::render_icon;
use super::theme::{TailwindColors, font_medium, font_semibold};
use crate::core::models::{AggregatedEndpoint, DaySummary, HourlyBucket};
use crate::store::analysis_store::ChartLayout;
use crate::utils::format::{format_date, format_ms, format_num};
use egui::{Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect, Stroke, Ui, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ChartViewMode {
    #[default]
    TimeOfDay,
    DailyTrend,
    Throughput,
    Distribution,
    TopP95,
}

pub struct ChartProps<'a> {
    pub rows: &'a [AggregatedEndpoint],
    pub hourly: &'a [HourlyBucket],
    pub daily: &'a [DaySummary],
    pub layout: ChartLayout,
    pub date_filter: &'a str,
    pub dark: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChartAction {
    None,
    ToggleLayout,
}

pub fn render_chart(ui: &mut Ui, props: ChartProps<'_>, mode: &mut ChartViewMode) -> ChartAction {
    let mut action = ChartAction::None;
    let dark = props.dark;

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

    let has_multi_day = props.daily.len() > 1;

    // Header with title, date filter badge, layout toggle, and mode buttons
    Frame::NONE
        .fill(bg_color)
        .stroke(Stroke::new(1.0, border_color))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("API VISUAL ANALYTICS")
                            .font(font_semibold(12.0))
                            .extra_letter_spacing(0.3)
                            .line_height(Some(super::theme::LH_XS))
                            .color(if dark {
                                TailwindColors::SLATE_300
                            } else {
                                TailwindColors::SLATE_700
                            }),
                    );

                    if props.date_filter != "all" {
                        let badge_bg = if dark {
                            TailwindColors::BLUE_950
                        } else {
                            TailwindColors::BLUE_50
                        };
                        let badge_fg = if dark {
                            TailwindColors::BLUE_400
                        } else {
                            TailwindColors::BLUE_700
                        };
                        let badge_border = if dark {
                            TailwindColors::BLUE_950
                        } else {
                            TailwindColors::BLUE_200
                        };
                        Frame::NONE
                            .fill(badge_bg)
                            .stroke(Stroke::new(1.0, badge_border))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::symmetric(7, 2))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format_date(Some(props.date_filter)))
                                        .size(10.0)
                                        .strong()
                                        .color(badge_fg),
                                );
                            });
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let is_wide = props.layout == ChartLayout::Wide;
                        let icon_name = if is_wide { "columns2" } else { "rows2" };
                        let label_text = if is_wide { "Split View" } else { "Wide View" };

                        let toggle_frame = Frame::NONE
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
                            .inner_margin(Margin::symmetric(8, 4));

                        let toggle_resp = toggle_frame
                            .show(ui, |ui| {
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.add(render_icon(
                                            icon_name,
                                            if dark {
                                                TailwindColors::SLATE_300
                                            } else {
                                                TailwindColors::SLATE_600
                                            },
                                            12.0,
                                        ));
                                        ui.add_space(4.0);
                                        ui.label(
                                            egui::RichText::new(label_text)
                                                .font(font_medium(11.0))
                                                .color(if dark {
                                                    TailwindColors::SLATE_300
                                                } else {
                                                    TailwindColors::SLATE_600
                                                }),
                                        );
                                    },
                                );
                            })
                            .response;

                        if toggle_resp.interact(egui::Sense::click()).clicked() {
                            action = ChartAction::ToggleLayout;
                        }
                    });
                });

                ui.add_space(8.0);

                // Mode switcher bar with equal-width buttons matching React flex-1
                let mode_bg = if dark {
                    TailwindColors::SLATE_950
                } else {
                    TailwindColors::SLATE_100
                };
                let num_btns = if has_multi_day { 5 } else { 4 };
                let total_mode_w = ui.available_width() - 8.0;
                let btn_w =
                    ((total_mode_w - (num_btns as f32 - 1.0) * 4.0) / num_btns as f32).floor();

                Frame::NONE
                    .fill(mode_bg)
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::same(2))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        ui.horizontal(|ui| {
                            if has_multi_day {
                                render_mode_btn(
                                    ui,
                                    mode,
                                    ChartViewMode::DailyTrend,
                                    "calendar-days",
                                    "Trend",
                                    btn_w,
                                    dark,
                                );
                            }
                            render_mode_btn(
                                ui,
                                mode,
                                ChartViewMode::TimeOfDay,
                                "clock",
                                if has_multi_day || btn_w < 88.0 {
                                    "Latency"
                                } else {
                                    "Time vs Latency"
                                },
                                btn_w,
                                dark,
                            );
                            render_mode_btn(
                                ui,
                                mode,
                                ChartViewMode::Throughput,
                                "chart-column",
                                if has_multi_day || btn_w < 88.0 {
                                    "Volume"
                                } else {
                                    "Hourly Volume"
                                },
                                btn_w,
                                dark,
                            );
                            render_mode_btn(
                                ui,
                                mode,
                                ChartViewMode::Distribution,
                                "activity",
                                if has_multi_day || btn_w < 88.0 {
                                    "Dist"
                                } else {
                                    "Distribution"
                                },
                                btn_w,
                                dark,
                            );
                            render_mode_btn(
                                ui,
                                mode,
                                ChartViewMode::TopP95,
                                "flame",
                                if has_multi_day || btn_w < 88.0 {
                                    "Slowest"
                                } else {
                                    "Top Slowest"
                                },
                                btn_w,
                                dark,
                            );
                        });
                    });

                ui.add_space(10.0);

                // Draw chart canvas
                let chart_height = if props.layout == ChartLayout::Wide {
                    400.0
                } else {
                    340.0
                };
                let (rect, resp) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), chart_height),
                    egui::Sense::hover(),
                );
                let painter = ui.painter_at(rect);

                // Background
                painter.rect_filled(rect, CornerRadius::ZERO, bg_color);

                let hover_pos = resp.hover_pos();

                match *mode {
                    ChartViewMode::TimeOfDay => {
                        render_hourly_latency(&painter, rect, props.hourly, hover_pos, dark);
                    }
                    ChartViewMode::DailyTrend => {
                        render_daily_trend(&painter, rect, props.daily, hover_pos, dark);
                    }
                    ChartViewMode::Throughput => {
                        render_throughput(&painter, rect, props.hourly, hover_pos, dark);
                    }
                    ChartViewMode::Distribution => {
                        render_distribution(&painter, rect, props.rows, hover_pos, dark);
                    }
                    ChartViewMode::TopP95 => {
                        render_top_slowest(&painter, rect, props.rows, hover_pos, dark);
                    }
                }
            });
        });

    action
}

fn render_mode_btn(
    ui: &mut Ui,
    current: &mut ChartViewMode,
    target: ChartViewMode,
    icon_name: &str,
    label: &str,
    btn_w: f32,
    dark: bool,
) {
    let active = *current == target;
    let bg = if active {
        if dark {
            TailwindColors::BLUE_600
        } else {
            Color32::WHITE
        }
    } else {
        Color32::TRANSPARENT
    };
    let fg = if active {
        if dark {
            Color32::WHITE
        } else {
            TailwindColors::BLUE_600
        }
    } else if dark {
        TailwindColors::SLATE_400
    } else {
        TailwindColors::SLATE_600
    };

    let frame = Frame::NONE
        .fill(bg)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(4, 4));

    let resp = frame
        .show(ui, |ui| {
            ui.set_width(btn_w - 8.0);
            ui.centered_and_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.add(render_icon(icon_name, fg, 13.0));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(label).font(font_medium(11.0)).color(fg));
                });
            });
        })
        .response;

    if resp.interact(egui::Sense::click()).clicked() {
        *current = target;
    }
}

fn paint_dashed_h_line(painter: &egui::Painter, x0: f32, x1: f32, y: f32, stroke: Stroke) {
    let mut x = x0;
    while x < x1 {
        let seg_end = (x + 3.0).min(x1);
        painter.line_segment([Pos2::new(x, y), Pos2::new(seg_end, y)], stroke);
        x += 6.0;
    }
}

fn paint_area_gradient(
    painter: &egui::Painter,
    pts: &[Pos2],
    y_base: f32,
    color: Color32,
    top_alpha: u8,
) {
    if pts.len() < 2 {
        return;
    }
    let mut mesh = egui::epaint::Mesh::default();
    let top_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), top_alpha);
    let bot_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 0);

    for i in 0..pts.len() - 1 {
        let p0 = pts[i];
        let p1 = pts[i + 1];
        let b0 = Pos2::new(p0.x, y_base);
        let b1 = Pos2::new(p1.x, y_base);

        let idx = mesh.vertices.len() as u32;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: p0,
            uv: Pos2::ZERO,
            color: top_color,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: p1,
            uv: Pos2::ZERO,
            color: top_color,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: b1,
            uv: Pos2::ZERO,
            color: bot_color,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: b0,
            uv: Pos2::ZERO,
            color: bot_color,
        });

        mesh.indices
            .extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
    }
    painter.add(egui::Shape::Mesh(mesh.into()));
}

fn nice_y_max(val: f64) -> f64 {
    if val <= 0.0 {
        return 10.0;
    }
    let rough = val / 4.0;
    let exp = 10.0f64.powf(rough.log10().floor());
    let f = rough / exp;
    let nice_step = if f <= 1.0 {
        1.0 * exp
    } else if f <= 2.0 {
        2.0 * exp
    } else if f <= 2.5 {
        2.5 * exp
    } else if f <= 5.0 {
        5.0 * exp
    } else if f <= 6.0 {
        6.0 * exp
    } else {
        10.0 * exp
    };
    nice_step * 4.0
}

// 1. Hourly Latency Area Chart matching Recharts
fn render_hourly_latency(
    painter: &egui::Painter,
    rect: Rect,
    hourly: &[HourlyBucket],
    hover_pos: Option<Pos2>,
    dark: bool,
) {
    let pad_left = 55.0;
    let pad_bottom = 35.0;
    let pad_top = 20.0;
    let pad_right = 20.0;

    let plot_rect = Rect::from_min_max(
        Pos2::new(rect.min.x + pad_left, rect.min.y + pad_top),
        Pos2::new(rect.max.x - pad_right, rect.max.y - pad_bottom),
    );

    let raw_max = hourly
        .iter()
        .map(|h| h.p99_ms.max(h.p95_ms))
        .fold(0.0f64, f64::max)
        .max(10.0);
    let max_val = nice_y_max(raw_max);

    // Draw horizontal dashed grid lines matching CartesianGrid
    let text_color = TailwindColors::SLATE_500;
    let grid_stroke = Stroke::new(
        1.0,
        if dark {
            Color32::from_rgb(30, 41, 59)
        } else {
            TailwindColors::SLATE_200
        },
    );

    for i in 0..=4 {
        let frac = i as f32 / 4.0;
        let y = plot_rect.max.y - frac * plot_rect.height();
        paint_dashed_h_line(painter, plot_rect.min.x, plot_rect.max.x, y, grid_stroke);

        let val = max_val * frac as f64;
        painter.text(
            Pos2::new(plot_rect.min.x - 6.0, y),
            egui::Align2::RIGHT_CENTER,
            format_ms(val),
            FontId::monospace(10.0),
            text_color,
        );
    }

    if hourly.is_empty() {
        return;
    }

    let n = hourly.len();
    let dx = plot_rect.width() / (n.max(1) as f32);

    let mut p95_pts = Vec::with_capacity(n);
    let mut avg_pts = Vec::with_capacity(n);
    let mut p99_pts = Vec::with_capacity(n);

    for (i, h) in hourly.iter().enumerate() {
        let x = plot_rect.min.x + (i as f32 + 0.5) * dx;
        let y_p95 = plot_rect.max.y
            - ((h.p95_ms / max_val) as f32 * plot_rect.height()).clamp(0.0, plot_rect.height());
        let y_avg = plot_rect.max.y
            - ((h.avg_ms / max_val) as f32 * plot_rect.height()).clamp(0.0, plot_rect.height());
        let y_p99 = plot_rect.max.y
            - ((h.p99_ms / max_val) as f32 * plot_rect.height()).clamp(0.0, plot_rect.height());

        p95_pts.push(Pos2::new(x, y_p95));
        avg_pts.push(Pos2::new(x, y_avg));
        p99_pts.push(Pos2::new(x, y_p99));

        // Draw x-axis label every 3 hours (stopping at 21:00 like Recharts interval={2})
        if i % 3 == 0 && h.hour <= 21 {
            let hour_str = format!("{:02}:00", h.hour);
            painter.text(
                Pos2::new(x, plot_rect.max.y + 12.0),
                egui::Align2::CENTER_CENTER,
                hour_str,
                FontId::monospace(10.0),
                text_color,
            );
        }
    }

    // Colors matching PALETTE
    let p95_color = Color32::from_rgb(37, 99, 235); // #2563eb
    let avg_color = Color32::from_rgb(13, 148, 136); // #0d9488
    let p99_color = Color32::from_rgb(124, 58, 237); // #7c3aed

    // 1. Draw gradient area fills (under curve)
    paint_area_gradient(painter, &p99_pts, plot_rect.max.y, p99_color, 70);
    paint_area_gradient(painter, &p95_pts, plot_rect.max.y, p95_color, 90);
    paint_area_gradient(painter, &avg_pts, plot_rect.max.y, avg_color, 75);

    // 2. Draw line strokes
    for i in 0..p95_pts.len() - 1 {
        painter.line_segment([avg_pts[i], avg_pts[i + 1]], Stroke::new(1.5, avg_color));
        painter.line_segment([p99_pts[i], p99_pts[i + 1]], Stroke::new(1.5, p99_color));
        painter.line_segment([p95_pts[i], p95_pts[i + 1]], Stroke::new(2.0, p95_color));
    }

    // Centered Legend at bottom matching reference screenshot
    let legend_y = rect.max.y - 10.0;
    let legend_w = 270.0;
    let mut lx = rect.center().x - (legend_w / 2.0);

    // Avg Latency
    painter.circle_filled(Pos2::new(lx, legend_y), 4.0, avg_color);
    painter.text(
        Pos2::new(lx + 8.0, legend_y),
        egui::Align2::LEFT_CENTER,
        "Avg Latency",
        FontId::proportional(11.0),
        text_color,
    );
    lx += 95.0;

    // P95 Latency
    painter.circle_filled(Pos2::new(lx, legend_y), 4.0, p95_color);
    painter.text(
        Pos2::new(lx + 8.0, legend_y),
        egui::Align2::LEFT_CENTER,
        "P95 Latency",
        FontId::proportional(11.0),
        text_color,
    );
    lx += 95.0;

    // P99 Latency
    painter.circle_filled(Pos2::new(lx, legend_y), 4.0, p99_color);
    painter.text(
        Pos2::new(lx + 8.0, legend_y),
        egui::Align2::LEFT_CENTER,
        "P99 Latency",
        FontId::proportional(11.0),
        text_color,
    );

    // Hover tooltip with vertical guideline
    if let Some(pos) = hover_pos {
        if plot_rect.contains(pos) {
            let idx = (((pos.x - plot_rect.min.x) / dx) as usize).min(n - 1);
            let h = &hourly[idx];
            let pt = p95_pts[idx];

            painter.line_segment(
                [
                    Pos2::new(pt.x, plot_rect.min.y),
                    Pos2::new(pt.x, plot_rect.max.y),
                ],
                Stroke::new(1.0, TailwindColors::BLUE_400),
            );

            let tip_text = format!(
                "Time: {:02}:00\nAvg: {}\np95: {}\np99: {}\nTotal: {}",
                h.hour,
                format_ms(h.avg_ms),
                format_ms(h.p95_ms),
                format_ms(h.p99_ms),
                format_num(h.count)
            );
            render_tooltip_box(painter, pos, &tip_text, dark);
        }
    }
}

// 2. Daily Trend Bar + Line chart
fn render_daily_trend(
    painter: &egui::Painter,
    rect: Rect,
    daily: &[DaySummary],
    hover_pos: Option<Pos2>,
    dark: bool,
) {
    let pad_left = 55.0;
    let pad_bottom = 35.0;
    let pad_top = 20.0;
    let pad_right = 20.0;

    let plot_rect = Rect::from_min_max(
        Pos2::new(rect.min.x + pad_left, rect.min.y + pad_top),
        Pos2::new(rect.max.x - pad_right, rect.max.y - pad_bottom),
    );

    let max_req = daily.iter().map(|d| d.count).max().unwrap_or(1).max(10) as f32;
    let max_p95 = daily
        .iter()
        .map(|d| d.p95_ms)
        .fold(0.0f64, f64::max)
        .max(10.0);

    let text_color = TailwindColors::SLATE_500;
    let grid_stroke = Stroke::new(
        1.0,
        if dark {
            Color32::from_rgb(30, 41, 59)
        } else {
            Color32::from_rgb(241, 245, 249)
        },
    );

    for i in 0..=4 {
        let frac = i as f32 / 4.0;
        let y = plot_rect.max.y - frac * plot_rect.height();
        painter.line_segment(
            [Pos2::new(plot_rect.min.x, y), Pos2::new(plot_rect.max.x, y)],
            grid_stroke,
        );

        let val = (max_req * frac) as u64;
        painter.text(
            Pos2::new(plot_rect.min.x - 6.0, y),
            egui::Align2::RIGHT_CENTER,
            format_num(val),
            FontId::monospace(10.0),
            text_color,
        );
    }

    if daily.is_empty() {
        return;
    }

    let n = daily.len();
    let slot_w = plot_rect.width() / (n as f32);
    let bar_w = (slot_w * 0.5).clamp(4.0, 36.0);

    let mut p95_pts = Vec::with_capacity(n);

    let bar_color = if dark {
        TailwindColors::BLUE_400
    } else {
        TailwindColors::BLUE_500
    };
    let p95_color = Color32::from_rgb(124, 58, 237);

    for (i, d) in daily.iter().enumerate() {
        let cx = plot_rect.min.x + (i as f32 + 0.5) * slot_w;

        // Bar (requests)
        let bar_h = (d.count as f32 / max_req) * plot_rect.height();
        let bar_rect = Rect::from_min_max(
            Pos2::new(cx - bar_w / 2.0, plot_rect.max.y - bar_h),
            Pos2::new(cx + bar_w / 2.0, plot_rect.max.y),
        );
        painter.rect_filled(bar_rect, CornerRadius::same(2), bar_color);

        // p95 line point
        let y_p95 = plot_rect.max.y
            - ((d.p95_ms / max_p95) as f32 * plot_rect.height()).clamp(0.0, plot_rect.height());
        p95_pts.push(Pos2::new(cx, y_p95));

        // Date label
        painter.text(
            Pos2::new(cx, plot_rect.max.y + 12.0),
            egui::Align2::CENTER_CENTER,
            format_date(Some(&d.date)),
            FontId::monospace(10.0),
            text_color,
        );
    }

    for i in 0..p95_pts.len() - 1 {
        painter.line_segment([p95_pts[i], p95_pts[i + 1]], Stroke::new(2.0, p95_color));
    }
    for pt in &p95_pts {
        painter.circle_filled(*pt, 3.0, p95_color);
    }

    // Legend
    let legend_y = rect.max.y - 10.0;
    let mut lx = plot_rect.min.x + 30.0;
    painter.rect_filled(
        Rect::from_min_size(Pos2::new(lx, legend_y - 4.0), Vec2::new(8.0, 8.0)),
        CornerRadius::same(2),
        bar_color,
    );
    painter.text(
        Pos2::new(lx + 12.0, legend_y),
        egui::Align2::LEFT_CENTER,
        "Requests",
        FontId::proportional(11.0),
        text_color,
    );
    lx += 100.0;
    painter.circle_filled(Pos2::new(lx, legend_y), 4.0, p95_color);
    painter.text(
        Pos2::new(lx + 8.0, legend_y),
        egui::Align2::LEFT_CENTER,
        "p95 Latency",
        FontId::proportional(11.0),
        text_color,
    );

    if let Some(pos) = hover_pos {
        if plot_rect.contains(pos) {
            let idx = (((pos.x - plot_rect.min.x) / slot_w) as usize).min(n - 1);
            let d = &daily[idx];
            let tip = format!(
                "{}\nReqs: {}\np95: {}\nErrors: {}",
                format_date(Some(&d.date)),
                format_num(d.count),
                format_ms(d.p95_ms),
                format_num(d.error_count)
            );
            render_tooltip_box(painter, pos, &tip, dark);
        }
    }
}

// 3. Hourly Throughput Bars + Errors
fn render_throughput(
    painter: &egui::Painter,
    rect: Rect,
    hourly: &[HourlyBucket],
    hover_pos: Option<Pos2>,
    dark: bool,
) {
    let pad_left = 55.0;
    let pad_bottom = 35.0;
    let pad_top = 20.0;
    let pad_right = 20.0;

    let plot_rect = Rect::from_min_max(
        Pos2::new(rect.min.x + pad_left, rect.min.y + pad_top),
        Pos2::new(rect.max.x - pad_right, rect.max.y - pad_bottom),
    );

    let max_req = hourly.iter().map(|h| h.count).max().unwrap_or(1).max(10) as f32;

    let text_color = TailwindColors::SLATE_500;
    let grid_stroke = Stroke::new(
        1.0,
        if dark {
            Color32::from_rgb(30, 41, 59)
        } else {
            Color32::from_rgb(241, 245, 249)
        },
    );

    for i in 0..=4 {
        let frac = i as f32 / 4.0;
        let y = plot_rect.max.y - frac * plot_rect.height();
        painter.line_segment(
            [Pos2::new(plot_rect.min.x, y), Pos2::new(plot_rect.max.x, y)],
            grid_stroke,
        );

        let val = (max_req * frac) as u64;
        painter.text(
            Pos2::new(plot_rect.min.x - 6.0, y),
            egui::Align2::RIGHT_CENTER,
            format_num(val),
            FontId::monospace(10.0),
            text_color,
        );
    }

    if hourly.is_empty() {
        return;
    }

    let n = hourly.len();
    let slot_w = plot_rect.width() / (n as f32);
    let bar_w = (slot_w * 0.65).clamp(3.0, 32.0);

    let req_color = if dark {
        TailwindColors::BLUE_400
    } else {
        TailwindColors::BLUE_500
    };
    let err_color = if dark {
        TailwindColors::ROSE_400
    } else {
        TailwindColors::ROSE_500
    };

    for (i, h) in hourly.iter().enumerate() {
        let cx = plot_rect.min.x + (i as f32 + 0.5) * slot_w;

        let bar_h = (h.count as f32 / max_req) * plot_rect.height();
        let err_h = (h.error_count as f32 / max_req) * plot_rect.height();

        // Requests bar
        let req_rect = Rect::from_min_max(
            Pos2::new(cx - bar_w / 2.0, plot_rect.max.y - bar_h),
            Pos2::new(cx + bar_w / 2.0, plot_rect.max.y),
        );
        painter.rect_filled(req_rect, CornerRadius::same(2), req_color);

        // Errors bar overlay
        if h.error_count > 0 {
            let err_rect = Rect::from_min_max(
                Pos2::new(cx - bar_w / 2.0, plot_rect.max.y - err_h),
                Pos2::new(cx + bar_w / 2.0, plot_rect.max.y),
            );
            painter.rect_filled(err_rect, CornerRadius::same(2), err_color);
        }

        if i % 3 == 0 || i == n - 1 {
            let hour_str = format!("{:02}:00", h.hour);
            painter.text(
                Pos2::new(cx, plot_rect.max.y + 12.0),
                egui::Align2::CENTER_CENTER,
                hour_str,
                FontId::monospace(10.0),
                text_color,
            );
        }
    }

    // Legend
    let legend_y = rect.max.y - 10.0;
    let mut lx = plot_rect.min.x + 30.0;
    painter.rect_filled(
        Rect::from_min_size(Pos2::new(lx, legend_y - 4.0), Vec2::new(8.0, 8.0)),
        CornerRadius::same(2),
        req_color,
    );
    painter.text(
        Pos2::new(lx + 12.0, legend_y),
        egui::Align2::LEFT_CENTER,
        "Requests",
        FontId::proportional(11.0),
        text_color,
    );
    lx += 100.0;
    painter.rect_filled(
        Rect::from_min_size(Pos2::new(lx, legend_y - 4.0), Vec2::new(8.0, 8.0)),
        CornerRadius::same(2),
        err_color,
    );
    painter.text(
        Pos2::new(lx + 12.0, legend_y),
        egui::Align2::LEFT_CENTER,
        "Errors",
        FontId::proportional(11.0),
        text_color,
    );

    if let Some(pos) = hover_pos {
        if plot_rect.contains(pos) {
            let idx = (((pos.x - plot_rect.min.x) / slot_w) as usize).min(n - 1);
            let h = &hourly[idx];
            let tip = format!(
                "{:02}:00\nTotal: {}\nErrors: {}",
                h.hour,
                format_num(h.count),
                format_num(h.error_count)
            );
            render_tooltip_box(painter, pos, &tip, dark);
        }
    }
}

// 4. Latency Distribution buckets (<50ms, 50-100ms, ..., >3s)
fn render_distribution(
    painter: &egui::Painter,
    rect: Rect,
    rows: &[AggregatedEndpoint],
    hover_pos: Option<Pos2>,
    dark: bool,
) {
    let pad_left = 55.0;
    let pad_bottom = 35.0;
    let pad_top = 20.0;
    let pad_right = 20.0;

    let plot_rect = Rect::from_min_max(
        Pos2::new(rect.min.x + pad_left, rect.min.y + pad_top),
        Pos2::new(rect.max.x - pad_right, rect.max.y - pad_bottom),
    );

    // Distribution buckets matching PALETTE.distribution
    let bucket_labels = [
        "<50ms",
        "50-100ms",
        "100-300ms",
        "300-500ms",
        "500ms-1s",
        "1s-3s",
        ">3s",
    ];
    let bucket_colors = [
        Color32::from_rgb(34, 197, 94),  // #22c55e green
        Color32::from_rgb(132, 204, 22), // #84cc16 lime
        Color32::from_rgb(234, 179, 8),  // #eab308 yellow
        Color32::from_rgb(249, 115, 22), // #f97316 orange
        Color32::from_rgb(239, 68, 68),  // #ef4444 red
        Color32::from_rgb(185, 28, 28),  // #b91c1c dark red
        Color32::from_rgb(127, 29, 29),  // #7f1d1d maroon
    ];

    let mut counts = [0u64; 7];

    let classify = |ms: f64| -> usize {
        if ms < 50.0 {
            0
        } else if ms < 100.0 {
            1
        } else if ms < 300.0 {
            2
        } else if ms < 500.0 {
            3
        } else if ms < 1000.0 {
            4
        } else if ms < 3000.0 {
            5
        } else {
            6
        }
    };

    for r in rows {
        let c = r.count;
        if c == 0 {
            continue;
        }
        let c50 = (c as f64 * 0.50).round() as u64;
        let c90 = (c as f64 * 0.40).round() as u64;
        let c95 = (c as f64 * 0.05).round() as u64;
        let c99 = (c as f64 * 0.04).round() as u64;
        let c_max = c.saturating_sub(c50 + c90 + c95 + c99);

        counts[classify(r.p50_ms)] += c50;
        counts[classify((r.p50_ms + r.p90_ms) / 2.0)] += c90;
        counts[classify((r.p90_ms + r.p95_ms) / 2.0)] += c95;
        counts[classify((r.p95_ms + r.p99_ms) / 2.0)] += c99;
        counts[classify(r.max_ms)] += c_max;
    }

    let max_count = *counts.iter().max().unwrap_or(&1).max(&10) as f32;

    let text_color = TailwindColors::SLATE_500;
    let grid_stroke = Stroke::new(
        1.0,
        if dark {
            Color32::from_rgb(30, 41, 59)
        } else {
            Color32::from_rgb(241, 245, 249)
        },
    );

    for i in 0..=4 {
        let frac = i as f32 / 4.0;
        let y = plot_rect.max.y - frac * plot_rect.height();
        painter.line_segment(
            [Pos2::new(plot_rect.min.x, y), Pos2::new(plot_rect.max.x, y)],
            grid_stroke,
        );

        let val = (max_count * frac) as u64;
        painter.text(
            Pos2::new(plot_rect.min.x - 6.0, y),
            egui::Align2::RIGHT_CENTER,
            format_num(val),
            FontId::monospace(10.0),
            text_color,
        );
    }

    let slot_w = plot_rect.width() / 7.0;
    let bar_w = (slot_w * 0.65).clamp(8.0, 44.0);

    for (i, &cnt) in counts.iter().enumerate() {
        let cx = plot_rect.min.x + (i as f32 + 0.5) * slot_w;
        let bar_h = (cnt as f32 / max_count) * plot_rect.height();

        let bar_rect = Rect::from_min_max(
            Pos2::new(cx - bar_w / 2.0, plot_rect.max.y - bar_h),
            Pos2::new(cx + bar_w / 2.0, plot_rect.max.y),
        );
        painter.rect_filled(bar_rect, CornerRadius::same(3), bucket_colors[i]);

        painter.text(
            Pos2::new(cx, plot_rect.max.y + 12.0),
            egui::Align2::CENTER_CENTER,
            bucket_labels[i],
            FontId::proportional(10.0),
            text_color,
        );
    }

    if let Some(pos) = hover_pos {
        if plot_rect.contains(pos) {
            let idx = (((pos.x - plot_rect.min.x) / slot_w) as usize).min(6);
            let tip = format!(
                "{}\nRequests: {}",
                bucket_labels[idx],
                format_num(counts[idx])
            );
            render_tooltip_box(painter, pos, &tip, dark);
        }
    }
}

// 5. Top 20 Slowest Endpoints
fn render_top_slowest(
    painter: &egui::Painter,
    rect: Rect,
    rows: &[AggregatedEndpoint],
    hover_pos: Option<Pos2>,
    dark: bool,
) {
    let pad_left = 140.0;
    let pad_bottom = 20.0;
    let pad_top = 10.0;
    let pad_right = 50.0;

    let plot_rect = Rect::from_min_max(
        Pos2::new(rect.min.x + pad_left, rect.min.y + pad_top),
        Pos2::new(rect.max.x - pad_right, rect.max.y - pad_bottom),
    );

    let top_rows: Vec<_> = rows.iter().take(15).collect();
    if top_rows.is_empty() {
        return;
    }

    let max_p95 = top_rows
        .iter()
        .map(|r| r.p95_ms)
        .fold(0.0f64, f64::max)
        .max(10.0);

    let text_color = if dark {
        TailwindColors::SLATE_300
    } else {
        TailwindColors::SLATE_700
    };
    let row_h = plot_rect.height() / (top_rows.len() as f32);
    let bar_h = (row_h * 0.65).clamp(4.0, 18.0);
    let bar_color = if dark {
        TailwindColors::BLUE_500
    } else {
        TailwindColors::BLUE_600
    };

    for (i, r) in top_rows.iter().enumerate() {
        let cy = plot_rect.min.y + (i as f32 + 0.5) * row_h;
        let bar_len =
            ((r.p95_ms / max_p95) as f32 * plot_rect.width()).clamp(4.0, plot_rect.width());

        // Label
        let label = format!(
            "{} {}",
            r.method.as_str(),
            if r.path.len() > 18 {
                format!("{}…", &r.path[..16])
            } else {
                r.path.clone()
            }
        );
        painter.text(
            Pos2::new(plot_rect.min.x - 8.0, cy),
            egui::Align2::RIGHT_CENTER,
            label,
            FontId::monospace(10.0),
            text_color,
        );

        // Bar
        let bar_rect = Rect::from_min_max(
            Pos2::new(plot_rect.min.x, cy - bar_h / 2.0),
            Pos2::new(plot_rect.min.x + bar_len, cy + bar_h / 2.0),
        );
        painter.rect_filled(bar_rect, CornerRadius::same(2), bar_color);

        // Value text
        painter.text(
            Pos2::new(plot_rect.min.x + bar_len + 6.0, cy),
            egui::Align2::LEFT_CENTER,
            format_ms(r.p95_ms),
            FontId::monospace(10.0),
            text_color,
        );
    }

    if let Some(pos) = hover_pos {
        if plot_rect.contains(pos) {
            let idx = (((pos.y - plot_rect.min.y) / row_h) as usize).min(top_rows.len() - 1);
            let r = &top_rows[idx];
            let tip = format!(
                "{} {}\np95: {}\nAvg: {}\np99: {}\nMax: {}\nReqs: {}",
                r.method.as_str(),
                r.path,
                format_ms(r.p95_ms),
                format_ms(r.avg_ms),
                format_ms(r.p99_ms),
                format_ms(r.max_ms),
                format_num(r.count)
            );
            render_tooltip_box(painter, pos, &tip, dark);
        }
    }
}

fn render_tooltip_box(painter: &egui::Painter, pos: Pos2, text: &str, dark: bool) {
    let font = FontId::monospace(11.0);
    let lines: Vec<&str> = text.lines().collect();
    let line_height = 16.0;
    let height = lines.len() as f32 * line_height + 12.0;

    let mut max_w = 60.0f32;
    for line in &lines {
        max_w = max_w.max(line.len() as f32 * 6.8);
    }
    let width = max_w + 16.0;

    let mut tip_rect = Rect::from_min_size(pos + Vec2::new(12.0, 12.0), Vec2::new(width, height));
    if tip_rect.max.x > painter.clip_rect().max.x {
        tip_rect = Rect::from_min_size(
            pos - Vec2::new(width + 12.0, -12.0),
            Vec2::new(width, height),
        );
    }

    let bg = if dark {
        Color32::from_rgb(15, 23, 42) // #0f172a
    } else {
        Color32::WHITE
    };
    let border = if dark {
        Color32::from_rgb(51, 65, 85) // #334155
    } else {
        Color32::from_rgb(203, 213, 225) // #cbd5e1
    };

    painter.rect_filled(tip_rect, CornerRadius::same(6), bg);
    painter.rect_stroke(
        tip_rect,
        CornerRadius::same(6),
        Stroke::new(1.0, border),
        egui::StrokeKind::Outside,
    );

    let fg = if dark {
        Color32::from_rgb(248, 250, 252)
    } else {
        Color32::from_rgb(15, 23, 42)
    };
    for (i, line) in lines.iter().enumerate() {
        painter.text(
            tip_rect.min + Vec2::new(8.0, 6.0 + i as f32 * line_height),
            egui::Align2::LEFT_TOP,
            *line,
            font.clone(),
            fg,
        );
    }
}
