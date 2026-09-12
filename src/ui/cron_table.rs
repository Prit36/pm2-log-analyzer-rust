//! `CronTable` — port of `src/components/CronTable.tsx`.

use std::fmt;

use iced::widget::text::Wrapping;
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, responsive, row, scrollable, space,
    text, text_input,
};
use iced::{Center, Element, Fill, Length, Padding, Right};

use crate::app::{App, Message};
use crate::core::models::CronAggregated;
use crate::store::analysis_store::{CronSortKey, SortDirection};
use crate::ui::virtualize;
use crate::ui::{hrule, icons, style};
use crate::utils::format::{format_ms, format_num};
use crate::utils::text_fit;

const ROW_HEIGHT: f32 = 32.0;
const HEADER_HEIGHT: f32 = 32.0;
const OVERSCAN: usize = 6;
/// Numeric column widths, mirroring the reference grid template.
const COL_RUNS: f32 = 56.0;
const COL_STARTS: f32 = 56.0;
const COL_FAILS: f32 = 56.0;
const COL_AVG: f32 = 64.0;
const COL_P95: f32 = 64.0;
const COL_P99: f32 = 64.0;
const COL_MAX: f32 = 64.0;
const COL_LAST: f32 = 64.0;
const NUMERIC_WIDTH: f32 = COL_RUNS
    + COL_STARTS
    + COL_FAILS
    + COL_AVG
    + COL_P95
    + COL_P99
    + COL_MAX
    + COL_LAST;
/// `px-3` on the row/header grid plus the eight `gap-1` gutters.
const NAME_INSET: f32 = 24.0 + 8.0 * 4.0;

const SORT_OPTIONS: &[CronSortKey] = &[
    CronSortKey::P95Ms,
    CronSortKey::P99Ms,
    CronSortKey::AvgMs,
    CronSortKey::MaxMs,
    CronSortKey::Runs,
    CronSortKey::Starts,
    CronSortKey::Fails,
    CronSortKey::LastDurationMs,
    CronSortKey::Name,
];

impl fmt::Display for CronSortKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CronSortKey::P95Ms => "p95",
            CronSortKey::P99Ms => "p99",
            CronSortKey::AvgMs => "avg",
            CronSortKey::MaxMs => "max",
            CronSortKey::Runs => "runs",
            CronSortKey::Starts => "starts",
            CronSortKey::Fails => "fails",
            CronSortKey::LastDurationMs => "last",
            CronSortKey::Name => "job",
        })
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let rows = &app.cron_rows;
    let filters = &app.analysis.filters;
    let is_dark = app.analysis.is_dark();

    let query = text_input("Filter jobs…", &filters.cron_query)
        .on_input(Message::CronQueryChanged)
        .padding([7, 8])
        .size(12)
        .line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
        .width(Length::Fixed(160.0))
        .style(style::field);
    let min_ms = text_input("Min ms", &app.cron_min_ms_input)
        .on_input(Message::CronMinMsChanged)
        .padding([7, 8])
        .size(12)
        .line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
        .width(Length::Fixed(80.0))
        .style(style::field);
    let failures = checkbox(filters.cron_show_failed_only)
        .label("Failures only")
        .on_toggle(Message::CronFailedOnly)
        .size(14)
        .text_size(11)
        .style(style::check);
    let sort = pick_list(
        SORT_OPTIONS,
        Some(filters.cron_sort_key),
        Message::CronSortKeyChanged,
    )
    .padding([7, 8])
    .text_size(12)
    .text_line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
    .style(style::picker);

    let mut copy = button(
        row![
            icons::icon(
                "copy",
                12.0,
                if is_dark {
                    style::SLATE_400
                } else {
                    style::SLATE_500
                }
            ),
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
        copy = copy.on_press(Message::CopyCronTsv);
    }

    let controls = container(
        row![
            crate::ui::lined_styled(
                "CRON JOBS",
                12.0,
                style::SEMIBOLD,
                crate::ui::lh::TEXT_XS,
                style::text_subheading,
            ),
            space().width(Fill),
            query,
            min_ms,
            failures,
            sort,
            copy,
        ]
        .spacing(8)
        .align_y(Center)
        .padding(Padding {
            top: 8.0,
            right: 12.0,
            bottom: 8.0,
            left: 12.0,
        }),
    )
    .width(Fill);

    let header = container(
        row![
            sort_header(
                "Job",
                CronSortKey::Name,
                filters.cron_sort_key,
                filters.cron_sort_dir,
                0.0,
                is_dark,
                app.cron_header_hover,
                true,
            ),
            sort_header(
                "Runs",
                CronSortKey::Runs,
                filters.cron_sort_key,
                filters.cron_sort_dir,
                COL_RUNS,
                is_dark,
                app.cron_header_hover,
                false,
            ),
            sort_header(
                "Starts",
                CronSortKey::Starts,
                filters.cron_sort_key,
                filters.cron_sort_dir,
                COL_STARTS,
                is_dark,
                app.cron_header_hover,
                false,
            ),
            sort_header(
                "Fails",
                CronSortKey::Fails,
                filters.cron_sort_key,
                filters.cron_sort_dir,
                COL_FAILS,
                is_dark,
                app.cron_header_hover,
                false,
            ),
            sort_header(
                "Avg",
                CronSortKey::AvgMs,
                filters.cron_sort_key,
                filters.cron_sort_dir,
                COL_AVG,
                is_dark,
                app.cron_header_hover,
                false,
            ),
            sort_header(
                "p95",
                CronSortKey::P95Ms,
                filters.cron_sort_key,
                filters.cron_sort_dir,
                COL_P95,
                is_dark,
                app.cron_header_hover,
                false,
            ),
            sort_header(
                "p99",
                CronSortKey::P99Ms,
                filters.cron_sort_key,
                filters.cron_sort_dir,
                COL_P99,
                is_dark,
                app.cron_header_hover,
                false,
            ),
            sort_header(
                "Max",
                CronSortKey::MaxMs,
                filters.cron_sort_key,
                filters.cron_sort_dir,
                COL_MAX,
                is_dark,
                app.cron_header_hover,
                false,
            ),
            sort_header(
                "Last",
                CronSortKey::LastDurationMs,
                filters.cron_sort_key,
                filters.cron_sort_dir,
                COL_LAST,
                is_dark,
                app.cron_header_hover,
                false,
            ),
        ]
        .spacing(4)
        .align_y(Center)
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        }),
    )
    .width(Fill)
    .height(Length::Fixed(HEADER_HEIGHT))
    .style(style::table_header);

    let body: Element<'_, Message> = if rows.is_empty() {
        container(
            text("No cron jobs match filters.")
                .size(14)
                .style(style::text_faint),
        )
        .width(Fill)
        .center_x(Fill)
        .padding(Padding {
            top: 32.0,
            right: 12.0,
            bottom: 32.0,
            left: 12.0,
        })
        .into()
    } else {
        let total_height = (rows.len() as f32 * ROW_HEIGHT + HEADER_HEIGHT).clamp(120.0, 360.0);
        let viewport_height = (total_height - HEADER_HEIGHT).max(0.0);
        responsive(move |size| {
            let name_width = (f64::from(size.width) - f64::from(NUMERIC_WIDTH + NAME_INSET)).max(0.0);
            let (start, end) = virtualize::visible_range(
                rows.len(),
                app.cron_scroll,
                viewport_height,
                ROW_HEIGHT,
                OVERSCAN,
            );

            let mut list = column![].spacing(0).width(Fill);
            if start > 0 {
                list = list.push(space().height(start as f32 * ROW_HEIGHT));
            }
            for (index, row_data) in rows[start..end].iter().enumerate() {
                list = list.push(cron_row(start + index, row_data, name_width as f32));
                list = list.push(hrule(style::row_divider));
            }
            if end < rows.len() {
                list = list.push(space().height((rows.len() - end) as f32 * ROW_HEIGHT));
            }

            scrollable(list)
                .on_scroll(Message::CronScrolled)
                .height(Length::Fixed(viewport_height))
                .width(Fill)
                .into()
        })
        .height(Length::Fixed(viewport_height))
        .into()
    };

    container(column![
        controls,
        hrule(style::divider),
        header,
        hrule(style::divider),
        body
    ]
    .spacing(0))
        .width(Fill)
        .style(style::card)
        .into()
}

fn cron_row(index: usize, row_data: &CronAggregated, name_width: f32) -> Element<'static, Message> {
    let name = text_fit::truncate_mono(&row_data.name, 11.0, name_width);
    container(
        row![
            container(
                text(name.into_owned())
                    .size(11)
                    .font(style::MONO)
                    .style(style::text_strong)
                    .wrapping(Wrapping::None)
            )
            .width(Fill)
            .clip(true),
            numeric(format_num(row_data.runs), COL_RUNS, style::text_body, false),
            numeric(
                format_num(row_data.starts),
                COL_STARTS,
                style::text_faint,
                false
            ),
            numeric(
                format_num(row_data.fails),
                COL_FAILS,
                if row_data.fails > 0 {
                    style::text_danger
                } else {
                    style::text_fainter
                },
                row_data.fails > 0
            ),
            numeric(format_ms(row_data.avg_ms), COL_AVG, style::text_body, false),
            numeric(format_ms(row_data.p95_ms), COL_P95, style::text_accent, true),
            numeric(format_ms(row_data.p99_ms), COL_P99, style::text_body, false),
            numeric(format_ms(row_data.max_ms), COL_MAX, style::text_body, false),
            numeric(
                row_data
                    .last_duration_ms
                    .map(format_ms)
                    .unwrap_or_else(|| "-".to_string()),
                COL_LAST,
                style::text_faint,
                false
            ),
        ]
        .spacing(4)
        .align_y(Center)
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        }),
    )
    .width(Fill)
    .height(Length::Fixed(ROW_HEIGHT - 1.0))
    .align_y(Center)
    .style(move |theme| style::table_row(theme, index.is_multiple_of(2)))
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

#[allow(clippy::too_many_arguments)]
fn sort_header(
    label: &'static str,
    key: CronSortKey,
    current_key: CronSortKey,
    current_dir: SortDirection,
    width: f32,
    is_dark: bool,
    hovered: Option<CronSortKey>,
    align_left: bool,
) -> Element<'static, Message> {
    let active = current_key == key;
    let accent = if is_dark {
        style::BLUE_400
    } else {
        style::BLUE_600
    };
    let (icon, size, color) = if active {
        let icon = if current_dir == SortDirection::Asc {
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
    let label = text(label)
        .size(10)
        .font(font)
        .style(text_style)
        .wrapping(Wrapping::None);

    let content: Element<'static, Message> = if show_icon {
        row![label, icons::icon(icon, size, color)]
            .spacing(4)
            .align_y(Center)
            .into()
    } else {
        row![label, space().width(size).height(size)]
            .spacing(4)
            .align_y(Center)
            .into()
    };

    let header = button(content)
        .on_press(Message::CronSortToggled(key))
        .padding(0)
        .style(style::sort_header(active));

    let aligned = if align_left {
        container(header).align_x(iced::Left)
    } else {
        container(header).width(Length::Fixed(width)).align_x(Right)
    };

    mouse_area(aligned)
        .on_enter(Message::CronHeaderHovered(Some(key)))
        .on_exit(Message::CronHeaderHovered(None))
        .into()
}
