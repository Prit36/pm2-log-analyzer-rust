//! `AppHeader` — port of `src/components/AppHeader.tsx`.

use iced::widget::{button, container, row, space, text};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::store::analysis_store::SourceKind;
use crate::store::app_mode_store::AppMode;
use crate::ui::{icons, mongo, style};
use crate::utils::format::{format_bytes, format_date};

fn build_date_range_badge(dates: &[String]) -> Option<String> {
    if dates.is_empty() {
        return None;
    }
    if dates.len() > 1 {
        return Some(format!(
            "{} → {} ({} days)",
            format_date(dates.first().map(|s| s.as_str())),
            format_date(dates.last().map(|s| s.as_str())),
            dates.len()
        ));
    }
    Some(format_date(dates.first().map(|s| s.as_str())))
}

fn build_source_label(
    kind: SourceKind,
    file_name: Option<&str>,
    file_size: Option<u64>,
) -> Option<String> {
    match kind {
        SourceKind::File => {
            let name = file_name?;
            match file_size {
                Some(size) => Some(format!("{name} · {}", format_bytes(size))),
                None => Some(name.to_string()),
            }
        }
        SourceKind::Paste => Some("Pasted text".to_string()),
        SourceKind::None => None,
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let is_mongo = app.mode == AppMode::Mongo;
    let is_dark = app.analysis.is_dark();
    let accent = if is_dark {
        style::BLUE_400
    } else {
        style::BLUE_600
    };

    let title: Element<'_, Message> = if is_mongo {
        mongo::header_info(app)
    } else {
        pm2_info(app, accent)
    };

    let switcher = container(
        row![
            tab(
                "PM2 Logs",
                "file-text",
                !is_mongo,
                app.analysis.has_data && is_mongo,
                accent,
                is_dark,
                Message::SetMode(AppMode::Pm2),
            ),
            tab(
                "MongoDB Logs",
                "database",
                is_mongo,
                false,
                if is_dark {
                    style::EMERALD_400
                } else {
                    style::EMERALD_700
                },
                is_dark,
                Message::SetMode(AppMode::Mongo),
            ),
        ]
        .spacing(4),
    )
    .padding(4)
    .style(style::segmented);

    container(
        row![
            container(title).width(Length::Shrink),
            space().width(Fill),
            switcher,
            space().width(Fill),
            row![
                theme_button(is_dark),
                export_button(app, is_mongo, is_dark),
                clear_button(app, is_mongo, is_dark),
            ]
            .spacing(8),
        ]
        .align_y(Center),
    )
    .width(Fill)
    .padding(Padding {
        top: 10.0,
        right: 16.0,
        bottom: 10.0,
        left: 16.0,
    })
    .style(style::header)
    .into()
}

fn pm2_info(app: &App, accent: iced::Color) -> Element<'_, Message> {
    let date_badge = app
        .analysis
        .result
        .as_ref()
        .and_then(|result| build_date_range_badge(&result.dates));
    let source_label = build_source_label(
        app.analysis.source_kind,
        app.analysis.file_name.as_deref(),
        app.analysis.file_size,
    );

    let mut heading = row![
        icons::icon("file-text", 20.0, accent),
        text("PM2 Log Analyzer")
            .size(16)
            .font(style::TIGHT_SEMIBOLD)
            .style(style::text_heading),
    ]
    .spacing(8)
    .align_y(Center);
    if let Some(badge) = date_badge {
        heading = heading.push(
            container(crate::ui::lined_styled(
                badge,
                11.0,
                style::REGULAR,
                crate::ui::lh::TEXT_11,
                style::text_info,
            ))
            .padding([2, 8])
            .style(style::info_pill),
        );
    }

    let (subtitle, is_mono) = match source_label {
        Some(label) => (label, true),
        None => ("API latency & cron insight".to_string(), false),
    };

    iced::widget::column![
        container(heading)
            .height(Length::Fixed(crate::ui::lh::TEXT_BASE))
            .align_y(iced::Center),
        crate::ui::lined(
            subtitle,
            12.0,
            if is_mono {
                style::MONO
            } else {
                style::REGULAR
            },
            crate::ui::lh::TEXT_XS,
        )
    ]
    .spacing(2)
    .into()
}

fn theme_button(is_dark: bool) -> Element<'static, Message> {
    button(icons::icon(
        if is_dark { "sun" } else { "moon" },
        16.0,
        if is_dark {
            style::AMBER_400
        } else {
            style::SLATE_600
        },
    ))
    .on_press(Message::ToggleTheme)
    .padding(7)
    .style(style::btn_secondary)
    .into()
}

fn export_button(app: &App, is_mongo: bool, is_dark: bool) -> Element<'static, Message> {
    let enabled = if is_mongo {
        app.mongo.has_data && !app.mongo.is_parsing
    } else {
        app.analysis.has_data && !app.analysis.is_parsing
    };
    let message = if is_mongo {
        Message::ExportMongo
    } else {
        Message::ExportPm2
    };

    button(
        row![
            icons::icon(
                "download",
                14.0,
                if is_dark {
                    style::SLATE_200
                } else {
                    style::SLATE_700
                }
            ),
            crate::ui::boxed_text(
                "Export",
                12.0,
                style::MEDIUM,
                crate::ui::lh::TEXT_XS,
                style::text_button,
            ),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press_maybe(enabled.then_some(message))
    .padding([7, 13])
    .style(style::btn_secondary)
    .into()
}

fn clear_button(app: &App, is_mongo: bool, is_dark: bool) -> Element<'static, Message> {
    let enabled = if is_mongo {
        app.mongo.has_data || app.mongo.source_kind != SourceKind::None
    } else {
        app.analysis.has_data || app.analysis.source_kind != SourceKind::None
    };
    let message = if is_mongo {
        Message::MongoClear
    } else {
        Message::ClearAll
    };

    button(
        row![
            icons::icon(
                "eraser",
                14.0,
                if is_dark {
                    style::SLATE_200
                } else {
                    style::SLATE_700
                }
            ),
            crate::ui::boxed_text(
                "Clear",
                12.0,
                style::MEDIUM,
                crate::ui::lh::TEXT_XS,
                style::text_button,
            ),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press_maybe(enabled.then_some(message))
    .padding([7, 13])
    .style(style::btn_secondary)
    .into()
}

fn tab(
    label: &'static str,
    icon: &str,
    active: bool,
    show_dot: bool,
    accent: iced::Color,
    is_dark: bool,
    on_press: Message,
) -> Element<'static, Message> {
    let text_color = if active {
        accent
    } else if is_dark {
        style::SLATE_400
    } else {
        style::SLATE_500
    };
    let mut content = row![
        icons::icon(icon, 14.0, text_color),
        crate::ui::boxed_text(
            label,
            12.0,
            style::SEMIBOLD,
            crate::ui::lh::TEXT_XS,
            style::text_tab(active, accent),
        )
    ]
    .spacing(6)
    .align_y(Center);
    if show_dot {
        content = content.push(
            container(space().width(6.0).height(6.0))
                .width(Length::Fixed(6.0))
                .height(Length::Fixed(6.0))
                .style(style::dot(accent)),
        );
    }

    button(content)
        .on_press(on_press)
        .padding([6, 12])
        .style(style::nav_tab(active))
        .into()
}
