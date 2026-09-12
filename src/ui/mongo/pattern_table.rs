//! `MongoPatternTable` — query patterns with index suggestions.

use iced::widget::{button, column, container, mouse_area, row, scrollable, space, text};
use iced::{Center, Element, Fill, Length, Padding, Right};

use crate::app::{App, Message};
use crate::core::mongo_models::{MongoQueryPattern, MongoSortField, MongoSortDirection};
use crate::ui::{icons, style, virtualize};
use crate::utils::format::{format_ms, format_num, format_ratio};
use crate::utils::text_fit;

const ROW_HEIGHT: f32 = 58.0;
const HEADER_HEIGHT: f32 = 40.0;
const OVERSCAN: usize = 4;
const COL_COUNT: f32 = 80.0;
const COL_TOTAL: f32 = 90.0;
const COL_AVG: f32 = 70.0;
const COL_P95: f32 = 75.0;
const COL_MAX: f32 = 75.0;
const COL_RATIO: f32 = 85.0;
const COL_INSPECT: f32 = 40.0;
const NUMERIC_WIDTH: f32 = COL_COUNT
    + COL_TOTAL
    + COL_AVG
    + COL_P95
    + COL_MAX
    + COL_RATIO
    + COL_INSPECT;
const ROW_PAD: f32 = 12.0;
/// `minmax(0,2.2fr) … minmax(0,1.6fr) …` — the fingerprint column gets 1.6fr.
const INDEX_SHARE: f32 = 1.6 / (2.2 + 1.6);

pub fn view(app: &App) -> Element<'_, Message> {
    let Some(result) = app.mongo.result.as_ref() else {
        return container(text("")).into();
    };
    let filters = &app.mongo.filters;
    let sorted = sorted_patterns(result.patterns.clone(), filters.sort_field, filters.sort_direction);

    if sorted.is_empty() {
        return empty_state(app);
    }

    let total_height = (sorted.len() as f32 * ROW_HEIGHT + HEADER_HEIGHT).clamp(280.0, 620.0);
    let viewport_height = (total_height - HEADER_HEIGHT).max(0.0);

    let body = responsive_body(app, sorted, viewport_height);

    container(
        column![
            header_row(filters.sort_field, filters.sort_direction, app.window_width),
            body
        ]
        .spacing(0),
    )
    .width(Fill)
    .style(style::mongo_card)
    .into()
}

fn responsive_body(
    app: &App,
    sorted: Vec<MongoQueryPattern>,
    viewport_height: f32,
) -> Element<'_, Message> {
    iced::widget::responsive(move |size| {
        let table_width = f64::from(size.width).max(760.0) as f32;
        let pattern_width = table_width - NUMERIC_WIDTH - ROW_PAD * 2.0;
        let index_width = (pattern_width * INDEX_SHARE).max(80.0);
        let first_width = (pattern_width - index_width).max(120.0);

        let (start, end) = virtualize::visible_range(
            sorted.len(),
            app.mongo_scroll,
            viewport_height,
            ROW_HEIGHT,
            OVERSCAN,
        );

        let mut list = column![].spacing(0).width(Length::Fixed(table_width));
        if start > 0 {
            list = list.push(space().height(start as f32 * ROW_HEIGHT));
        }
        for (index, pattern) in sorted[start..end].iter().enumerate() {
            list = list.push(pattern_row(
                start + index,
                pattern,
                first_width,
                index_width,
                app.mongo_copied_index.as_deref(),
            ));
            list = list.push(crate::ui::hrule(style::row_divider));
        }
        if end < sorted.len() {
            list = list.push(space().height((sorted.len() - end) as f32 * ROW_HEIGHT));
        }

        scrollable(list)
            .on_scroll(Message::MongoScrolled)
            .height(Length::Fixed(viewport_height))
            .width(Fill)
            .into()
    })
    .height(Length::Fixed(viewport_height))
    .into()
}

fn header_row(
    sort_field: MongoSortField,
    direction: MongoSortDirection,
    window_width: f32,
) -> Element<'static, Message> {
    let table_width = (window_width - 64.0).max(760.0);
    let pattern_width = table_width - NUMERIC_WIDTH - ROW_PAD * 2.0;
    let index_width = (pattern_width * INDEX_SHARE).max(80.0);
    let first_width = (pattern_width - index_width).max(120.0);

    container(
        row![
            sort_header("Query Pattern / Collection", MongoSortField::Collection, sort_field, direction, first_width, false),
            sort_header("Count", MongoSortField::Count, sort_field, direction, COL_COUNT, true),
            sort_header("Total Time", MongoSortField::TotalDurationMs, sort_field, direction, COL_TOTAL, true),
            sort_header("Avg", MongoSortField::AvgDurationMs, sort_field, direction, COL_AVG, true),
            sort_header("P95", MongoSortField::P95DurationMs, sort_field, direction, COL_P95, true),
            sort_header("Max", MongoSortField::MaxDurationMs, sort_field, direction, COL_MAX, true),
            sort_header("Scan Ratio", MongoSortField::ScanRatio, sort_field, direction, COL_RATIO, true),
            container(text("Suggested Index (1-Click Copy)").size(11).font(style::SEMIBOLD).style(style::text_muted))
                .width(Length::Fixed(index_width))
                .padding(Padding { top: 0.0, right: 8.0, bottom: 0.0, left: 8.0 }),
            container(text("View").size(11).font(style::SEMIBOLD).style(style::text_muted))
                .width(Length::Fixed(COL_INSPECT))
                .align_x(Center),
        ]
        .spacing(0)
        .align_y(Center)
        .padding(Padding { top: 10.0, right: ROW_PAD, bottom: 10.0, left: ROW_PAD }),
    )
    .width(Fill)
    .height(Length::Fixed(HEADER_HEIGHT))
    .style(style::table_header)
    .into()
}

fn pattern_row(
    index: usize,
    pattern: &MongoQueryPattern,
    first_width: f32,
    index_width: f32,
    copied: Option<&str>,
) -> Element<'static, Message> {
    let is_copied = copied == Some(pattern.id.as_str());
    let fingerprint = text_fit::truncate_mono(&pattern.fingerprint, 11.0, first_width - 8.0);
    let suggestion = text_fit::truncate_mono(&pattern.index_suggestion, 10.0, index_width - 28.0);
    let collection = text_fit::truncate_mono(&pattern.collection, 12.0, first_width - 70.0);

    let mut title = row![
        container(
            text(pattern.op.clone())
                .size(10)
                .font(style::BOLD),
        )
        .padding([2, 6])
        .style(style::mongo_op_badge(&pattern.op)),
        text(collection.into_owned())
            .size(12)
            .font(style::SEMIBOLD)
            .style(style::text_heading),
    ]
    .spacing(6)
    .align_y(Center);

    if pattern.is_collscan {
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

    let index_cell: Element<'static, Message> = if pattern.index_suggestion.is_empty() {
        text("Indexed / Covered")
            .size(10)
            .style(style::text_faint)
            .into()
    } else {
        let label = suggestion.into_owned();
        let suggestion_text = pattern.index_suggestion.clone();
        button(
            row![
                text(label)
                    .size(10)
                    .font(style::MONO)
                    .style(style::text_emerald),
                space().width(Fill),
                icons::icon(
                    if is_copied { "check" } else { "copy" },
                    12.0,
                    style::EMERALD_600,
                ),
            ]
            .spacing(4)
            .align_y(Center)
            .width(Fill),
        )
        .on_press(Message::MongoCopyIndex(suggestion_text))
        .padding([4, 8])
        .width(Fill)
        .style(style::index_chip_button)
        .into()
    };

    let ratio = if pattern.scan_ratio > 0.0 {
        format_ratio(pattern.scan_ratio)
    } else {
        "0x".to_string()
    };

    let row_content = container(
        row![
            container(
                column![
                    title,
                    text(fingerprint.into_owned())
                        .size(11)
                        .font(style::MONO)
                        .style(style::text_muted),
                ]
                .spacing(2)
            )
            .width(Length::Fixed(first_width))
            .padding(Padding {
                top: 0.0,
                right: 8.0,
                bottom: 0.0,
                left: 0.0,
            }),
            numeric(format_num(pattern.count), COL_COUNT, style::text_body, false),
            numeric(
                format!("{:.1}s", pattern.total_duration_ms as f64 / 1000.0),
                COL_TOTAL,
                style::text_heading,
                true,
            ),
            numeric(format_ms(pattern.avg_duration_ms), COL_AVG, style::text_muted, false),
            numeric(format_ms(pattern.p95_duration_ms as f64), COL_P95, style::text_accent, true),
            numeric(format_ms(pattern.max_duration_ms as f64), COL_MAX, style::text_danger, true),
            numeric(
                ratio,
                COL_RATIO,
                if pattern.scan_ratio > 100.0 {
                    style::text_danger
                } else {
                    style::text_muted
                },
                false,
            ),
            container(index_cell)
                .width(Length::Fixed(index_width))
                .padding(Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 8.0,
                }),
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
    .width(Length::Fixed(first_width + index_width + NUMERIC_WIDTH + ROW_PAD * 2.0))
    .height(Length::Fixed(ROW_HEIGHT - 1.0))
    .style(move |theme| style::table_row(theme, index.is_multiple_of(2)));

    let example = pattern.example_query.clone();
    mouse_area(row_content)
        .on_press(Message::MongoOpenSlowQuery(Some(Box::new(example))))
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
    field: MongoSortField,
    current: MongoSortField,
    direction: MongoSortDirection,
    width: f32,
    right_aligned: bool,
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
    let color = if active {
        style::EMERALD_600
    } else {
        style::SLATE_400
    };

    let content: Element<'static, Message> = row![
        text(label).size(11).font(style::SEMIBOLD).style(style::text_muted),
        icons::icon(icon, 12.0, color),
    ]
    .spacing(4)
    .align_y(Center)
    .into();

    let button = button(content)
        .on_press(Message::MongoSortToggled(field))
        .padding(0)
        .style(style::transparent_button);

    let wrapper = container(button).width(Length::Fixed(width));
    if right_aligned {
        wrapper.align_x(Right).into()
    } else {
        wrapper.into()
    }
}

fn empty_state(app: &App) -> Element<'_, Message> {
    let active_count = app.mongo.filters.active_count();
    let mut content = column![
        icons::icon("lightbulb", 32.0, style::SLATE_400),
        text("No query patterns found")
            .size(14)
            .font(style::SEMIBOLD)
            .style(style::text_body),
        text("Try adjusting the search query, duration, or plan filters.")
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

/// Reference sorts client-side; the kernel returns total-duration order.
fn sorted_patterns(
    mut patterns: Vec<MongoQueryPattern>,
    field: MongoSortField,
    direction: MongoSortDirection,
) -> Vec<MongoQueryPattern> {
    patterns.sort_by(|a, b| {
        let ordering = match field {
            MongoSortField::Count => a.count.cmp(&b.count),
            MongoSortField::AvgDurationMs => a.avg_duration_ms.total_cmp(&b.avg_duration_ms),
            MongoSortField::P95DurationMs => a.p95_duration_ms.cmp(&b.p95_duration_ms),
            MongoSortField::MaxDurationMs => a.max_duration_ms.cmp(&b.max_duration_ms),
            MongoSortField::ScanRatio => a.scan_ratio.total_cmp(&b.scan_ratio),
            MongoSortField::Collection => a.collection.cmp(&b.collection),
            MongoSortField::CollscanCount => a.collscan_count.cmp(&b.collscan_count),
            MongoSortField::TotalDocsExamined => {
                a.total_docs_examined.cmp(&b.total_docs_examined)
            }
            MongoSortField::TotalDurationMs => a.total_duration_ms.cmp(&b.total_duration_ms),
        };
        match direction {
            MongoSortDirection::Asc => ordering,
            MongoSortDirection::Desc => ordering.reverse(),
        }
    });
    patterns
}
