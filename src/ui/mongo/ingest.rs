//! `MongoIngestPanel` — the dashed drop zone (empty / loaded / parsing), the
//! pending-drop banner and the paste editor.
//!
//! Reference geometry (`dump-dom.mjs --mongo`): the zone is
//! `rounded-2xl border-2 border-dashed` with `p-6` (24px) or `py-4` once data
//! is loaded, all three states centred inside it. The Add/Replace buttons are
//! `px-3 py-1 text-xs font-medium` pills with 14px icons.

use iced::widget::{button, column, container, progress_bar, row, space, text, text_editor};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::ui::{BORDER, boxed_text, icons, lh, style};
use crate::utils::format::format_bytes;

const PASTE_LIMIT: usize = 8 * 1024 * 1024;

pub fn view(app: &App) -> Element<'_, Message> {
    let mongo = &app.mongo;

    let content: Element<'_, Message> = if mongo.is_parsing {
        status_column(app)
    } else if mongo.has_data {
        loaded_row(app)
    } else {
        empty_state()
    };

    let compact = mongo.has_data && !mongo.is_parsing;
    let is_dark = app.analysis.is_dark();
    let dash_color = if app.drag_over {
        if is_dark { style::EMERALD_400 } else { style::EMERALD_500 }
    } else if is_dark {
        style::SLATE_800
    } else {
        style::SLATE_300
    };
    let zone = crate::ui::dashed_border(
        container(content)
            .width(Fill)
            .padding(Padding {
                top: if compact { 16.0 } else { 24.0 },
                right: 24.0,
                bottom: if compact { 16.0 } else { 24.0 },
                left: 24.0,
            })
            .center_x(Fill)
            .style(move |theme: &iced::Theme| style::mongo_drop_zone(theme, app.drag_over)),
        dash_color,
        2.0,
        16.0,
    );

    let mut card = column![zone].spacing(12).width(Fill);

    if let Some(pending) = &app.pending_drop {
        card = card.push(pending_banner(app, pending.len()));
    }
    if mongo.paste_open {
        card = card.push(paste_editor(app));
    }
    if let Some(error) = &mongo.error {
        card = card.push(
            container(
                boxed_text(
                    error.clone(),
                    12.0,
                    style::REGULAR,
                    lh::TEXT_XS,
                    style::text_danger,
                ),
            )
            .width(Fill)
            .padding(8)
            .style(style::danger_surface),
        );
    }

    card.into()
}

/// Parsing state: spinner, stage/percent legend, progress bar, cancel link.
fn status_column(app: &App) -> Element<'_, Message> {
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

    column![
        icons::icon("refresh-cw", 24.0, style::EMERALD_600),
        column![
            row![
                boxed_text(
                    format!("{stage}…"),
                    12.0,
                    style::REGULAR,
                    lh::TEXT_XS,
                    style::text_muted,
                ),
                space().width(Fill),
                boxed_text(
                    format!("{percent}%"),
                    12.0,
                    style::REGULAR,
                    lh::TEXT_XS,
                    style::text_muted,
                ),
            ]
            .align_y(Center),
            container(
                progress_bar(0.0..=100.0, percent as f32)
                    .girth(Length::Fixed(8.0))
                    .style(style::mongo_progress),
            )
            .width(Fill),
        ]
        .spacing(4),
        button(boxed_text(
            "Cancel parsing",
            12.0,
            style::REGULAR,
            lh::TEXT_XS,
            style::text_muted,
        ))
        .on_press(Message::MongoCancel)
        .padding(0)
        .style(style::btn_ghost),
    ]
    .spacing(12)
    .width(Length::Fixed(448.0))
    .align_x(Center)
    .into()
}

/// Loaded state: `Loaded 1 file` + Add More Files / Replace.
fn loaded_row(app: &App) -> Element<'_, Message> {
    let count = app.mongo.loaded_files.len();
    row![
        boxed_text(
            format!("Loaded {count} file{}", if count == 1 { "" } else { "s" }),
            12.0,
            style::MEDIUM,
            lh::TEXT_XS,
            style::text_body,
        ),
        row![
            pill_button(
                "plus",
                style::EMERALD_600,
                "Add More Files",
                style::MEDIUM,
                Message::MongoBrowse { append: true },
            ),
            pill_button(
                "upload",
                style::SLATE_500,
                "Replace",
                style::MEDIUM,
                Message::MongoBrowse { append: false },
            ),
        ]
        .spacing(8),
    ]
    .spacing(12)
    .align_y(Center)
    .into()
}

/// Empty state: emerald icon tile, heading, hint, Browse / Paste actions.
fn empty_state() -> Element<'static, Message> {
    column![
        container(icons::icon("database", 24.0, style::EMERALD_600))
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .center_x(Length::Fixed(48.0))
            .center_y(Length::Fixed(48.0))
            .style(style::emerald_icon_tile),
        crate::ui::lined(
            "Drop MongoDB logs or .zip archive here",
            14.0,
            style::SEMIBOLD,
            lh::TEXT_SM,
        ),
        crate::ui::lined_styled(
            "Supports mongod log files (.log, .log.1, .gz, .zip) · Auto-classifies API & MongoDB logs",
            12.0,
            style::REGULAR,
            lh::TEXT_XS,
            style::text_muted,
        ),
        row![
            button(
                row![
                    icons::icon("upload", 14.0, iced::Color::WHITE),
                    boxed_text(
                        "Browse Files",
                        12.0,
                        style::SEMIBOLD,
                        lh::TEXT_XS,
                        style::text_white,
                    ),
                ]
                .spacing(6)
                .align_y(Center),
            )
            .on_press(Message::MongoBrowse { append: false })
            .padding([8, 16])
            .style(style::btn_emerald),
            button(
                row![
                    icons::icon("clipboard-paste", 14.0, style::SLATE_500),
                    boxed_text(
                        "Paste Text",
                        12.0,
                        style::SEMIBOLD,
                        lh::TEXT_XS,
                        style::text_button,
                    ),
                ]
                .spacing(6)
                .align_y(Center),
            )
            .on_press(Message::MongoPasteToggle)
            .padding([9, 13])
            .style(style::btn_secondary),
        ]
        .spacing(8)
        .align_y(Center),
    ]
    .spacing(8)
    .align_x(Center)
    .into()
}

/// `inline-flex items-center gap-1.5 rounded-lg border px-3 py-1 text-xs
/// font-medium` with a 14px icon.
fn pill_button(
    icon: &'static str,
    icon_color: iced::Color,
    label: &'static str,
    font: iced::Font,
    message: Message,
) -> Element<'static, Message> {
    button(
        row![
            icons::icon(icon, 14.0, icon_color),
            boxed_text(label, 12.0, font, lh::TEXT_XS, style::text_button),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press(message)
    .padding([4.0 + BORDER, 12.0 + BORDER])
    .style(style::btn_secondary)
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
            boxed_text(
                "Paste MongoDB Logs",
                14.0,
                style::BOLD,
                lh::TEXT_SM,
                style::text_heading,
            ),
            space().width(Fill),
            boxed_text(
                if bytes > 0 {
                    format!("{} pasted", format_bytes(bytes as u64))
                } else {
                    "Paste MongoDB JSON log lines here…".to_string()
                },
                11.0,
                style::REGULAR,
                lh::TEXT_11,
                if over_limit {
                    style::text_danger
                } else {
                    style::text_muted
                },
            ),
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
            boxed_text(
                label,
                12.0,
                style::SEMIBOLD,
                lh::TEXT_XS,
                style::text_white,
            ),
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
            boxed_text(label, 12.0, style::MEDIUM, lh::TEXT_XS, style::text_button),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press(message)
    .padding([6, 12])
    .style(style::btn_secondary)
    .into()
}
