//! `MongoFilterBar` — view switcher, search, plan/op/collection/user filters.
//!
//! Reference geometry (`dump-dom.mjs --mongo` at 1024x768): the card is
//! `p-3.5` (content inset 15px), the tab strip is `rounded-lg bg-slate-100 p-1`
//! with `px-3 py-1.5` buttons, the search is 320px on its own line once the
//! strip no longer fits, and the option chips are `px-2 py-0.5 text-xs`
//! (20px tall, 22px when they carry a real border). Line heights inherit
//! Tailwind v4's unitless `text-xs` ratio: 12px text on 16px, 11px labels on
//! 14.6667px, 10px badges on 13.3333px.

use std::fmt;

use iced::widget::{button, column, container, pick_list, row, space, text_input};
use iced::{Center, Element, Fill, Length, Padding};

use crate::app::{App, Message};
use crate::core::mongo_models::{MongoFilters, MongoPlanFilter};
use crate::store::mongo_store::MongoActiveView;
use crate::ui::{BORDER, boxed_text, icons, style};
use crate::utils::format::format_num;

const DURATION_PRESETS: [(u32, &str); 5] = [
    (0, "All"),
    (100, ">100ms"),
    (500, ">500ms"),
    (1000, ">1s"),
    (5000, ">5s"),
];

/// `text-[10px]` inside a `text-xs` row: 4/3 em.
const XS_10: f32 = 13.3333;
/// `text-[11px]` inside a `text-xs` row: 4/3 em.
const XS_11: f32 = 14.6667;
const SEARCH_WIDTH: f32 = 320.0;
/// Below this content width the reference's `flex-wrap` option row breaks
/// after the User select (measured: it needs ~1468px + card gutters).
const OPTIONS_ONE_LINE: f32 = 1520.0;

/// A pick-list entry whose label can differ from its value (`<option
/// value="all">All Operations</option>`).
#[derive(Clone, PartialEq)]
struct SelectOption {
    value: String,
    label: String,
}

impl fmt::Display for SelectOption {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

fn selected(options: &[SelectOption], value: &str) -> Option<SelectOption> {
    options.iter().find(|option| option.value == value).cloned()
}

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

    let tabs_width = tab_strip_width(pattern_count, slow_count, user_count, error_count);
    let active_view = mongo.active_view;

    let query = filters.search_query.clone();
    let top_row = iced::widget::responsive(move |size| {
        let strip = tab_strip(active_view, pattern_count, slow_count, user_count, error_count);
        let search = search_field(&query);
        if f64::from(size.width) >= f64::from(tabs_width) + 10.0 + f64::from(SEARCH_WIDTH) {
            row![strip, space().width(Fill), search]
                .spacing(10)
                .align_y(Center)
                .into()
        } else {
            column![strip, search].spacing(10).into()
        }
    });

    let top = container(top_row)
        .width(Fill)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 12.0 + BORDER,
            left: 0.0,
        })
        .style(style::mongo_filter_divider);

    let mut op_options = vec![SelectOption {
        value: "all".to_string(),
        label: "All Operations".to_string(),
    }];
    op_options.extend(
        result
            .map(|r| r.operations.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|op| SelectOption {
                value: op.clone(),
                label: op,
            }),
    );

    let mut collection_options = vec![SelectOption {
        value: "all".to_string(),
        label: format!("All Collections ({})", mongo.collections().len()),
    }];
    collection_options.extend(mongo.collections().into_iter().map(|name| SelectOption {
        value: name.clone(),
        label: name,
    }));

    let mut user_options = vec![SelectOption {
        value: "all".to_string(),
        label: format!("All Users ({})", user_count),
    }];
    user_options.extend(
        result
            .map(|r| r.user_names.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|user| SelectOption {
                value: user.clone(),
                label: user,
            }),
    );

    let active_count = filters.active_count();
    let options = iced::widget::responsive(move |size| {
        let first_line = options_first_line(
            filters,
            &op_options,
            &collection_options,
            &user_options,
            collscan_count,
        );
        let second_line = options_second_line(filters, active_count);
        if f64::from(size.width) >= f64::from(OPTIONS_ONE_LINE) {
            row![first_line, space().width(Fill), second_line]
                .spacing(12)
                .align_y(Center)
                .width(Fill)
                .into()
        } else {
            column![first_line, second_line]
                .spacing(12)
                .width(Fill)
                .into()
        }
    });

    container(column![top, options].spacing(12).width(Fill))
        .width(Fill)
        .padding(14.0 + BORDER)
        .style(style::mongo_card)
        .into()
}

/// Width of the `bg-slate-100 p-1 gap-1` strip, measured with the bundled
/// faces so the search field wraps exactly when the reference's does.
fn tab_strip_width(patterns: u64, slow: u64, users: u64, errors: u64) -> f32 {
    let tabs = [
        (MongoActiveView::Patterns, patterns),
        (MongoActiveView::SlowQueries, slow),
        (MongoActiveView::Users, users),
        (MongoActiveView::Charts, 0),
        (MongoActiveView::Diagnostics, errors),
    ];
    let mut width = 8.0;
    for (index, (view, badge)) in tabs.iter().enumerate() {
        if index > 0 {
            width += 4.0;
        }
        width += 24.0 + 14.0 + 6.0 + label_width(view.label());
        if *badge > 0 {
            width += 6.0 + 12.0 + number_width(*badge);
        }
    }
    width
}

fn label_width(label: &str) -> f32 {
    crate::utils::text_path::measure(label, crate::utils::text_path::FaceKind::SansSemiBold, 12.0)
        as f32
}

fn number_width(count: u64) -> f32 {
    let text = format_num(count);
    crate::utils::text_path::measure(&text, crate::utils::text_path::FaceKind::SansSemiBold, 10.0)
        as f32
}

fn tab_strip(
    active_view: MongoActiveView,
    patterns: u64,
    slow: u64,
    users: u64,
    errors: u64,
) -> Element<'static, Message> {
    let mut strip = row![].spacing(4).align_y(Center);
    for view in MongoActiveView::ALL {
        let badge = match view {
            MongoActiveView::Patterns => patterns,
            MongoActiveView::SlowQueries => slow,
            MongoActiveView::Users => users,
            MongoActiveView::Diagnostics => errors,
            MongoActiveView::Charts => 0,
        };
        strip = strip.push(tab(view, active_view == view, badge));
    }

    container(strip).padding(4).style(style::mongo_tab_strip).into()
}

fn tab(view: MongoActiveView, active: bool, badge: u64) -> Element<'static, Message> {
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
        boxed_text(
            view.label(),
            12.0,
            style::SEMIBOLD,
            crate::ui::lh::TEXT_XS,
            style::text_tab(active, style::EMERALD_700),
        ),
    ]
    .spacing(6)
    .align_y(Center);

    if badge > 0 {
        content = content.push(
            container(boxed_text(
                format_num(badge),
                10.0,
                style::BOLD,
                XS_10,
                move |theme: &iced::Theme| style::tab_badge_text(theme, active),
            ))
            .padding([0, 6])
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

/// `py-1.5 pl-8 pr-7` search input with the magnifier (and the clear "X").
fn search_field(query: &str) -> Element<'static, Message> {
    let input = text_input("Search collection, plan, IP...", query)
        .on_input(Message::MongoSearchChanged)
        .size(12)
        .line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
        .padding(Padding {
            top: 7.0,
            right: 28.0,
            bottom: 7.0,
            left: 32.0,
        })
        .width(Length::Fixed(SEARCH_WIDTH))
        .style(style::mongo_search);

    let mut layers: Vec<Element<'static, Message>> = vec![
        input.into(),
        container(icons::icon("search", 14.0, style::SLATE_400))
            .width(Fill)
            .height(Fill)
            .align_x(iced::Left)
            .align_y(Center)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 10.0,
            })
            .into(),
    ];

    if !query.is_empty() {
        layers.push(
            container(icons::icon("x", 14.0, style::SLATE_400))
                .width(Fill)
                .height(Fill)
                .align_x(iced::Right)
                .align_y(Center)
                .padding(Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 0.0,
                })
                .into(),
        );
    }

    iced::widget::Stack::with_children(layers).into()
}

/// Plan / separator / Op / Collection / User — the first wrapped line.
fn options_first_line(
    filters: &MongoFilters,
    op_options: &[SelectOption],
    collection_options: &[SelectOption],
    user_options: &[SelectOption],
    collscan_count: u64,
) -> Element<'static, Message> {
    let mut line = row![
        mongo_label("Plan:"),
        plan_chip(
            "All Plans".to_string(),
            filters.plan_filter == MongoPlanFilter::All,
            PlanChipKind::Neutral,
            Some(Message::MongoPlanFilter(MongoPlanFilter::All)),
        ),
        plan_chip(
            format!("COLLSCAN Only ({})", format_num(collscan_count)),
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
            "IXSCAN".to_string(),
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

    // `gap-3` between the wrapped groups, `gap-1` inside them. iced rows
    // carry a single gap, so each group boundary adds a 4px filler (4+4+4).
    line = line.push(space().width(4.0));
    line = line.push(separator());
    line = line.push(space().width(4.0));
    line = line.push(mongo_label("Op:"));
    line = line.push(
        pick_list(
            op_options.to_vec(),
            selected(op_options, &filters.operation),
            |option: SelectOption| Message::MongoOperationChanged(option.value),
        )
        .text_size(12)
        .text_line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
        .padding([3.5, 9.0])
        .style(style::mongo_select),
    );

    if collection_options.len() > 1 {
        line = line.push(space().width(8.0));
        line = line.push(mongo_label("Collection:"));
        line = line.push(
            pick_list(
                collection_options.to_vec(),
                selected(collection_options, &filters.collection),
                |option: SelectOption| Message::MongoCollectionChanged(option.value),
            )
            .text_size(12)
            .text_line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
            .padding([3.5, 9.0])
            .style(style::mongo_select),
        );
    }

    if user_options.len() > 1 {
        let active = filters.user_filter != "all";
        line = line.push(space().width(8.0));
        line = line.push(mongo_label("User:"));
        line = line.push(
            pick_list(
                user_options.to_vec(),
                selected(user_options, &filters.user_filter),
                |option: SelectOption| Message::MongoUserChanged(option.value),
            )
            .text_size(12)
            .text_line_height(iced::Pixels(crate::ui::lh::TEXT_XS))
            .padding([3.5, 9.0])
            .style(style::mongo_select_user(active)),
        );
    }

    line.into()
}

/// Duration chips / scan-ratio toggle / Reset all filters.
fn options_second_line(
    filters: &MongoFilters,
    active_count: usize,
) -> Element<'static, Message> {
    let mut duration = row![mongo_label("Duration:")].spacing(4).align_y(Center);
    for (ms, label) in DURATION_PRESETS {
        duration = duration.push(duration_chip(label, filters.min_duration_ms == ms, ms));
    }

    row![
        duration,
        scan_ratio_chip(filters.high_scan_ratio_only),
        space().width(Fill),
        reset_chip(active_count),
    ]
    .spacing(12)
    .align_y(Center)
    .into()
}

fn mongo_label(value: &'static str) -> Element<'static, Message> {
    boxed_text(value, 11.0, style::MEDIUM, XS_11, style::text_faint).into()
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
    label_text: String,
    active: bool,
    kind: PlanChipKind,
    message: Option<Message>,
) -> Element<'static, Message> {
    let mut content = row![].spacing(4).align_y(Center);
    let font = if matches!(kind, PlanChipKind::Collscan) {
        style::SEMIBOLD
    } else {
        style::MEDIUM
    };
    if matches!(kind, PlanChipKind::Collscan) {
        content = content.push(icons::icon("flame", 12.0, style::AMBER_500));
    }
    content = content.push(boxed_text(
        label_text,
        12.0,
        font,
        crate::ui::lh::TEXT_XS,
        style::text_inherit,
    ));

    let padding = Padding {
        top: 2.0,
        right: 8.0,
        bottom: 2.0,
        left: 8.0,
    };

    let surface = match kind {
        PlanChipKind::Neutral => container(content)
            .padding(padding)
            .style(move |theme: &iced::Theme| style::mongo_chip(theme, active)),
        PlanChipKind::Collscan => container(content)
            .padding(padding)
            .style(move |theme: &iced::Theme| style::collscan_filter_chip(theme, active)),
        PlanChipKind::Ixscan => container(content)
            .padding(padding)
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
        container(boxed_text(
            label_text.to_string(),
            12.0,
            style::MEDIUM,
            crate::ui::lh::TEXT_XS,
            style::text_inherit,
        ))
        .padding([2, 8])
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
                boxed_text(
                    "Scan Ratio >100x",
                    12.0,
                    style::MEDIUM,
                    crate::ui::lh::TEXT_XS,
                    style::text_inherit,
                ),
            ]
            .spacing(4)
            .align_y(Center),
        )
        .padding([3, 9])
        .style(move |theme| style::scan_ratio_chip(theme, active)),
    )
    .on_press(Message::MongoScanRatioToggled)
    .padding(0)
    .style(style::transparent_button)
    .into()
}

fn reset_chip(active_count: usize) -> Element<'static, Message> {
    let active = active_count > 0;
    let mut content = row![
        icons::icon(
            "rotate-ccw",
            12.0,
            if active {
                style::ROSE_700
            } else {
                style::SLATE_400
            }
        ),
        boxed_text(
            "Reset all filters",
            12.0,
            style::SEMIBOLD,
            crate::ui::lh::TEXT_XS,
            if active {
                style::text_reset_active
            } else {
                style::text_reset_idle
            },
        ),
    ]
    .spacing(6)
    .align_y(Center);

    if active {
        content = content.push(
            container(boxed_text(
                format_num(active_count as u64),
                10.0,
                style::BOLD,
                XS_10,
                style::text_reset_count,
            ))
            .padding([0, 6])
            .style(style::reset_count),
        );
    }

    button(content)
        .on_press_maybe(active.then_some(Message::MongoResetFilters))
        .padding([4.0 + BORDER, 10.0 + BORDER])
        .style(style::reset_chip(active))
        .into()
}
