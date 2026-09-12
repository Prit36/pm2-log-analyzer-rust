//! `IngestPanel` — port of `src/components/IngestPanel.tsx`.

use iced::widget::{
    button, column, container, progress_bar, row, space, text, text_editor,
};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message, PASTE_EDITOR_ID};
use crate::core::pm2::LoadedSource;
use crate::ui::{icons, style};
use crate::utils::format::format_bytes;

const ACCEPT_EXTENSIONS: &[&str] = &["log", "txt", "out", "err", "zip", "gz", "json"];

pub fn is_valid_file_path(path: &std::path::Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if name.ends_with(".zip") || name.ends_with(".gz") {
        return true;
    }
    let ext = name.rsplit('.').next().unwrap_or("");
    if ACCEPT_EXTENSIONS.contains(&ext) {
        return true;
    }
    // `.log.1`, `.log.2`, … and bare numeric suffixes.
    if name.split('.').count() >= 2 {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() >= 2 && parts[parts.len() - 2] == "log" {
            return true;
        }
        if name
            .rsplit('.')
            .next()
            .is_some_and(|e| e.chars().all(|c| c.is_ascii_digit()))
        {
            return true;
        }
    }
    false
}

pub fn pick_files() -> Vec<LoadedSource> {
    let Some(paths) = rfd::FileDialog::new()
        .add_filter(
            "Log files",
            &["log", "txt", "out", "err", "gz", "zip", "json", "1", "2", "3"],
        )
        .pick_files()
    else {
        return Vec::new();
    };
    paths
        .into_iter()
        .filter(|p| is_valid_file_path(p))
        .map(LoadedSource::Path)
        .collect()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let mut card = column![].spacing(0).width(Fill);
    if app.analysis.has_data {
        card = card.push(compact_row(app));
        if app.analysis.is_parsing {
            card = card.push(progress_strip(app));
        }
    } else {
        card = card.push(drop_zone(app));
    }

    if let Some(pending) = &app.pending_drop {
        card = card.push(pending_banner(app, pending.len()));
    }
    if app.analysis.paste_open {
        card = card.push(paste_editor(app));
    }

    container(card)
        .width(Fill)
        .padding(crate::ui::BORDER)
        .style(style::card)
        .into()
}

fn compact_row(app: &App) -> Element<'_, Message> {
    let is_dark = app.analysis.is_dark();
    let is_parsing = app.analysis.is_parsing;

    let files = loaded_files(&app.analysis.loaded_files);

    let actions: Element<'_, Message> = if is_parsing {
        row![
            parse_status(app),
            cancel_button(),
        ]
        .spacing(12)
        .align_y(Center)
        .into()
    } else {
        row![
            primary_action("file-plus", "Add / Append", Message::Browse { append: true }),
            secondary_action("refresh-cw", "Replace", Message::Browse { append: false }),
            secondary_action("clipboard-paste", "Paste", Message::TogglePaste),
        ]
        .spacing(8)
        .into()
    };

    container(
        row![
            row![
                icons::icon(
                    "upload",
                    16.0,
                    if is_dark {
                        style::SLATE_500
                    } else {
                        style::SLATE_400
                    }
                ),
                files,
            ]
            .spacing(8)
            .align_y(Center),
            space().width(Fill),
            actions,
        ]
        .spacing(12)
        .align_y(Center)
        .padding(Padding {
            top: 10.0,
            right: 16.0,
            bottom: 10.0,
            left: 16.0,
        }),
    )
    .width(Fill)
    .style(style::card)
    .into()
}

fn drop_zone(app: &App) -> Element<'_, Message> {
    let is_dark = app.analysis.is_dark();
    let is_parsing = app.analysis.is_parsing;

    let mut content = column![
        icons::icon(
            "upload",
            32.0,
            if is_dark {
                style::SLATE_500
            } else {
                style::SLATE_400
            }
        ),
        text(if is_parsing {
            "Parsing PM2 log file(s)…"
        } else {
            "Drop PM2 / API logs or .zip archive"
        })
        .size(14)
        .font(style::MEDIUM)
        .style(style::text_primary),
        text(".log / .log.1 / .gz / .zip (auto-classifies API & MongoDB logs)")
            .size(12)
            .style(style::text_muted),
    ]
    .spacing(8)
    .align_x(Center);

    if is_parsing {
        let percent = app
            .analysis
            .progress
            .as_ref()
            .map(|progress| progress.percent)
            .unwrap_or(0);
        content = content.push(
            column![
                row![
                    text(
                        app.analysis
                            .progress
                            .as_ref()
                            .map(|progress| progress.stage.clone())
                            .unwrap_or_default()
                    )
                    .size(11)
                    .style(style::text_muted),
                    space().width(Fill),
                    text(format!("{percent}%"))
                        .size(11)
                        .font(style::SEMIBOLD)
                        .style(style::text_accent),
                ]
                .align_y(Center),
                progress_bar(0.0..=100.0, percent as f32)
                    .girth(8.0)
                    .style(style::progress),
                cancel_button(),
            ]
            .spacing(10)
            .align_x(Center)
            .width(Fill),
        );
    } else {
        content = content.push(
            row![
                primary_action(
                    "upload",
                    "Browse files",
                    Message::Browse { append: false }
                ),
                secondary_action("clipboard-paste", "Paste logs", Message::TogglePaste),
            ]
            .spacing(8),
        );
        content = content.push(
            text("Example: 2026-07-24T00:00:10: GET /api/health 200 12.5 ms - 42")
                .size(11)
                .font(style::MONO)
                .style(style::text_faint),
        );
    }

    container(content)
        .width(Fill)
        .padding(Padding {
            top: 40.0,
            right: 24.0,
            bottom: 40.0,
            left: 24.0,
        })
        .center_x(Fill)
        .style(move |theme| style::drop_zone(theme, app.drag_over))
        .into()
}

fn loaded_files(files: &[LoadedSource]) -> Element<'_, Message> {
    if files.is_empty() {
        return text("Logs loaded")
            .size(12)
            .font(style::MEDIUM)
            .style(style::text_primary)
            .into();
    }

    let mut chips = row![text(format!(
        "{} file{}:",
        files.len(),
        if files.len() > 1 { "s" } else { "" }
    ))
    .size(12)
    .font(style::SEMIBOLD)
    .style(style::text_primary)]
    .spacing(6)
    .align_y(Center);

    for file in files.iter().take(5) {
        let name = file.name();
        let size = format_bytes(file.size());
        chips = chips.push(
            container(
                row![
                    crate::ui::boxed_text(
                        name,
                        11.0,
                        style::MONO,
                        crate::ui::lh::TEXT_11,
                        style::text_body,
                    )
                    .width(iced::Length::Shrink),
                    crate::ui::boxed_text(
                        size,
                        10.0,
                        style::REGULAR,
                        crate::ui::lh::TEXT_10,
                        style::text_faint,
                    ),
                ]
                .spacing(4)
                .align_y(Center),
            )
            .padding([2, 8])
            .style(style::file_chip),
        );
    }

    let extra = files.len().saturating_sub(5);
    if extra > 0 {
        chips = chips.push(
            text(format!("+{extra} more"))
                .size(11)
                .style(style::text_muted),
        );
    }

    chips.into()
}

fn parse_status(app: &App) -> Element<'_, Message> {
    let (stage, percent) = app
        .analysis
        .progress
        .as_ref()
        .map(|progress| (progress.stage.clone(), progress.percent))
        .unwrap_or_default();

    row![
        text(stage)
            .size(12)
            .font(style::MONO)
            .style(style::text_muted),
        text(format!("{percent}%"))
            .size(12)
            .font(style::MONO)
            .style(style::text_accent),
    ]
    .spacing(8)
    .align_y(Center)
    .into()
}

fn progress_strip(app: &App) -> Element<'_, Message> {
    let percent = app
        .analysis
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

fn pending_banner(app: &App, count: usize) -> Element<'_, Message> {
    let loaded = app.analysis.loaded_files.len();
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
                primary_action("plus", "Append to current", Message::PendingDropAppend),
                secondary_action("refresh-cw", "Replace current", Message::PendingDropReplace),
                button(icons::icon(
                    "x",
                    14.0,
                    if app.analysis.is_dark() {
                        style::SLATE_400
                    } else {
                        style::SLATE_500
                    }
                ))
                .on_press(Message::PendingDropCancel)
                .padding(4)
                .style(style::btn_ghost),
            ]
            .spacing(8)
            .align_y(Center),
        ]
        .spacing(4),
    )
    .width(Fill)
    .padding(Padding {
        top: 14.0,
        right: 16.0,
        bottom: 14.0,
        left: 16.0,
    })
    .style(style::pending_banner)
    .into()
}

fn paste_editor(app: &App) -> Element<'_, Message> {
    let disabled = app.analysis.is_parsing;
    let editor = text_editor(&app.paste_text)
        .id(iced::widget::Id::new(PASTE_EDITOR_ID))
        .placeholder("Paste PM2 stdout/stderr here…")
        .on_action(Message::PasteEdit)
        .padding(8)
        .size(12)
        .height(Length::Fixed(120.0))
        .style(style::editor);

    let mut analyze = button(
        text("Analyze paste")
            .size(12)
            .font(style::MEDIUM)
            .style(style::text_white),
    )
    .padding([4, 10])
    .style(style::btn_solid);
    if !disabled {
        analyze = analyze.on_press(Message::PasteAnalyze);
    }

    container(
        column![
            text(format!(
                "Paste log lines (not persisted; max ~{})",
                format_bytes(crate::store::analysis_store::PASTE_WARN_BYTES as u64)
            ))
            .size(12)
            .style(style::text_muted),
            editor,
            container(analyze)
                .width(Fill)
                .align_x(iced::Right)
                .padding(iced::Padding {
                    top: 8.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                }),
        ]
        .spacing(6),
    )
    .width(Fill)
    .padding(Padding {
        top: 12.0,
        right: 16.0,
        bottom: 12.0,
        left: 16.0,
    })
    .into()
}

fn primary_action(icon: &'static str, label: &'static str, message: Message) -> Element<'static, Message> {
    button(
        row![
            icons::icon(icon, 14.0, iced::Color::WHITE),
            crate::ui::boxed_text(
                label,
                12.0,
                style::MEDIUM,
                crate::ui::lh::TEXT_XS,
                style::text_white,
            ),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press(message)
    .padding([4, 10])
    .style(style::btn_solid)
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
            crate::ui::boxed_text(
                label,
                12.0,
                style::MEDIUM,
                crate::ui::lh::TEXT_XS,
                style::text_button,
            ),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press(message)
    .padding([5, 11])
    .style(style::btn_secondary)
    .into()
}

fn cancel_button() -> Element<'static, Message> {
    button(
        crate::ui::boxed_text(
            "Cancel",
            12.0,
            style::MEDIUM,
            crate::ui::lh::TEXT_XS,
            style::text_danger,
        ),
    )
    .on_press(Message::CancelParse)
    .padding([5, 11])
    .style(style::btn_danger)
    .into()
}
