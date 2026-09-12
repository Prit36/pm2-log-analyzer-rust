pub mod api_table;
pub mod charts;
pub mod cron_table;
pub mod filters;
pub mod fonts;
pub mod header;
pub mod icons;
pub mod ingest;
pub mod kpi;
pub mod mongo;
pub mod skipped;
pub mod style;
pub mod toast;
pub mod virtualize;

/// A 1px horizontal rule (`border-b`) used for table section and row separators.
pub fn hrule<'a, Message: 'a>(
    style: fn(&iced::Theme) -> iced::widget::container::Style,
) -> iced::Element<'a, Message> {
    use iced::widget::{container, space};
    use iced::{Fill, Length};

    container(space())
        .width(Fill)
        .height(Length::Fixed(1.0))
        .style(style)
        .into()
}

/// Tailwind line heights, measured from the reference app (IBM Plex ships a
/// 1.5em normal line height at small sizes, text-* utilities pin the rest).
pub mod lh {
    pub const TEXT_10: f32 = 15.0;
    pub const TEXT_11: f32 = 16.5;
    pub const TEXT_XS: f32 = 16.0;
    pub const TEXT_SM: f32 = 20.0;
    pub const TEXT_BASE: f32 = 24.0;
    pub const TEXT_LG: f32 = 28.0;
}

/// Text with an explicit line box, so section heights match the reference.
pub fn lined<'a, Message: 'a>(
    value: impl Into<String>,
    size: f32,
    font: iced::Font,
    line_height: f32,
) -> iced::Element<'a, Message> {
    use iced::widget::{container, text};
    use iced::Length;

    container(text(value.into()).size(size).font(font))
        .height(Length::Fixed(line_height))
        .into()
}

/// [`lined`] with an explicit text colour function.
pub fn lined_styled<'a, Message: 'a>(
    value: impl Into<String>,
    size: f32,
    font: iced::Font,
    line_height: f32,
    style: fn(&iced::Theme) -> iced::widget::text::Style,
) -> iced::Element<'a, Message> {
    use iced::widget::{container, text};
    use iced::Length;

    container(
        text(value.into())
            .size(size)
            .font(font)
            .style(style),
    )
    .height(Length::Fixed(line_height))
    .into()
}
