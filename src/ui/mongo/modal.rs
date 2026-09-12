//! `MongoQueryDetailModal` — full slow-query document inspector.

use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::ui::{icons, style};
use crate::utils::format::{format_ms, format_num, format_ratio, round_to_tenth};

pub fn view(app: &App) -> Element<'_, Message> {
    let Some(query) = app.mongo.active_slow_query.as_ref() else {
        return container(text("")).into();
    };

    let header = row![
        text("Slow Query Detail")
            .size(14)
            .font(style::BOLD)
            .style(style::text_heading),
        space().width(Fill),
        button(icons::icon("x", 16.0, style::SLATE_400))
            .on_press(Message::MongoOpenSlowQuery(None))
            .padding(4)
            .style(style::btn_ghost),
    ]
    .align_y(Center);

    let facts = row![
        fact("Timestamp", query.timestamp.clone()),
        fact("Duration", format_ms(query.duration_ms as f64)),
        fact("Operation", query.op.clone()),
        fact("Plan", query.plan_summary.clone()),
        fact("Scan ratio", format_ratio(round_to_tenth(query.scan_ratio))),
        fact("Docs / Keys", format!(
            "{} / {}",
            format_num(query.docs_examined),
            format_num(query.keys_examined)
        )),
        fact("Returned", format_num(query.nreturned)),
        fact("Reslen", format_num(query.reslen)),
    ]
    .spacing(16)
    .width(Fill);

    let mut meta = column![
        row![
            meta_pill("ns", query.ns.clone()),
            meta_pill("collection", query.collection.clone()),
            meta_pill("severity", query.severity.clone()),
            if query.is_collscan {
                meta_pill("scan", "COLLSCAN".to_string())
            } else {
                meta_pill("scan", "IXSCAN".to_string())
            },
            meta_pill("user", query.user.clone().unwrap_or_else(|| "—".to_string())),
            meta_pill("remote", query.remote.clone().unwrap_or_else(|| "—".to_string())),
        ]
        .spacing(8)
        .align_y(Center),
        meta_pill("fingerprint", query.fingerprint.clone()),
    ]
    .spacing(8)
    .width(Fill);

    if !query.index_suggestion.as_deref().unwrap_or("").is_empty() {
        meta = meta.push(
            container(
                row![
                    icons::icon("lightbulb", 14.0, style::EMERALD_600),
                    text(query.index_suggestion.clone().unwrap_or_default())
                        .size(11)
                        .font(style::MONO)
                        .style(style::text_emerald),
                ]
                .spacing(6)
                .align_y(Center),
            )
            .padding(8)
            .width(Fill)
            .style(style::index_chip),
        );
    }

    let mut body = column![header, facts, meta].spacing(12).width(Fill);

    body = body.push(json_block("command", &query.command));
    if let Some(originating) = &query.originating_command {
        body = body.push(json_block("originatingCommand", originating));
    }
    if let Some(locks) = &query.locks {
        body = body.push(json_block("locks", locks));
    }
    if let Some(storage) = &query.storage {
        body = body.push(json_block("storage", storage));
    }

    if let Some(hash) = &query.query_hash {
        body = body.push(
            text(format!("queryHash: {hash}"))
                .size(11)
                .font(style::MONO)
                .style(style::text_faint),
        );
    }
    if let Some(plan_cache_key) = &query.plan_cache_key {
        body = body.push(
            text(format!("planCacheKey: {plan_cache_key}"))
                .size(11)
                .font(style::MONO)
                .style(style::text_faint),
        );
    }

    container(
        container(scrollable(body).height(Length::Fixed(520.0)))
            .width(Length::Fill)
            .padding(20)
            .style(style::mongo_card),
    )
    .width(Fill)
    .padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: 12.0,
        left: 0.0,
    })
    .into()
}

fn fact(label: &'static str, value: String) -> Element<'static, Message> {
    column![
        text(label)
            .size(10)
            .font(style::MEDIUM)
            .style(style::text_faint),
        text(value)
            .size(12)
            .font(style::SEMIBOLD)
            .style(style::text_body),
    ]
    .spacing(1)
    .into()
}

fn meta_pill(label: &'static str, value: String) -> Element<'static, Message> {
    container(
        row![
            text(label)
                .size(10)
                .font(style::MEDIUM)
                .style(style::text_faint),
            text(value)
                .size(11)
                .font(style::MONO)
                .style(style::text_body),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .padding([3, 8])
    .style(style::chip(false))
    .into()
}

fn json_block(title: &'static str, value: &serde_json::Value) -> Element<'static, Message> {
    let pretty = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    container(
        column![
            text(title)
                .size(11)
                .font(style::SEMIBOLD)
                .style(style::text_muted),
            container(
                text(pretty)
                    .size(11)
                    .font(style::MONO)
                    .style(style::text_body),
            )
            .width(Fill)
            .padding(10)
            .style(style::code_block),
        ]
        .spacing(4),
    )
    .width(Fill)
    .into()
}
