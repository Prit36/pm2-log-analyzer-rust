//! `LatencyChart` — port of `src/components/LatencyChart.tsx`.
//!
//! Recharts is reproduced from the observed reference output: the same plot
//! rects, tick algorithm (`getNiceTickValues` adaptive), monotone cubic curves,
//! bar sizing and legend markup.

use dioxus::prelude::*;

use crate::core::models::{AggregatedEndpoint, DaySummary, HourlyBucket};
use crate::store::{toggle_chart_layout, use_analysis_store, ChartLayout};
use crate::ui::icons::Icon;
use crate::utils::cn::cn;
use crate::utils::format::{format_date, format_ms, format_num_i64};

// Palette (src/utils/palette.ts)
const GRID_LIGHT: &str = "#f1f5f9";
const GRID_DARK: &str = "#1e293b";
const TICK_LIGHT: &str = "#64748b";
const TICK_DARK: &str = "#94a3b8";
const CATEGORY_TICK_LIGHT: &str = "#334155";
const CATEGORY_TICK_DARK: &str = "#cbd5e1";
const P99: &str = "#7c3aed";
const P95: &str = "#2563eb";
const AVG: &str = "#0d9488";
const BAR_BLUE: &str = "#3b82f6";
const ERROR_RED: &str = "#ef4444";

const Y_AXIS_WIDTH: f64 = 60.0;
const X_AXIS_HEIGHT: f64 = 30.0;
const LEGEND_HEIGHT: f64 = 20.5;
const FONT_SIZE: f64 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ChartMode {
    DailyTrend,
    TimeOfDay,
    Throughput,
    Distribution,
    TopP95,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AxisKind {
    /// `type="number"` with the reference `formatMs` tick formatter.
    NumberMs,
    /// `type="number"` with `formatNum`.
    NumberCount,
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

struct Tick {
    value: String,
    coord: f64,
    show: bool,
}

struct Axis {
    orientation: Orientation,
    len: f64,
    ticks: Vec<Tick>,
    axis_line: f64,
    label_offset: f64,
    font_family: Option<&'static str>,
    font_size: f64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Orientation {
    Bottom,
    Left,
    Right,
}

#[component]
pub fn LatencyChart(rows: Vec<AggregatedEndpoint>) -> Element {
    let store = use_analysis_store();
    let result = store.result();
    let hourly: Vec<HourlyBucket> = result
        .as_ref()
        .map(|r| r.hourly_stats.clone())
        .unwrap_or_default();
    let daily: Vec<DaySummary> = result
        .as_ref()
        .map(|r| r.daily_stats.clone())
        .unwrap_or_default();
    let is_dark = store.is_dark();
    let date_filter = store.filters().date_filter;
    let layout = store.chart_layout();
    let mut mode = use_signal(|| ChartMode::TimeOfDay);

    let has_data = !rows.is_empty() || hourly.iter().any(|h| h.count > 0);

    // Chart host geometry (mirrors the ResponsiveContainer inside the layout).
    let window_w = store.window_width();
    let content_w = window_w.min(1280.0) - 32.0;
    let is_lg = window_w >= 1024.0;
    let section_w = if layout == ChartLayout::Wide || !is_lg {
        content_w
    } else {
        (content_w - 64.0) * 2.0 / 5.0 + 16.0
    };
    // Recharts' ResponsiveContainer rounds the measured size to whole pixels.
    let svg_w = (section_w - 2.0 - 24.0).max(50.0).round();
    let chart_h: f64 = if layout == ChartLayout::Wide { 400.0 } else { 340.0 };
    let svg_h = (chart_h - 24.0).max(50.0).round();

    let grid_color = if is_dark { GRID_DARK } else { GRID_LIGHT };
    let tick_color = if is_dark { TICK_DARK } else { TICK_LIGHT };
    let category_tick = if is_dark {
        CATEGORY_TICK_DARK
    } else {
        CATEGORY_TICK_LIGHT
    };
    let bar_color = if is_dark { "#60a5fa" } else { BAR_BLUE };
    let error_color = if is_dark { "#f87171" } else { ERROR_RED };

    let multi_day = daily.len() > 1;
    let svg = if has_data {
        Some(render_chart(
            mode(),
            svg_w,
            svg_h,
            &hourly,
            &daily,
            &rows,
            grid_color,
            tick_color,
            category_tick,
            bar_color,
            error_color,
        ))
    } else {
        None
    };
    let legend_width = svg_w - 16.0;
    let legend = if has_data {
        match mode() {
            ChartMode::DailyTrend => Some(legend_html(
                &[
                    ("Avg Latency", AVG),
                    ("Errors", error_color),
                    ("P95 Latency", P99),
                    ("Requests", bar_color),
                ],
                legend_width,
                tick_color,
            )),
            ChartMode::TimeOfDay => Some(legend_html(
                &[("Avg Latency", AVG), ("P95 Latency", P95), ("P99 Latency", P99)],
                legend_width,
                tick_color,
            )),
            ChartMode::Throughput => Some(legend_html(
                &[("Errors (4xx/5xx)", error_color), ("Total Requests", bar_color)],
                legend_width,
                tick_color,
            )),
            _ => None,
        }
    } else {
        None
    };

    rsx! {
        section { class: "flex flex-col rounded border border-slate-200 bg-white shadow-xs dark:border-slate-800/80 dark:bg-slate-900/80 dark:shadow-md dark:shadow-black/20",
            div { class: "flex flex-col gap-2 border-b border-slate-200 px-3.5 py-2.5 dark:border-slate-800",
                div { class: "flex items-center justify-between gap-2",
                    div { class: "flex items-center gap-2",
                        h2 { class: "text-xs font-semibold uppercase tracking-wide text-slate-700 dark:text-slate-300",
                            "API Visual Analytics"
                        }
                        if date_filter != "all" {
                            span { class: "inline-flex items-center rounded-full bg-blue-50 px-2 py-0.5 text-[10px] font-semibold text-blue-700 dark:bg-blue-950/60 dark:text-blue-300",
                                "{format_date(Some(&date_filter))}"
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| toggle_chart_layout(store),
                        class: "inline-flex items-center gap-1 rounded border border-slate-200 bg-slate-50 px-2 py-0.5 text-[11px] font-medium text-slate-600 hover:bg-slate-100 dark:border-slate-800 dark:bg-slate-950 dark:text-slate-300 dark:hover:bg-slate-800 cursor-pointer",
                        title: if layout == ChartLayout::Wide { "Switch to Split Side-by-Side View" } else { "Expand Chart to Full Width View" },
                        if layout == ChartLayout::Wide {
                            Icon { name: "columns2", class: "h-3 w-3" }
                            span { "Split View" }
                        } else {
                            Icon { name: "rows", class: "h-3 w-3" }
                            span { "Wide View" }
                        }
                    }
                }
                div { class: "flex w-full items-center gap-1 rounded bg-slate-100 p-0.5 text-xs dark:bg-slate-950",
                    if multi_day {
                        ChartTab {
                            active: mode() == ChartMode::DailyTrend,
                            icon: "calendar-days",
                            label: "Trend",
                            title: "Daily Trend (Requests, Latency, Errors across all days)",
                            onclick: move |_| mode.set(ChartMode::DailyTrend),
                        }
                    }
                    ChartTab {
                        active: mode() == ChartMode::TimeOfDay,
                        icon: "clock",
                        label: if multi_day { "Latency" } else { "Time vs Latency" },
                        title: "Time of Day vs Latency Trend",
                        onclick: move |_| mode.set(ChartMode::TimeOfDay),
                    }
                    ChartTab {
                        active: mode() == ChartMode::Throughput,
                        icon: "bar-chart3",
                        label: if multi_day { "Volume" } else { "Hourly Volume" },
                        title: "Hourly Request Volume & Error Rate",
                        onclick: move |_| mode.set(ChartMode::Throughput),
                    }
                    ChartTab {
                        active: mode() == ChartMode::Distribution,
                        icon: "activity",
                        label: if multi_day { "Dist" } else { "Distribution" },
                        title: "Latency Distribution Buckets",
                        onclick: move |_| mode.set(ChartMode::Distribution),
                    }
                    ChartTab {
                        active: mode() == ChartMode::TopP95,
                        icon: "flame",
                        label: if multi_day { "Slowest" } else { "Top Slowest" },
                        title: "Top p95 Slowest Endpoints",
                        onclick: move |_| mode.set(ChartMode::TopP95),
                    }
                }
            }

            if let Some(svg) = svg {
                div {
                    class: "px-3 py-3 w-full",
                    style: "height: {chart_h}px;",
                    div { class: "recharts-responsive-container", style: "width: 100%; height: 100%; min-width: 0px;",
                        div { style: "width: 0px; height: 0px; overflow: visible;",
                            div {
                                width: "{svg_w}",
                                height: "{svg_h}",
                                class: "recharts-wrapper",
                                style: "position: relative; cursor: default; width: {svg_w}px; height: {svg_h}px;",
                                // Recharts renders the hidden tooltip and the legend
                                // before the chart SVG inside `.recharts-wrapper`.
                                div {
                                    tabindex: "-1",
                                    class: "recharts-tooltip-wrapper",
                                    style: "visibility: hidden; pointer-events: none; position: absolute; top: 0px; left: 0px;",
                                }
                                if let Some(legend) = legend {
                                    div { dangerous_inner_html: legend }
                                }
                                div { dangerous_inner_html: svg }
                            }
                        }
                    }
                }
            } else {
                div { class: "px-3 py-12 text-center text-sm text-slate-400 dark:text-slate-500",
                    "No chart data available yet."
                }
            }
        }
    }
}

#[component]
fn ChartTab(
    active: bool,
    icon: String,
    label: String,
    title: String,
    onclick: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| onclick.call(()),
            class: cn(&[
                "flex flex-1 items-center justify-center gap-1.5 whitespace-nowrap rounded px-2 py-1 font-medium transition-colors cursor-pointer border-0",
                if active {
                    "bg-white text-blue-600 shadow-xs dark:bg-blue-600 dark:text-white dark:shadow-md dark:shadow-blue-950/50"
                } else {
                    "text-slate-600 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200"
                },
            ]),
            title: "{title}",
            Icon { name: icon, class: "h-3.5 w-3.5" }
            span { "{label}" }
        }
    }
}

// ── Legend ──────────────────────────────────────────────────────────────────

fn legend_html(items: &[(&str, &str)], width: f64, color: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "<div class=\"recharts-legend-wrapper\" style=\"position: absolute; width: {width}px; height: auto; left: 0px; bottom: 4px; font-size: 11px; padding-top: 4px; color: {color};\">"
    ));
    out.push_str("<ul class=\"recharts-default-legend\" style=\"padding: 0px; margin: 0px; text-align: center;\">");
    for (i, (name, item_color)) in items.iter().enumerate() {
        out.push_str(&format!(
            "<li class=\"recharts-legend-item legend-item-{i}\" style=\"display: inline-block; margin-right: 10px; white-space: nowrap;\">"
        ));
        out.push_str(&format!(
            "<svg aria-label=\"{name} legend icon\" class=\"recharts-surface\" width=\"14\" height=\"14\" viewBox=\"0 0 32 32\" style=\"display: inline-block; vertical-align: middle; margin-right: 4px;\">"
        ));
        out.push_str("<title></title><desc></desc>");
        out.push_str(&format!(
            "<path fill=\"{item_color}\" cx=\"16\" cy=\"16\" class=\"recharts-symbols\" transform=\"translate(16, 16)\" d=\"M16,0A16,16,0,1,1,-16,0A16,16,0,1,1,16,0\"></path>"
        ));
        out.push_str("</svg>");
        out.push_str(&format!(
            "<span class=\"recharts-legend-item-text\" style=\"color: {item_color}; white-space: normal; overflow-wrap: break-word;\">{name}</span>"
        ));
        out.push_str("</li>");
    }
    out.push_str("</ul></div>");
    out
}

// ── Recharts math ───────────────────────────────────────────────────────────

fn get_digit_count(value: f64) -> i32 {
    if value == 0.0 {
        1
    } else {
        value.abs().log10().floor() as i32 + 1
    }
}

fn adaptive_step(rough: f64, allow_decimals: bool, correction: i32) -> f64 {
    if rough <= 0.0 {
        return 0.0;
    }
    let digit_count = get_digit_count(rough);
    let digit_count_value = 10f64.powi(digit_count);
    let step_ratio = rough / digit_count_value;
    let step_ratio_scale = if digit_count != 1 { 0.05 } else { 0.1 };
    let amend_step_ratio = (step_ratio / step_ratio_scale).ceil() + correction as f64;
    let format_step = amend_step_ratio * step_ratio_scale * digit_count_value;
    if allow_decimals {
        format_step
    } else {
        format_step.ceil()
    }
}

fn calculate_step(
    min: f64,
    max: f64,
    tick_count: usize,
    allow_decimals: bool,
    correction: i32,
) -> (f64, f64, f64) {
    let step = adaptive_step((max - min) / (tick_count as f64 - 1.0), allow_decimals, correction);
    let middle = if min <= 0.0 && max >= 0.0 {
        0.0
    } else {
        let m = (min + max) / 2.0;
        m - (m % step)
    };
    let below_count = ((middle - min) / step).ceil();
    let mut up_count = ((max - middle) / step).ceil();
    let scale_count = below_count + up_count + 1.0;
    if scale_count > tick_count as f64 {
        return calculate_step(min, max, tick_count, allow_decimals, correction + 1);
    }
    if scale_count < tick_count as f64 {
        if max > 0.0 {
            up_count += tick_count as f64 - scale_count;
        } else {
            // below_count adjustment handled by returning tick_min below
        }
    }
    let below_count = if scale_count < tick_count as f64 && max <= 0.0 {
        below_count + (tick_count as f64 - scale_count)
    } else {
        below_count
    };
    (
        step,
        middle - below_count * step,
        middle + up_count * step,
    )
}

fn nice_ticks(min: f64, max: f64, tick_count: usize, allow_decimals: bool) -> Vec<f64> {
    let count = tick_count.max(2);
    let (cormin, cormax) = if min > max { (max, min) } else { (min, max) };
    if cormin == cormax {
        // getTickOfSingleValue
        let mut middle = cormin;
        if cormin == 0.0 {
            middle = ((tick_count - 1) / 2) as f64;
        } else if !allow_decimals {
            middle = cormin.floor();
        }
        let middle_index = (tick_count - 1) / 2;
        return (0..tick_count)
            .map(|i| middle + (i as f64 - middle_index as f64))
            .collect();
    }
    let (step, tick_min, tick_max) = calculate_step(cormin, cormax, count, allow_decimals, 0);
    let end = tick_max + 0.1 * step;
    let mut values = Vec::new();
    let mut num = tick_min;
    let mut i = 0;
    while num < end && i < 100_000 {
        values.push(num);
        num += step;
        i += 1;
    }
    if min > max {
        values.reverse();
    }
    values
}

fn format_tick(value: f64, kind: AxisKind) -> String {
    match kind {
        AxisKind::NumberMs => format_ms(value),
        AxisKind::NumberCount => {
            if !value.is_finite() {
                "-".to_string()
            } else if value.fract() == 0.0 && value.abs() < 1e15 {
                format_num_i64(value as i64)
            } else {
                format_num_i64(value.round() as i64)
            }
        }
    }
}

/// `getTicksEnd` + `isVisible` from the reference Recharts build.
fn select_ticks_end(ticks: &mut [Tick], sign: f64, start: f64, end: f64, size: impl Fn(usize) -> f64, min_gap: f64) {
    let len = ticks.len();
    let mut end = end;
    for i in (0..len).rev() {
        let coord = ticks[i].coord;
        if i == len - 1 {
            let gap = sign * (coord + sign * size(i) / 2.0 - end);
            if gap > 0.0 {
                ticks[i].coord = coord - gap * sign;
            }
        }
        let tick_coord = ticks[i].coord;
        let s = size(i);
        // isVisible
        let visible = if sign * tick_coord < sign * start || sign * tick_coord > sign * end {
            false
        } else {
            sign * (tick_coord - sign * s / 2.0 - start) >= 0.0
                && sign * (tick_coord + sign * s / 2.0 - end) <= 0.0
        };
        if visible {
            end = tick_coord - sign * (s / 2.0 + min_gap);
            ticks[i].show = true;
        }
    }
}

fn text_width(text: &str, font_size: f64, mono: bool) -> f64 {
    // Widths measured from the reference app's DOM (IBM Plex Sans/Mono).
    if mono {
        return text.chars().count() as f64 * font_size * 0.6;
    }
    match text {
        "<50ms" => 32.0,
        "50-100ms" => 48.0,
        "100-300ms" => 54.0,
        "300-500ms" => 54.0,
        "500ms-1s" => 46.0,
        "1s-3s" => 26.0,
        ">3s" => 17.0,
        _ => text.chars().count() as f64 * font_size * 0.55,
    }
}

fn text_height(font_size: f64) -> f64 {
    // Recharts measures offsetHeight of a span: ~15px at 10px, 14px at 9px mono.
    if (font_size - 9.0).abs() < f64::EPSILON {
        14.0
    } else {
        15.0
    }
}

// ── Scales ──────────────────────────────────────────────────────────────────

fn linear(v: f64, d0: f64, d1: f64, r0: f64, r1: f64) -> f64 {
    if d1 == d0 {
        return r0;
    }
    r0 + (v - d0) / (d1 - d0) * (r1 - r0)
}

fn point_coord(i: usize, n: usize, x: f64, w: f64) -> f64 {
    if n <= 1 {
        return x + w / 2.0;
    }
    x + w * i as f64 / (n as f64 - 1.0)
}

fn band_coord(i: usize, n: usize, x: f64, w: f64) -> f64 {
    let band = w / n.max(1) as f64;
    x + band * (i as f64 + 0.5)
}

fn band_size(n: usize, w: f64) -> f64 {
    w / n.max(1) as f64
}

/// Recharts `getBarPositions` for a single series.
fn bar_position(band: f64, max_bar: f64) -> (f64, f64) {
    let offset = 0.1 * band; // barCategoryGap default "10%"
    let mut original = band - 2.0 * offset;
    if original > 1.0 {
        original = original.round();
    }
    let size = original.min(max_bar);
    (offset + (original - size) / 2.0, size)
}

// ── Path building ───────────────────────────────────────────────────────────

fn pn(v: f64) -> String {
    // d3-path default: 3 decimals, no trailing zeros.
    let r = (v * 1000.0).round() / 1000.0;
    if r == 0.0 {
        return "0".to_string();
    }
    let s = format!("{r}");
    s
}

fn rn(v: f64) -> String {
    // Rectangle path: 4 decimals, no trailing zeros.
    let r = (v * 10000.0).round() / 10000.0;
    if r == 0.0 {
        return "0".to_string();
    }
    format!("{r}")
}

fn sign(x: f64) -> f64 {
    if x < 0.0 { -1.0 } else { 1.0 }
}

fn slope3(x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let h0 = x1 - x0;
    let h1 = x2 - x1;
    let s0 = (y1 - y0) / if h0 != 0.0 { h0 } else { 1.0 };
    let s1 = (y2 - y1) / if h1 != 0.0 { h1 } else { 1.0 };
    let p = (s0 * h1 + s1 * h0) / (h0 + h1);
    (sign(s0) + sign(s1)) * s0.abs().min(s1.abs()).min(0.5 * p.abs())
}

fn slope2(x0: f64, y0: f64, x1: f64, y1: f64, t: f64) -> f64 {
    let h = x1 - x0;
    if h != 0.0 {
        (3.0 * (y1 - y0) / h - t) / 2.0
    } else {
        t
    }
}

fn bezier(out: &mut String, x0: f64, y0: f64, x1: f64, y1: f64, t0: f64, t1: f64) {
    let dx = (x1 - x0) / 3.0;
    out.push_str(&format!(
        "C{},{} {},{} {},{}",
        pn(x0 + dx),
        pn(y0 + dx * t0),
        pn(x1 - dx),
        pn(y1 - dx * t1),
        pn(x1),
        pn(y1)
    ));
}

/// d3 `curveMonotoneX` path over the given points.
fn monotone_curve(points: &[(f64, f64)]) -> String {
    let n = points.len();
    let mut out = String::new();
    if n == 0 {
        return out;
    }
    out.push_str(&format!("M{},{}", pn(points[0].0), pn(points[0].1)));
    if n == 1 {
        return out;
    }
    if n == 2 {
        out.push_str(&format!("L{},{}", pn(points[1].0), pn(points[1].1)));
        return out;
    }
    // Fritsch-Carlson tangents at interior points (d3 slope3).
    let mut m = vec![0.0f64; n];
    for i in 1..n - 1 {
        m[i] = slope3(
            points[i - 1].0,
            points[i - 1].1,
            points[i].0,
            points[i].1,
            points[i + 1].0,
            points[i + 1].1,
        );
    }
    let t0 = slope2(points[0].0, points[0].1, points[1].0, points[1].1, m[1]);
    bezier(
        &mut out,
        points[0].0,
        points[0].1,
        points[1].0,
        points[1].1,
        t0,
        m[1],
    );
    for i in 1..n - 2 {
        bezier(
            &mut out,
            points[i].0,
            points[i].1,
            points[i + 1].0,
            points[i + 1].1,
            m[i],
            m[i + 1],
        );
    }
    let last = n - 1;
    let t_last = slope2(
        points[last - 1].0,
        points[last - 1].1,
        points[last].0,
        points[last].1,
        m[last - 1],
    );
    bezier(
        &mut out,
        points[last - 1].0,
        points[last - 1].1,
        points[last].0,
        points[last].1,
        m[last - 1],
        t_last,
    );
    out
}

fn monotone_area(points: &[(f64, f64)], baseline: f64) -> String {
    if points.is_empty() {
        return String::new();
    }
    let mut out = monotone_curve(points);
    let last_x = points[points.len() - 1].0;
    out.push_str(&format!("L{},{}", pn(last_x), pn(baseline)));
    let reversed: Vec<(f64, f64)> = points.iter().rev().map(|(x, _)| (*x, baseline)).collect();
    let tail = monotone_curve(&reversed);
    if let Some(idx) = tail.find('C') {
        out.push_str(&tail[idx..]);
    }
    out.push('Z');
    out
}

fn rounded_rect_path(x: f64, y: f64, w: f64, h: f64, r: [f64; 4]) -> String {
    let [tl, tr, br, bl] = r;
    let mut out = String::new();
    out.push_str(&format!("M{},{}", rn(x), rn(y + tl)));
    if tl > 0.0 {
        out.push_str(&format!("A {tl},{tl},0,0,1,{},{}", rn(x + tl), rn(y)));
    } else {
        out.push_str(&format!("L{},{}", rn(x), rn(y)));
    }
    out.push_str(&format!("L{},{}", rn(x + w - tr), rn(y)));
    if tr > 0.0 {
        out.push_str(&format!("A {tr},{tr},0,0,1,{},{}", rn(x + w), rn(y + tr)));
    }
    out.push_str(&format!("L{},{}", rn(x + w), rn(y + h - br)));
    if br > 0.0 {
        out.push_str(&format!("A {br},{br},0,0,1,{},{}", rn(x + w - br), rn(y + h)));
    }
    out.push_str(&format!("L{},{}", rn(x + bl), rn(y + h)));
    if bl > 0.0 {
        out.push_str(&format!("A {bl},{bl},0,0,1,{},{}", rn(x), rn(y + h - bl)));
    }
    out.push('Z');
    out
}

// ── SVG emission ────────────────────────────────────────────────────────────

struct Svg {
    out: String,
}

impl Svg {
    fn new(w: f64, h: f64) -> Self {
        let mut out = String::new();
        out.push_str(&format!(
            "<svg role=\"application\" tabindex=\"0\" class=\"recharts-surface\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" style=\"width: 100%; height: 100%; display: block;\">",
            trim_num(w),
            trim_num(h),
            trim_num(w),
            trim_num(h)
        ));
        out.push_str("<title></title><desc></desc>");
        Self { out }
    }

    fn finish(mut self) -> String {
        self.out.push_str("</svg>");
        self.out
    }
}

fn trim_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn grid_lines(svg: &mut Svg, plot: Rect, ticks: &[Tick], color: &str, vertical: bool) {
    if ticks.is_empty() {
        return;
    }
    svg.out.push_str("<g class=\"recharts-cartesian-grid\">");
    if vertical {
        svg.out.push_str("<g class=\"recharts-cartesian-grid-vertical\">");
    } else {
        svg.out.push_str("<g class=\"recharts-cartesian-grid-horizontal\">");
    }
    for tick in ticks {
        if !tick.show {
            continue;
        }
        if vertical {
            svg.out.push_str(&format!(
                "<line stroke-dasharray=\"3 3\" stroke=\"{color}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" fill=\"none\"></line>",
                trim_num(plot.x), trim_num(plot.y), trim_num(plot.w), trim_num(plot.h),
                trim_num(tick.coord), trim_num(plot.y), trim_num(tick.coord), trim_num(plot.y + plot.h)
            ));
        } else {
            svg.out.push_str(&format!(
                "<line stroke-dasharray=\"3 3\" stroke=\"{color}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" fill=\"none\"></line>",
                trim_num(plot.x), trim_num(plot.y), trim_num(plot.w), trim_num(plot.h),
                trim_num(plot.x), trim_num(tick.coord), trim_num(plot.x + plot.w), trim_num(tick.coord)
            ));
        }
    }
    svg.out.push_str("</g></g>");
}

#[allow(clippy::too_many_arguments)]
fn emit_axis(svg: &mut Svg, axis: &Axis, plot: Rect) {
    let (line_x1, line_y1, line_x2, line_y2) = match axis.orientation {
        Orientation::Bottom => (plot.x, axis.axis_line, plot.x + plot.w, axis.axis_line),
        Orientation::Left => (axis.axis_line, plot.y, axis.axis_line, plot.y + plot.h),
        Orientation::Right => (axis.axis_line, plot.y, axis.axis_line, plot.y + plot.h),
    };
    let axis_class = match axis.orientation {
        Orientation::Bottom => "recharts-xAxis xAxis",
        _ => "recharts-yAxis yAxis",
    };
    let orientation = match axis.orientation {
        Orientation::Bottom => "bottom",
        Orientation::Left => "left",
        Orientation::Right => "right",
    };
    let (axis_w, axis_h) = match axis.orientation {
        Orientation::Bottom => (plot.w, X_AXIS_HEIGHT),
        _ => (axis.len, plot.h),
    };
    let _ = axis_w;
    let axis_w = if axis.orientation == Orientation::Bottom {
        plot.w
    } else {
        // Distance from the plot edge to the axis line (60 or 150).
        (axis.axis_line - plot.x).abs()
    };
    svg.out.push_str(&format!(
        "<g class=\"recharts-layer recharts-cartesian-axis {axis_class}\">"
    ));
    svg.out.push_str(&format!(
        "<line angle=\"0\" height=\"{}\" orientation=\"{orientation}\" x=\"{}\" y=\"{}\" width=\"{}\" class=\"recharts-cartesian-axis-line\" stroke=\"#666\" fill=\"none\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"></line>",
        trim_num(axis_h), trim_num(plot.x), trim_num(plot.y), trim_num(axis_w),
        trim_num(line_x1), trim_num(line_y1), trim_num(line_x2), trim_num(line_y2)
    ));
    svg.out.push_str("<g class=\"recharts-cartesian-axis-ticks\">");
    svg.out.push_str("<g class=\"recharts-cartesian-axis-tick-lines\">");
    for tick in &axis.ticks {
        if !tick.show {
            continue;
        }
        let (x1, y1, x2, y2) = match axis.orientation {
            Orientation::Bottom => (tick.coord, axis.axis_line + 6.0, tick.coord, axis.axis_line),
            Orientation::Left => (axis.axis_line - 6.0, tick.coord, axis.axis_line, tick.coord),
            Orientation::Right => (axis.axis_line + 6.0, tick.coord, axis.axis_line, tick.coord),
        };
        svg.out.push_str(&format!(
            "<line angle=\"0\" height=\"{}\" orientation=\"{orientation}\" x=\"{}\" y=\"{}\" width=\"{}\" class=\"recharts-cartesian-axis-tick-line\" stroke=\"#666\" fill=\"none\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"></line>",
            trim_num(axis_h), trim_num(plot.x), trim_num(plot.y), trim_num(axis_w),
            trim_num(x1), trim_num(y1), trim_num(x2), trim_num(y2)
        ));
    }
    svg.out.push_str("</g></g></g>");

    // Labels layer.
    let label_class = match axis.orientation {
        Orientation::Bottom => "recharts-xAxis-tick-labels",
        _ => "recharts-yAxis-tick-labels",
    };
    svg.out.push_str(&format!(
        "<g class=\"recharts-cartesian-axis-tick-labels {label_class}\">"
    ));
    for tick in &axis.ticks {
        if !tick.show {
            continue;
        }
        let (tx, ty, anchor, dy) = match axis.orientation {
            Orientation::Bottom => (tick.coord, axis.label_offset, "middle", "0.71em"),
            Orientation::Left => (axis.label_offset, tick.coord, "end", "0.355em"),
            Orientation::Right => (axis.label_offset, tick.coord, "start", "0.355em"),
        };
        let font_family = match axis.font_family {
            Some(family) => format!(" font-family=\"{family}\""),
            None => String::new(),
        };
        svg.out.push_str(&format!(
            "<g class=\"recharts-layer recharts-cartesian-axis-tick-label\"><text width=\"{}\" orientation=\"{orientation}\" height=\"{}\" stroke=\"none\" font-size=\"{}\"{font_family} x=\"{}\" y=\"{}\" class=\"recharts-text recharts-cartesian-axis-tick-value\" text-anchor=\"{anchor}\" fill=\"#64748b\"><tspan x=\"{}\" dy=\"{dy}\">{}</tspan></text></g>",
            trim_num(axis_w), trim_num(axis_h), trim_num(axis.font_size),
            trim_num(tx), trim_num(ty), trim_num(tx), tick.value
        ));
    }
    svg.out.push_str("</g>");
}

fn emit_dots(svg: &mut Svg, points: &[(f64, f64)], name: &str, color: &str, r: f64, plot: Rect) {
    for (x, y) in points {
        svg.out.push_str(&format!(
            "<circle r=\"{r}\" name=\"{name}\" stroke=\"{color}\" stroke-width=\"2\" fill=\"{color}\" height=\"{}\" width=\"{}\" cx=\"{}\" cy=\"{}\" class=\"recharts-dot recharts-line-dot\"></circle>",
            trim_num(plot.h), trim_num(plot.w), trim_num(*x), trim_num(*y)
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn render_chart(
    mode: ChartMode,
    width: f64,
    height: f64,
    hourly: &[HourlyBucket],
    daily: &[DaySummary],
    rows: &[AggregatedEndpoint],
    grid_color: &str,
    tick_color: &str,
    category_tick: &str,
    bar_color: &str,
    error_color: &str,
) -> String {
    let multi_day = daily.len() > 1;
    let mut svg = Svg::new(width, height);

    match mode {
        ChartMode::DailyTrend if multi_day => {
            let n = daily.len();
            let margins = (10.0, 16.0, 0.0, 4.0);
            let offset = (
                margins.0,
                margins.1 + Y_AXIS_WIDTH,
                margins.2 + X_AXIS_HEIGHT + LEGEND_HEIGHT,
                margins.3 + Y_AXIS_WIDTH,
            );
            let plot = Rect {
                x: offset.3,
                y: offset.0,
                w: width - offset.3 - offset.1,
                h: height - offset.0 - offset.2,
            };
            let count_max = daily.iter().map(|d| d.count).max().unwrap_or(0) as f64;
            let latency_max = daily
                .iter()
                .flat_map(|d| [d.p95_ms, d.avg_ms])
                .fold(0.0f64, f64::max);
            let left_ticks = nice_ticks(0.0, count_max, 5, true);
            let right_ticks = nice_ticks(0.0, latency_max, 5, true);
            let left_domain = left_ticks.last().copied().unwrap_or(0.0);
            let right_domain = right_ticks.last().copied().unwrap_or(0.0);

            let y_left = build_numeric_axis(
                &left_ticks,
                AxisKind::NumberCount,
                Orientation::Left,
                plot,
                plot.x - 8.0,
                plot.x,
            );
            let y_right = build_numeric_axis(
                &right_ticks,
                AxisKind::NumberMs,
                Orientation::Right,
                plot,
                plot.x + plot.w + 8.0,
                plot.x + plot.w,
            );

            let mut x_ticks: Vec<Tick> = (0..n)
                .map(|i| Tick {
                    value: format_date(Some(&daily[i].date)),
                    coord: band_coord(i, n, plot.x, plot.w),
                    show: true,
                })
                .collect();
            let x_widths: Vec<f64> = x_ticks
                .iter()
                .map(|t| text_width(&t.value, FONT_SIZE, false))
                .collect();
            select_ticks_end(
                &mut x_ticks,
                1.0,
                plot.x,
                plot.x + plot.w,
                |i| x_widths[i],
                5.0,
            );

            grid_lines(&mut svg, plot, &y_left.ticks, grid_color, false);

            // Bar: count on the left axis.
            svg.out.push_str("<g class=\"recharts-layer recharts-bar\">");
            let band = band_size(n, plot.w);
            let (bar_offset, bar_w) = bar_position(band, 32.0);
            let base_y = linear(0.0, 0.0, left_domain, plot.y + plot.h, plot.y);
            for (i, day) in daily.iter().enumerate() {
                let v = day.count as f64;
                let y = linear(v, 0.0, left_domain, plot.y + plot.h, plot.y);
                let x = plot.x + band * i as f64 + bar_offset;
                let h = base_y - y;
                if h <= 0.0 {
                    continue;
                }
                svg.out.push_str(&format!(
                    "<g class=\"recharts-layer recharts-bar-rectangle\"><path fill=\"{bar_color}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" radius=\"3,3,0,0\" class=\"recharts-rectangle\" d=\"{}\"></path></g>",
                    rn(x), rn(y), rn(bar_w), rn(h),
                    rounded_rect_path(x, y, bar_w, h, [3.0, 3.0, 0.0, 0.0])
                ));
            }
            svg.out.push_str("</g>");

            // Lines.
            let errors: Vec<(f64, f64)> = daily
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    (
                        band_coord(i, n, plot.x, plot.w),
                        linear(d.error_count as f64, 0.0, left_domain, plot.y + plot.h, plot.y),
                    )
                })
                .collect();
            emit_line(&mut svg, &errors, "Errors", error_color, 2.0, None, Some(3.0), plot);

            let p95s: Vec<(f64, f64)> = daily
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    (
                        band_coord(i, n, plot.x, plot.w),
                        linear(d.p95_ms, 0.0, right_domain, plot.y + plot.h, plot.y),
                    )
                })
                .collect();
            emit_line(&mut svg, &p95s, "P95 Latency", P99, 2.0, None, Some(3.0), plot);

            let avgs: Vec<(f64, f64)> = daily
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    (
                        band_coord(i, n, plot.x, plot.w),
                        linear(d.avg_ms, 0.0, right_domain, plot.y + plot.h, plot.y),
                    )
                })
                .collect();
            emit_line(&mut svg, &avgs, "Avg Latency", AVG, 1.5, Some("4 4"), Some(2.0), plot);

            emit_axis(&mut svg, &x_axis(
                &x_ticks,
                plot.x,
                plot.y + plot.h,
                plot.w,
                plot.x + plot.w,
                tick_color,
            ), plot);
            emit_axis(&mut svg, &y_left, plot);
            emit_axis(&mut svg, &y_right, plot);
        }
        ChartMode::TimeOfDay | ChartMode::DailyTrend => {
            let n = hourly.len();
            let margins = (10.0, 16.0, 0.0, 4.0);
            let offset = (
                margins.0,
                margins.1,
                margins.2 + X_AXIS_HEIGHT + LEGEND_HEIGHT,
                margins.3 + Y_AXIS_WIDTH,
            );
            let plot = Rect {
                x: offset.3,
                y: offset.0,
                w: width - offset.3 - offset.1,
                h: height - offset.0 - offset.2,
            };
            let data_max = hourly
                .iter()
                .flat_map(|h| [h.p99_ms, h.p95_ms, h.avg_ms])
                .fold(0.0f64, f64::max);
            let ticks = nice_ticks(0.0, data_max, 5, true);
            let domain = ticks.last().copied().unwrap_or(0.0);
            let y_axis = build_numeric_axis(
                &ticks,
                AxisKind::NumberMs,
                Orientation::Left,
                plot,
                plot.x - 8.0,
                plot.x,
            );

            let x_ticks: Vec<Tick> = (0..n)
                .enumerate()
                .filter(|(i, _)| i % 3 == 0)
                .map(|(i, _)| Tick {
                    value: hourly[i].label.clone(),
                    coord: point_coord(i, n, plot.x, plot.w),
                    show: true,
                })
                .collect();

            grid_lines(&mut svg, plot, &y_axis.ticks, grid_color, false);

            fn p99_of(h: &HourlyBucket) -> f64 { h.p99_ms }
            fn p95_of(h: &HourlyBucket) -> f64 { h.p95_ms }
            fn avg_of(h: &HourlyBucket) -> f64 { h.avg_ms }
            let series: [(&str, &str, &str, f64, f64, fn(&HourlyBucket) -> f64); 3] = [
                ("P99 Latency", P99, "p99Grad", 2.0, 0.3, p99_of),
                ("P95 Latency", P95, "p95Grad", 2.0, 0.35, p95_of),
                ("Avg Latency", AVG, "avgGrad", 1.5, 0.3, avg_of),
            ];
            for (name, color, grad, stroke_w, opacity, get) in series {
                let points: Vec<(f64, f64)> = hourly
                    .iter()
                    .enumerate()
                    .map(|(i, h)| {
                        (
                            point_coord(i, n, plot.x, plot.w),
                            linear(get(h), 0.0, domain, plot.y + plot.h, plot.y),
                        )
                    })
                    .collect();
                let area = monotone_area(&points, plot.y + plot.h);
                let curve = monotone_curve(&points);
                svg.out.push_str(&format!(
                    "<g class=\"recharts-layer recharts-area\"><g class=\"recharts-layer\"><g class=\"recharts-layer recharts-shape\"><g class=\"recharts-layer\">"
                ));
                svg.out.push_str(&format!(
                    "<defs><linearGradient id=\"{grad}\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\"><stop offset=\"5%\" stop-color=\"{color}\" stop-opacity=\"{opacity}\"></stop><stop offset=\"95%\" stop-color=\"{color}\" stop-opacity=\"0\"></stop></linearGradient></defs>"
                ));
                svg.out.push_str(&format!(
                    "<path name=\"{name}\" stroke-width=\"{stroke_w}\" fill=\"url(#{grad})\" fill-opacity=\"1\" height=\"{}\" width=\"{}\" stroke=\"none\" class=\"recharts-curve recharts-area-area\" d=\"{area}\"></path>",
                    trim_num(plot.h), trim_num(plot.w)
                ));
                svg.out.push_str(&format!(
                    "<path name=\"{name}\" stroke-width=\"{stroke_w}\" fill=\"none\" fill-opacity=\"1\" height=\"{}\" width=\"{}\" class=\"recharts-curve recharts-area-curve\" stroke=\"{color}\" d=\"{curve}\"></path>",
                    trim_num(plot.h), trim_num(plot.w)
                ));
                svg.out.push_str("</g></g></g></g>");
            }

            emit_axis(&mut svg, &x_axis(
                &x_ticks,
                plot.x,
                plot.y + plot.h,
                plot.w,
                plot.x + plot.w,
                tick_color,
            ), plot);
            emit_axis(&mut svg, &y_axis, plot);
        }
        ChartMode::Throughput => {
            let n = hourly.len();
            let margins = (10.0, 16.0, 0.0, 4.0);
            let offset = (
                margins.0,
                margins.1 + Y_AXIS_WIDTH,
                margins.2 + X_AXIS_HEIGHT + LEGEND_HEIGHT,
                margins.3 + Y_AXIS_WIDTH,
            );
            let plot = Rect {
                x: offset.3,
                y: offset.0,
                w: width - offset.3 - offset.1,
                h: height - offset.0 - offset.2,
            };
            let count_max = hourly.iter().map(|h| h.count).max().unwrap_or(0) as f64;
            let error_max = hourly.iter().map(|h| h.error_count).max().unwrap_or(0) as f64;
            let left_ticks = nice_ticks(0.0, count_max, 5, true);
            let right_ticks = nice_ticks(0.0, error_max, 5, true);
            let left_domain = left_ticks.last().copied().unwrap_or(0.0);
            let right_domain = right_ticks.last().copied().unwrap_or(0.0);

            let y_left = build_numeric_axis(
                &left_ticks,
                AxisKind::NumberCount,
                Orientation::Left,
                plot,
                plot.x - 8.0,
                plot.x,
            );
            let y_right = build_numeric_axis(
                &right_ticks,
                AxisKind::NumberCount,
                Orientation::Right,
                plot,
                plot.x + plot.w + 8.0,
                plot.x + plot.w,
            );

            let x_ticks: Vec<Tick> = (0..n)
                .enumerate()
                .filter(|(i, _)| i % 3 == 0)
                .map(|(i, _)| Tick {
                    value: hourly[i].label.clone(),
                    coord: band_coord(i, n, plot.x, plot.w),
                    show: true,
                })
                .collect();

            grid_lines(&mut svg, plot, &y_left.ticks, grid_color, false);

            svg.out.push_str("<g class=\"recharts-layer recharts-bar\">");
            let band = band_size(n, plot.w);
            let (bar_offset, bar_w) = bar_position(band, 22.0);
            let base_y = linear(0.0, 0.0, left_domain, plot.y + plot.h, plot.y);
            for (i, h) in hourly.iter().enumerate() {
                let v = h.count as f64;
                let y = linear(v, 0.0, left_domain, plot.y + plot.h, plot.y);
                let x = plot.x + band * i as f64 + bar_offset;
                let bar_h = base_y - y;
                if bar_h <= 0.0 {
                    continue;
                }
                svg.out.push_str(&format!(
                    "<g class=\"recharts-layer recharts-bar-rectangle\"><path fill=\"{bar_color}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" radius=\"3,3,0,0\" class=\"recharts-rectangle\" d=\"{}\"></path></g>",
                    rn(x), rn(y), rn(bar_w), rn(bar_h),
                    rounded_rect_path(x, y, bar_w, bar_h, [3.0, 3.0, 0.0, 0.0])
                ));
            }
            svg.out.push_str("</g>");

            let errors: Vec<(f64, f64)> = hourly
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    (
                        band_coord(i, n, plot.x, plot.w),
                        linear(h.error_count as f64, 0.0, right_domain, plot.y + plot.h, plot.y),
                    )
                })
                .collect();
            emit_line(
                &mut svg,
                &errors,
                "Errors (4xx/5xx)",
                error_color,
                2.0,
                None,
                Some(3.0),
                plot,
            );

            emit_axis(&mut svg, &x_axis(
                &x_ticks,
                plot.x,
                plot.y + plot.h,
                plot.w,
                plot.x + plot.w,
                tick_color,
            ), plot);
            emit_axis(&mut svg, &y_left, plot);
            emit_axis(&mut svg, &y_right, plot);
        }
        ChartMode::Distribution => {
            let buckets = distribution_buckets(rows);
            let n = buckets.len();
            let margins = (10.0, 16.0, 0.0, 4.0);
            let offset = (
                margins.0,
                margins.1,
                margins.2 + X_AXIS_HEIGHT,
                margins.3 + Y_AXIS_WIDTH,
            );
            let plot = Rect {
                x: offset.3,
                y: offset.0,
                w: width - offset.3 - offset.1,
                h: height - offset.0 - offset.2,
            };
            let count_max = buckets.iter().map(|(_, c)| *c).max().unwrap_or(0) as f64;
            let ticks = nice_ticks(0.0, count_max, 5, true);
            let domain = ticks.last().copied().unwrap_or(0.0);
            let y_axis = build_numeric_axis(
                &ticks,
                AxisKind::NumberCount,
                Orientation::Left,
                plot,
                plot.x - 8.0,
                plot.x,
            );

            let mut x_ticks: Vec<Tick> = buckets
                .iter()
                .enumerate()
                .map(|(i, (label, _))| Tick {
                    value: label.clone(),
                    coord: band_coord(i, n, plot.x, plot.w),
                    show: true,
                })
                .collect();
            let x_widths: Vec<f64> = x_ticks
                .iter()
                .map(|t| text_width(&t.value, FONT_SIZE, false))
                .collect();
            select_ticks_end(
                &mut x_ticks,
                1.0,
                plot.x,
                plot.x + plot.w,
                |i| x_widths[i],
                5.0,
            );

            grid_lines(&mut svg, plot, &y_axis.ticks, grid_color, false);

            svg.out.push_str("<g class=\"recharts-layer recharts-bar\">");
            let band = band_size(n, plot.w);
            let (bar_offset, bar_w) = bar_position(band, 36.0);
            let base_y = linear(0.0, 0.0, domain, plot.y + plot.h, plot.y);
            for (i, (_, count)) in buckets.iter().enumerate() {
                let v = *count as f64;
                let y = linear(v, 0.0, domain, plot.y + plot.h, plot.y);
                let x = plot.x + band * i as f64 + bar_offset;
                let h = base_y - y;
                if h <= 0.0 {
                    continue;
                }
                svg.out.push_str(&format!(
                    "<g class=\"recharts-layer recharts-bar-rectangle\"><path fill=\"{P95}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" radius=\"4,4,0,0\" class=\"recharts-rectangle\" d=\"{}\"></path></g>",
                    rn(x), rn(y), rn(bar_w), rn(h),
                    rounded_rect_path(x, y, bar_w, h, [4.0, 4.0, 0.0, 0.0])
                ));
            }
            svg.out.push_str("</g>");

            emit_axis(&mut svg, &x_axis(
                &x_ticks,
                plot.x,
                plot.y + plot.h,
                plot.w,
                plot.x + plot.w,
                tick_color,
            ), plot);
            emit_axis(&mut svg, &y_axis, plot);
        }
        ChartMode::TopP95 => {
            let top: Vec<&AggregatedEndpoint> = rows.iter().take(20).collect();
            let n = top.len().max(1);
            let margins = (4.0, 16.0, 4.0, 4.0);
            let offset = (
                margins.0,
                margins.1,
                margins.2 + X_AXIS_HEIGHT,
                margins.3 + 150.0,
            );
            let plot = Rect {
                x: offset.3,
                y: offset.0,
                w: width - offset.3 - offset.1,
                h: height - offset.0 - offset.2,
            };
            let p95_max = top.iter().map(|r| r.p95_ms).fold(0.0f64, f64::max);
            let ticks = nice_ticks(0.0, p95_max, 5, true);
            let domain = ticks.last().copied().unwrap_or(0.0);
            let x_axis_scale = |v: f64| linear(v, 0.0, domain, plot.x, plot.x + plot.w);

            let mut x_ticks: Vec<Tick> = ticks
                .iter()
                .map(|v| Tick {
                    value: format_tick(*v, AxisKind::NumberMs),
                    coord: linear(*v, 0.0, domain, plot.x, plot.x + plot.w),
                    show: true,
                })
                .collect();
            let x_widths: Vec<f64> = x_ticks
                .iter()
                .map(|t| text_width(&t.value, FONT_SIZE, false))
                .collect();
            select_ticks_end(
                &mut x_ticks,
                1.0,
                plot.x,
                plot.x + plot.w,
                |i| x_widths[i],
                5.0,
            );

            let mut y_ticks: Vec<Tick> = top
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let truncated = if r.path.chars().count() > 36 {
                        format!("{} {}", r.method.as_str(), truncate_chars(&r.path, 34))
                    } else {
                        format!("{} {}", r.method.as_str(), r.path)
                    };
                    Tick {
                        value: truncated,
                        coord: band_coord(i, n, plot.y, plot.h),
                        show: true,
                    }
                })
                .collect();
            select_ticks_end(
                &mut y_ticks,
                -1.0,
                plot.y + plot.h,
                plot.y,
                |_| text_height(9.0),
                5.0,
            );

            grid_lines(&mut svg, plot, &x_ticks, grid_color, true);

            svg.out.push_str("<g class=\"recharts-layer recharts-bar\">");
            let band = band_size(n, plot.h);
            let (bar_offset, bar_h) = bar_position(band, 18.0);
            for (i, r) in top.iter().enumerate() {
                let x = x_axis_scale(0.0);
                let w = x_axis_scale(r.p95_ms) - x;
                let y = plot.y + band * i as f64 + bar_offset;
                if w <= 0.0 {
                    continue;
                }
                let name = if r.path.chars().count() > 36 {
                    format!("{} {}", r.method.as_str(), truncate_chars(&r.path, 34))
                } else {
                    format!("{} {}", r.method.as_str(), r.path)
                };
                svg.out.push_str(&format!(
                    "<g class=\"recharts-layer recharts-bar-rectangle\"><path fill=\"{P95}\" name=\"{name}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" radius=\"0,3,3,0\" class=\"recharts-rectangle\" d=\"{}\"></path></g>",
                    rn(x), rn(y), rn(w), rn(bar_h),
                    rounded_rect_path(x, y, w, bar_h, [0.0, 3.0, 3.0, 0.0])
                ));
            }
            svg.out.push_str("</g>");

            let x_axis = Axis {
                orientation: Orientation::Bottom,
                len: plot.w,
                ticks: x_ticks,
                axis_line: plot.y + plot.h,
                label_offset: plot.y + plot.h + 8.0,
                font_family: None,
                font_size: FONT_SIZE,
            };
            emit_axis(&mut svg, &x_axis, plot);

            let y_axis = Axis {
                orientation: Orientation::Left,
                len: plot.h,
                ticks: y_ticks,
                axis_line: plot.x,
                label_offset: plot.x - 8.0,
                font_family: Some("IBM Plex Mono, monospace"),
                font_size: 9.0,
            };
            emit_axis(&mut svg, &y_axis, plot);
            let _ = category_tick;
        }
    }

    svg.finish()
}

fn truncate_chars(s: &str, max: usize) -> String {
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

#[allow(clippy::too_many_arguments)]
fn build_numeric_axis(
    tick_values: &[f64],
    kind: AxisKind,
    orientation: Orientation,
    plot: Rect,
    label_offset: f64,
    axis_line: f64,
) -> Axis {
    let domain = tick_values.last().copied().unwrap_or(0.0);
    let ticks: Vec<Tick> = tick_values
        .iter()
        .map(|v| Tick {
            value: format_tick(*v, kind),
            coord: if orientation == Orientation::Bottom {
                linear(*v, 0.0, domain, plot.x, plot.x + plot.w)
            } else {
                linear(*v, 0.0, domain, plot.y + plot.h, plot.y)
            },
            show: true,
        })
        .collect();
    Axis {
        orientation,
        len: if orientation == Orientation::Bottom {
            plot.w
        } else {
            plot.h
        },
        ticks,
        axis_line,
        label_offset,
        font_family: None,
        font_size: FONT_SIZE,
    }
}

fn x_axis(
    ticks: &[Tick],
    _x: f64,
    axis_line: f64,
    w: f64,
    _end: f64,
    _tick_color: &str,
) -> Axis {
    Axis {
        orientation: Orientation::Bottom,
        len: w,
        ticks: ticks
            .iter()
            .map(|t| Tick {
                value: t.value.clone(),
                coord: t.coord,
                show: t.show,
            })
            .collect(),
        axis_line,
        label_offset: axis_line + 8.0,
        font_family: None,
        font_size: FONT_SIZE,
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_line(
    svg: &mut Svg,
    points: &[(f64, f64)],
    name: &str,
    color: &str,
    stroke_width: f64,
    dash: Option<&str>,
    dot_r: Option<f64>,
    plot: Rect,
) {
    svg.out.push_str("<g class=\"recharts-layer recharts-line\">");
    let dash_attr = match dash {
        Some(d) => format!(" stroke-dasharray=\"{d}\""),
        None => String::new(),
    };
    let curve = monotone_curve(points);
    svg.out.push_str(&format!(
        "<path name=\"{name}\" stroke=\"{color}\" stroke-width=\"{stroke_width}\"{dash_attr} fill=\"none\" height=\"{}\" width=\"{}\" class=\"recharts-curve recharts-line-curve\" d=\"{curve}\"></path>",
        trim_num(plot.h), trim_num(plot.w)
    ));
    if let Some(r) = dot_r {
        emit_dots(svg, points, name, color, r, plot);
    }
    svg.out.push_str("</g>");
}

fn distribution_buckets(rows: &[AggregatedEndpoint]) -> Vec<(String, u64)> {
    let mut buckets: Vec<(&str, u64, &str)> = vec![
        ("<50ms", 0, "#22c55e"),
        ("50-100ms", 0, "#84cc16"),
        ("100-300ms", 0, "#eab308"),
        ("300-500ms", 0, "#f97316"),
        ("500ms-1s", 0, "#ef4444"),
        ("1s-3s", 0, "#b91c1c"),
        (">3s", 0, "#7f1d1d"),
    ];
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
        let c50 = (c as f64 * 0.5).round() as u64;
        let c90 = (c as f64 * 0.4).round() as u64;
        let c95 = (c as f64 * 0.05).round() as u64;
        let c99 = (c as f64 * 0.04).round() as u64;
        let c_max = c.saturating_sub(c50 + c90 + c95 + c99);
        buckets[classify(r.p50_ms)].1 += c50;
        buckets[classify((r.p50_ms + r.p90_ms) / 2.0)].1 += c90;
        buckets[classify((r.p90_ms + r.p95_ms) / 2.0)].1 += c95;
        buckets[classify((r.p95_ms + r.p99_ms) / 2.0)].1 += c99;
        buckets[classify(r.max_ms)].1 += c_max;
    }
    buckets
        .into_iter()
        .map(|(label, count, _)| (label.to_string(), count))
        .collect()
}
