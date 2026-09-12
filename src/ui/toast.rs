//! `Toast` — port of `src/components/Toast.tsx`.

use iced::widget::{container, text};
use iced::{Bottom, Element, Fill, Padding, Right};

use crate::app::Message;
use crate::ui::style;

pub fn view(message: &str) -> Element<'_, Message> {
    container(
        container(text(message.to_string()).size(12).font(style::MEDIUM))
            .padding(Padding {
                top: 10.0,
                right: 14.0,
                bottom: 10.0,
                left: 14.0,
            })
            .style(style::toast),
    )
    .width(Fill)
    .height(Fill)
    .align_x(Right)
    .align_y(Bottom)
    .padding(16)
    .into()
}
