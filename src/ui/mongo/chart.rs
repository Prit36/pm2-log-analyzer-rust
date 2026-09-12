//! `MongoLatencyChart` — port of `src/components/mongo/MongoLatencyChart.tsx`.
//!
//! Three Recharts modes (ComposedChart for throughput+latency, stacked BarChart
//! for plan mix, vertical BarChart for the slowest collections) rendered through
//! the same SVG emitter the PM2 chart uses, plus a hover tooltip per mode.

use iced::widget::{button, column, container, mouse_area, row, space, stack, svg, text};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::core::mongo_models::{MongoCollectionMetric, MongoTimeBucket};
use crate::store::mongo_store::MongoChartMode;
use crate::ui::charts::{
    AxisKind, Orientation, Rect, Svg, Tick, band_coord, build_numeric_axis, emit_axis, grid_lines,
    linear, nice_ticks, point_coord, x_axis,
};
use crate::ui::style;
use crate::utils::format::{format_ms, format_num};

const CHART_HEIGHT: f64 = 320.0;
/// Recharts margins from the reference (`margin={{ top: 10, right: 20, left: 0, bottom: 4 }}`).
const MARGIN_TOP: f64 = 10.0;
const MARGIN_RIGHT: f64 = 20.0;
const MARGIN_BOTTOM: f64 = 4.0;
/// Recharts default axis sizes.
const X_AXIS_HEIGHT: f64 = 30.0;
const Y_AXIS_WIDTH: f64 = 60.0;
const LEGEND_HEIGHT: f64 = 30.0;

const BAR_GREEN: &str = "#10b981";
const BAR_AMBER: &str = "#f59e0b";
const BAR_PURPLE: &str = "#8b5cf6";
const LINE_BLUE: &str = "#3b82f6";

pub fn view(app: &App) -> Element<'_, Message> {
    let Some(result) = app.mongo.result.as_ref() else {
        return container(text("")).into();
    };
    let time_buckets = &result.time_buckets;
    let collections = &result.collections;

    if time_buckets.is_empty() && collections.is_empty() {
        return container(
            text("No chart data available for the current filters.")
                .size(14)
                .style(style::text_muted),
        )
        .width(Fill)
        .padding(Padding {
            top: 32.0,
            right: 16.0,
            bottom: 32.0,
            left: 16.0,
        })
        .center_x(Fill)
        .style(style::mongo_card)
        .into();
    }

    let header = container(
        row![
            column![
                text("Database Performance Over Time")
                    .size(14)
                    .font(style::SEMIBOLD)
                    .style(style::text_heading),
                text("Latency percentiles, collection scans, and top resource consumers")
                    .size(12)
                    .style(style::text_muted),
            ]
            .spacing(2),
            space().width(Fill),
            mode_switcher(app),
        ]
        .spacing(8)
        .align_y(Center)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 12.0,
            left: 0.0,
        }),
    )
    .width(Fill);

    let plot: Element<'_, Message> = match app.mongo_chart_mode {
        MongoChartMode::ThroughputLatency => composed_chart(app, time_buckets),
        MongoChartMode::Plans => plans_chart(app, time_buckets),
        MongoChartMode::TopCollections => collections_chart(app, collections),
    };

    container(
        column![
            header,
            crate::ui::hrule(style::divider),
            container(plot).padding(Padding {
                top: 8.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            }),
        ]
        .spacing(12),
    )
    .width(Fill)
    .padding(16)
    .style(style::mongo_card)
    .into()
}

fn mode_switcher(app: &App) -> Element<'static, Message> {
    let mut row_items = row![].spacing(4).align_y(Center);
    for mode in MongoChartMode::ALL {
        let active = app.mongo_chart_mode == mode;
        row_items = row_items.push(
            button(
                text(mode.label())
                    .size(12)
                    .font(style::MEDIUM),
            )
            .on_press(Message::MongoChartMode(mode))
            .padding([4, 10])
            .style(move |theme: &iced::Theme, status| {
                style::mongo_chart_tab(theme, mode, active, status)
            }),
        );
    }
    container(row_items)
        .padding(4)
        .style(style::segmented)
        .into()
}

// ── Mode 1: queries (bars) + p95/avg latency (lines) ───────────────────────

fn composed_chart<'a>(app: &'a App, buckets: &[MongoTimeBucket]) -> Element<'a, Message> {
    let is_dark = app.analysis.is_dark();
    let width = chart_width(app);
    let height = CHART_HEIGHT;
    let left_ticks = nice_ticks(0.0, bucket_max(buckets, |b| b.query_count as f64), 5, false);
    let right_ticks = nice_ticks(0.0, bucket_max(buckets, |b| b.p95_duration_ms as f64), 5, true);
    let left_domain = left_ticks.last().copied().unwrap_or(1.0);
    let right_domain = right_ticks.last().copied().unwrap_or(1.0);

    let plot = chart_plot(width, height, Y_AXIS_WIDTH, Y_AXIS_WIDTH, X_AXIS_HEIGHT + LEGEND_HEIGHT);
    let mut canvas = Svg::new(width, height);
    grid_lines(
        &mut canvas,
        plot,
        &build_numeric_axis(
            &left_ticks,
            AxisKind::NumberCount,
            Orientation::Left,
            plot,
            plot.x - 8.0,
            plot.x,
            tick_color(is_dark),
        )
        .ticks,
        grid_color(is_dark),
        false,
    );
    emit_axis(
        &mut canvas,
        &x_axis(
            &category_ticks(buckets, plot, |b| b.hour_label.clone()),
            plot.x,
            plot.y + plot.h,
            plot.w,
            plot.x + plot.w,
            tick_color(is_dark),
        ),
        plot,
    );
    emit_axis(
        &mut canvas,
        &build_numeric_axis(
            &left_ticks,
            AxisKind::NumberCount,
            Orientation::Left,
            plot,
            plot.x - 8.0,
            plot.x,
            tick_color(is_dark),
        ),
        plot,
    );
    emit_axis(
        &mut canvas,
        &right_axis_with_unit(&right_ticks, plot, tick_color(is_dark)),
        plot,
    );

    // Bars: queryCount, opacity .65, maxBarSize 40, top radius 3.
    let band = plot.w / buckets.len().max(1) as f64;
    let bar_width = (band * 0.6).min(40.0);
    canvas.raw("<g class=\"recharts-layer recharts-bar\">");
    for (index, bucket) in buckets.iter().enumerate() {
        let x = band_coord(index, buckets.len(), plot.x, plot.w) - bar_width / 2.0;
        let value = bucket.query_count as f64;
        let y = linear(value, 0.0, left_domain, plot.y + plot.h, plot.y);
        canvas.bar(
            x,
            y,
            bar_width,
            plot.y + plot.h - y,
            3.0,
            BAR_GREEN,
            0.65,
            "Slow Queries",
            value,
        );
    }
    canvas.raw("</g>");

    // Lines on the right axis (ms).
    canvas.raw("<g class=\"recharts-layer recharts-line\">");
    let p95_points: Vec<(f64, f64)> = buckets
        .iter()
        .enumerate()
        .map(|(index, bucket)| {
            (
                band_coord(index, buckets.len(), plot.x, plot.w),
                linear(bucket.p95_duration_ms as f64, 0.0, right_domain, plot.y + plot.h, plot.y),
            )
        })
        .collect();
    canvas.curve(&p95_points, LINE_BLUE, 2.0, "P95 Latency");
    canvas.raw("</g>");
    canvas.raw("<g class=\"recharts-layer recharts-line\">");
    let avg_points: Vec<(f64, f64)> = buckets
        .iter()
        .enumerate()
        .map(|(index, bucket)| {
            (
                band_coord(index, buckets.len(), plot.x, plot.w),
                linear(bucket.avg_duration_ms, 0.0, right_domain, plot.y + plot.h, plot.y),
            )
        })
        .collect();
    canvas.curve(&avg_points, BAR_AMBER, 1.5, "Avg Latency");
    canvas.raw("</g>");

    let document = canvas.finish();
    let slots: Vec<TooltipSlot> = buckets
        .iter()
        .map(|bucket| TooltipSlot {
            label: bucket.hour_label.clone(),
            rows: vec![
                ("Slow Queries", format_num(bucket.query_count), BAR_GREEN),
                ("P95 Latency", format_ms(bucket.p95_duration_ms as f64), LINE_BLUE),
                ("Avg Latency", format_ms(bucket.avg_duration_ms), BAR_AMBER),
            ],
        })
        .collect();
    chart_host(app, document, width, plot, slots, true)
}

// ── Mode 2: stacked COLLSCAN vs IXSCAN bars ────────────────────────────────

fn plans_chart<'a>(app: &'a App, buckets: &[MongoTimeBucket]) -> Element<'a, Message> {
    let is_dark = app.analysis.is_dark();
    let width = chart_width(app);
    let height = CHART_HEIGHT;
    let max_total = buckets
        .iter()
        .map(|bucket| bucket.query_count as f64)
        .fold(0.0_f64, f64::max);
    let ticks = nice_ticks(0.0, max_total, 5, false);
    let domain = ticks.last().copied().unwrap_or(1.0);
    let plot = chart_plot(width, height, Y_AXIS_WIDTH, 0.0, X_AXIS_HEIGHT + LEGEND_HEIGHT);

    let mut canvas = Svg::new(width, height);
    let axis = build_numeric_axis(
        &ticks,
        AxisKind::NumberCount,
        Orientation::Left,
        plot,
        plot.x - 8.0,
        plot.x,
        tick_color(is_dark),
    );
    grid_lines(&mut canvas, plot, &axis.ticks, grid_color(is_dark), false);
    emit_axis(
        &mut canvas,
        &x_axis(
            &category_ticks(buckets, plot, |b| b.hour_label.clone()),
            plot.x,
            plot.y + plot.h,
            plot.w,
            plot.x + plot.w,
            tick_color(is_dark),
        ),
        plot,
    );
    emit_axis(&mut canvas, &axis, plot);

    let band = plot.w / buckets.len().max(1) as f64;
    let bar_width = (band * 0.6).min(40.0);
    canvas.raw("<g class=\"recharts-layer recharts-bar\">");
    for (index, bucket) in buckets.iter().enumerate() {
        let x = band_coord(index, buckets.len(), plot.x, plot.w) - bar_width / 2.0;
        let collscan = bucket.collscan_count as f64;
        let ixscan = (bucket.query_count as f64 - collscan).max(0.0);
        let base = plot.y + plot.h;
        let collscan_top = linear(collscan, 0.0, domain, base, plot.y);
        canvas.bar(
            x,
            collscan_top,
            bar_width,
            base - collscan_top,
            0.0,
            BAR_AMBER,
            1.0,
            "COLLSCAN (Table Scan)",
            collscan,
        );
        let total_top = linear(collscan + ixscan, 0.0, domain, base, plot.y);
        canvas.bar(
            x,
            total_top,
            bar_width,
            collscan_top - total_top,
            3.0,
            BAR_GREEN,
            1.0,
            "IXSCAN (Indexed)",
            ixscan,
        );
    }
    canvas.raw("</g>");

    let slots: Vec<TooltipSlot> = buckets
        .iter()
        .map(|bucket| TooltipSlot {
            label: bucket.hour_label.clone(),
            rows: vec![
                ("COLLSCAN (Table Scan)", format_num(bucket.collscan_count), BAR_AMBER),
                (
                    "IXSCAN (Indexed)",
                    format_num(bucket.query_count.saturating_sub(bucket.collscan_count)),
                    BAR_GREEN,
                ),
            ],
        })
        .collect();
    chart_host(app, canvas.finish(), width, plot, slots, true)
}

// ── Mode 3: slowest collections (horizontal bars) ──────────────────────────

fn collections_chart<'a>(app: &'a App, collections: &[MongoCollectionMetric]) -> Element<'a, Message> {
    let is_dark = app.analysis.is_dark();
    let width = chart_width(app);
    let height = CHART_HEIGHT;
    let data: Vec<(&MongoCollectionMetric, f64)> = collections
        .iter()
        .take(10)
        .map(|collection| {
            let seconds = (collection.total_duration_ms as f64 / 1000.0 * 10.0).round() / 10.0;
            (collection, seconds)
        })
        .collect();

    // `layout="vertical"` swaps the axes: category on Y (width 120), number on X.
    let plot = chart_plot(width, height, 120.0, 0.0, X_AXIS_HEIGHT + LEGEND_HEIGHT);
    let max_value = data.iter().map(|(_, value)| *value).fold(0.0_f64, f64::max);
    let ticks = nice_ticks(0.0, max_value, 5, true);
    let domain = ticks.last().copied().unwrap_or(1.0);

    let mut canvas = Svg::new(width, height);
    let axis = numeric_axis_with_unit(
        &ticks,
        AxisKind::NumberPlain,
        Orientation::Bottom,
        plot,
        plot.y + plot.h + 16.0,
        plot.y + plot.h,
        "s",
        tick_color(is_dark),
    );
    grid_lines(&mut canvas, plot, &axis.ticks, grid_color(is_dark), true);
    emit_axis(&mut canvas, &axis, plot);

    let band = plot.h / data.len().max(1) as f64;
    let bar_height = (band * 0.6).min(40.0);
    canvas.raw("<g class=\"recharts-layer recharts-bar\">");
    for (index, (collection, seconds)) in data.iter().enumerate() {
        let y = plot.y + band * index as f64 + (band - bar_height) / 2.0;
        let w = linear(*seconds, 0.0, domain, 0.0, plot.w);
        canvas.bar(
            plot.x,
            y,
            w,
            bar_height,
            0.0,
            BAR_PURPLE,
            1.0,
            "Total DB Time (Seconds)",
            *seconds,
        );
        canvas.tick_text(
            plot.x - 8.0,
            y + bar_height / 2.0 + 4.0,
            "end",
            tick_color(is_dark),
            &text_clip(&collection.collection, 16),
        );
    }
    canvas.raw("</g>");

    let slots: Vec<TooltipSlot> = data
        .iter()
        .map(|(collection, seconds)| TooltipSlot {
            label: collection.collection.clone(),
            rows: vec![(
                "Total Database Time",
                format!("{seconds} seconds"),
                BAR_PURPLE,
            )],
        })
        .collect();
    chart_host(app, canvas.finish(), width, plot, slots, false)
}

// ── Shared host: SVG + hover hit zones + tooltip card ──────────────────────

struct TooltipSlot {
    label: String,
    rows: Vec<(&'static str, String, &'static str)>,
}

fn chart_host(
    app: &App,
    document: String,
    width: f64,
    plot: Rect,
    slots: Vec<TooltipSlot>,
    horizontal_bands: bool,
) -> Element<'_, Message> {
    let handle = svg::Handle::from_memory(document.into_bytes());
    let canvas: Element<'_, Message> = svg(handle)
        .width(Length::Fixed(width as f32))
        .height(Length::Fixed(CHART_HEIGHT as f32))
        .content_fit(iced::ContentFit::Fill)
        .into();

    if slots.is_empty() {
        return canvas;
    }

    let hovered = app.mongo_chart_hover;
    let count = slots.len();
    let mut zones = column![].spacing(0).width(Length::Fixed(plot.w as f32));
    if horizontal_bands {
        // One band per category column.
        let band = plot.w / count as f64;
        let mut band_row = row![].spacing(0);
        for index in 0..count {
            band_row = band_row.push(
                mouse_area(
                    container(space())
                        .width(Length::Fixed(band as f32))
                        .height(Length::Fixed(plot.h as f32)),
                )
                .on_enter(Message::MongoChartHover(Some(index)))
                .on_exit(Message::MongoChartHover(None)),
            );
        }
        zones = zones.push(band_row);
    } else {
        let band = plot.h / count as f64;
        for index in 0..count {
            zones = zones.push(
                mouse_area(
                    container(space())
                        .width(Length::Fixed(plot.w as f32))
                        .height(Length::Fixed(band as f32)),
                )
                .on_enter(Message::MongoChartHover(Some(index)))
                .on_exit(Message::MongoChartHover(None)),
            );
        }
    }

    let positioned = container(zones).padding(Padding {
        top: plot.y as f32,
        right: 0.0,
        bottom: 0.0,
        left: plot.x as f32,
    });

    match hovered.and_then(|index| slots.get(index).map(tooltip_owned)) {
        Some(slot) => column![
            stack![canvas, stack![positioned, tooltip_card(slot)]],
            legend(app)
        ]
        .spacing(0)
        .into(),
        None => column![stack![canvas, positioned], legend(app)].spacing(0).into(),
    }
}

fn tooltip_owned(slot: &TooltipSlot) -> TooltipSlot {
    TooltipSlot {
        label: slot.label.clone(),
        rows: slot
            .rows
            .iter()
            .map(|(name, value, color)| (*name, value.clone(), *color))
            .collect(),
    }
}

fn tooltip_card(slot: TooltipSlot) -> Element<'static, Message> {
    let mut rows = column![text(slot.label)
        .size(12)
        .font(style::REGULAR)
        .style(style::text_heading)]
    .spacing(3);

    for (name, value, color) in slot.rows {
        rows = rows.push(
            row![
                container(space().width(7.0).height(7.0))
                    .width(Length::Fixed(7.0))
                    .height(Length::Fixed(7.0))
                    .style(style::dot(style::from_hex(color))),
                text(name).size(12).style(style::text_body),
                space().width(Length::Fixed(12.0)),
                text(value)
                    .size(12)
                    .font(style::SEMIBOLD)
                    .style(style::text_heading),
            ]
            .spacing(4)
            .align_y(Center),
        );
    }

    container(rows)
        .padding(10)
        .style(style::chart_tooltip)
        .into()
}

// ── Geometry helpers ───────────────────────────────────────────────────────

fn chart_width(app: &App) -> f64 {
    let window = f64::from(app.window_width).min(1280.0);
    (window - 32.0 - 32.0).max(320.0)
}

/// Plot rect for the given axis sizes (Recharts reserves the axis + legend).
fn chart_plot(width: f64, height: f64, y_left: f64, y_right: f64, bottom: f64) -> Rect {
    Rect {
        x: y_left,
        y: MARGIN_TOP,
        w: (width - y_left - y_right - MARGIN_RIGHT).max(10.0),
        h: (height - MARGIN_TOP - MARGIN_BOTTOM - bottom).max(10.0),
    }
}

fn category_ticks(
    buckets: &[MongoTimeBucket],
    plot: Rect,
    label: impl Fn(&MongoTimeBucket) -> String,
) -> Vec<Tick> {
    let count = buckets.len();
    buckets
        .iter()
        .enumerate()
        .map(|(index, bucket)| Tick {
            value: label(bucket),
            coord: point_coord(index, count, plot.x, plot.w),
            show: true,
        })
        .collect()
}

fn bucket_max(buckets: &[MongoTimeBucket], get: impl Fn(&MongoTimeBucket) -> f64) -> f64 {
    buckets.iter().map(|bucket| get(bucket)).fold(0.0_f64, f64::max)
}

/// Right-hand latency axis: `tickFormatter={(v) => `${v}ms`}`.
fn right_axis_with_unit(ticks: &[f64], plot: Rect, color: &'static str) -> crate::ui::charts::Axis {
    numeric_axis_with_unit(
        ticks,
        AxisKind::NumberPlain,
        Orientation::Right,
        plot,
        plot.x + plot.w + 8.0,
        plot.x + plot.w,
        "ms",
        color,
    )
}

/// Numeric axis whose tick labels carry a unit suffix (`85s`, `8000ms`).
fn numeric_axis_with_unit(
    ticks: &[f64],
    kind: AxisKind,
    orientation: Orientation,
    plot: Rect,
    label_offset: f64,
    axis_line: f64,
    unit: &str,
    color: &'static str,
) -> crate::ui::charts::Axis {
    let mut axis =
        build_numeric_axis(ticks, kind, orientation, plot, label_offset, axis_line, color);
    for tick in &mut axis.ticks {
        tick.value.push_str(unit);
    }
    axis
}

fn grid_color(is_dark: bool) -> &'static str {
    if is_dark {
        "#1e293b"
    } else {
        "#f1f5f9"
    }
}

fn tick_color(is_dark: bool) -> &'static str {
    if is_dark {
        "#94a3b8"
    } else {
        "#64748b"
    }
}

/// `maxBarSize` + ellipsis clip for the collection labels.
fn text_clip(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut clipped: String = value.chars().take(max_chars.saturating_sub(1)).collect();
    clipped.push('…');
    clipped
}

/// Legend row shared by the bar/line modes (`Legend` with 11px labels).
pub fn legend(app: &App) -> Element<'_, Message> {
    let entries: Vec<(&str, &str)> = match app.mongo_chart_mode {
        MongoChartMode::ThroughputLatency => vec![
            ("Slow Queries", BAR_GREEN),
            ("P95 Latency", LINE_BLUE),
            ("Avg Latency", BAR_AMBER),
        ],
        MongoChartMode::Plans => vec![
            ("COLLSCAN (Table Scan)", BAR_AMBER),
            ("IXSCAN (Indexed)", BAR_GREEN),
        ],
        MongoChartMode::TopCollections => vec![("Total DB Time (Seconds)", BAR_PURPLE)],
    };

    let mut items = row![].spacing(12).align_y(Center);
    for (label, color) in entries {
        items = items.push(
            row![
                container(space().width(8.0).height(8.0))
                    .width(Length::Fixed(8.0))
                    .height(Length::Fixed(8.0))
                    .style(style::dot(style::from_hex(color))),
                text(label).size(11).style(style::text_muted),
            ]
            .spacing(4)
            .align_y(Center),
        );
    }
    container(items)
        .width(Fill)
        .center_x(Fill)
        .padding(Padding {
            top: 10.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .into()
}