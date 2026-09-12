//! `MongoKpiRow` — six MongoDB KPI cards.
//!
//! Reference metrics (`dump-dom.mjs --mongo`): each card is `rounded-xl
//! border p-3.5` (content inset 15px in iced's border-over-padding model),
//! the label is `text-xs font-medium` on a 16px line, the value is
//! `text-xl font-bold tracking-tight` (20px/28, -0.025em) and the caption is
//! `text-[11px]` on the font's normal 16.5px line, offset by `mt-2`/`mt-0.5`.

use iced::widget::{column, container, row, space};
use iced::{Center, Element, Fill, Length};

use crate::app::{App, Message};
use crate::ui::{BORDER, boxed_text, lh, style};
use crate::utils::format::{format_ms, format_num, format_ratio};

pub fn view(app: &App) -> Option<Element<'_, Message>> {
    let result = app.mongo.result.as_ref()?;
    let summary = &result.summary;
    let connections = &result.connections;
    let error_types = result.errors.len() as u64;
    let error_events: u64 = result.errors.iter().map(|error| error.count).sum();
    let has_collscan = summary.collscan_count > 0;
    let collscan_percent = if summary.slow_query_count > 0 {
        (summary.collscan_count * 100).div_ceil(summary.slow_query_count)
    } else {
        0
    };

    let mut cards = row![].spacing(12).width(Fill).align_y(iced::Top);

    cards = cards.push(card(
        "Slow Queries",
        "database",
        style::EMERALD_600,
        false,
        format_num(summary.slow_query_count),
        text_style_neutral,
        format!(
            "{} patterns · {} colls",
            summary.unique_patterns, summary.unique_collections
        ),
        text_style_muted,
        style::REGULAR,
        None,
    ));

    cards = cards.push(card(
        "COLLSCANs",
        if has_collscan { "flame" } else { "alert-triangle" },
        if has_collscan { style::AMBER_600 } else { style::SLATE_400 },
        has_collscan,
        format_num(summary.collscan_count),
        if has_collscan { text_style_amber } else { text_style_neutral },
        if has_collscan {
            format!("{collscan_percent}% unindexed scans")
        } else {
            "Zero table scans".to_string()
        },
        text_style_amber,
        style::REGULAR,
        if has_collscan { Some(style::amber_card) } else { None },
    ));

    cards = cards.push(card(
        "P95 Latency",
        "clock",
        style::BLUE_600,
        false,
        format_ms(summary.p95_duration_ms as f64),
        text_style_accent,
        format!(
            "Avg: {} · P99: {}",
            format_ms(summary.avg_duration_ms),
            format_ms(summary.p99_duration_ms as f64)
        ),
        text_style_muted,
        style::REGULAR,
        None,
    ));

    cards = cards.push(card(
        "Max Duration",
        "flame",
        style::ROSE_600,
        false,
        format_ms(summary.max_duration_ms as f64),
        text_style_danger,
        format!("P50: {}", format_ms(summary.p50_duration_ms as f64)),
        text_style_muted,
        style::REGULAR,
        None,
    ));

    // The reference keeps the docs value slate-900 and moves the alert colour
    // (plus `font-medium`) onto the caption.
    let scan_hot = summary.overall_scan_ratio > 100.0;
    cards = cards.push(card(
        "Docs Examined",
        "network",
        style::EMERALD_600,
        false,
        format_num(summary.total_docs_examined),
        text_style_neutral,
        if summary.overall_scan_ratio > 1.0 {
            format!("{} scan ratio", format_ratio(summary.overall_scan_ratio))
        } else {
            "Direct index hits".to_string()
        },
        if scan_hot {
            text_style_danger
        } else {
            text_style_muted
        },
        style::MEDIUM,
        None,
    ));

    cards = cards.push(card(
        "Diagnostics",
        "shield-alert",
        style::AMBER_500,
        false,
        if connections.peak_concurrent > 0 {
            format!("{} peak", format_num(connections.peak_concurrent))
        } else {
            "Normal".to_string()
        },
        text_style_neutral,
        if error_events > 0 {
            format!("{} events ({} types)", format_num(error_events), error_types)
        } else {
            "No engine errors".to_string()
        },
        text_style_muted,
        style::REGULAR,
        None,
    ));

    Some(cards.into())
}

type StyleFn = fn(&iced::Theme) -> iced::widget::text::Style;

fn text_style_neutral(theme: &iced::Theme) -> iced::widget::text::Style {
    style::text_heading(theme)
}
fn text_style_accent(theme: &iced::Theme) -> iced::widget::text::Style {
    style::text_accent(theme)
}
fn text_style_danger(theme: &iced::Theme) -> iced::widget::text::Style {
    style::text_danger(theme)
}
fn text_style_amber(theme: &iced::Theme) -> iced::widget::text::Style {
    style::text_amber(theme)
}
fn text_style_muted(theme: &iced::Theme) -> iced::widget::text::Style {
    style::text_muted(theme)
}

#[allow(clippy::too_many_arguments)]
fn card<'a>(
    label: &'a str,
    icon: &'static str,
    icon_color: iced::Color,
    label_amber: bool,
    value: String,
    value_style: StyleFn,
    caption: String,
    caption_style: StyleFn,
    caption_font: iced::Font,
    surface: Option<fn(&iced::Theme) -> iced::widget::container::Style>,
) -> Element<'a, Message> {
    let header = row![
        boxed_text(
            label,
            12.0,
            if label_amber { style::SEMIBOLD } else { style::MEDIUM },
            lh::TEXT_XS,
            move |theme| if label_amber {
                style::text_amber(theme)
            } else {
                style::text_muted(theme)
            },
        ),
        space().width(Fill),
        crate::ui::icons::icon(icon, 16.0, icon_color),
    ]
    .align_y(Center);

    container(column![
        header,
        column![
            boxed_text(value, 20.0, style::TIGHT_BOLD, lh::TEXT_LG, value_style),
            boxed_text(caption, 11.0, caption_font, lh::TEXT_11, caption_style),
        ]
        .spacing(2),
    ]
    .spacing(8))
    .padding(14.0 + BORDER)
    .width(Length::FillPortion(1))
    .height(Length::Shrink)
    .style(surface.unwrap_or(style::mongo_card))
    .into()
}
