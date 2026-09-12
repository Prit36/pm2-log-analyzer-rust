//! `MongoKpiRow` — six MongoDB KPI cards.

use iced::widget::{column, container, row, space, text};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::ui::style;
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
        format_num(summary.slow_query_count),
        Some(text_style_neutral),
        format!("{} patterns · {} colls", summary.unique_patterns, summary.unique_collections),
        None,
    ));

    cards = cards.push(card(
        "COLLSCANs",
        if has_collscan { "flame" } else { "alert-triangle" },
        if has_collscan { style::AMBER_600 } else { style::SLATE_400 },
        format_num(summary.collscan_count),
        Some(if has_collscan { text_style_amber } else { text_style_neutral }),
        if has_collscan {
            format!("{collscan_percent}% unindexed scans")
        } else {
            "Zero table scans".to_string()
        },
        if has_collscan { Some(style::amber_card) } else { None },
    ));

    cards = cards.push(card(
        "P95 Latency",
        "clock",
        style::BLUE_600,
        format_ms(summary.p95_duration_ms as f64),
        Some(text_style_accent),
        format!(
            "Avg: {} · P99: {}",
            format_ms(summary.avg_duration_ms),
            format_ms(summary.p99_duration_ms as f64)
        ),
        None,
    ));

    cards = cards.push(card(
        "Max Duration",
        "flame",
        style::ROSE_600,
        format_ms(summary.max_duration_ms as f64),
        Some(text_style_danger),
        format!("P50: {}", format_ms(summary.p50_duration_ms as f64)),
        None,
    ));

    let scan_hot = summary.overall_scan_ratio > 100.0;
    let docs_value_style: Option<StyleFn> = Some(if scan_hot {
        text_style_danger
    } else {
        text_style_neutral
    });
    cards = cards.push(card(
        "Docs Examined",
        "network",
        style::EMERALD_600,
        format_num(summary.total_docs_examined),
        docs_value_style,
        if summary.overall_scan_ratio > 1.0 {
            format!("{} scan ratio", format_ratio(summary.overall_scan_ratio))
        } else {
            "Direct index hits".to_string()
        },
        None,
    ));

    cards = cards.push(card(
        "Diagnostics",
        "shield-alert",
        style::AMBER_500,
        if connections.peak_concurrent > 0 {
            format!("{} peak", format_num(connections.peak_concurrent))
        } else {
            "Normal".to_string()
        },
        Some(text_style_neutral),
        if error_events > 0 {
            format!("{} events ({} types)", format_num(error_events), error_types)
        } else {
            "No engine errors".to_string()
        },
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

#[allow(clippy::too_many_arguments)]
fn card<'a>(
    label: &'a str,
    icon: &'static str,
    icon_color: iced::Color,
    value: String,
    value_style: Option<StyleFn>,
    caption: String,
    surface: Option<fn(&iced::Theme) -> iced::widget::container::Style>,
) -> Element<'a, Message> {
    let is_amber = surface.is_some();
    let header = row![
        text(label)
            .size(12)
            .font(if is_amber { style::SEMIBOLD } else { style::MEDIUM })
            .style(if is_amber { style::text_amber } else { style::text_muted }),
        space().width(Fill),
        crate::ui::icons::icon(icon, 16.0, icon_color),
    ]
    .align_y(Center);

    let value_text = text(value).size(20).font(style::BOLD);
    let value_text = match value_style {
        Some(style_fn) => value_text.style(style_fn),
        None => value_text.style(style::text_heading),
    };

    container(column![
        header,
        container(column![
            value_text,
            text(caption).size(11).style(if is_amber {
                style::text_amber
            } else {
                style::text_muted
            }),
        ]
        .spacing(2))
        .padding(Padding {
            top: 8.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }),
    ])
    .padding(14)
    .width(Length::FillPortion(1))
    .height(Length::Shrink)
    .style(surface.unwrap_or(style::mongo_card))
    .into()
}
