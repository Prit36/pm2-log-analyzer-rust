//! `MongoSlowQueryTable` — individual slow queries with sortable columns.

use iced::widget::{button, column, container, mouse_area, row, scrollable, space, text};
use iced::{Center, Element, Fill, Length, Padding, Right};

use crate::app::{App, Message};
use crate::core::mongo_models::{MongoSlowQuery, MongoSlowQuerySortField, MongoSortDirection};
use crate::ui::{icons, style, virtualize};
use crate::utils::format::{format_ms, format_num, format_ratio, round_to_tenth};
use crate::utils::text_fit;

const ROW_HEIGHT: f32 = 44.0;
const HEADER_HEIGHT: f32 = 40.0;
const OVERSCAN: usize = 6;
const COL_TIME: f32 = 92.0;
const COL_OP: f32 = 74.0;
const COL_DURATION: f32 = 84.0;
const COL_DOCS: f32 = 80.0;
const COL_KEYS: f32 = 80.0;
const COL_RETURNED: f32 = 80.0;
const COL_RATIO: f32 = 74.0;
const COL_INSPECT: f32 = 36.0;
const NUMERIC_WIDTH: f32 =
    COL_TIME + COL_OP + COL_DURATION + COL_DOCS + COL_KEYS + COL_RETURNED + COL_RATIO + COL_INSPECT;
const ROW_PAD: f32 = 12.0;
const TABLE_FLOOR: f64 = 720.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let Some(result) = app.mongo.result.as_ref() else {
        return container(text("")).into();
    };
    let filters = &app.mongo.filters;
    let queries = sorted_queries(
        result.slow_queries.clone(),
        filters.slow_sort_field,
        filters.slow_sort_direction,
    );

    if queries.is_empty() {
        return empty_state(app);
    }

    let total_height = (queries.len() as f32 * ROW_HEIGHT + HEADER_HEIGHT).clamp(280.0, 620.0);
    let viewport_height = (total_height - HEADER_HEIGHT).max(0.0);

    let body = iced::widget::responsive(move |size| {
        let table_width = f64::from(size.width).max(TABLE_FLOOR) as f32;
        let query_width = table_width - NUMERIC_WIDTH - ROW_PAD * 2.0;
        let (start, end) = virtualize::visible_range(
            queries.len(),
            app.mongo_slow_scroll,
            viewport_height,
            ROW_HEIGHT,
            OVERSCAN,
        );

        let mut list = column![].spacing(0).width(Length::Fixed(table_width));
        if start > 0 {
            list = list.push(space().height(start as f32 * ROW_HEIGHT));
        }
        for (index, query) in queries[start..end].iter().enumerate() {
            list = list.push(query_row(start + index, query, query_width));
            list = list.push(crate::ui::hrule(style::row_divider));
        }
        if end < queries.len() {
            list = list.push(space().height((queries.len() - end) as f32 * ROW_HEIGHT));
        }

        scrollable(list)
            .on_scroll(Message::MongoSlowScrolled)
            .height(Length::Fixed(viewport_height))
            .width(Fill)
            .into()
    })
    .height(Length::Fixed(viewport_height));

    let sort_field = filters.slow_sort_field;
    let sort_direction = filters.slow_sort_direction;
    container(
        column![
            header_row(sort_field, sort_direction),
            Element::from(body)
        ]
        .spacing(0),
    )
    .width(Fill)
    .style(style::mongo_card)
    .into()
}

fn header_row(
    sort_field: MongoSlowQuerySortField,
    direction: MongoSortDirection,
) -> Element<'static, Message> {
    container(
        row![
            container(text("Query").size(11).font(style::SEMIBOLD).style(style::text_muted))
                .width(Fill),
            sort_header("Time", MongoSlowQuerySortField::Timestamp, sort_field, direction, COL_TIME),
            sort_header("Op", MongoSlowQuerySortField::Collection, sort_field, direction, COL_OP),
            sort_header("Duration", MongoSlowQuerySortField::DurationMs, sort_field, direction, COL_DURATION),
            sort_header("Docs", MongoSlowQuerySortField::DocsExamined, sort_field, direction, COL_DOCS),
            sort_header("Keys", MongoSlowQuerySortField::KeysExamined, sort_field, direction, COL_KEYS),
            sort_header("Returned", MongoSlowQuerySortField::Nreturned, sort_field, direction, COL_RETURNED),
            sort_header("Ratio", MongoSlowQuerySortField::ScanRatio, sort_field, direction, COL_RATIO),
            container(text("").size(11)).width(Length::Fixed(COL_INSPECT)),
        ]
        .spacing(0)
        .align_y(Center)
        .padding(Padding {
            top: 10.0,
            right: ROW_PAD,
            bottom: 10.0,
            left: ROW_PAD,
        }),
    )
    .width(Fill)
    .height(Length::Fixed(HEADER_HEIGHT))
    .style(style::table_header)
    .into()
}

fn query_row(index: usize, query: &MongoSlowQuery, query_width: f32) -> Element<'static, Message> {
    let ns = text_fit::truncate_mono(&query.ns, 11.0, query_width - 60.0);
    let time = query
        .timestamp
        .get(11..19)
        .map(|slice| slice.to_string())
        .unwrap_or_else(|| query.timestamp.clone());

    let mut title = row![
        container(
            text(query.op.clone())
                .size(10)
                .font(style::BOLD),
        )
        .padding([2, 6])
        .style(style::mongo_op_badge(&query.op)),
        text(ns.into_owned())
            .size(11)
            .font(style::MONO)
            .style(style::text_strong),
    ]
    .spacing(6)
    .align_y(Center);

    if query.is_collscan {
        title = title.push(
            container(
                row![
                    icons::icon("flame", 10.0, style::AMBER_600),
                    text("COLLSCAN").size(10).font(style::BOLD),
                ]
                .spacing(2)
                .align_y(Center),
            )
            .padding([1, 4])
            .style(style::collscan_chip),
        );
    }

    let row_content = container(
        row![
            container(title)
                .width(Fill)
                .padding(Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 0.0,
                }),
            plain(time, COL_TIME, style::text_body),
            plain(query.op.clone(), COL_OP, style::text_muted),
            numeric(format_ms(query.duration_ms as f64), COL_DURATION, style::text_heading, true),
            numeric(format_num(query.docs_examined), COL_DOCS, style::text_body, false),
            numeric(format_num(query.keys_examined), COL_KEYS, style::text_muted, false),
            numeric(format_num(query.nreturned), COL_RETURNED, style::text_body, false),
            numeric(
                if query.scan_ratio > 0.0 {
                    format_ratio(round_to_tenth(query.scan_ratio))
                } else {
                    "0x".to_string()
                },
                COL_RATIO,
                if query.scan_ratio > 100.0 {
                    style::text_danger
                } else {
                    style::text_muted
                },
                false,
            ),
            container(icons::icon("external-link", 14.0, style::SLATE_400))
                .width(Length::Fixed(COL_INSPECT))
                .align_x(Center),
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
    .style(move |theme| style::table_row(theme, index.is_multiple_of(2)));

    mouse_area(row_content)
        .on_press(Message::MongoOpenSlowQuery(Some(Box::new(query.clone()))))
        .into()
}

fn plain(
    value: String,
    width: f32,
    text_style: fn(&iced::Theme) -> iced::widget::text::Style,
) -> Element<'static, Message> {
    container(
        text(value)
            .size(11)
            .font(style::MONO)
            .style(text_style),
    )
    .width(Length::Fixed(width))
    .align_x(Right)
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
    field: MongoSlowQuerySortField,
    current: MongoSlowQuerySortField,
    direction: MongoSortDirection,
    width: f32,
) -> Element<'static, Message> {
    let active = current == field;
    let icon = if active {
        match direction {
            MongoSortDirection::Asc => "arrow-up",
            MongoSortDirection::Desc => "arrow-down",
        }
    } else {
        "arrow-up-down"
    };

    container(
        button(
            row![
                text(label).size(11).font(style::SEMIBOLD).style(style::text_muted),
                icons::icon(
                    icon,
                    12.0,
                    if active {
                        style::EMERALD_600
                    } else {
                        style::SLATE_400
                    }
                ),
            ]
            .spacing(4)
            .align_y(Center),
        )
        .on_press(Message::MongoSlowSortToggled(field))
        .padding(0)
        .style(style::transparent_button),
    )
    .width(Length::Fixed(width))
    .align_x(Right)
    .into()
}

fn empty_state(app: &App) -> Element<'_, Message> {
    let active_count = app.mongo.filters.active_count();
    let mut content = column![
        icons::icon("file-text", 32.0, style::SLATE_400),
        text("No slow queries match")
            .size(14)
            .font(style::SEMIBOLD)
            .style(style::text_body),
        text("Adjust the filters to widen the result set.")
            .size(12)
            .style(style::text_muted),
    ]
    .spacing(6)
    .align_x(Center);

    if active_count > 0 {
        content = content.push(
            button(
                row![
                    icons::icon("rotate-ccw", 14.0, style::ROSE_700),
                    text(format!("Reset all filters ({active_count})"))
                        .size(12)
                        .font(style::SEMIBOLD),
                ]
                .spacing(6)
                .align_y(Center),
            )
            .on_press(Message::MongoResetFilters)
            .padding([6, 12])
            .style(style::btn_danger),
        );
    }

    container(content)
        .width(Fill)
        .height(Length::Fixed(256.0))
        .center_x(Fill)
        .center_y(Length::Fixed(256.0))
        .style(style::mongo_card)
        .into()
}

fn sorted_queries(
    mut queries: Vec<MongoSlowQuery>,
    field: MongoSlowQuerySortField,
    direction: MongoSortDirection,
) -> Vec<MongoSlowQuery> {
    queries.sort_by(|a, b| {
        let ordering = match field {
            MongoSlowQuerySortField::Timestamp => a.timestamp.cmp(&b.timestamp),
            MongoSlowQuerySortField::DurationMs => a.duration_ms.cmp(&b.duration_ms),
            MongoSlowQuerySortField::DocsExamined => a.docs_examined.cmp(&b.docs_examined),
            MongoSlowQuerySortField::KeysExamined => a.keys_examined.cmp(&b.keys_examined),
            MongoSlowQuerySortField::Nreturned => a.nreturned.cmp(&b.nreturned),
            MongoSlowQuerySortField::ScanRatio => a.scan_ratio.total_cmp(&b.scan_ratio),
            MongoSlowQuerySortField::Collection => a.collection.cmp(&b.collection),
        };
        match direction {
            MongoSortDirection::Asc => ordering,
            MongoSortDirection::Desc => ordering.reverse(),
        }
    });
    queries
}
