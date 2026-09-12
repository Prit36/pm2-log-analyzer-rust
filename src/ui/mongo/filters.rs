//! `MongoFilterBar` — view switcher, search, plan/op/collection/user filters.

use iced::widget::{button, column, container, pick_list, row, space, text, text_input};
use iced::{Center, Element, Fill, Length};

use crate::app::{App, Message};
use crate::core::mongo_models::MongoPlanFilter;
use crate::store::mongo_store::MongoActiveView;
use crate::ui::{icons, style};
use crate::utils::format::format_num;

const DURATION_PRESETS: [(u32, &str); 5] = [
    (0, "All"),
    (100, ">100ms"),
    (500, ">500ms"),
    (1000, ">1s"),
    (5000, ">5s"),
];

pub fn view(app: &App) -> Element<'_, Message> {
    let mongo = &app.mongo;
    let filters = &mongo.filters;
    let result = mongo.result.as_ref();

    let pattern_count = result.map(|r| r.patterns.len() as u64).unwrap_or(0);
    let slow_count = result.map(|r| r.summary.slow_query_count).unwrap_or(0);
    let user_count = result.map(|r| r.users.len() as u64).unwrap_or(0);
    let error_count: u64 = result
        .map(|r| r.errors.iter().map(|error| error.count).sum())
        .unwrap_or(0);
    let collscan_count = result.map(|r| r.summary.collscan_count).unwrap_or(0);

    let mut tabs = row![].spacing(4).align_y(Center);
    for view in MongoActiveView::ALL {
        let badge = match view {
            MongoActiveView::Patterns => Some(pattern_count),
            MongoActiveView::SlowQueries => Some(slow_count),
            MongoActiveView::Users => Some(user_count),
            MongoActiveView::Diagnostics => Some(error_count),
            MongoActiveView::Charts => None,
        };
        tabs = tabs.push(tab(view, mongo.active_view == view, badge));
    }

    let search = text_input("Search collection, plan, IP…", &filters.search_query)
        .on_input(Message::MongoSearchChanged)
        .padding([7, 10])
        .size(12)
        .width(Length::Fixed(260.0))
        .style(style::field);

    let operations: Vec<String> = result.map(|r| r.operations.clone()).unwrap_or_default();
    let op_items = std::iter::once("all".to_string())
        .chain(operations.iter().cloned())
        .collect::<Vec<_>>();
    let op_selected = Some(filters.operation.clone());

    let collections: Vec<String> = std::iter::once("all".to_string())
        .chain(mongo.collections())
        .collect();
    let users: Vec<String> = std::iter::once("all".to_string())
        .chain(result.map(|r| r.user_names.clone()).unwrap_or_default())
        .collect();

    let plan_chips = row![
        plan_chip(
            "All Plans",
            filters.plan_filter == MongoPlanFilter::All,
            PlanChipKind::Neutral,
            Some(Message::MongoPlanFilter(MongoPlanFilter::All)),
        ),
        plan_chip(
            &format!("COLLSCAN Only ({})", format_num(collscan_count)),
            filters.plan_filter == MongoPlanFilter::CollscanOnly,
            PlanChipKind::Collscan,
            Some(Message::MongoPlanFilter(if filters.plan_filter
                == MongoPlanFilter::CollscanOnly
            {
                MongoPlanFilter::All
            } else {
                MongoPlanFilter::CollscanOnly
            })),
        ),
        plan_chip(
            "IXSCAN",
            filters.plan_filter == MongoPlanFilter::IxscanOnly,
            PlanChipKind::Ixscan,
            Some(Message::MongoPlanFilter(if filters.plan_filter
                == MongoPlanFilter::IxscanOnly
            {
                MongoPlanFilter::All
            } else {
                MongoPlanFilter::IxscanOnly
            })),
        ),
    ]
    .spacing(4)
    .align_y(Center);

    let mut duration_row = row![].spacing(4).align_y(Center);
    for (ms, label) in DURATION_PRESETS {
        duration_row = duration_row.push(duration_chip(
            label,
            filters.min_duration_ms == ms,
            ms,
        ));
    }

    let active_count = filters.active_count();
    let reset = button(
        row![
            icons::icon(
                "rotate-ccw",
                12.0,
                if active_count > 0 {
                    style::ROSE_700
                } else {
                    style::SLATE_400
                }
            ),
            text("Reset all filters")
                .size(12)
                .font(style::SEMIBOLD),
            if active_count > 0 {
                container(text(format_num(active_count as u64)).size(10).font(style::BOLD))
                    .padding([1, 6])
                    .style(style::reset_count)
            } else {
                container(text("").size(10))
            },
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press_maybe((active_count > 0).then_some(Message::MongoResetFilters))
    .padding([7, 10])
    .style(style::reset_chip(active_count > 0));

    let mut controls = row![
        label("Plan:"),
        plan_chips,
        separator(),
        label("Op:"),
        pick_list(op_items, op_selected, Message::MongoOperationChanged)
            .text_size(12)
            .padding([4, 8])
            .style(style::picker)
            .width(Length::Fixed(150.0)),
    ]
    .spacing(8)
    .align_y(Center);

    if mongo.collections().len() > 1 {
        controls = controls.push(label("Collection:"));
        controls = controls.push(
            pick_list(collections, Some(filters.collection.clone()), Message::MongoCollectionChanged)
                .text_size(12)
                .padding([4, 8])
                .style(style::picker)
                .width(Length::Fixed(200.0)),
        );
    }

    if users.len() > 1 {
        controls = controls.push(label("User:"));
        controls = controls.push(
            pick_list(users, Some(filters.user_filter.clone()), Message::MongoUserChanged)
                .text_size(12)
                .padding([4, 8])
                .style(style::picker)
                .width(Length::Fixed(180.0)),
        );
    }

    controls = controls
        .push(separator())
        .push(label("Duration:"))
        .push(duration_row)
        .push(scan_ratio_chip(filters.high_scan_ratio_only))
        .push(space().width(Fill))
        .push(reset);

    let header = container(
        column![
            row![tabs, space().width(Fill), search]
                .spacing(12)
                .align_y(Center),
            container(space().height(1.0))
                .width(Fill)
                .style(style::divider),
            // `flex flex-wrap` in the reference: the filter row wraps instead of
            // clipping when the window is narrow.
            controls.wrap(),
        ]
        .spacing(12),
    )
    .width(Fill)
    .padding(14)
    .style(style::mongo_card);

    header.into()
}

fn tab(view: MongoActiveView, active: bool, badge: Option<u64>) -> Element<'static, Message> {
    let mut content = row![
        icons::icon(
            view_icon(view),
            14.0,
            if active {
                style::EMERALD_600
            } else {
                style::SLATE_400
            }
        ),
        text(view.label()).size(12).font(style::SEMIBOLD),
    ]
    .spacing(6)
    .align_y(Center);

    if let Some(count) = badge.filter(|count| *count > 0) {
        content = content.push(
            container(text(format_num(count)).size(10).font(style::BOLD))
                .padding([1, 6])
                .style(move |theme: &iced::Theme| style::tab_badge(theme, active)),
        );
    }

    button(content)
        .on_press(Message::MongoActiveView(view))
        .padding([6, 12])
        .style(style::mongo_tab_button(active))
        .into()
}

fn view_icon(view: MongoActiveView) -> &'static str {
    match view {
        MongoActiveView::Patterns => "database",
        MongoActiveView::SlowQueries => "file-text",
        MongoActiveView::Users => "user",
        MongoActiveView::Charts => "bar-chart3",
        MongoActiveView::Diagnostics => "shield-alert",
    }
}

fn label(text_value: &'static str) -> Element<'static, Message> {
    text(text_value)
        .size(11)
        .font(style::MEDIUM)
        .style(style::text_faint)
        .into()
}

fn separator<'a>() -> Element<'a, Message> {
    container(space().width(1.0))
        .width(1.0)
        .height(Length::Fixed(16.0))
        .style(style::divider)
        .into()
}

enum PlanChipKind {
    Neutral,
    Collscan,
    Ixscan,
}

fn plan_chip(
    label_text: &str,
    active: bool,
    kind: PlanChipKind,
    message: Option<Message>,
) -> Element<'static, Message> {
    let mut content = row![].spacing(4).align_y(Center);
    if matches!(kind, PlanChipKind::Collscan) {
        content = content.push(icons::icon("flame", 12.0, style::AMBER_500));
    }
    content = content.push(
        text(label_text.to_string())
            .size(12)
            .font(if matches!(kind, PlanChipKind::Collscan) {
                style::SEMIBOLD
            } else {
                style::MEDIUM
            }),
    );

    let surface = match kind {
        PlanChipKind::Neutral => container(content)
            .padding([3, 8])
            .style(move |theme: &iced::Theme| style::mongo_chip(theme, active)),
        PlanChipKind::Collscan => container(content)
            .padding([3, 8])
            .style(move |theme: &iced::Theme| style::collscan_filter_chip(theme, active)),
        PlanChipKind::Ixscan => container(content)
            .padding([3, 8])
            .style(move |theme: &iced::Theme| style::ixscan_filter_chip(theme, active)),
    };

    button(surface)
        .on_press_maybe(message)
        .padding(0)
        .style(style::transparent_button)
        .into()
}

fn duration_chip(label_text: &str, active: bool, ms: u32) -> Element<'static, Message> {
    button(
        container(text(label_text.to_string()).size(12).font(style::MEDIUM))
            .padding([3, 8])
            .style(move |theme| style::mongo_chip(theme, active)),
    )
    .on_press(Message::MongoMinDuration(ms))
    .padding(0)
    .style(style::transparent_button)
    .into()
}

fn scan_ratio_chip(active: bool) -> Element<'static, Message> {
    button(
        container(
            row![
                icons::icon(
                    "filter",
                    12.0,
                    if active {
                        style::ROSE_700
                    } else {
                        style::SLATE_500
                    }
                ),
                text("Scan Ratio >100x").size(12).font(style::MEDIUM),
            ]
            .spacing(4)
            .align_y(Center),
        )
        .padding([3, 8])
        .style(move |theme| style::scan_ratio_chip(theme, active)),
    )
    .on_press(Message::MongoScanRatioToggled)
    .padding(0)
    .style(style::transparent_button)
    .into()
}
