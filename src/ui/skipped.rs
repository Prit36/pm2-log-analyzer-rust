//! `SkippedDisclosure` — port of `src/components/SkippedDisclosure.tsx`.

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::ui::{icons, style};
use crate::utils::format::format_num;

const SAMPLE_ROW_HEIGHT: f32 = 18.0;

pub fn view(app: &App) -> Option<Element<'_, Message>> {
    if !app.analysis.has_data {
        return None;
    }
    let result = app.analysis.result.as_ref()?;
    if result.unmatched_count == 0 {
        return None;
    }

    let is_dark = app.analysis.is_dark();
    let sample = &result.unmatched_sample;
    let summary = row![
        text(format!("{} lines skipped", format_num(result.unmatched_count)))
            .size(12)
            .font(style::MEDIUM)
            .style(style::text_muted),
        text("(non-HTTP / unmatched)")
            .size(12)
            .style(style::text_faint),
    ]
    .spacing(8)
    .align_y(Center);

    let toggle = button(
        row![
            icons::icon(
                if app.skipped_open {
                    "arrow-down"
                } else {
                    "arrow-right"
                },
                12.0,
                if is_dark {
                    style::SLATE_400
                } else {
                    style::SLATE_500
                },
            ),
            summary,
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press(Message::ToggleSkipped)
    .padding([8, 12])
    .width(Fill)
    .style(style::transparent_button);

    let mut body = column![].spacing(0).width(Fill);
    if app.skipped_open {
        let list_height = (sample.len() as f32 * SAMPLE_ROW_HEIGHT).min(192.0);
        let mut lines = column![].spacing(2).width(Fill);
        for line in sample {
            lines = lines.push(
                text(line.clone())
                    .size(11)
                    .font(style::MONO)
                    .style(style::text_muted),
            );
        }
        body = body.push(
            container(scrollable(lines).height(Length::Fixed(list_height)))
                .padding(Padding {
                    top: 8.0,
                    right: 12.0,
                    bottom: 8.0,
                    left: 12.0,
                })
                .width(Fill),
        );
        if result.unmatched_count > sample.len() as u64 {
            body = body.push(
                container(
                    text(format!(
                        "Showing {} of {} samples",
                        sample.len(),
                        format_num(result.unmatched_count)
                    ))
                    .size(11)
                    .style(style::text_faint),
                )
                .padding(Padding {
                    top: 0.0,
                    right: 12.0,
                    bottom: 10.0,
                    left: 12.0,
                }),
            );
        }
    }

    Some(
        container(column![toggle, body].spacing(0))
            .width(Fill)
            .style(style::card)
            .into(),
    )
}
