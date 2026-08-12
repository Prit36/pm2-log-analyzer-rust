use dioxus::prelude::*;
use crate::core::models::{EndpointStats, LogAnalysisSummary};

fn format_ms(val: f32) -> String {
    if val >= 1000.0 {
        format!("{:.2}s", val / 1000.0)
    } else {
        format!("{:.0}ms", val)
    }
}

fn format_num(val: u64) -> String {
    if val >= 1_000_000 {
        format!("{:.1}M", val as f64 / 1_000_000.0)
    } else if val >= 1_000 {
        format!("{:.1}K", val as f64 / 1_000.0)
    } else {
        val.to_string()
    }
}

#[component]
pub fn LatencyChart(
    has_data: Signal<bool>,
    summary: Signal<LogAnalysisSummary>,
    endpoints: Signal<Vec<EndpointStats>>,
    theme: Signal<String>,
) -> Element {
    let mut mode = use_signal(|| "timeOfDay".to_string());
    let is_dark = theme() == "dark";

    if !has_data() {
        return rsx! {
            section { class: "flex flex-col rounded border border-slate-200 bg-white shadow-xs dark:border-slate-800/80 dark:bg-slate-900/80 dark:shadow-md dark:shadow-black/20",
                div { class: "flex items-center justify-between border-b border-slate-200 px-3 py-2 dark:border-slate-800",
                    h2 { class: "text-xs font-semibold uppercase tracking-wide text-slate-700 dark:text-slate-300",
                        "API Visual Analytics"
                    }
                }
                div { class: "px-3 py-12 text-center text-sm text-slate-400 dark:text-slate-500",
                    "No chart data available yet."
                }
            }
        };
    }

    let sum = summary();
    let hourly = &sum.hourly_buckets;
    let ep_list = endpoints();

    let tab_active = "flex items-center gap-1.5 rounded px-2 py-1 font-medium transition-colors bg-white text-blue-600 shadow-xs dark:bg-blue-600 dark:text-white dark:shadow-md text-xs cursor-pointer border-0";
    let tab_inactive = "flex items-center gap-1.5 rounded px-2 py-1 font-medium transition-colors text-slate-600 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200 text-xs cursor-pointer border-0";

    rsx! {
        section { class: "flex flex-col rounded border border-slate-200 bg-white shadow-xs dark:border-slate-800/80 dark:bg-slate-900/80 dark:shadow-md dark:shadow-black/20",
            div { class: "flex flex-wrap items-center justify-between gap-2 border-b border-slate-200 px-3 py-2 dark:border-slate-800",
                h2 { class: "text-xs font-semibold uppercase tracking-wide text-slate-700 dark:text-slate-300",
                    "API Visual Analytics"
                }
                div { class: "flex items-center gap-1 rounded bg-slate-100 p-0.5 text-xs dark:bg-slate-950",
                    button {
                        r#type: "button",
                        onclick: move |_| mode.set("timeOfDay".to_string()),
                        class: if mode() == "timeOfDay" { "{tab_active}" } else { "{tab_inactive}" },
                        title: "Time of Day vs Latency Trend",
                        svg { width: "14", height: "14", class: "h-3.5 w-3.5", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            circle { cx: "12", cy: "12", r: "10" }
                            polyline { points: "12 6 12 12 16 14" }
                        }
                        span { "Time vs Latency" }
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| mode.set("throughput".to_string()),
                        class: if mode() == "throughput" { "{tab_active}" } else { "{tab_inactive}" },
                        title: "Hourly Request Volume & Error Rate",
                        svg { width: "14", height: "14", class: "h-3.5 w-3.5", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            line { x1: "18", y1: "20", x2: "18", y2: "10" }
                            line { x1: "12", y1: "20", x2: "12", y2: "4" }
                            line { x1: "6", y1: "20", x2: "6", y2: "14" }
                        }
                        span { "Hourly Volume" }
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| mode.set("distribution".to_string()),
                        class: if mode() == "distribution" { "{tab_active}" } else { "{tab_inactive}" },
                        title: "Latency Distribution Buckets",
                        svg { width: "14", height: "14", class: "h-3.5 w-3.5", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            polyline { points: "22 12 18 12 15 21 9 3 6 12 2 12" }
                        }
                        span { "Distribution" }
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| mode.set("topP95".to_string()),
                        class: if mode() == "topP95" { "{tab_active}" } else { "{tab_inactive}" },
                        title: "Top p95 Slowest Endpoints",
                        svg { width: "14", height: "14", class: "h-3.5 w-3.5", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            path { d: "M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z" }
                        }
                        span { "Top Slowest" }
                    }
                }
            }

            div { class: "h-[340px] px-3 py-3 w-full flex items-center justify-center",
                if mode() == "timeOfDay" {
                    { render_area_chart(hourly, is_dark) }
                } else if mode() == "throughput" {
                    { render_volume_chart(hourly, is_dark) }
                } else if mode() == "distribution" {
                    { render_distribution_chart(&ep_list, is_dark) }
                } else {
                    { render_top_slowest_chart(&ep_list, is_dark) }
                }
            }
        }
    }
}

fn render_area_chart(hourly: &[crate::core::models::HourlyBucket], is_dark: bool) -> Element {
    let grid_color = if is_dark { "#1e293b" } else { "#f1f5f9" };
    let text_color = if is_dark { "#94a3b8" } else { "#64748b" };

    let mut max_val: f32 = 10.0;
    for h in hourly {
        if h.p99_ms > max_val { max_val = h.p99_ms; }
    }
    max_val *= 1.1;

    let width = 500.0;
    let height = 300.0;
    let padding_left = 45.0;
    let padding_bottom = 35.0;
    let padding_top = 30.0;
    let padding_right = 15.0;

    let chart_w = width - padding_left - padding_right;
    let chart_h = height - padding_top - padding_bottom;

    let step_x = if hourly.len() > 1 { chart_w / (hourly.len() - 1) as f32 } else { chart_w };

    let mut p99_pts = String::new();
    let mut p95_pts = String::new();
    let mut avg_pts = String::new();

    for (i, h) in hourly.iter().enumerate() {
        let x = padding_left + i as f32 * step_x;
        let y_p99 = padding_top + chart_h * (1.0 - (h.p99_ms / max_val).clamp(0.0, 1.0));
        let y_p95 = padding_top + chart_h * (1.0 - (h.p95_ms / max_val).clamp(0.0, 1.0));
        let y_avg = padding_top + chart_h * (1.0 - (h.avg_ms / max_val).clamp(0.0, 1.0));

        if i == 0 {
            p99_pts.push_str(&format!("{:.1},{:.1}", x, y_p99));
            p95_pts.push_str(&format!("{:.1},{:.1}", x, y_p95));
            avg_pts.push_str(&format!("{:.1},{:.1}", x, y_avg));
        } else {
            p99_pts.push_str(&format!(" L{:.1},{:.1}", x, y_p99));
            p95_pts.push_str(&format!(" L{:.1},{:.1}", x, y_p95));
            avg_pts.push_str(&format!(" L{:.1},{:.1}", x, y_avg));
        }
    }

    let p99_fill = format!("M{},{} L{} L{},{} Z", padding_left, padding_top + chart_h, p99_pts, padding_left + chart_w, padding_top + chart_h);
    let p95_fill = format!("M{},{} L{} L{},{} Z", padding_left, padding_top + chart_h, p95_pts, padding_left + chart_w, padding_top + chart_h);
    let avg_fill = format!("M{},{} L{} L{},{} Z", padding_left, padding_top + chart_h, avg_pts, padding_left + chart_w, padding_top + chart_h);

    rsx! {
        svg { width: "100%", height: "100%", view_box: "0 0 500 300", class: "w-full h-full",
            defs {
                linearGradient { id: "p99Grad", x1: "0", y1: "0", x2: "0", y2: "1",
                    stop { offset: "5%", stop_color: "#7c3aed", stop_opacity: "0.3" }
                    stop { offset: "95%", stop_color: "#7c3aed", stop_opacity: "0" }
                }
                linearGradient { id: "p95Grad", x1: "0", y1: "0", x2: "0", y2: "1",
                    stop { offset: "5%", stop_color: "#2563eb", stop_opacity: "0.35" }
                    stop { offset: "95%", stop_color: "#2563eb", stop_opacity: "0" }
                }
                linearGradient { id: "avgGrad", x1: "0", y1: "0", x2: "0", y2: "1",
                    stop { offset: "5%", stop_color: "#0d9488", stop_opacity: "0.3" }
                    stop { offset: "95%", stop_color: "#0d9488", stop_opacity: "0" }
                }
            }

            for i in 0..=4 {
                {
                    let y = padding_top + chart_h * (1.0 - i as f32 / 4.0);
                    let val = max_val * (i as f32 / 4.0);
                    rsx! {
                        line { x1: "{padding_left}", y1: "{y}", x2: "{padding_left + chart_w}", y2: "{y}", stroke: "{grid_color}", stroke_dasharray: "3 3" }
                        text { x: "{padding_left - 6.0}", y: "{y + 3.0}", text_anchor: "end", fill: "{text_color}", font_size: "10", "{format_ms(val)}" }
                    }
                }
            }

            for (i, h) in hourly.iter().enumerate() {
                {
                    if i % 3 == 0 {
                        let x = padding_left + i as f32 * step_x;
                        let y = padding_top + chart_h + 16.0;
                        rsx! {
                            text { x: "{x}", y: "{y}", text_anchor: "middle", fill: "{text_color}", font_size: "9", "{h.label}" }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }

            path { d: "{p99_fill}", fill: "url(#p99Grad)" }
            path { d: "{p95_fill}", fill: "url(#p95Grad)" }
            path { d: "{avg_fill}", fill: "url(#avgGrad)" }

            path { d: "M{p99_pts}", fill: "none", stroke: "#7c3aed", stroke_width: "2" }
            path { d: "M{p95_pts}", fill: "none", stroke: "#2563eb", stroke_width: "2" }
            path { d: "M{avg_pts}", fill: "none", stroke: "#0d9488", stroke_width: "1.5" }

            g { transform: "translate(110, 14)",
                circle { cx: "0", cy: "0", r: "4", fill: "#7c3aed" }
                text { x: "8", y: "4", fill: "{text_color}", font_size: "10", "P99 Latency" }
                circle { cx: "100", cy: "0", r: "4", fill: "#2563eb" }
                text { x: "108", y: "4", fill: "{text_color}", font_size: "10", "P95 Latency" }
                circle { cx: "200", cy: "0", r: "4", fill: "#0d9488" }
                text { x: "208", y: "4", fill: "{text_color}", font_size: "10", "Avg Latency" }
            }
        }
    }
}

fn render_volume_chart(hourly: &[crate::core::models::HourlyBucket], is_dark: bool) -> Element {
    let grid_color = if is_dark { "#1e293b" } else { "#f1f5f9" };
    let text_color = if is_dark { "#94a3b8" } else { "#64748b" };

    let mut max_count = 10u64;
    let mut max_errors = 1u64;
    for h in hourly {
        if h.count > max_count { max_count = h.count; }
        if h.error_count > max_errors { max_errors = h.error_count; }
    }

    let width = 500.0f32;
    let height = 300.0f32;
    let padding_left = 40.0f32;
    let padding_bottom = 35.0f32;
    let padding_top = 30.0f32;
    let padding_right = 40.0f32;

    let chart_w = width - padding_left - padding_right;
    let chart_h = height - padding_top - padding_bottom;
    let bar_w: f32 = (chart_w / (hourly.len().max(1) as f32) * 0.6).clamp(4.0, 18.0);

    let mut err_pts = String::new();
    let step = chart_w / hourly.len().max(1) as f32;

    for (i, h) in hourly.iter().enumerate() {
        let x = padding_left + i as f32 * step + step / 2.0;
        let y_err = padding_top + chart_h * (1.0 - (h.error_count as f32 / max_errors as f32).clamp(0.0, 1.0));
        if i == 0 {
            err_pts.push_str(&format!("{:.1},{:.1}", x, y_err));
        } else {
            err_pts.push_str(&format!(" L{:.1},{:.1}", x, y_err));
        }
    }

    rsx! {
        svg { width: "100%", height: "100%", view_box: "0 0 500 300", class: "w-full h-full",
            for i in 0..=4 {
                {
                    let y = padding_top + chart_h * (1.0 - i as f32 / 4.0);
                    let val = (max_count as f32 * (i as f32 / 4.0)) as u64;
                    let val_err = (max_errors as f32 * (i as f32 / 4.0)) as u64;
                    rsx! {
                        line { x1: "{padding_left}", y1: "{y}", x2: "{padding_left + chart_w}", y2: "{y}", stroke: "{grid_color}", stroke_dasharray: "3 3" }
                        text { x: "{padding_left - 6.0}", y: "{y + 3.0}", text_anchor: "end", fill: "{text_color}", font_size: "10", "{format_num(val)}" }
                        text { x: "{padding_left + chart_w + 6.0}", y: "{y + 3.0}", text_anchor: "start", fill: "#ef4444", font_size: "10", "{format_num(val_err)}" }
                    }
                }
            }

            for (i, h) in hourly.iter().enumerate() {
                {
                    let x = padding_left + i as f32 * step + (step - bar_w) / 2.0;
                    let h_bar = chart_h * (h.count as f32 / max_count as f32).clamp(0.0, 1.0);
                    let y = padding_top + chart_h - h_bar;

                    rsx! {
                        rect { x: "{x}", y: "{y}", width: "{bar_w}", height: "{h_bar}", rx: "3", fill: "#3b82f6" }
                    }
                }
            }

            path { d: "M{err_pts}", fill: "none", stroke: "#ef4444", stroke_width: "2" }

            for (i, h) in hourly.iter().enumerate() {
                {
                    let x = padding_left + i as f32 * step + step / 2.0;
                    let y_err = padding_top + chart_h * (1.0 - (h.error_count as f32 / max_errors as f32).clamp(0.0, 1.0));
                    rsx! {
                        circle { cx: "{x}", cy: "{y_err}", r: "3", fill: "#ef4444" }
                    }
                }
            }

            for (i, h) in hourly.iter().enumerate() {
                {
                    if i % 3 == 0 {
                        let x = padding_left + i as f32 * step + step / 2.0;
                        let y = padding_top + chart_h + 16.0;
                        rsx! {
                            text { x: "{x}", y: "{y}", text_anchor: "middle", fill: "{text_color}", font_size: "9", "{h.label}" }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }

            g { transform: "translate(120, 14)",
                rect { x: "0", y: "-4", width: "8", height: "8", rx: "1", fill: "#3b82f6" }
                text { x: "12", y: "3", fill: "{text_color}", font_size: "10", "Total Requests" }
                line { x1: "120", y1: "0", x2: "135", y2: "0", stroke: "#ef4444", stroke_width: "2" }
                circle { cx: "127.5", cy: "0", r: "3", fill: "#ef4444" }
                text { x: "140", y: "3", fill: "{text_color}", font_size: "10", "Errors (4xx/5xx)" }
            }
        }
    }
}

fn classify_ms(ms: f32) -> usize {
    if ms < 50.0 { 0 }
    else if ms < 100.0 { 1 }
    else if ms < 300.0 { 2 }
    else if ms < 500.0 { 3 }
    else if ms < 1000.0 { 4 }
    else if ms < 3000.0 { 5 }
    else { 6 }
}

fn render_distribution_chart(endpoints: &[EndpointStats], is_dark: bool) -> Element {
    let text_color = if is_dark { "#94a3b8" } else { "#64748b" };

    let mut bucket_counts = [0u64; 7];

    for r in endpoints {
        let c = r.total_calls;
        if c == 0 { continue; }
        let c50 = (c as f64 * 0.50).round() as u64;
        let c90 = (c as f64 * 0.40).round() as u64;
        let c95 = (c as f64 * 0.05).round() as u64;
        let c99 = (c as f64 * 0.04).round() as u64;
        let c_max = c.saturating_sub(c50 + c90 + c95 + c99);

        bucket_counts[classify_ms(r.p50_ms)] += c50;
        bucket_counts[classify_ms((r.p50_ms + r.p90_ms) / 2.0)] += c90;
        bucket_counts[classify_ms((r.p90_ms + r.p95_ms) / 2.0)] += c95;
        bucket_counts[classify_ms((r.p95_ms + r.p99_ms) / 2.0)] += c99;
        bucket_counts[classify_ms(r.max_duration_ms)] += c_max;
    }

    let buckets = vec![
        ("<50ms", bucket_counts[0], "#22c55e"),
        ("50-100ms", bucket_counts[1], "#84cc16"),
        ("100-300ms", bucket_counts[2], "#eab308"),
        ("300-500ms", bucket_counts[3], "#f97316"),
        ("500ms-1s", bucket_counts[4], "#ef4444"),
        ("1s-3s", bucket_counts[5], "#b91c1c"),
        (">3s", bucket_counts[6], "#7f1d1d"),
    ];

    let max_val = buckets.iter().map(|b| b.1).max().unwrap_or(1).max(1);

    rsx! {
        svg { width: "100%", height: "100%", view_box: "0 0 500 300", class: "w-full h-full",
            for (i, (label, count, color)) in buckets.iter().enumerate() {
                {
                    let step = 440.0 / 7.0;
                    let x = 35.0 + i as f32 * step + 6.0;
                    let h_bar = 200.0 * (*count as f32 / max_val as f32).clamp(0.02, 1.0);
                    let y = 240.0 - h_bar;

                    rsx! {
                        rect { x: "{x}", y: "{y}", width: "36", height: "{h_bar}", rx: "4", fill: "{color}" }
                        text { x: "{x + 18.0}", y: "260", text_anchor: "middle", fill: "{text_color}", font_size: "9", "{label}" }
                        if *count > 0 {
                            text { x: "{x + 18.0}", y: "{y - 5.0}", text_anchor: "middle", fill: "{text_color}", font_size: "9", "{format_num(*count)}" }
                        }
                    }
                }
            }
        }
    }
}

fn render_top_slowest_chart(endpoints: &[EndpointStats], is_dark: bool) -> Element {
    let text_color = if is_dark { "#cbd5e1" } else { "#334155" };
    let mut top20 = endpoints.to_vec();
    top20.sort_by(|a, b| b.p95_ms.partial_cmp(&a.p95_ms).unwrap_or(std::cmp::Ordering::Equal));
    top20.truncate(15);

    let max_p95: f32 = top20.iter().map(|e| e.p95_ms).fold(10.0f32, |a, b| a.max(b));

    rsx! {
        svg { width: "100%", height: "100%", view_box: "0 0 500 300", class: "w-full h-full",
            for (i, ep) in top20.iter().enumerate() {
                {
                    let y = 10.0 + i as f32 * 19.0;
                    let bar_w = (280.0 * (ep.p95_ms / max_p95)).clamp(4.0, 280.0);
                    let raw_name = format!("{} {}", ep.method.as_str(), ep.path);
                    let label = if raw_name.len() > 32 { format!("{}…", &raw_name[..30]) } else { raw_name };

                    rsx! {
                        text { x: "155", y: "{y + 11.0}", text_anchor: "end", fill: "{text_color}", font_size: "8.5", font_family: "IBM Plex Mono, monospace", "{label}" }
                        rect { x: "162", y: "{y}", width: "{bar_w}", height: "13", rx: "3", fill: "#2563eb" }
                        text { x: "{168.0 + bar_w}", y: "{y + 10.0}", fill: "{text_color}", font_size: "8.5", "{format_ms(ep.p95_ms)}" }
                    }
                }
            }
        }
    }
}
