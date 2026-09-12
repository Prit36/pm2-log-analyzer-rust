//! `MongoDiagnosticsPanel` — four local sub-tabs: errors, connections,
//! collections and checkpoints.
//!
//! Reference geometry: `rounded-xl border p-4` (content inset 17 in iced),
//! `gap-3` column, a `gap-1 border-b pb-3` tab bar of `px-3 py-1.5 text-xs
//! font-semibold rounded-lg` buttons tinted amber/emerald/purple/blue.

use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::store::mongo_store::MongoDiagTab;
use crate::ui::{BORDER, boxed_text, icons, lh, style};
use crate::ui::style::DiagTint;
use crate::utils::format::{format_date_time, format_ms, format_num};

pub fn view(app: &App) -> Element<'_, Message> {
    let Some(result) = app.mongo.result.as_ref() else {
        return container(text("")).into();
    };

    let errors = &result.errors;
    let connections = &result.connections;
    let collections = &result.collections;
    let checkpoints = &result.checkpoints;

    let is_dark = app.analysis.is_dark();
    let tabs = row![
        diag_tab(
            "shield-alert",
            format!("Errors & Warnings ({})", errors.len()),
            MongoDiagTab::Errors,
            DiagTint::Errors,
            app.mongo.diag_tab,
            is_dark,
        ),
        diag_tab(
            "network",
            "Connection Pool & Drivers".to_string(),
            MongoDiagTab::Connections,
            DiagTint::Connections,
            app.mongo.diag_tab,
            is_dark,
        ),
        diag_tab(
            "database",
            format!("Collections Summary ({})", collections.len()),
            MongoDiagTab::Collections,
            DiagTint::Collections,
            app.mongo.diag_tab,
            is_dark,
        ),
        diag_tab(
            "timer",
            format!("Checkpoints ({})", checkpoints.len()),
            MongoDiagTab::Checkpoints,
            DiagTint::Checkpoints,
            app.mongo.diag_tab,
            is_dark,
        ),
    ]
    .spacing(4)
    .align_y(Center);

    let tab_bar = container(tabs)
        .width(Fill)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 12.0 + BORDER,
            left: 0.0,
        })
        .style(style::mongo_filter_divider);

    let body: Element<'_, Message> = match app.mongo.diag_tab {
        MongoDiagTab::Errors => errors_view(errors),
        MongoDiagTab::Connections => connections_view(connections),
        MongoDiagTab::Collections => collections_view(collections),
        MongoDiagTab::Checkpoints => checkpoints_view(checkpoints),
    };

    container(
        column![
            tab_bar,
            container(body).width(Fill).padding(Padding {
                top: 4.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            }),
        ]
        .spacing(12)
        .width(Fill),
    )
    .width(Fill)
    .padding(16.0 + BORDER)
    .style(style::mongo_card)
    .into()
}

fn diag_tab(
    icon: &'static str,
    label: String,
    tab: MongoDiagTab,
    tint: DiagTint,
    active: MongoDiagTab,
    is_dark: bool,
) -> Element<'static, Message> {
    let is_active = active == tab;
    let theme = if is_dark {
        iced::Theme::Dark
    } else {
        iced::Theme::Light
    };
    let icon_color = if is_active {
        style::diag_tab_icon_color(&theme, tint)
    } else {
        style::SLATE_400
    };
    button(
        row![
            icons::icon(icon, 14.0, icon_color),
            boxed_text(
                label,
                12.0,
                style::SEMIBOLD,
                lh::TEXT_XS,
                move |theme: &iced::Theme| style::text_diag_tab(theme, is_active, tint),
            ),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press(Message::MongoDiagTab(tab))
    .padding([6, 12])
    .style(style::diag_tab_button(is_active, tint))
    .into()
}

fn errors_view(errors: &[crate::core::mongo_models::MongoErrorInfo]) -> Element<'_, Message> {
    if errors.is_empty() {
        return container(boxed_text(
            "No engine warnings or errors recorded.",
            12.0,
            style::REGULAR,
            lh::TEXT_XS,
            style::text_muted,
        ))
        .width(Fill)
        .center_x(Fill)
        .padding(Padding {
            top: 24.0,
            right: 0.0,
            bottom: 24.0,
            left: 0.0,
        })
        .into();
    }

    let mut list = column![].spacing(8).width(Fill);
    for error in errors {
        let severity = error.severity.clone();
        let badge_color_is_rose = matches!(severity.as_str(), "E" | "F");
        let badge = container(boxed_text(
            severity,
            10.0,
            style::BOLD,
            lh::TEXT_10,
            if badge_color_is_rose {
                style::text_danger
            } else {
                style::text_amber
            },
        ))
        .padding([2, 6])
        .style(move |theme: &iced::Theme| {
            if badge_color_is_rose {
                style::severity_badge(theme, "E")
            } else {
                style::severity_badge(theme, "W")
            }
        });

        let mut meta = row![boxed_text(
            format!("Component: {}", error.component),
            11.0,
            style::REGULAR,
            lh::TEXT_11,
            style::text_muted,
        )]
        .spacing(8)
        .align_y(Center);
        if let Some(id) = error.id {
            meta = meta.push(boxed_text(
                format!("· ID: {id}"),
                11.0,
                style::REGULAR,
                lh::TEXT_11,
                style::text_muted,
            ));
        }
        meta = meta.push(boxed_text(
            format!("· {}", format_date_time(Some(error.timestamp.as_str()))),
            11.0,
            style::REGULAR,
            lh::TEXT_11,
            style::text_muted,
        ));

        let card = container(
            row![
                row![
                    badge,
                    column![
                        boxed_text(error.msg.clone(), 12.0, style::SEMIBOLD, lh::TEXT_XS, style::text_strong),
                        meta,
                    ]
                    .spacing(4),
                ]
                .spacing(10)
                .align_y(iced::Top),
                space().width(Fill),
                container(boxed_text(
                    format!("{}x", format_num(error.count)),
                    11.0,
                    style::BOLD,
                    lh::TEXT_11,
                    style::text_body,
                ))
                .padding([2, 8])
                .style(style::diag_count_pill),
            ]
            .spacing(12)
            .align_y(iced::Top),
        )
        .width(Fill)
        .padding(12)
        .style(style::diag_note_card);
        list = list.push(card);
    }
    list.into()
}

fn connections_view(
    connections: &crate::core::mongo_models::MongoConnectionStats,
) -> Element<'_, Message> {
    let mut content = column![
        row![
            stat_card(
                "Accepted Connections",
                format_num(connections.accepted),
                None,
            ),
            stat_card("Closed Connections", format_num(connections.ended), None),
            stat_card(
                "Peak Concurrent",
                format_num(connections.peak_concurrent),
                Some(style::text_emerald),
            ),
            stat_card(
                "Auth Success / Fail",
                format!("{} / {}", format_num(connections.auth_success), format_num(connections.auth_failed)),
                None,
            ),
        ]
        .spacing(12)
        .width(Fill),
    ]
    .spacing(16)
    .width(Fill);

    if !connections.drivers.is_empty() {
        let mut drivers = column![].spacing(8).width(Fill);
        for driver in &connections.drivers {
            let name = if driver.driver_name.is_empty() {
                "unknown".to_string()
            } else {
                driver.driver_name.clone()
            };
            let detail = format!(
                "{} · {} {}",
                if driver.platform.is_empty() { "unknown" } else { driver.platform.as_str() },
                driver.os_name.trim(),
                driver.os_version.trim()
            );
            drivers = drivers.push(
                container(
                    row![
                        column![
                            boxed_text(
                                format!("{name} ({})", driver.driver_version),
                                12.0,
                                style::SEMIBOLD,
                                lh::TEXT_XS,
                                style::text_heading,
                            ),
                            boxed_text(detail, 11.0, style::REGULAR, lh::TEXT_11, style::text_muted),
                        ]
                        .spacing(0),
                        space().width(Fill),
                        container(boxed_text(
                            format!("{} conns", format_num(driver.count)),
                            10.0,
                            style::BOLD,
                            lh::TEXT_10,
                            style::text_body,
                        ))
                        .padding([2, 8])
                        .style(style::diag_count_pill),
                    ]
                    .spacing(12)
                    .align_y(Center),
                )
                .width(Fill)
                .padding(10)
                .style(style::diag_note_card),
            );
        }
        content = content.push(
            column![
                crate::ui::lined(
                    "Detected Client Drivers & Frameworks",
                    12.0,
                    style::SEMIBOLD,
                    lh::TEXT_XS,
                ),
                drivers,
            ]
            .spacing(8),
        );
    }

    if !connections.client_ips.is_empty() {
        let mut chips = row![].spacing(8);
        for client in connections.client_ips.iter().take(12) {
            chips = chips.push(
                container(
                    row![
                        boxed_text(client.ip.clone(), 11.0, style::MONO, lh::TEXT_11, style::text_body),
                        container(boxed_text(
                            format_num(client.count),
                            10.0,
                            style::BOLD,
                            lh::TEXT_10,
                            style::text_muted,
                        ))
                        .padding([2, 4])
                        .style(style::diag_ip_count),
                    ]
                    .spacing(6)
                    .align_y(Center),
                )
                .padding([4, 10])
                .style(style::diag_ip_chip),
            );
        }
        content = content.push(
            column![
                crate::ui::lined(
                    "Active Client IP Addresses",
                    12.0,
                    style::SEMIBOLD,
                    lh::TEXT_XS,
                ),
                chips.wrap(),
            ]
            .spacing(8),
        );
    }

    content.into()
}

fn stat_card(
    label: &'static str,
    value: String,
    value_style: Option<fn(&iced::Theme) -> iced::widget::text::Style>,
) -> Element<'static, Message> {
    container(
        column![
            boxed_text(label, 12.0, style::REGULAR, lh::TEXT_XS, style::text_muted),
            boxed_text(
                value,
                18.0,
                style::BOLD,
                lh::TEXT_LG,
                value_style.unwrap_or(style::text_heading),
            ),
        ]
        .spacing(4),
    )
    .padding(12)
    .width(Length::FillPortion(1))
    .style(style::diag_stat_card)
    .into()
}

fn collections_view(collections: &[crate::core::mongo_models::MongoCollectionMetric]) -> Element<'_, Message> {
    let mut table = column![].spacing(0).width(Fill);
    table = table.push(
        container(
            row![
                header_cell("Collection", false, true),
                header_cell("Queries", true, false),
                header_cell("Total Time", true, false),
                header_cell("Avg", true, false),
                header_cell("P95", true, false),
                header_cell("Max", true, false),
                header_cell("COLLSCANs", true, false),
                header_cell("Docs Examined", true, false),
                header_cell("Scan Ratio", true, false),
            ]
            .spacing(0)
            .width(Fill),
        )
        .width(Fill)
        .height(Length::Fixed(32.0))
        .align_y(Center)
        .style(style::diag_table_header),
    );

    for collection in collections {
        let collscan_style: fn(&iced::Theme) -> iced::widget::text::Style =
            if collection.collscan_count > 0 {
                style::text_amber
            } else {
                style::text_faint
            };
        let collscan_font = if collection.collscan_count > 0 {
            style::SEMIBOLD
        } else {
            style::REGULAR
        };
        table = table.push(
            container(
                row![
                    container(boxed_text(
                        collection.collection.clone(),
                        12.0,
                        style::SEMIBOLD,
                        lh::TEXT_XS,
                        style::text_heading,
                    ))
                    .width(Fill),
                    table_cell(
                        format_num(collection.query_count),
                        style::text_body,
                        style::REGULAR,
                    ),
                    table_cell(
                        format!("{:.1}s", collection.total_duration_ms as f64 / 1000.0),
                        style::text_heading,
                        style::BOLD,
                    ),
                    table_cell(format_ms(collection.avg_duration_ms), style::text_muted, style::REGULAR),
                    table_cell(format_ms(collection.p95_duration_ms as f64), style::text_accent, style::SEMIBOLD),
                    table_cell(format_ms(collection.max_duration_ms as f64), style::text_danger, style::SEMIBOLD),
                    table_cell(format_num(collection.collscan_count), collscan_style, collscan_font),
                    table_cell(format_num(collection.total_docs_examined), style::text_muted, style::REGULAR),
                    table_cell(
                        format!("{}x", collection.scan_ratio),
                        style::text_muted,
                        style::MEDIUM,
                    ),
                ]
                .spacing(0)
                .width(Fill)
                .align_y(Center),
            )
            .width(Fill)
            .padding(Padding {
                top: 8.0,
                right: 0.0,
                bottom: 8.0,
                left: 0.0,
            })
            .style(style::diag_table_row),
        );
    }
    table.into()
}

fn header_cell(label: &'static str, right: bool, left: bool) -> Element<'static, Message> {
    let cell = container(boxed_text(label, 11.0, style::SEMIBOLD, lh::TEXT_11, style::text_muted));
    let cell = if right {
        cell.width(Length::Fixed(96.0)).align_x(iced::Right)
    } else if left {
        cell.width(Fill)
    } else {
        cell.width(Length::Fixed(96.0))
    };
    cell.into()
}

fn table_cell(
    value: String,
    text_style: fn(&iced::Theme) -> iced::widget::text::Style,
    font: iced::Font,
) -> Element<'static, Message> {
    container(boxed_text(value, 12.0, font, lh::TEXT_XS, text_style))
        .width(Length::Fixed(96.0))
        .align_x(iced::Right)
        .into()
}

fn checkpoints_view(checkpoints: &[crate::core::mongo_models::MongoCheckpointInfo]) -> Element<'_, Message> {
    if checkpoints.is_empty() {
        return container(boxed_text(
            "No checkpoint events recorded.",
            12.0,
            style::REGULAR,
            lh::TEXT_XS,
            style::text_muted,
        ))
        .width(Fill)
        .center_x(Fill)
        .padding(Padding {
            top: 24.0,
            right: 0.0,
            bottom: 24.0,
            left: 0.0,
        })
        .into();
    }

    let mut list = column![].spacing(8).width(Fill);
    for checkpoint in checkpoints {
        list = list.push(
            container(
                row![
                    boxed_text(
                        format_date_time(Some(checkpoint.timestamp.as_str())),
                        11.0,
                        style::MONO,
                        lh::TEXT_11,
                        style::text_faint,
                    ),
                    boxed_text(
                        checkpoint.msg.clone(),
                        11.0,
                        style::MONO,
                        lh::TEXT_11,
                        style::text_body,
                    ),
                ]
                .spacing(8)
                .align_y(Center),
            )
            .width(Fill)
            .padding(10)
            .style(style::diag_note_card),
        );
    }
    list.into()
}

/// The diagnostics body scrolls with the page; the card is content-sized.
#[allow(dead_code)]
fn _scroll(content: Element<'_, Message>) -> Element<'_, Message> {
    scrollable(content).height(Length::Fixed(560.0)).into()
}
