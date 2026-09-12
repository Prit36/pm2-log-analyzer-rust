//! `KpiRow` — port of `src/components/KpiRow.tsx`.

use iced::widget::{column, container, row, space};
use iced::{Border, Center, Element, Fill, FillPortion};

use crate::app::{App, Message};
use crate::ui::style;
use crate::utils::format::{format_ms, format_num};

struct KpiItem {
    label: &'static str,
    value: String,
    accent: bool,
    danger: bool,
}

pub fn view(app: &App) -> Option<Element<'_, Message>> {
    let summary = app.analysis.summary()?;

    let mut items = vec![
        KpiItem {
            label: "REQUESTS",
            value: format_num(summary.matched),
            accent: false,
            danger: false,
        },
        KpiItem {
            label: "AVG",
            value: format_ms(summary.avg),
            accent: false,
            danger: false,
        },
        KpiItem {
            label: "P95",
            value: format_ms(summary.p95_ms),
            accent: true,
            danger: false,
        },
        KpiItem {
            label: "ERRORS",
            value: format_num(summary.errors),
            accent: false,
            danger: summary.errors > 0,
        },
        KpiItem {
            label: "SLOW ≥3S",
            value: format_num(summary.slow),
            accent: false,
            danger: false,
        },
    ];

    if app.analysis.has_cron_events() {
        items.push(KpiItem {
            label: "CRON JOBS",
            value: format_num(
                app.analysis
                    .result
                    .as_ref()
                    .map(|r| r.cron_summary.jobs)
                    .unwrap_or(0),
            ),
            accent: false,
            danger: false,
        });
    }

    let mut tiles = row![].spacing(0).width(Fill).align_y(Center);
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            tiles = tiles.push(
                container(space().width(1.0))
                    .width(1.0)
                    .height(Fill)
                    .style(style::divider),
            );
        }
        // Intrinsic tile height drives the row; the divider stretches to it.
        tiles = tiles.push(tile(item).width(FillPortion(1)));
    }

    let border = if app.analysis.is_dark() {
        style::SLATE_800
    } else {
        style::SLATE_200
    };
    let card = container(tiles)
        .width(Fill)
        .style(move |_theme| iced::widget::container::Style {
            border: Border {
                color: border,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    Some(card.into())
}

fn tile(item: &KpiItem) -> iced::widget::Container<'static, Message> {
    let danger = item.danger;
    let accent = item.accent;
    container(
        column![
            crate::ui::lined_styled(
                item.label,
                10.0,
                style::SEMIBOLD,
                crate::ui::lh::TEXT_10,
                style::text_muted,
            ),
            crate::ui::lined_styled(
                item.value.clone(),
                18.0,
                style::MONO_SEMIBOLD,
                crate::ui::lh::TEXT_LG,
                value_style(danger, accent),
            ),
        ]
        .spacing(4),
    )
    .padding(iced::Padding {
        top: 12.0,
        right: 12.0,
        bottom: 12.0,
        left: 12.0,
    })
    .style(style::kpi_tile)
}

/// `text-rose-600` / `text-blue-600` / `text-slate-900` for the KPI value.
fn value_style(danger: bool, accent: bool) -> fn(&iced::Theme) -> iced::widget::text::Style {
    if danger {
        style::text_danger
    } else if accent {
        style::text_accent
    } else {
        style::text_primary
    }
}

