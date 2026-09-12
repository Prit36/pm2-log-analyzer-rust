//! `MongoIngestPanel` — drop zone, Add/Replace, paste panel and progress.

use iced::widget::{button, column, container, progress_bar, row, space, text, text_editor};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::ui::{icons, style};
use crate::utils::format::format_bytes;

const PASTE_LIMIT: usize = 8 * 1024 * 1024;

pub fn view(app: &App) -> Element<'_, Message> {
    let mongo = &app.mongo;
    let mut card = column![].spacing(12).width(Fill);

    if mongo.is_parsing {
        card = card.push(status_row(app));
        card = card.push(progress_strip(app));
    } else if mongo.has_data {
        card = card.push(
            row![
                text(format!(
                    "{} loaded",
                    mongo
                        .file_name
                        .clone()
                        .unwrap_or_else(|| "MongoDB logs".to_string())
                ))
                .size(12)
                .font(style::MEDIUM)
                .style(style::text_body),
                space().width(Fill),
                secondary_action("plus", "Add More Files", Message::MongoBrowse { append: true }),
                secondary_action("upload", "Replace", Message::MongoBrowse { append: false }),
                secondary_action("clipboard-paste", "Paste", Message::MongoPasteToggle),
            ]
            .spacing(8)
            .align_y(Center),
        );
    } else {
        card = card.push(drop_zone(app));
    }

    if let Some(pending) = &app.pending_drop {
        card = card.push(pending_banner(app, pending.len()));
    }
    if mongo.paste_open {
        card = card.push(paste_editor(app));
    }
    if let Some(error) = &mongo.error {
        card = card.push(
            container(text(error.clone()).size(12).style(style::text_danger))
                .width(Fill)
                .padding(8)
                .style(style::danger_surface),
        );
    }

    container(card).width(Fill).style(style::card).into()
}

fn status_row(app: &App) -> Element<'_, Message> {
    let percent = app
        .mongo
        .progress
        .as_ref()
        .map(|progress| progress.percent)
        .unwrap_or(0);
    let stage = app
        .mongo
        .progress
        .as_ref()
        .map(|progress| progress.stage.clone())
        .unwrap_or_else(|| "Parsing".to_string());

    row![
        text(format!("{stage}…"))
            .size(12)
            .style(style::text_muted),
        space().width(Fill),
        text(format!("{percent}%"))
            .size(12)
            .style(style::text_muted),
        button(text("Cancel parsing").size(12).style(style::text_muted))
            .on_press(Message::MongoCancel)
            .padding(0)
            .style(style::btn_ghost),
    ]
    .spacing(12)
    .align_y(Center)
    .into()
}

fn progress_strip(app: &App) -> Element<'_, Message> {
    let percent = app
        .mongo
        .progress
        .as_ref()
        .map(|progress| progress.percent)
        .unwrap_or(0);
    container(
        progress_bar(0.0..=100.0, percent as f32)
            .girth(2.0)
            .style(style::progress),
    )
    .width(Fill)
    .into()
}

fn drop_zone(app: &App) -> Element<'_, Message> {
    let mut content = column![
        icons::icon("database", 32.0, style::EMERALD_600),
        text("Drop MongoDB JSON logs, or choose files")
            .size(14)
            .font(style::MEDIUM)
            .style(style::text_primary),
        text("mongod / mongos JSON logs — slow queries, COLLSCANs and index hints")
            .size(12)
            .style(style::text_muted),
    ]
    .spacing(8)
    .align_x(Center);

    content = content.push(
        row![
            button(
                row![
                    icons::icon("upload", 14.0, iced::Color::WHITE),
                    text("Choose Files")
                        .size(12)
                        .font(style::SEMIBOLD)
                        .style(style::text_white),
                ]
                .spacing(6)
                .align_y(Center),
            )
            .on_press(Message::MongoBrowse { append: false })
            .padding([8, 16])
            .style(style::btn_emerald),
            secondary_action("clipboard-paste", "Paste Text", Message::MongoPasteToggle),
        ]
        .spacing(8)
        .align_y(Center),
    );

    container(content)
        .width(Fill)
        .padding(Padding {
            top: 28.0,
            right: 16.0,
            bottom: 28.0,
            left: 16.0,
        })
        .center_x(Fill)
        .style(move |theme: &iced::Theme| style::drop_zone(theme, app.drag_over))
        .into()
}

fn pending_banner(app: &App, count: usize) -> Element<'_, Message> {
    let loaded = app.mongo.loaded_files.len();
    container(
        column![
            text(format!(
                "You dropped {count} file{}",
                if count > 1 { "s" } else { "" }
            ))
            .size(12)
            .font(style::SEMIBOLD)
            .style(style::text_primary),
            text(format!(
                "{loaded} file{} currently loaded. How would you like to proceed?",
                if loaded > 1 { "s" } else { "" }
            ))
            .size(11)
            .style(style::text_muted),
            row![
                primary_action("plus", "Add & Combine", Message::PendingDropAppend),
                secondary_action("refresh-cw", "Replace", Message::PendingDropReplace),
                button(icons::icon("x", 14.0, style::SLATE_400))
                    .on_press(Message::PendingDropCancel)
                    .padding(4)
                    .style(style::btn_ghost),
            ]
            .spacing(8)
            .align_y(Center),
        ]
        .spacing(6),
    )
    .width(Fill)
    .padding(12)
    .style(style::pending_banner)
    .into()
}

fn paste_editor(app: &App) -> Element<'_, Message> {
    let bytes = app.mongo_paste_text.text().len();
    let over_limit = bytes > PASTE_LIMIT;

    let mut content = column![
        row![
            text("Paste MongoDB Logs")
                .size(14)
                .font(style::BOLD)
                .style(style::text_heading),
            space().width(Fill),
            text(if bytes > 0 {
                format!("{} pasted", format_bytes(bytes as u64))
            } else {
                "Paste MongoDB JSON log lines here…".to_string()
            })
            .size(11)
            .style(if over_limit {
                style::text_danger
            } else {
                style::text_muted
            }),
        ]
        .align_y(Center),
        container(
            text_editor(&app.mongo_paste_text)
                .placeholder("Paste MongoDB JSON log lines here…")
                .on_action(Message::MongoPasteEdit)
                .padding(8)
                .size(12)
                .height(Length::Fixed(160.0))
                .style(style::editor),
        ),
        row![
            text("").size(11),
            space().width(Fill),
            secondary_action("x", "Cancel", Message::MongoPasteToggle),
            button(
                text("Analyze paste")
                    .size(12)
                    .font(style::SEMIBOLD)
                    .style(style::text_white),
            )
            .on_press_maybe((!over_limit && bytes > 0).then_some(Message::MongoPasteAnalyze))
            .padding([8, 16])
            .style(style::btn_emerald),
        ]
        .spacing(8)
        .align_y(Center),
    ]
    .spacing(10);

    if over_limit {
        content = content.push(
            text("Paste is limited to ~8 MB — use a file for larger logs.")
                .size(11)
                .style(style::text_danger),
        );
    }

    content.into()
}

fn primary_action(icon: &'static str, label: &'static str, message: Message) -> Element<'static, Message> {
    button(
        row![
            icons::icon(icon, 14.0, iced::Color::WHITE),
            text(label)
                .size(12)
                .font(style::SEMIBOLD)
                .style(style::text_white),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press(message)
    .padding([6, 12])
    .style(style::btn_emerald)
    .into()
}

fn secondary_action(
    icon: &'static str,
    label: &'static str,
    message: Message,
) -> Element<'static, Message> {
    button(
        row![
            icons::icon(icon, 14.0, style::SLATE_500),
            text(label).size(12).font(style::MEDIUM).style(style::text_body),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press(message)
    .padding([6, 12])
    .style(style::btn_secondary)
    .into()
}
