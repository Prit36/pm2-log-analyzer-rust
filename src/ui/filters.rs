//! `FilterBar` — port of `src/components/FilterBar.tsx`.

use std::fmt;

use iced::widget::{button, column, container, pick_list, row, space, text, text_input, Column};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message, SEARCH_INPUT_ID};
use crate::core::models::{NormalizeMode, StatusFamily};
use crate::store::analysis_store::{count_active_analysis_filters, ApiSortKey};
use crate::ui::{icons, style};
use crate::utils::format::format_date;

const FIELD_PADDING: Padding = Padding {
    top: 7.0,
    right: 8.0,
    bottom: 7.0,
    left: 8.0,
};

impl fmt::Display for NormalizeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            NormalizeMode::CollapseIds => "Collapse IDs",
            NormalizeMode::StripQuery => "Strip query",
            NormalizeMode::Exact => "Exact path",
        })
    }
}

impl fmt::Display for StatusFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            StatusFamily::All => "All",
            StatusFamily::X2xx => "2xx",
            StatusFamily::X3xx => "3xx",
            StatusFamily::X4xx => "4xx",
            StatusFamily::X5xx => "5xx",
        })
    }
}

impl fmt::Display for ApiSortKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ApiSortKey::P95Ms => "p95",
            ApiSortKey::P99Ms => "p99",
            ApiSortKey::AvgMs => "avg",
            ApiSortKey::MaxMs => "max",
            ApiSortKey::Count => "count",
            ApiSortKey::ErrorCount => "errors",
            ApiSortKey::Path => "endpoint",
        })
    }
}

const NORMALIZE_OPTIONS: &[NormalizeMode] = &[
    NormalizeMode::CollapseIds,
    NormalizeMode::StripQuery,
    NormalizeMode::Exact,
];
const STATUS_OPTIONS: &[StatusFamily] = &[
    StatusFamily::All,
    StatusFamily::X2xx,
    StatusFamily::X3xx,
    StatusFamily::X4xx,
    StatusFamily::X5xx,
];
const SORT_OPTIONS: &[ApiSortKey] = &[
    ApiSortKey::P95Ms,
    ApiSortKey::P99Ms,
    ApiSortKey::AvgMs,
    ApiSortKey::MaxMs,
    ApiSortKey::Count,
    ApiSortKey::ErrorCount,
    ApiSortKey::Path,
];

pub fn view(app: &App) -> Option<Element<'_, Message>> {
    if !app.analysis.has_data {
        return None;
    }
    let filters = &app.analysis.filters;
    let active_count = count_active_analysis_filters(filters);
    let methods = app
        .analysis
        .result
        .as_ref()
        .map(|r| r.methods.clone())
        .unwrap_or_default();
    let dates = app
        .analysis
        .result
        .as_ref()
        .map(|r| r.dates.clone())
        .unwrap_or_default();
    let all_selected = filters.methods.is_empty();

    let search = text_input("Filter endpoints… (/)", &filters.query)
        .id(iced::widget::Id::new(SEARCH_INPUT_ID))
        .on_input(Message::QueryChanged)
        .padding(FIELD_PADDING)
        .size(12)
        .line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
        .style(style::field);
    let normalize = pick_list(
        NORMALIZE_OPTIONS,
        Some(filters.normalize_mode),
        Message::NormalizeChanged,
    )
    .padding(FIELD_PADDING)
    .text_size(12)
    .text_line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
    .style(style::picker);
    let status = pick_list(
        STATUS_OPTIONS,
        Some(filters.status_family),
        Message::StatusChanged,
    )
    .padding(FIELD_PADDING)
    .text_size(12)
    .text_line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
    .style(style::picker);
    let min_ms = text_input("", &app.min_ms_input)
        .on_input(Message::MinMsChanged)
        .padding(FIELD_PADDING)
        .size(12)
        .line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
        .width(Length::Fixed(80.0))
        .style(style::field);
    let sort = pick_list(SORT_OPTIONS, Some(filters.sort_key), Message::ApiSortKeyChanged)
        .padding(FIELD_PADDING)
        .text_size(12)
        .text_line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
        .style(style::picker);
    let top_n = text_input("", &app.top_n_input)
        .on_input(Message::TopNChanged)
        .padding(FIELD_PADDING)
        .size(12)
        .line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
        .width(Length::Fixed(80.0))
        .style(style::field);

    let reset = reset_button(active_count);

    let controls = row![
        field("SEARCH", search.into(), Fill),
        field("NORMALIZE", normalize.into(), Length::Shrink),
        field("STATUS", status.into(), Length::Shrink),
        field("MIN MS", min_ms.into(), Length::Shrink),
        field("SORT", sort.into(), Length::Shrink),
        field("TOP N", top_n.into(), Length::Shrink),
        container(reset).padding(iced::Padding {
            top: 0.0,
            right: 0.0,
            bottom: 1.0,
            left: 0.0,
        }),
    ]
    .spacing(12)
    .align_y(iced::Bottom)
    .width(Fill);

    let mut content = column![controls].spacing(12).width(Fill);

    if dates.len() > 1 || !methods.is_empty() {
        let mut meta = row![].spacing(12).align_y(Center);

        if dates.len() > 1 {
            meta = meta.push(
                row![
                    crate::ui::lined_styled(
                        "DAY",
                        10.0,
                        style::SEMIBOLD,
                        crate::ui::lh::TEXT_10,
                        style::text_muted,
                    ),
                    chip(
                        format!("All Days ({})", dates.len()),
                        filters.date_filter == "all",
                        style::MEDIUM,
                        Message::DateFilterChanged("all".to_string()),
                    ),
                ]
                .spacing(6)
                .align_y(Center),
            );
            for date in &dates {
                meta = meta.push(chip(
                    format_date(Some(date)),
                    filters.date_filter == *date,
                    style::MONO,
                    Message::DateFilterChanged(date.clone()),
                ));
            }
        }

        if dates.len() > 1 && !methods.is_empty() {
            meta = meta.push(
                container(space().width(1.0))
                    .width(1.0)
                    .height(Length::Fixed(16.0))
                    .style(style::divider),
            );
        }

        if !methods.is_empty() {
            let mut method_row = row![
                crate::ui::lined_styled(
                    "METHODS",
                    10.0,
                    style::SEMIBOLD,
                    crate::ui::lh::TEXT_10,
                    style::text_muted,
                ),
                chip(
                    "All".to_string(),
                    all_selected,
                    style::BOLD,
                    Message::MethodChipToggled(None)
                ),
            ]
            .spacing(6)
            .align_y(Center);
            for method in &methods {
                method_row = method_row.push(chip(
                    method.clone(),
                    all_selected || filters.methods.iter().any(|m| m == method),
                    style::BOLD,
                    Message::MethodChipToggled(Some(method.clone())),
                ));
            }
            if !all_selected {
                method_row = method_row.push(
                    button(text("Reset").size(11).style(style::text_muted))
                        .on_press(Message::MethodChipToggled(None))
                        .padding(0)
                        .style(style::btn_ghost),
                );
            }
            meta = meta.push(method_row);
        }

        content = content.push(meta);
    }

    Some(
        container(content)
            .width(Fill)
            .padding(Padding {
                top: 12.0,
                right: 12.0,
                bottom: 12.0,
                left: 12.0,
            })
            .style(style::card)
            .into(),
    )
}

fn field<'a>(
    label: &'static str,
    control: Element<'a, Message>,
    width: Length,
) -> Column<'a, Message> {
    column![
        crate::ui::lined_styled(
            label,
            10.0,
            style::SEMIBOLD,
            crate::ui::lh::TEXT_10,
            style::text_muted,
        ),
        control,
    ]
    .spacing(4)
    .width(width)
}

fn reset_button(active_count: usize) -> Element<'static, Message> {
    let mut content = row![icons::icon(
        "rotate-ccw",
        12.0,
        if active_count > 0 {
            style::ROSE_700
        } else {
            style::SLATE_400
        }
    )]
    .spacing(6)
    .align_y(Center);
    content = content.push(text("Reset all filters").size(12).font(style::SEMIBOLD));
    if active_count > 0 {
        content = content.push(
            container(text(active_count.to_string()).size(10).font(style::BOLD))
                .padding([1, 6])
                .style(style::reset_count),
        );
    }

    button(content)
        .on_press_maybe((active_count > 0).then_some(Message::ResetFilters))
        .padding([6, 10])
        .style(style::reset_chip(active_count > 0))
        .into()
}

fn chip(
    label: String,
    active: bool,
    font: iced::Font,
    on_press: Message,
) -> Element<'static, Message> {
    let content = container(
        container(text(label).size(10).font(font))
            .height(Length::Fixed(crate::ui::lh::TEXT_10)),
    )
    .padding(Padding {
        top: 2.0,
        right: 8.0,
        bottom: 2.0,
        left: 8.0,
    })
    .style(style::chip(active));

    button(content)
        .on_press(on_press)
        .padding(0)
        .style(style::transparent_button)
        .into()
}
