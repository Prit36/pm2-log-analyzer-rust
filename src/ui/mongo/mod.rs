//! MongoDB analyzer UI — port of `src/components/mongo/*`.

pub mod chart;
pub mod diagnostics;
pub mod filters;
pub mod ingest;
pub mod kpi;
pub mod modal;
pub mod pattern_table;
pub mod slow_query_table;
pub mod user_activity;

use iced::widget::{column, container, text};
use iced::{Element, Fill, Padding};

use crate::app::{App, Message};
use crate::ui::style;
use crate::ui::boxed_text;
use crate::utils::format::{format_bytes, format_date};

/// `MongoAppView` — ingest, then KPIs/filters/one active view, then the footer.
pub fn view(app: &App) -> Element<'_, Message> {
    let mut content = column![ingest::view(app)].spacing(16).width(Fill);

    if app.mongo.has_data {
        content = content
            .push(kpi::view(app))
            .push(filters::view(app));
        content = content.push(match app.mongo.active_view {
            crate::store::mongo_store::MongoActiveView::Patterns => pattern_table::view(app),
            crate::store::mongo_store::MongoActiveView::SlowQueries => slow_query_table::view(app),
            crate::store::mongo_store::MongoActiveView::Charts => chart::view(app),
            crate::store::mongo_store::MongoActiveView::Diagnostics => diagnostics::view(app),
            crate::store::mongo_store::MongoActiveView::Users => user_activity::view(app),
        });
    }

    content = content.push(footer()).push(modal::view(app));
    content.into()
}

/// Header block for the Mongo mode (`MongoHeaderInfo`).
pub fn header_info(app: &App) -> Element<'_, Message> {
    let mongo = &app.mongo;
    let dates = mongo
        .result
        .as_ref()
        .map(|result| result.dates.clone())
        .unwrap_or_default();
    let source_label = match mongo.source_kind {
        crate::store::analysis_store::SourceKind::File => mongo.file_name.as_ref().map(|name| {
            match mongo.file_size {
                Some(size) => format!("{name} · {}", format_bytes(size)),
                None => name.clone(),
            }
        }),
        crate::store::analysis_store::SourceKind::Paste => Some("Pasted text".to_string()),
        crate::store::analysis_store::SourceKind::None => None,
    };

    let mut heading = iced::widget::row![
        crate::ui::icons::icon("database", 20.0, style::EMERALD_600),
        boxed_text(
            "MongoDB Log Analyzer",
            16.0,
            style::TIGHT_SEMIBOLD,
            crate::ui::lh::TEXT_BASE,
            style::text_heading,
        ),
    ]
    .spacing(8)
    .align_y(iced::Center);

    if let Some(badge) = date_range_badge(&dates) {
        heading = heading.push(
            container(boxed_text(
                badge,
                11.0,
                style::MEDIUM,
                crate::ui::lh::TEXT_11,
                style::text_emerald,
            ))
            .padding([2, 8])
            .style(style::emerald_pill),
        );
    }

    let subtitle = source_label.unwrap_or_else(|| {
        "Slow queries, COLLSCAN detection & smart index advisor".to_string()
    });
    let is_mono = mongo.source_kind != crate::store::analysis_store::SourceKind::None;

    iced::widget::column![
        heading,
        boxed_text(
            subtitle,
            12.0,
            if is_mono { style::MONO } else { style::REGULAR },
            crate::ui::lh::TEXT_XS,
            style::text_muted,
        ),
    ]
    .spacing(2)
    .into()
}

fn date_range_badge(dates: &[String]) -> Option<String> {
    match dates {
        [] => None,
        [only] => Some(format_date(Some(only.as_str()))),
        [first, ..] => Some(format!(
            "{} → {} ({} days)",
            format_date(Some(first.as_str())),
            format_date(dates.last().map(String::as_str)),
            dates.len()
        )),
    }
}

fn footer() -> Element<'static, Message> {
    container(
        text("MongoDB JSON Log Analyzer · Parses locally · Logs never leave your machine")
            .size(11)
            .style(style::text_faint),
    )
    .width(Fill)
    .center_x(Fill)
    .padding(Padding {
        top: 8.0,
        right: 0.0,
        bottom: 24.0,
        left: 0.0,
    })
    .into()
}
