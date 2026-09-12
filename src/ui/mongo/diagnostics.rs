//! `MongoDiagnosticsPanel` — connections, drivers, client IPs, errors, checkpoints.

use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::ui::{icons, style};
use crate::utils::format::format_num;

pub fn view(app: &App) -> Element<'_, Message> {
    let Some(result) = app.mongo.result.as_ref() else {
        return container(text("")).into();
    };
    let connections = &result.connections;

    let stats = row![
        stat("Accepted", format_num(connections.accepted)),
        stat("Ended", format_num(connections.ended)),
        stat("Peak Concurrent", format_num(connections.peak_concurrent)),
        stat("Auth Success", format_num(connections.auth_success)),
        stat("Auth Failed", format_num(connections.auth_failed)),
    ]
    .spacing(12)
    .width(Fill);

    let drivers: Element<'_, Message> = if connections.drivers.is_empty() {
        muted("No driver handshakes found")
    } else {
        let mut rows = column![].spacing(4).width(Fill);
        for driver in &connections.drivers {
            rows = rows.push(
                row![
                    text(if driver.driver_name.is_empty() {
                        "unknown".to_string()
                    } else {
                        driver.driver_name.clone()
                    })
                    .size(12)
                    .font(style::MONO)
                    .style(style::text_body),
                    text(driver.driver_version.clone())
                        .size(11)
                        .style(style::text_muted),
                    space().width(Fill),
                    text(format!(
                        "{} {}",
                        driver.os_name.trim(),
                        driver.os_version.trim()
                    ))
                    .size(11)
                    .style(style::text_faint),
                    text(format_num(driver.count))
                        .size(11)
                        .font(style::SEMIBOLD)
                        .style(style::text_body),
                ]
                .spacing(8)
                .align_y(Center),
            );
        }
        rows.into()
    };

    let ips: Element<'_, Message> = if connections.client_ips.is_empty() {
        muted("No client addresses recorded")
    } else {
        let mut rows = column![].spacing(4).width(Fill);
        for client in connections.client_ips.iter().take(12) {
            rows = rows.push(
                row![
                    icons::icon("globe", 12.0, style::SLATE_400),
                    text(client.ip.clone())
                        .size(11)
                        .font(style::MONO)
                        .style(style::text_body),
                    space().width(Fill),
                    text(format!("{} conns", format_num(client.count)))
                        .size(11)
                        .style(style::text_muted),
                ]
                .spacing(6)
                .align_y(Center),
            );
        }
        rows.into()
    };

    let errors: Element<'_, Message> = if result.errors.is_empty() {
        muted("No engine errors")
    } else {
        let mut rows = column![].spacing(6).width(Fill);
        for error in result.errors.iter().take(20) {
            rows = rows.push(
                row![
                    container(
                        text(error.severity.clone())
                            .size(10)
                            .font(style::BOLD),
                    )
                    .padding([1, 6])
                    .style({
                        let severity = error.severity.clone();
                        move |theme: &iced::Theme| style::severity_badge(theme, &severity)
                    }),
                    text(error.component.clone())
                        .size(11)
                        .font(style::MEDIUM)
                        .style(style::text_muted),
                    text(short_time(&error.timestamp))
                        .size(11)
                        .font(style::MONO)
                        .style(style::text_faint),
                    space().width(Fill),
                    text(format!("×{}", format_num(error.count)))
                        .size(11)
                        .font(style::SEMIBOLD)
                        .style(style::text_body),
                ]
                .spacing(8)
                .align_y(Center),
            );
            rows = rows.push(
                text(error.msg.clone())
                    .size(11)
                    .style(style::text_muted),
            );
        }
        rows.into()
    };

    let checkpoints: Element<'_, Message> = if result.checkpoints.is_empty() {
        muted("No checkpoint activity")
    } else {
        let mut rows = column![].spacing(4).width(Fill);
        for checkpoint in result.checkpoints.iter().rev().take(12) {
            rows = rows.push(
                row![
                    icons::icon("terminal", 12.0, style::SLATE_400),
                    text(short_time(&checkpoint.timestamp))
                        .size(11)
                        .font(style::MONO)
                        .style(style::text_body),
                    text(checkpoint.msg.clone())
                        .size(11)
                        .style(style::text_muted),
                    space().width(Fill),
                    text(
                        checkpoint
                            .bytes_written
                            .map(format_num)
                            .unwrap_or_default()
                    )
                    .size(11)
                    .style(style::text_faint),
                ]
                .spacing(8)
                .align_y(Center),
            );
        }
        rows.into()
    };

    let mut content = column![
        stats,
        panel("Connections by driver", drivers),
        row![
            container(panel("Top client IPs", ips)).width(Length::FillPortion(1)),
            container(panel("Engine errors", errors)).width(Length::FillPortion(1)),
        ]
        .spacing(12),
        panel("WiredTiger checkpoints", checkpoints),
    ]
    .spacing(12)
    .width(Fill);

    if result.errors.is_empty() && result.checkpoints.is_empty() {
        content = content.push(
            container(
                row![
                    icons::icon("circle-check", 16.0, style::EMERALD_600),
                    text("No errors or checkpoint warnings in this log window")
                        .size(12)
                        .style(style::text_muted),
                ]
                .spacing(8)
                .align_y(Center),
            )
            .padding(12)
            .style(style::mongo_card),
        );
    }

    scrollable(content).height(Length::Fixed(560.0)).into()
}

fn stat(label: &str, value: String) -> Element<'static, Message> {
    container(
        column![
            text(label.to_string())
                .size(11)
                .font(style::MEDIUM)
                .style(style::text_muted),
            text(value)
                .size(18)
                .font(style::BOLD)
                .style(style::text_heading),
        ]
        .spacing(2),
    )
    .padding(12)
    .width(Length::FillPortion(1))
    .style(style::mongo_card)
    .into()
}

fn panel<'a>(title: &'a str, body: Element<'a, Message>) -> Element<'a, Message> {
    container(
        column![
            text(title.to_string())
                .size(12)
                .font(style::SEMIBOLD)
                .style(style::text_body),
            container(body).padding(Padding {
                top: 6.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            }),
        ]
        .spacing(2),
    )
    .width(Fill)
    .padding(14)
    .style(style::mongo_card)
    .into()
}

fn muted<'a>(message: &'a str) -> Element<'a, Message> {
    text(message.to_string())
        .size(11)
        .style(style::text_faint)
        .into()
}

fn short_time(timestamp: &str) -> String {
    timestamp
        .get(11..19)
        .map(|slice| slice.to_string())
        .unwrap_or_else(|| timestamp.to_string())
}

/// Unused import guard kept for symmetry with the other views.
#[allow(dead_code)]
fn _unused<'a>() -> Element<'a, Message> {
    button(text("")).into()
}
