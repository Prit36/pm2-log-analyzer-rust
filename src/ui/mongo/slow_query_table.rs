//! `MongoSlowQueryTable` — individual slow queries with sortable columns.
//!
//! Reference grid (`dump-dom.mjs --mongo --view "Slow Query Log"`):
//! `[100px_85px_110px_1fr_80px_80px_80px_70px_110px_36px]` with `px-4`,
//! 44px rows, a `px-4 py-2.5 text-[11px]` header whose numeric labels wrap to
//! two lines (54px strip), severity-tinted duration badges and a mono 10px
//! plan chip.

use iced::widget::{button, column, container, mouse_area, row, scrollable, space, text};
use iced::{Center, Element, Fill, Length, Padding, Right};

use crate::app::{App, Message};
use crate::core::mongo_models::{MongoSlowQuery, MongoSlowQuerySortField, MongoSortDirection};
use crate::ui::{BORDER, boxed_text, icons, lh, style, virtualize};
use crate::utils::format::{format_ms, format_num, round_to_tenth};
use crate::utils::text_fit;

const ROW_HEIGHT: f32 = 44.0;
/// Header content: `py-2.5` (10px) plus the 33px two-line numeric labels.
const HEADER_CONTENT: f32 = 53.0;
const OVERSCAN: usize = 6;
const COL_TIME: f32 = 100.0;
const COL_DURATION: f32 = 85.0;
const COL_USER: f32 = 110.0;
const COL_DOCS: f32 = 80.0;
const COL_KEYS: f32 = 80.0;
const COL_RETURNED: f32 = 80.0;
const COL_RATIO: f32 = 70.0;
const COL_REMOTE: f32 = 110.0;
const COL_INSPECT: f32 = 36.0;
const NUMERIC_WIDTH: f32 = COL_TIME
    + COL_DURATION
    + COL_USER
    + COL_DOCS
    + COL_KEYS
    + COL_RETURNED
    + COL_RATIO
    + COL_REMOTE
    + COL_INSPECT;
const ROW_PAD: f32 = 16.0;
const TABLE_FLOOR: f64 = 760.0;
/// `text-[10px]` inside a `text-xs` row: 4/3 em.
const XS_10: f32 = 13.3333;
/// `text-[11px]` inside a `text-xs` row: 4/3 em.
const XS_11: f32 = 14.6667;

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

    // Reference: react-window's list is `min(620, max(280, queries * 44))`.
    let viewport_height = (queries.len() as f32 * ROW_HEIGHT).clamp(280.0, 620.0);

    let body = iced::widget::responsive(move |size| {
        let table_width = f64::from(size.width).max(TABLE_FLOOR) as f32;
        let collection_width = table_width - NUMERIC_WIDTH - ROW_PAD * 2.0;
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
            list = list.push(query_row(
                start + index,
                query,
                collection_width.max(120.0),
            ));
            list = list.push(crate::ui::hrule(style::row_divider));
        }
        if end < queries.len() {
            list = list.push(space().height((queries.len() - end) as f32 * ROW_HEIGHT));
        }

        scrollable(list)
            .on_scroll(Message::MongoSlowScrolled)
            .height(Length::Fixed(viewport_height))
            // Chromium's overlay scrollbar takes no layout width.
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new().width(0.0).scroller_width(0.0),
            ))
            .width(Fill)
            .into()
    })
    .height(Length::Fixed(viewport_height));

    let sort_field = filters.slow_sort_field;
    let sort_direction = filters.slow_sort_direction;
    container(
        column![
            header_row(sort_field, sort_direction, app.window_width),
            crate::ui::hrule(style::divider),
            Element::from(body)
        ]
        .spacing(0),
    )
    .width(Fill)
    .padding(BORDER)
    .style(style::mongo_card)
    .into()
}

fn header_row(
    sort_field: MongoSlowQuerySortField,
    direction: MongoSortDirection,
    window_width: f32,
) -> Element<'static, Message> {
    let table_width = (window_width - 34.0).max(TABLE_FLOOR as f32);
    let collection_width = (table_width - NUMERIC_WIDTH - ROW_PAD * 2.0).max(120.0);

    container(
        row![
            sort_header("Time", MongoSlowQuerySortField::Timestamp, sort_field, direction, COL_TIME, false, false),
            sort_header("Duration", MongoSlowQuerySortField::DurationMs, sort_field, direction, COL_DURATION, false, false),
            header_label("User", COL_USER, false, false),
            sort_header("Collection & Plan", MongoSlowQuerySortField::Collection, sort_field, direction, collection_width, false, false),
            sort_header("Docs\nScanned", MongoSlowQuerySortField::DocsExamined, sort_field, direction, COL_DOCS, true, true),
            sort_header("Keys\nScanned", MongoSlowQuerySortField::KeysExamined, sort_field, direction, COL_KEYS, true, true),
            sort_header("Returned", MongoSlowQuerySortField::Nreturned, sort_field, direction, COL_RETURNED, true, false),
            sort_header("Scan\nRatio", MongoSlowQuerySortField::ScanRatio, sort_field, direction, COL_RATIO, true, true),
            container(boxed_text("Client IP", 11.0, style::SEMIBOLD, lh::TEXT_11, style::text_muted))
                .width(Length::Fixed(COL_REMOTE))
                .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 8.0 }),
            container(boxed_text("View", 11.0, style::SEMIBOLD, lh::TEXT_11, style::text_muted))
                .width(Length::Fixed(COL_INSPECT))
                .align_x(Center),
        ]
        .spacing(0)
        .align_y(Center)
        .padding(Padding { top: 10.0, right: ROW_PAD, bottom: 10.0, left: ROW_PAD }),
    )
    .width(Fill)
    .height(Length::Fixed(HEADER_CONTENT))
    .align_y(Center)
    .style(style::table_header)
    .into()
}

fn header_label(
    label: &'static str,
    width: f32,
    right_aligned: bool,
    two_line: bool,
) -> Element<'static, Message> {
    let height = if two_line { 33.0 } else { lh::TEXT_11 };
    let inner = container(crate::ui::lined_styled(
        label,
        11.0,
        style::SEMIBOLD,
        height,
        style::text_muted,
    ))
    .width(Length::Fixed(width));
    if right_aligned {
        inner.align_x(Right).into()
    } else {
        inner.into()
    }
}

fn query_row(index: usize, query: &MongoSlowQuery, collection_width: f32) -> Element<'static, Message> {
    let time = query
        .timestamp
        .get(11..19)
        .map(|slice| slice.to_string())
        .unwrap_or_else(|| query.timestamp.clone());
    let duration_ms = query.duration_ms;

    let user_name = query
        .user
        .clone()
        .filter(|user| !user.is_empty())
        .unwrap_or_else(|| "system".to_string());
    let user_style: fn(&iced::Theme) -> iced::widget::text::Style =
        if user_name != "system" {
            style::text_emerald
        } else {
            style::text_muted
        };
    let mut user_cell = column![boxed_text(
        user_name,
        11.0,
        style::SEMIBOLD,
        XS_11,
        user_style,
    )]
    .spacing(0);
    if !query.ctx.is_empty() {
        user_cell = user_cell.push(boxed_text(
            query.ctx.clone(),
            9.0,
            style::MONO,
            12.0,
            style::text_faint,
        ));
    }

    let collection = query.collection.clone();
    // The reference row `truncate`s the collection and shrinks it to zero once
    // the op and plan chips take the cell (`min-w-0` flex behaviour).
    let op_label = query.op.to_uppercase();
    let op_width = 8.0 + text_width(&op_label, 10.0);
    let plan_width = if query.is_collscan {
        8.0 + 12.0 + 2.0 + text_width("COLLSCAN", 10.0)
    } else if !query.plan_summary.is_empty() {
        (8.0 + text_width(&query.plan_summary, 10.0)).min(120.0)
    } else {
        0.0
    };
    let chip_gaps = if plan_width > 0.0 { 12.0 } else { 6.0 };
    let collection_avail =
        (collection_width - 12.0 - op_width - plan_width - chip_gaps).max(0.0);
    let collection_hidden = collection_avail <= 0.0;
    let collection_cell: Element<'static, Message> = if collection_hidden {
        space().width(0.0).into()
    } else {
        let collection = text_fit::truncate_mono(&collection, 12.0, collection_avail);
        boxed_text(
            collection.into_owned(),
            12.0,
            style::SEMIBOLD,
            lh::TEXT_XS,
            style::text_heading,
        )
        .into()
    };
    let mut namespace = row![].spacing(6).align_y(Center);
    if !collection_hidden {
        namespace = namespace.push(collection_cell);
    }
    namespace = namespace.push(
        container(boxed_text(
            op_label,
            10.0,
            style::BOLD,
            XS_10,
            style::text_inherit,
        ))
        .padding([2, 4])
        .style(style::mongo_op_plain),
    );
    if query.is_collscan {
        namespace = namespace.push(
            container(
                row![
                    icons::icon("flame", 12.0, style::AMBER_600),
                    boxed_text("COLLSCAN", 10.0, style::BOLD, XS_10, style::text_inherit),
                ]
                .spacing(2)
                .align_y(Center),
            )
            .padding([2, 4])
            .style(style::collscan_chip),
        );
    } else if !query.plan_summary.is_empty() {
        let plan = text_fit::truncate_mono(&query.plan_summary, 10.0, 120.0);
        namespace = namespace.push(
            container(boxed_text(
                plan.into_owned(),
                10.0,
                style::SEMIBOLD,
                XS_10,
                style::text_inherit,
            ))
            .padding([2, 4])
            .style(style::mongo_plan_chip),
        );
    }

    let ratio = format!("{}x", round_to_tenth(query.scan_ratio));
    let ratio_font = if query.scan_ratio >= 1000.0 {
        style::BOLD
    } else if query.scan_ratio >= 100.0 {
        style::SEMIBOLD
    } else {
        style::REGULAR
    };
    let ratio_style: fn(&iced::Theme) -> iced::widget::text::Style = if query.scan_ratio >= 1000.0 {
        style::text_danger
    } else if query.scan_ratio >= 100.0 {
        style::text_amber
    } else {
        style::text_muted
    };

    let row_content = container(
        row![
            container(boxed_text(time, 11.0, style::MONO, XS_11, style::text_muted))
                .width(Length::Fixed(COL_TIME)),
            container(
                container(boxed_text(
                    format_ms(query.duration_ms as f64),
                    11.0,
                    style::MONO_SEMIBOLD,
                    XS_11,
                    style::text_inherit,
                ))
                .padding([2, 6])
                .style(move |theme: &iced::Theme| {
                    style::slow_duration_badge(theme, duration_ms)
                }),
            )
            .width(Length::Fixed(COL_DURATION)),
            container(user_cell)
                .width(Length::Fixed(COL_USER))
                .padding(Padding { top: 0.0, right: 8.0, bottom: 0.0, left: 0.0 }),
            container(namespace)
                .width(Length::Fixed(collection_width))
                .padding(Padding {
                    top: 0.0,
                    right: 12.0,
                    bottom: 0.0,
                    left: if collection_hidden { 6.0 } else { 0.0 },
                }),
            mono_numeric(format_num(query.docs_examined), COL_DOCS, style::text_body),
            mono_numeric(format_num(query.keys_examined), COL_KEYS, style::text_muted),
            mono_numeric(format_num(query.nreturned), COL_RETURNED, style::text_muted),
            container(boxed_text(ratio, 12.0, ratio_font, lh::TEXT_XS, ratio_style))
                .width(Length::Fixed(COL_RATIO))
                .align_x(Right),
            container(boxed_text(
                query
                    .remote
                    .clone()
                    .filter(|remote| !remote.is_empty())
                    .unwrap_or_else(|| "unknown".to_string()),
                11.0,
                style::MONO,
                XS_11,
                style::text_muted,
            ))
            .width(Length::Fixed(COL_REMOTE))
            .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 8.0 }),
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
    .align_y(Center)
    .style(move |theme| style::table_row(theme, index.is_multiple_of(2)));

    mouse_area(row_content)
        .on_press(Message::MongoOpenSlowQuery(Some(Box::new(query.clone()))))
        .into()
}

fn mono_numeric(
    value: String,
    width: f32,
    text_style: fn(&iced::Theme) -> iced::widget::text::Style,
) -> Element<'static, Message> {
    container(boxed_text(value, 12.0, style::MONO, lh::TEXT_XS, text_style))
        .width(Length::Fixed(width))
        .align_x(Right)
        .into()
}

/// Advance width of `text` in the bundled sans at `size` (chip sizing).
fn text_width(text: &str, size: f32) -> f32 {
    crate::utils::text_path::measure(
        text,
        crate::utils::text_path::FaceKind::SansSemiBold,
        f64::from(size),
    ) as f32
}

fn sort_header(
    label: &'static str,
    field: MongoSlowQuerySortField,
    current: MongoSlowQuerySortField,
    direction: MongoSortDirection,
    width: f32,
    right_aligned: bool,
    two_line: bool,
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
    let height = if two_line { 33.0 } else { lh::TEXT_11 };

    let content: Element<'static, Message> = row![
        crate::ui::lined_styled(label, 11.0, style::SEMIBOLD, height, style::text_muted),
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
    .align_y(Center)
    .into();

    let button = button(content)
        .on_press(Message::MongoSlowSortToggled(field))
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
        icons::icon("info", 32.0, style::SLATE_400),
        crate::ui::lined(
            "No slow query occurrences match filters",
            14.0,
            style::SEMIBOLD,
            lh::TEXT_SM,
        ),
    ]
    .spacing(8)
    .align_x(Center);

    if active_count > 0 {
        content = content.push(
            button(
                row![
                    icons::icon("rotate-ccw", 14.0, style::ROSE_700),
                    boxed_text(
                        format!("Reset all filters ({active_count})"),
                        12.0,
                        style::SEMIBOLD,
                        lh::TEXT_XS,
                        style::text_danger,
                    ),
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
            MongoSlowQuerySortField::Timestamp => a.epoch_ms.total_cmp(&b.epoch_ms),
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
