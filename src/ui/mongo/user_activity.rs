//! `MongoUserActivityPanel` — per-user operation mix, latency and collections.

use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::core::mongo_models::MongoUserActivity;
use crate::ui::{icons, style};
use crate::utils::format::{format_ms, format_num, format_ratio, round_to_tenth};

pub fn view(app: &App) -> Element<'_, Message> {
    let Some(result) = app.mongo.result.as_ref() else {
        return container(text("")).into();
    };

    if result.users.is_empty() {
        return container(
            column![
                icons::icon("users", 32.0, style::SLATE_400),
                text("No authenticated users in this window")
                    .size(14)
                    .font(style::SEMIBOLD)
                    .style(style::text_body),
                text("User activity appears when logs contain auth or client metadata.")
                    .size(12)
                    .style(style::text_muted),
            ]
            .spacing(6)
            .align_x(Center),
        )
        .width(Fill)
        .height(Length::Fixed(256.0))
        .center_x(Fill)
        .center_y(Length::Fixed(256.0))
        .style(style::mongo_card)
        .into();
    }

    let selected = app.mongo.active_user_detail.as_ref();
    let mut content = column![].spacing(12).width(Fill);

    for user in &result.users {
        let is_selected = selected.is_some_and(|active| active.user_name == user.user_name);
        content = content.push(user_card(user, is_selected));
    }

    scrollable(content).height(Length::Fixed(600.0)).into()
}

fn user_card(user: &MongoUserActivity, expanded: bool) -> Element<'static, Message> {
    let header = row![
        container(icons::icon("user", 16.0, style::EMERALD_600))
            .padding(6)
            .style(style::ingest_badge),
        column![
            text(if user.user_name.is_empty() {
                "(unauthenticated)".to_string()
            } else {
                user.user_name.clone()
            })
            .size(13)
            .font(style::SEMIBOLD)
            .style(style::text_heading),
            text(format!(
                "{}{}",
                if user.app_name.is_empty() {
                    String::new()
                } else {
                    format!("{} · ", user.app_name)
                },
                if user.auth_db.is_empty() {
                    "auth: unknown".to_string()
                } else {
                    format!("auth: {}", user.auth_db)
                }
            ))
            .size(11)
            .style(style::text_muted),
        ]
        .spacing(2),
        space().width(Fill),
        stat_block("Slow queries", format_num(user.slow_query_count), style::text_accent),
        stat_block("COLLSCANs", format_num(user.collscan_count), style::text_amber),
        stat_block("p95", format_ms(user.p95_duration_ms as f64), style::text_body),
        stat_block("Scan ratio", format_ratio(round_to_tenth(user.scan_ratio)), style::text_body),
        icons::icon(
            if expanded { "arrow-down" } else { "arrow-right" },
            14.0,
            style::SLATE_400,
        ),
    ]
    .spacing(12)
    .align_y(Center);

    let mut card = column![
        button(header)
            .on_press(Message::MongoOpenUser(Some(Box::new(user.clone()))))
            .padding(0)
            .style(style::transparent_button)
    ]
    .spacing(10)
    .width(Fill);

    if expanded {
        card = card.push(crate::ui::hrule(style::divider));
        card = card.push(row![
            stat_block("Operations", format_num(user.total_operations), style::text_body),
            stat_block("Total time", format!("{:.1}s", user.total_duration_ms as f64 / 1000.0), style::text_body),
            stat_block("Avg", format_ms(user.avg_duration_ms), style::text_body),
            stat_block("Max", format_ms(user.max_duration_ms as f64), style::text_body),
            stat_block("Docs examined", format_num(user.total_docs_examined), style::text_body),
            stat_block("Returned", format_num(user.total_returned), style::text_body),
            stat_block("Auth ok/fail", format!("{} / {}", format_num(user.auth_success_count), format_num(user.auth_fail_count)), style::text_body),
        ]
        .spacing(16));

        card = card.push(
            row![
                detail_list(
                    "Operation mix",
                    user.operations
                        .iter()
                        .map(|(op, count)| format!("{op} · {}", format_num(*count)))
                        .collect(),
                ),
                detail_list(
                    "Top collections",
                    user.top_collections
                        .iter()
                        .map(|collection| {
                            format!(
                                "{} · {} ({}x, {:.1}s)",
                                collection.ns,
                                format_num(collection.count),
                                collection.collscan_count,
                                collection.total_duration_ms as f64 / 1000.0
                            )
                        })
                        .collect(),
                ),
                detail_list(
                    "Client IPs",
                    if user.client_ips.is_empty() {
                        vec!["—".to_string()]
                    } else {
                        user.client_ips.clone()
                    },
                ),
            ]
            .spacing(16),
        );

        card = card.push(
            text(format!(
                "Active {} → {}",
                short_time(&user.first_active),
                short_time(&user.last_active)
            ))
            .size(11)
            .style(style::text_faint),
        );
    }

    container(card)
        .width(Fill)
        .padding(14)
        .style(move |theme| style::user_card(theme, expanded))
        .into()
}

fn stat_block(
    label: &'static str,
    value: String,
    value_style: fn(&iced::Theme) -> iced::widget::text::Style,
) -> Element<'static, Message> {
    column![
        text(label)
            .size(10)
            .font(style::MEDIUM)
            .style(style::text_faint),
        text(value)
            .size(14)
            .font(style::SEMIBOLD)
            .style(value_style),
    ]
    .spacing(1)
    .width(Length::Fixed(96.0))
    .into()
}

fn detail_list(title: &'static str, items: Vec<String>) -> Element<'static, Message> {
    let mut list = column![
        text(title)
            .size(11)
            .font(style::SEMIBOLD)
            .style(style::text_muted)
    ]
    .spacing(3)
    .width(Fill);

    for item in items.into_iter().take(8) {
        list = list.push(
            text(item)
                .size(11)
                .font(style::MONO)
                .style(style::text_body),
        );
    }

    list.into()
}

fn short_time(timestamp: &str) -> String {
    timestamp
        .get(11..19)
        .map(|slice| slice.to_string())
        .unwrap_or_else(|| timestamp.to_string())
}

/// Kept for the empty-state padding symmetry.
#[allow(dead_code)]
const CARD_PADDING: Padding = Padding {
    top: 14.0,
    right: 14.0,
    bottom: 14.0,
    left: 14.0,
};
