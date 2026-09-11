//! Chart image renderer — port of the reference `src/utils/chartRenderer.ts`.
//!
//! The reference draws on a 600x320 canvas at `devicePixelRatio = 2` and exports a PNG data
//! URL. This module reproduces the same drawing commands as SVG and rasterizes them with
//! `resvg` at 2x, so the embedded images match the reference geometry 1:1.

use crate::core::models::{AggregatedEndpoint, DaySummary, HourlyBucket};
use crate::utils::format::{format_date, format_ms, format_num};

pub const CHART_W: f64 = 600.0;
pub const CHART_H: f64 = 320.0;
const DPR: f64 = 2.0;

const FONT_SANS: &str = "IBM Plex Sans, Inter, Segoe UI, sans-serif";

const PALETTE_DISTRIBUTION: [&str; 7] = [
    "#22c55e", "#84cc16", "#eab308", "#f97316", "#ef4444", "#b91c1c", "#7f1d1d",
];

pub struct ChartImages {
    pub time_vs_latency: String,
    pub hourly_volume: String,
    pub distribution: String,
    pub daily_trend: Option<String>,
}

// ── SVG helpers ─────────────────────────────────────────────────────────────

fn n(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r == r.trunc() {
        format!("{}", r as i64)
    } else {
        let s = format!("{r:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn text(out: &mut String, x: f64, y: f64, anchor: &str, weight: u32, size: f64, color: &str, s: &str) {
    out.push_str(&format!(
        "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\" dominant-baseline=\"middle\" \
         font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" fill=\"{}\">{}</text>",
        n(x),
        n(y),
        anchor,
        FONT_SANS,
        n(size),
        weight,
        color,
        esc(s)
    ));
}

fn line(out: &mut String, x1: f64, y1: f64, x2: f64, y2: f64, color: &str, width: f64) {
    out.push_str(&format!(
        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>",
        n(x1),
        n(y1),
        n(x2),
        n(y2),
        color,
        n(width)
    ));
}

fn circle(out: &mut String, cx: f64, cy: f64, r: f64, color: &str) {
    out.push_str(&format!(
        "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>",
        n(cx),
        n(cy),
        n(r),
        color
    ));
}

/// `ctx.roundRect(x, y, w, h, [tl, tr, br, bl])` equivalent.
fn rounded_rect_path(out: &mut String, x: f64, y: f64, w: f64, h: f64, radii: [f64; 4]) {
    let [tl, tr, br, bl] = radii.map(|r| r.min(w / 2.0).min(h / 2.0).max(0.0));
    out.push_str(&format!(
        "M{} {}H{}A{} {} 0 0 1 {} {}V{}A{} {} 0 0 1 {} {}H{}A{} {} 0 0 1 {} {}V{}A{} {} 0 0 1 {} {}Z",
        n(x + tl),
        n(y),
        n(x + w - tr),
        n(tr),
        n(tr),
        n(x + w),
        n(y + tr),
        n(y + h - br),
        n(br),
        n(br),
        n(x + w - br),
        n(y + h),
        n(x + bl),
        n(bl),
        n(bl),
        n(x),
        n(y + h - bl),
        n(y + tl),
        n(tl),
        n(tl),
        n(x + tl),
        n(y),
    ));
}

struct Init {
    out: String,
    graph_height: f64,
    padding_left: f64,
    padding_right: f64,
    is_categorical: bool,
}

impl Init {
    fn get_x(&self, index: usize, count: usize) -> f64 {
        let graph_width = CHART_W - self.padding_left - self.padding_right;
        if self.is_categorical {
            self.padding_left + (index as f64 + 0.5) * (graph_width / count.max(1) as f64)
        } else {
            self.padding_left + (index as f64 / (count.max(1) - 1) as f64) * graph_width
        }
    }
}

// ── Reagg helpers ───────────────────────────────────────────────────────────

fn latency_classify(ms: f64) -> usize {
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
}

fn create_latency_buckets() -> [(String, u64); 7] {
    [
        ("<50ms".to_string(), 0),
        ("50-100ms".to_string(), 0),
        ("100-300ms".to_string(), 0),
        ("300-500ms".to_string(), 0),
        ("500ms-1s".to_string(), 0),
        ("1s-3s".to_string(), 0),
        (">3s".to_string(), 0),
    ]
}

fn fill_latency_buckets(buckets: &mut [(String, u64); 7], rows: &[AggregatedEndpoint]) {
    for r in rows {
        if r.count == 0 {
            continue;
        }
        let c50 = (r.count as f64 * 0.5).round() as u64;
        let c90 = (r.count as f64 * 0.4).round() as u64;
        let c95 = (r.count as f64 * 0.05).round() as u64;
        let c99 = (r.count as f64 * 0.04).round() as u64;
        let c_max = r
            .count
            .saturating_sub(c50)
            .saturating_sub(c90)
            .saturating_sub(c95)
            .saturating_sub(c99);
        buckets[latency_classify(r.p50_ms)].1 += c50;
        buckets[latency_classify((r.p50_ms + r.p90_ms) / 2.0)].1 += c90;
        buckets[latency_classify((r.p90_ms + r.p95_ms) / 2.0)].1 += c95;
        buckets[latency_classify((r.p95_ms + r.p99_ms) / 2.0)].1 += c99;
        buckets[latency_classify(r.max_ms)].1 += c_max;
    }
}

fn max_latency_ms(stats: &[HourlyBucket]) -> f64 {
    if stats.is_empty() {
        return 100.0;
    }
    let raw = stats
        .iter()
        .map(|h| h.p99_ms.max(h.p95_ms).max(h.avg_ms))
        .fold(f64::NEG_INFINITY, f64::max);
    let ceil = (raw * 1.1).ceil();
    if ceil == 0.0 { 100.0 } else { ceil }
}

// ── Charts ──────────────────────────────────────────────────────────────────

/// Shared frame: background, title, grid, y labels, x labels.
#[allow(clippy::too_many_arguments)]
fn frame(
    title: &str,
    padding: (f64, f64, f64, f64),
    max_y: f64,
    format_y: &dyn Fn(f64) -> String,
    x_labels: &[String],
    is_categorical: bool,
    format_y_right: Option<&dyn Fn(f64) -> String>,
    max_y_right: Option<f64>,
) -> Init {
    let (top, right, bottom, left) = padding;
    let mut out = String::new();
    out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
        CHART_W, CHART_H, CHART_W, CHART_H
    ));
    out.push_str("<rect width=\"600\" height=\"320\" fill=\"#ffffff\"/>");
    text(&mut out, 18.0, 24.0, "start", 600, 15.0, "#0f172a", title);

    let graph_height = CHART_H - top - bottom;
    let mut init = Init {
        out,
        graph_height,
        padding_left: left,
        padding_right: right,
        is_categorical,
    };

    let y_steps = 4.0;
    for i in 0..=4 {
        let y = top + graph_height - (i as f64 / y_steps) * graph_height;
        line(
            &mut init.out,
            left,
            y,
            CHART_W - right,
            y,
            "#f1f5f9",
            1.0,
        );
        text(
            &mut init.out,
            left - 8.0,
            y + 1.0,
            "end",
            500,
            11.0,
            "#475569",
            &format_y((max_y / y_steps) * i as f64),
        );
        if let (Some(fmt_right), Some(max_right)) = (format_y_right, max_y_right) {
            text(
                &mut init.out,
                CHART_W - right + 8.0,
                y + 1.0,
                "start",
                500,
                11.0,
                "#dc2626",
                &fmt_right((max_right / y_steps) * i as f64),
            );
        }
    }

    let interval = std::cmp::max(1, x_labels.len() / 10) as usize;
    let mut i = 0usize;
    while i < x_labels.len() {
        let x = init.get_x(i, x_labels.len());
        text(
            &mut init.out,
            x,
            CHART_H - bottom + 18.0,
            "middle",
            500,
            11.0,
            "#475569",
            x_labels.get(i).map(String::as_str).unwrap_or(""),
        );
        i += interval;
    }

    init
}

fn draw_series(
    out: &mut String,
    pts: &[(f64, f64)],
    bottom_y: f64,
    stroke: &str,
    fill_start: &str,
    grad_id: &str,
) {
    if pts.is_empty() {
        return;
    }
    // Linear gradient from the series top to the baseline (canvas createLinearGradient(0,topY,0,bottomY)).
    let top_y = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    out.push_str(&format!(
        "<defs><linearGradient id=\"{grad_id}\" gradientUnits=\"userSpaceOnUse\" x1=\"0\" y1=\"{}\" x2=\"0\" y2=\"{}\">\
         <stop offset=\"0\" stop-color=\"{}\" stop-opacity=\"1\"/>\
         <stop offset=\"1\" stop-color=\"#ffffff\" stop-opacity=\"0\"/></linearGradient></defs>",
        n(top_y),
        n(bottom_y),
        fill_start
    ));
    let mut d = String::new();
    d.push_str(&format!("M{} {}", n(pts[0].0), n(bottom_y)));
    for p in pts {
        d.push_str(&format!("L{} {}", n(p.0), n(p.1)));
    }
    d.push_str(&format!("L{} {}Z", n(pts[pts.len() - 1].0), n(bottom_y)));
    out.push_str(&format!("<path d=\"{d}\" fill=\"url(#{grad_id})\"/>"));
    let mut d2 = String::new();
    for (i, p) in pts.iter().enumerate() {
        d2.push_str(&format!(
            "{} {} {}",
            if i == 0 { "M" } else { "L" },
            n(p.0),
            n(p.1)
        ));
    }
    out.push_str(&format!(
        "<path d=\"{d2}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"2\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>"
    ));
}

/// `Time vs Latency Trend` (P99 / P95 / Avg over hourly buckets).
pub fn render_time_vs_latency(hourly: &[HourlyBucket]) -> String {
    if hourly.is_empty() {
        return String::new();
    }
    let max_ms = max_latency_ms(hourly);
    let labels: Vec<String> = hourly.iter().map(|h| h.label.clone()).collect();
    let mut c = frame(
        "Time vs Latency Trend",
        (52.0, 24.0, 48.0, 68.0),
        max_ms,
        &|v| format_ms(v),
        &labels,
        false,
        None,
        None,
    );
    let top = 52.0;
    let bottom = top + c.graph_height;
    let count = hourly.len();
    let series: [(&str, &str, &str, fn(&HourlyBucket) -> f64); 3] = [
        ("p99Ms", "#7c3aed", "rgba(124, 58, 237, 0.25)", |h| h.p99_ms),
        ("p95Ms", "#2563eb", "rgba(37, 99, 235, 0.35)", |h| h.p95_ms),
        ("avgMs", "#0d9488", "rgba(13, 148, 136, 0.3)", |h| h.avg_ms),
    ];
    for (idx, (_, stroke, fill, get)) in series.iter().enumerate() {
        let pts: Vec<(f64, f64)> = hourly
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let x = c.get_x(i, count);
                let y = top + c.graph_height - (get(h) / max_ms.max(1.0)) * c.graph_height;
                (x, y.max(top).min(bottom))
            })
            .collect();
        let gid = format!("g{idx}");
        draw_series(&mut c.out, &pts, bottom, stroke, fill, &gid);
    }
    // Legend at y=24.
    let legend = [
        ("P99 Latency", "#7c3aed"),
        ("P95 Latency", "#2563eb"),
        ("Avg Latency", "#0d9488"),
    ];
    for (i, (label, color)) in legend.iter().enumerate() {
        let x = 340.0 + i as f64 * 90.0;
        circle(&mut c.out, x, 24.0, 4.0, color);
        text(&mut c.out, x + 8.0, 25.0, "start", 500, 11.0, "#334155", label);
    }
    c.out.push_str("</svg>");
    c.out
}

fn bar_path(out: &mut String, x_center: f64, bar_y: f64, bar_w: f64, bar_h: f64, radii: [f64; 4]) {
    let mut d = String::new();
    rounded_rect_path(
        &mut d,
        x_center - bar_w / 2.0,
        bar_y,
        bar_w,
        bar_h,
        radii,
    );
    out.push_str(&format!("<path d=\"{d}\"/>"));
}

/// `Hourly Request Volume & Errors` (bars + error line).
pub fn render_hourly_volume(hourly: &[HourlyBucket]) -> String {
    if hourly.is_empty() {
        return String::new();
    }
    let max_count = (hourly.iter().map(|h| h.count).max().unwrap_or(0) as f64 * 1.15).ceil();
    let max_count = if max_count == 0.0 { 10.0 } else { max_count };
    let max_error = (hourly.iter().map(|h| h.error_count).max().unwrap_or(0) as f64 * 1.2).ceil();
    let max_error = if max_error == 0.0 { 5.0 } else { max_error };
    let labels: Vec<String> = hourly.iter().map(|h| h.label.clone()).collect();
    let mut c = frame(
        "Hourly Request Volume & Errors",
        (52.0, 65.0, 48.0, 65.0),
        max_count,
        &|v| format_num(v as u64),
        &labels,
        true,
        Some(&|v| format_num(v as u64)),
        Some(max_error),
    );
    let top = 52.0;
    let bottom = top + c.graph_height;
    let count = hourly.len();
    let bar_w = (470.0 / count as f64 * 0.6).clamp(4.0, 22.0);
    for (i, h) in hourly.iter().enumerate() {
        let x_center = c.get_x(i, count);
        let bar_h = (h.count as f64 / max_count) * c.graph_height;
        let bar_y = bottom - bar_h;
        c.out.push_str("<g fill=\"#3b82f6\">");
        bar_path(&mut c.out, x_center, bar_y, bar_w, bar_h, [3.0, 3.0, 0.0, 0.0]);
        c.out.push_str("</g>");
    }
    // Error line + dots.
    let pts: Vec<(f64, f64)> = hourly
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let x = c.get_x(i, count);
            let y = top + c.graph_height - (h.error_count as f64 / max_error) * c.graph_height;
            (x, y)
        })
        .collect();
    let mut d = String::new();
    for (i, p) in pts.iter().enumerate() {
        d.push_str(&format!(
            "{} {} {}",
            if i == 0 { "M" } else { "L" },
            n(p.0),
            n(p.1)
        ));
    }
    c.out.push_str(&format!(
        "<path d=\"{d}\" fill=\"none\" stroke=\"#ef4444\" stroke-width=\"2\" stroke-linejoin=\"round\"/>"
    ));
    for p in &pts {
        circle(&mut c.out, p.0, p.1, 3.0, "#ef4444");
    }
    // Legend.
    c.out
        .push_str("<rect x=\"390\" y=\"18\" width=\"12\" height=\"10\" fill=\"#3b82f6\"/>");
    text(
        &mut c.out,
        406.0,
        25.0,
        "start",
        500,
        11.0,
        "#334155",
        "Total Requests",
    );
    circle(&mut c.out, 505.0, 24.0, 4.0, "#ef4444");
    text(
        &mut c.out,
        515.0,
        25.0,
        "start",
        500,
        11.0,
        "#334155",
        "Errors (4xx/5xx)",
    );
    c.out.push_str("</svg>");
    c.out
}

/// `Latency Distribution Breakdown`.
pub fn render_distribution(rows: &[AggregatedEndpoint]) -> String {
    let mut buckets = create_latency_buckets();
    fill_latency_buckets(&mut buckets, rows);
    let max_count = (buckets.iter().map(|b| b.1).max().unwrap_or(0) as f64 * 1.15).ceil();
    let max_count = if max_count == 0.0 { 10.0 } else { max_count };
    let labels: Vec<String> = buckets.iter().map(|b| b.0.clone()).collect();
    let mut c = frame(
        "Latency Distribution Breakdown",
        (52.0, 24.0, 48.0, 68.0),
        max_count,
        &|v| format_num(v as u64),
        &labels,
        true,
        None,
        None,
    );
    let top = 52.0;
    let bottom = top + c.graph_height;
    let count = buckets.len();
    let bar_w = (508.0 / count as f64 * 0.65).min(42.0);
    for (i, (_, value)) in buckets.iter().enumerate() {
        let x_center = c.get_x(i, count);
        let bar_h = (*value as f64 / max_count) * c.graph_height;
        let bar_y = bottom - bar_h;
        c.out
            .push_str(&format!("<g fill=\"{}\">", PALETTE_DISTRIBUTION[i]));
        bar_path(&mut c.out, x_center, bar_y, bar_w, bar_h, [4.0, 4.0, 0.0, 0.0]);
        c.out.push_str("</g>");
        if *value > 0 {
            text(
                &mut c.out,
                x_center,
                bar_y - 5.0,
                "middle",
                600,
                11.0,
                "#0f172a",
                &format_num(*value),
            );
        }
    }
    c.out.push_str("</svg>");
    c.out
}

/// `Daily Trend: Requests & Latency`.
pub fn render_daily_trend(daily: &[DaySummary]) -> String {
    if daily.is_empty() {
        return String::new();
    }
    let max_count = (daily.iter().map(|d| d.count).max().unwrap_or(0) as f64 * 1.15).ceil();
    let max_count = if max_count == 0.0 { 10.0 } else { max_count };
    let max_p95 = (daily.iter().map(|d| d.p95_ms).fold(0.0f64, f64::max) * 1.2).ceil();
    let max_p95 = if max_p95 == 0.0 { 100.0 } else { max_p95 };
    let labels: Vec<String> = daily
        .iter()
        .map(|d| format_date(Some(&d.date)))
        .collect();
    let mut c = frame(
        "Daily Trend: Requests & Latency",
        (52.0, 65.0, 48.0, 65.0),
        max_count,
        &|v| format_num(v as u64),
        &labels,
        true,
        Some(&|v| format_ms(v)),
        Some(max_p95),
    );
    let top = 52.0;
    let bottom = top + c.graph_height;
    let count = daily.len();
    let bar_w = (470.0 / count as f64 * 0.6).clamp(6.0, 32.0);
    for (i, day) in daily.iter().enumerate() {
        let x_center = c.get_x(i, count);
        let bar_h = (day.count as f64 / max_count) * c.graph_height;
        let bar_y = bottom - bar_h;
        c.out.push_str("<g fill=\"#3b82f6\">");
        bar_path(&mut c.out, x_center, bar_y, bar_w, bar_h, [3.0, 3.0, 0.0, 0.0]);
        c.out.push_str("</g>");
    }
    let pts: Vec<(f64, f64)> = daily
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let x = c.get_x(i, count);
            let y = top + c.graph_height - (d.p95_ms / max_p95) * c.graph_height;
            (x, y)
        })
        .collect();
    let mut d = String::new();
    for (i, p) in pts.iter().enumerate() {
        d.push_str(&format!(
            "{} {} {}",
            if i == 0 { "M" } else { "L" },
            n(p.0),
            n(p.1)
        ));
    }
    c.out.push_str(&format!(
        "<path d=\"{d}\" fill=\"none\" stroke=\"#7c3aed\" stroke-width=\"2\" stroke-linejoin=\"round\"/>"
    ));
    for p in &pts {
        circle(&mut c.out, p.0, p.1, 3.5, "#7c3aed");
    }
    c.out
        .push_str("<rect x=\"360\" y=\"18\" width=\"12\" height=\"10\" fill=\"#3b82f6\"/>");
    text(
        &mut c.out,
        376.0,
        25.0,
        "start",
        500,
        11.0,
        "#334155",
        "Total Requests",
    );
    circle(&mut c.out, 475.0, 24.0, 4.0, "#7c3aed");
    text(
        &mut c.out,
        485.0,
        25.0,
        "start",
        500,
        11.0,
        "#334155",
        "P95 Latency",
    );
    c.out.push_str("</svg>");
    c.out
}

/// Rasterize an SVG string to PNG bytes at 2x (canvas `devicePixelRatio = 2`).
pub fn svg_to_png(svg: &str) -> Option<Vec<u8>> {
    if svg.is_empty() {
        return None;
    }
    let mut opt = resvg::usvg::Options::default();
    let mut db = resvg::usvg::fontdb::Database::new();
    db.load_system_fonts();
    for dir in ["assets/fonts", "../assets/fonts"] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf") {
                        let _ = db.load_font_file(p);
                    }
                }
            }
        }
    }
    opt.fontdb = std::sync::Arc::new(db);
    let tree = resvg::usvg::Tree::from_str(svg, &opt).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(
        (CHART_W * DPR) as u32,
        (CHART_H * DPR) as u32,
    )?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(DPR as f32, DPR as f32),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().ok()
}

/// Port of `generateAllChartImages` (default light theme).
pub fn generate_all(rows: &[AggregatedEndpoint], hourly: &[HourlyBucket], daily: &[DaySummary]) -> ChartImages {
    ChartImages {
        time_vs_latency: svg_to_png(&render_time_vs_latency(hourly))
            .map(|b| base64(&b))
            .unwrap_or_default(),
        hourly_volume: svg_to_png(&render_hourly_volume(hourly))
            .map(|b| base64(&b))
            .unwrap_or_default(),
        distribution: svg_to_png(&render_distribution(rows))
            .map(|b| base64(&b))
            .unwrap_or_default(),
        daily_trend: if daily.len() > 1 {
            svg_to_png(&render_daily_trend(daily)).map(|b| base64(&b))
        } else {
            None
        },
    }
}

fn base64(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let v = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLE[(v >> 18) as usize & 63] as char);
        out.push(TABLE[(v >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(v >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[v as usize & 63] as char
        } else {
            '='
        });
    }
    out
}
