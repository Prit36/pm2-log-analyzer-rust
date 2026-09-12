//! `ApiTable` — port of `src/components/ApiTable.tsx`.

use iced::widget::text::Wrapping;
use iced::widget::{button, column, container, mouse_area, responsive, row, scrollable, space, text};
use iced::{Center, Element, Fill, Length, Padding, Right};

use crate::app::{App, Message};
use crate::core::models::AggregatedEndpoint;
use crate::store::analysis_store::{ApiSortKey, SortDirection};
use crate::ui::{hrule, icons, style, virtualize};
use crate::utils::format::{format_ms, format_num};
use crate::utils::text_fit;

const ROW_HEIGHT: f32 = 32.0;
const HEADER_HEIGHT: f32 = 32.0;
const OVERSCAN: usize = 6;
/// Numeric column widths, mirroring the reference grid template.
const COL_COUNT: f32 = 56.0;
const COL_AVG: f32 = 58.0;
const COL_P95: f32 = 58.0;
const COL_P99: f32 = 58.0;
const COL_MAX: f32 = 58.0;
const COL_ERRORS: f32 = 56.0;
const NUMERIC_WIDTH: f32 = COL_COUNT + COL_AVG + COL_P95 + COL_P99 + COL_MAX + COL_ERRORS;
/// `px-3` on the row/header grid.
const ROW_PAD: f32 = 12.0;
/// `pr-2` on the endpoint cell.
const CELL_PAD_RIGHT: f32 = 8.0;
/// `gap-2` between badge and path, `gap-1.5` between path and copy icon.
const BADGE_GAP: f32 = 8.0;
const ICON_GAP: f32 = 6.0;
const COPY_ICON: f32 = 12.0;
const BADGE_FONT_SIZE: f32 = 10.0;
const PATH_FONT_SIZE: f32 = 11.0;
/// `min-w-[680px]` on the reference grid.
const MIN_TABLE_WIDTH: f64 = 680.0;

fn header_row(
    filters: &crate::store::analysis_store::AnalysisFilters,
    is_dark: bool,
    hovered: Option<ApiSortKey>,
) -> Element<'static, Message> {
    container(
        row![
            container(
                text("Endpoint")
                    .size(11)
                    .font(style::SEMIBOLD)
                    .style(style::text_muted)
            )
            .width(Fill),
            sort_header("Count", ApiSortKey::Count, filters, COL_COUNT, is_dark, hovered),
            sort_header("Avg", ApiSortKey::AvgMs, filters, COL_AVG, is_dark, hovered),
            sort_header("p95", ApiSortKey::P95Ms, filters, COL_P95, is_dark, hovered),
            sort_header("p99", ApiSortKey::P99Ms, filters, COL_P99, is_dark, hovered),
            sort_header("Max", ApiSortKey::MaxMs, filters, COL_MAX, is_dark, hovered),
            sort_header(
                "Errors",
                ApiSortKey::ErrorCount,
                filters,
                COL_ERRORS,
                is_dark,
                hovered,
            ),
        ]
        .spacing(0)
        .align_y(Center)
        .padding(Padding {
            top: 0.0,
            right: ROW_PAD,
            bottom: 0.0,
            left: ROW_PAD,
        }),
    )
    .width(Fill)
    .height(Length::Fixed(HEADER_HEIGHT))
    .style(style::table_header)
    .into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let rows = &app.api_rows;
    let filters = &app.analysis.filters;
    let is_dark = app.analysis.is_dark();

    let mut copy = button(
        row![
            icons::icon("copy", 12.0, if is_dark { style::SLATE_400 } else { style::SLATE_500 }),
            text("Copy TSV")
                .size(11)
                .font(style::MEDIUM)
                .style(style::text_muted),
        ]
        .spacing(4)
        .align_y(Center),
    )
    .padding(0)
    .style(style::btn_ghost);
    if !rows.is_empty() {
        copy = copy.on_press(Message::CopyApiTsv);
    }

    let title = container(
        row![
            crate::ui::lined_styled(
                "SLOW API ENDPOINTS",
                12.0,
                style::SEMIBOLD,
                crate::ui::lh::TEXT_XS,
                style::text_subheading,
            ),
            space().width(Fill),
            copy,
        ]
        .align_y(Center)
        .padding(Padding {
            top: 8.0,
            right: 12.0,
            bottom: 8.0,
            left: 12.0,
        }),
    )
    .width(Fill);

    let body: Element<'_, Message> = if rows.is_empty() {        container(
            text("No matching endpoints")
                .size(14)
                .style(style::text_faint),
        )
        .width(Fill)
        .center_x(Fill)
        .padding(Padding {
            top: 40.0,
            right: 12.0,
            bottom: 40.0,
            left: 12.0,
        })
        .into()
    } else {
        // `responsive` reports the exact card width so path truncation measures
        // real pixels instead of guessing at layout. The reference grid keeps a
        // 680px floor inside `overflow-x-auto`, so the card scrolls sideways
        // instead of squeezing the columns.
        let total_height = (rows.len() as f32 * ROW_HEIGHT + HEADER_HEIGHT).clamp(120.0, 420.0);
        let viewport_height = (total_height - HEADER_HEIGHT).max(0.0);
        responsive(move |size| {
            let table_width = f64::from(size.width).max(MIN_TABLE_WIDTH) as f32;
            let (start, end) = virtualize::visible_range(
                rows.len(),
                app.api_scroll,
                viewport_height,
                ROW_HEIGHT,
                OVERSCAN,
            );

            let mut list = column![].spacing(0).width(Length::Fixed(table_width));
            if start > 0 {
                list = list.push(space().height(start as f32 * ROW_HEIGHT));
            }
            for (index, row_data) in rows[start..end].iter().enumerate() {
                list = list.push(api_row(
                    start + index,
                    row_data,
                    app.api_hover,
                    table_width,
                ));
                list = list.push(hrule(style::row_divider));
            }
            if end < rows.len() {
                list = list.push(space().height((rows.len() - end) as f32 * ROW_HEIGHT));
            }

            let grid = column![
                header_row(filters, is_dark, app.api_header_hover),
                hrule(style::divider),
                scrollable(list)
                    .on_scroll(Message::ApiScrolled)
                    .height(Length::Fixed(viewport_height))
                    .width(Fill),
            ]
            .spacing(0)
            .width(Length::Fixed(table_width));

            scrollable(grid)
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::default(),
                ))
                .width(Fill)
                .into()
        })
        .height(Length::Fixed(viewport_height))
        .into()
    };

    container(column![
        title,
        hrule(style::divider),
        body
    ]
    .spacing(0))
        .width(Fill)
        .style(style::card)
        .into()
}

/// Widest path that fits the endpoint cell at the given table width.
fn path_width(table_width: f32, method: &str) -> f32 {
    let endpoint = table_width as f32 - NUMERIC_WIDTH - ROW_PAD * 2.0;
    let badge = text_fit::sans_width(method, BADGE_FONT_SIZE, text_fit::SansWeight::Bold) + 14.0;
    (endpoint - badge - BADGE_GAP - CELL_PAD_RIGHT - COPY_ICON - ICON_GAP).max(0.0)
}

fn api_row(
    index: usize,
    row_data: &AggregatedEndpoint,
    hovered: Option<usize>,
    table_width: f32,
) -> Element<'static, Message> {
    let path = row_data.path.clone();
    let method = row_data.method.as_str();
    let is_hovered = hovered == Some(index);

    let label = text_fit::truncate_mono(
        &row_data.path,
        PATH_FONT_SIZE,
        path_width(table_width, method),
    );
    let trailing: Element<'static, Message> = if is_hovered {
        icons::icon("copy", COPY_ICON, style::SLATE_400).into()
    } else {
        space().width(COPY_ICON).into()
    };
    let path_button = button(
        row![
            text(label.into_owned())
                .size(PATH_FONT_SIZE)
                .font(style::MONO)
                .wrapping(Wrapping::None),
            trailing,
        ]
        .spacing(ICON_GAP)
        .align_y(Center),
    )
    .on_press(Message::CopyPath(path))
    .padding(0)
    .style(style::path_button(is_hovered));

    let badge = container(
        text(method)
            .size(BADGE_FONT_SIZE)
            .font(style::BOLD),
    )
    .padding([2, 6])
    .style(style::method_badge(method));

    let errors_style = if row_data.error_count > 0 {
        style::text_danger
    } else {
        style::text_fainter
    };

    let row_element: Element<'static, Message> = container(
        row![
            container(
                row![badge, path_button]
                    .spacing(BADGE_GAP)
                    .align_y(Center)
                    .width(Fill)
            )
            .width(Fill)
            .padding(Padding {
                top: 0.0,
                right: CELL_PAD_RIGHT,
                bottom: 0.0,
                left: 0.0,
            }),
            numeric(format_num(row_data.count), COL_COUNT, style::text_body, false),
            numeric(format_ms(row_data.avg_ms), COL_AVG, style::text_body, false),
            numeric(format_ms(row_data.p95_ms), COL_P95, style::text_accent, true),
            numeric(format_ms(row_data.p99_ms), COL_P99, style::text_body, false),
            numeric(format_ms(row_data.max_ms), COL_MAX, style::text_amber, true),
            numeric(
                format_num(row_data.error_count),
                COL_ERRORS,
                errors_style,
                row_data.error_count > 0,
            ),
        ]
        .spacing(0)
        .align_y(Center)
        .padding(Padding {
            top: 0.0,
            right: ROW_PAD,
            bottom: 0.0,
            left: ROW_PAD,
        }),
    )
    .width(Fill)
    .height(Length::Fixed(ROW_HEIGHT - 1.0))
    .style(move |theme| style::table_row(theme, index.is_multiple_of(2)))
    .into();

    mouse_area(row_element)
        .on_enter(Message::ApiRowHovered(Some(index)))
        .on_exit(Message::ApiRowHovered(None))
        .into()
}

fn numeric(
    value: String,
    width: f32,
    text_style: fn(&iced::Theme) -> iced::widget::text::Style,
    semibold: bool,
) -> Element<'static, Message> {
    container(
        text(value)
            .size(12)
            .font(if semibold {
                style::SEMIBOLD
            } else {
                style::REGULAR
            })
            .style(text_style),
    )
    .width(Length::Fixed(width))
    .align_x(Right)
    .into()
}

fn sort_header(
    label: &'static str,
    key: ApiSortKey,
    filters: &crate::store::analysis_store::AnalysisFilters,
    width: f32,
    is_dark: bool,
    hovered: Option<ApiSortKey>,
) -> Element<'static, Message> {
    let active = filters.sort_key == key;
    let accent = if is_dark {
        style::BLUE_400
    } else {
        style::BLUE_600
    };
    let (icon, size, color) = if active {
        let icon = if filters.sort_dir == SortDirection::Asc {
            "arrow-up"
        } else {
            "arrow-down"
        };
        (icon, 12.0, accent)
    } else {
        ("arrow-up-down", 10.0, style::SLATE_500)
    };
    let show_icon = active || hovered == Some(key);

    let text_style: fn(&iced::Theme) -> iced::widget::text::Style = if active {
        style::text_accent
    } else {
        style::text_muted
    };
    let font = if active { style::BOLD } else { style::SEMIBOLD };

    let content: Element<'static, Message> = if show_icon {
        row![text(label).size(11).font(font).style(text_style), icons::icon(icon, size, color)]
            .spacing(4)
            .align_y(Center)
            .into()
    } else {
        row![
            text(label).size(11).font(font).style(text_style),
            // `opacity-0` still reserves the icon slot in the reference.
            space().width(size).height(size)
        ]
        .spacing(4)
        .align_y(Center)
        .into()
    };

    let header = button(content)
        .on_press(Message::ApiSortToggled(key))
        .padding(0)
        .style(style::sort_header(active));

    mouse_area(
        container(header)
            .width(Length::Fixed(width))
            .align_x(Right),
    )
    .on_enter(Message::ApiHeaderHovered(Some(key)))
    .on_exit(Message::ApiHeaderHovered(None))
    .into()
}
