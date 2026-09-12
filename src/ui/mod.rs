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
///
/// CSS centres the font's content box inside the line box; iced always draws
/// the text at the top of its container, so the box is explicitly centred
/// here to keep the glyph baseline at the reference position.
pub fn lined<'a, Message: 'a>(
    value: impl Into<String>,
    size: f32,
    font: iced::Font,
    line_height: f32,
) -> iced::Element<'a, Message> {
    use iced::widget::{container, text};
    use iced::{Center, Length};

    container(text(value.into()).size(size).font(font))
        .height(Length::Fixed(line_height))
        .align_y(Center)
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
    use iced::{Center, Length};

    container(
        text(value.into())
            .size(size)
            .font(font)
            .style(style),
    )
    .height(Length::Fixed(line_height))
    .align_y(Center)
    .into()
}

/// One CSS pixel of `border`, reserved in layout. iced paints borders *over*
/// the container padding instead of adding to the box the way the CSS
/// border-box model does, so every bordered box adds this to its padding.
pub const BORDER: f32 = 1.0;

/// Draws a dashed border over `content` (`border-2 border-dashed`), which iced
/// containers cannot express natively.
pub fn dashed_border<'a, Message: 'a>(
    content: impl Into<iced::Element<'a, Message>>,
    color: iced::Color,
    width: f32,
    radius: f32,
) -> iced::Element<'a, Message> {
    use iced::widget::{Stack, canvas};
    use iced::{Fill, Length};

    let overlay: iced::Element<'a, Message> = canvas::Canvas::new(DashedBorder {
        color,
        width,
        radius,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    Stack::with_children([content.into(), overlay])
        .width(Fill)
        .height(Length::Shrink)
        .into()
}

struct DashedBorder {
    color: iced::Color,
    width: f32,
    radius: f32,
}

impl<Message> iced::widget::canvas::Program<Message> for DashedBorder {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, LineDash, Path, Stroke, stroke};

        const DASH: [f32; 2] = [6.0, 5.0];

        let mut frame = Frame::new(renderer, bounds.size());
        let inset = self.width / 2.0;
        let path = Path::rounded_rectangle(
            iced::Point::new(inset, inset),
            iced::Size::new(
                (bounds.width - self.width).max(0.0),
                (bounds.height - self.width).max(0.0),
            ),
            self.radius.into(),
        );
        frame.stroke(
            &path,
            Stroke {
                width: self.width,
                style: stroke::Style::Solid(self.color),
                line_dash: LineDash {
                    segments: &DASH,
                    offset: 0,
                },
                ..Stroke::default()
            },
        );
        vec![frame.into_geometry()]
    }
}

/// A text label whose line box is pinned, for use inside buttons/rows.
pub fn boxed_text<'a, Message: 'a>(
    value: impl Into<String>,
    size: f32,
    font: iced::Font,
    line_height: f32,
    style: impl Fn(&iced::Theme) -> iced::widget::text::Style + 'a,
) -> iced::widget::Container<'a, Message> {
    use iced::widget::{container, text};
    use iced::{Center, Length};

    container(
        text(value.into())
            .size(size)
            .font(font)
            .wrapping(iced::widget::text::Wrapping::None)
            .style(style),
    )
    .height(Length::Fixed(line_height))
    .align_y(Center)
}
