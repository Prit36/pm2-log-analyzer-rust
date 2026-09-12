//! `MongoUserActivityPanel` — user KPIs, the users table, and the master/detail
//! inspector.
//!
//! Reference geometry at 1024x768: four 239-wide KPI cards (`p-3.5`, 40px icon
//! tile, `text-xl font-bold` value), then a `lg:flex-row` split whose table
//! side is content-sized (~674px) and whose inspector takes the rest. The
//! table columns are `[User & Client, Queries, COLLSCANs, Total Time,
//! Avg (P95), Security, Actions]` with `py-2.5` cells.

use iced::widget::{button, column, container, mouse_area, row, scrollable, space, text, text_input};
use iced::{Center, Element, Fill, Length, Padding, Right};

use crate::app::{App, Message};
use crate::core::mongo_models::{MongoUserActivity, MongoUserTopCollection};
use crate::ui::style::{self, UserTint};
use crate::ui::{BORDER, boxed_text, icons, lh};
use crate::utils::format::{format_date_time, format_ms, format_num};

/// Reference auto-layout column widths at the 1024px viewport (css px).
const COL_QUERIES: f32 = 67.55;
const COL_COLLSCANS: f32 = 91.11;
const COL_TOTAL: f32 = 73.47;
const COL_AVG: f32 = 74.63;
const COL_SECURITY: f32 = 78.83;
const COL_ACTIONS: f32 = 96.59;
/// The reference table cannot shrink below its min-content width.
const TABLE_MIN_WIDTH: f32 = 674.0;
const SEARCH_WIDTH: f32 = 200.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let Some(result) = app.mongo.result.as_ref() else {
        return container(text("")).into();
    };
    let users = &result.users;
    let connections = &result.connections;
    let filters = &app.mongo.filters;

    let named_users = users.iter().filter(|user| user.user_name != "system").count();
    let top_user = users
        .iter()
        .filter(|user| user.user_name != "system")
        .max_by_key(|user| user.slow_query_count);

    let kpis = row![
        kpi_card(
            "users",
            UserTint::Emerald,
            "Identified Users",
            format!("{named_users}"),
            Some(format!("({} total)", users.len())),
            None,
        ),
        kpi_card(
            "user-check",
            UserTint::Blue,
            "Authenticated Sessions",
            format_num(connections.auth_success),
            None,
            None,
        ),
        kpi_card(
            "shield-alert",
            if connections.auth_failed > 0 { UserTint::Rose } else { UserTint::Slate },
            "Auth / Security Fails",
            format_num(connections.auth_failed),
            None,
            (connections.auth_failed > 0).then_some(style::text_danger),
        ),
        kpi_card(
            "flame",
            UserTint::Purple,
            "Top Querying User",
            top_user.map(|user| user.user_name.clone()).unwrap_or_else(|| "None".to_string()),
            Some(format!(
                "{} queries",
                format_num(top_user.map(|user| user.slow_query_count).unwrap_or(0))
            )),
            None,
        ),
    ]
    .spacing(12)
    .width(Fill);

    let search = app.mongo.user_search.clone();
    let query = search.to_lowercase();
    let filtered: Vec<MongoUserActivity> = users
        .iter()
        .filter(|user| {
            query.is_empty()
                || user.user_name.to_lowercase().contains(&query)
                || user.app_name.to_lowercase().contains(&query)
                || user.auth_db.to_lowercase().contains(&query)
                || user.client_ips.iter().any(|ip| ip.to_lowercase().contains(&query))
        })
        .cloned()
        .collect();

    let active_filter = filters.user_filter.clone();
    let active_count = filters.active_count();
    let total_users = users.len();
    let detail_user = app.mongo.active_user_detail.clone();
    let has_detail = detail_user.is_some();
    let search_owned = search.clone();

    let split = iced::widget::responsive(move |size| {
        let table = table_card(total_users, &filtered, &search_owned, &active_filter, active_count);
        let detail: Element<'static, Message> = match &detail_user {
            Some(user) => detail_card(user, &active_filter),
            None => placeholder_card(),
        };
        let total = f64::from(size.width);
        // `lg:` keys off the 1024px viewport; the page gutters leave 992 here.
        if total >= 992.0 {
            let inner = (total - 16.0).max(0.0) as f32;
            let table_width =
                ((inner as f64 * 7.0 / 12.0) as f32).max(TABLE_MIN_WIDTH).min(inner - 260.0);
            let detail_width = (inner - table_width).max(260.0);
            row![
                container(table).width(Length::Fixed(table_width)),
                container(detail).width(Length::Fixed(detail_width)),
            ]
            .spacing(16)
            .align_y(iced::Top)
            .into()
        } else if has_detail {
            column![table, detail].spacing(16).into()
        } else {
            column![table].spacing(16).into()
        }
    });

    column![kpis, split].spacing(16).width(Fill).into()
}

#[allow(clippy::too_many_arguments)]
fn kpi_card(
    icon: &'static str,
    tint: UserTint,
    label: &'static str,
    value: String,
    value_suffix: Option<String>,
    value_style: Option<fn(&iced::Theme) -> iced::widget::text::Style>,
) -> Element<'static, Message> {
    let theme_light = iced::Theme::Light;
    let icon_color = style::user_tint_color(&theme_light, tint);

    let value_row: Element<'static, Message> = match value_suffix {
        Some(suffix) => row![
            boxed_text(
                value,
                20.0,
                style::BOLD,
                lh::TEXT_LG,
                value_style.unwrap_or(style::text_heading),
            ),
            space().width(4.0),
            boxed_text(suffix, 12.0, style::REGULAR, lh::TEXT_XS, style::text_faint),
        ]
        .align_y(iced::Bottom)
        .into(),
        None => boxed_text(
            value,
            20.0,
            style::BOLD,
            lh::TEXT_LG,
            value_style.unwrap_or(style::text_heading),
        )
        .into(),
    };

    container(
        row![
            container(icons::icon(icon, 20.0, icon_color))
                .width(Length::Fixed(40.0))
                .height(Length::Fixed(40.0))
                .center_x(Length::Fixed(40.0))
                .center_y(Length::Fixed(40.0))
                .style(move |theme: &iced::Theme| style::user_icon_tile(theme, tint)),
            column![
                boxed_text(label, 11.0, style::MEDIUM, lh::TEXT_11, style::text_muted),
                value_row,
            ]
            .spacing(0),
        ]
        .spacing(12)
        .align_y(Center),
    )
    .padding(14.0 + BORDER)
    .width(Length::FillPortion(1))
    .height(Length::Fixed(87.0))
    .style(style::mongo_card)
    .into()
}

fn table_card(
    total_users: usize,
    users: &[MongoUserActivity],
    search: &str,
    active_filter: &str,
    active_count: usize,
) -> Element<'static, Message> {
    let mut header_right = row![].spacing(8).align_y(Center);
    if active_filter != "all" {
        header_right = header_right.push(
            container(
                row![
                    boxed_text(
                        format!("User: {active_filter}"),
                        12.0,
                        style::MEDIUM,
                        lh::TEXT_XS,
                        style::text_inherit,
                    ),
                    button(icons::icon("x", 12.0, style::EMERALD_800))
                        .on_press(Message::MongoUserChanged("all".to_string()))
                        .padding(0)
                        .style(style::transparent_button),
                ]
                .spacing(6)
                .align_y(Center),
            )
            .padding([4, 8])
            .style(style::user_filter_badge),
        );
    }
    if active_count > 0 {
        header_right = header_right.push(
            button(
                row![
                    icons::icon("rotate-ccw", 12.0, style::ROSE_700),
                    boxed_text(
                        format!("Reset filters ({active_count})"),
                        12.0,
                        style::SEMIBOLD,
                        lh::TEXT_XS,
                        style::text_danger,
                    ),
                ]
                .spacing(4)
                .align_y(Center),
            )
            .on_press(Message::MongoResetFilters)
            .padding([4, 8])
            .style(style::btn_danger),
        );
    }

    let search_input = {
        let input = text_input("Filter users, IPs, apps...", search)
            .on_input(Message::MongoUserSearchChanged)
            .size(12)
            .line_height(iced::Pixels(lh::TEXT_XS))
            .padding(Padding {
                top: 5.0,
                right: 26.0,
                bottom: 5.0,
                left: 30.0,
            })
            .width(Length::Fixed(SEARCH_WIDTH))
            .style(style::mongo_search);
        let mut layers: Vec<Element<'_, Message>> = vec![
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
        if !search.is_empty() {
            layers.push(
                container(
                    button(icons::icon("x", 12.0, style::SLATE_400))
                        .on_press(Message::MongoUserSearchChanged(String::new()))
                        .padding(0)
                        .style(style::transparent_button),
                )
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
        iced::widget::Stack::with_children(layers)
    };
    header_right = header_right.push(search_input);

    let header = container(
        row![
            row![
                icons::icon("user", 16.0, style::EMERALD_600),
                boxed_text(
                    format!("User Activity Tracking ({total_users})"),
                    14.0,
                    style::SEMIBOLD,
                    lh::TEXT_SM,
                    style::text_heading,
                ),
            ]
            .spacing(8)
            .align_y(Center),
            space().width(Fill),
            header_right,
        ]
        .spacing(8)
        .align_y(Center),
    )
    .width(Fill)
    .padding(Padding {
        top: 14.0,
        right: 14.0,
        bottom: 14.0 + BORDER,
        left: 14.0,
    })
    .style(style::mongo_filter_divider);

    let mut body = column![].spacing(0).width(Fill);
    body = body.push(table_header_row());
    if users.is_empty() {
        body = body.push(
            container(boxed_text(
                "No matching user activities found.",
                12.0,
                style::REGULAR,
                lh::TEXT_XS,
                style::text_faint,
            ))
            .width(Fill)
            .center_x(Fill)
            .padding(Padding {
                top: 32.0,
                right: 0.0,
                bottom: 32.0,
                left: 0.0,
            }),
        );
    } else {
        for (index, user) in users.iter().enumerate() {
            let selected = false;
            body = body.push(user_row(index, user, selected, active_filter));
            body = body.push(crate::ui::hrule(style::row_divider));
        }
    }

    container(column![header, body].spacing(0))
        .width(Fill)
        .height(Length::Shrink)
        .style(style::mongo_card)
        .into()
}

fn table_header_row() -> Element<'static, Message> {
    container(
        row![
            container(boxed_text("User & Client", 12.0, style::BOLD, lh::TEXT_XS, style::text_muted))
                .width(Fill)
                .padding(Padding {
                    top: 10.0,
                    right: 12.0,
                    bottom: 10.0,
                    left: 16.0
                }),
            header_cell("Queries", COL_QUERIES),
            header_cell("COLLSCANs", COL_COLLSCANS),
            header_cell("Total Time", COL_TOTAL),
            header_cell("Avg (P95)", COL_AVG),
            header_cell("Security", COL_SECURITY),
            container(boxed_text("Actions", 12.0, style::BOLD, lh::TEXT_XS, style::text_muted))
                .width(Length::Fixed(COL_ACTIONS))
                .align_x(Right)
                .padding(Padding {
                    top: 10.0,
                    right: 16.0,
                    bottom: 10.0,
                    left: 12.0
                }),
        ]
        .spacing(0)
        .align_y(Center)
        .width(Fill),
    )
    .width(Fill)
    .style(style::user_table_head)
    .into()
}

fn header_cell(label: &'static str, width: f32) -> Element<'static, Message> {
    container(boxed_text(label, 12.0, style::BOLD, lh::TEXT_XS, style::text_muted))
        .width(Length::Fixed(width))
        .align_x(Right)
        .padding(Padding {
            top: 10.0,
            right: 12.0,
            bottom: 10.0,
            left: 12.0,
        })
        .into()
}

fn user_row(
    index: usize,
    user: &MongoUserActivity,
    selected: bool,
    active_filter: &str,
) -> Element<'static, Message> {
    let is_system = user.user_name == "system";
    let is_filtered = active_filter == user.user_name;
    let initial = user
        .user_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();

    let mut name_line = row![boxed_text(
        user.user_name.clone(),
        12.0,
        style::SEMIBOLD,
        lh::TEXT_XS,
        style::text_heading,
    )]
    .spacing(6)
    .align_y(Center);
    if is_filtered {
        name_line = name_line.push(
            container(boxed_text(
                "FILTERED",
                9.0,
                style::BOLD,
                lh::TEXT_10,
                style::text_emerald,
            ))
            .padding([2, 4])
            .style(style::user_ok_chip),
        );
    }
    if !user.auth_db.is_empty() {
        name_line = name_line.push(
            container(boxed_text(
                format!("@{}", user.auth_db),
                9.0,
                style::REGULAR,
                lh::TEXT_10,
                style::text_muted,
            ))
            .padding([2, 4])
            .style(style::diag_ip_count),
        );
    }

    let mut sub_line = row![].spacing(0);
    if !user.app_name.is_empty() {
        sub_line = sub_line.push(boxed_text(
            user.app_name.clone(),
            11.0,
            style::REGULAR,
            lh::TEXT_11,
            style::text_faint,
        ));
    }
    if !user.client_ips.is_empty() {
        sub_line = sub_line.push(boxed_text(
            user.client_ips.join(", "),
            10.0,
            style::MONO,
            lh::TEXT_10,
            style::text_faint,
        ));
    }

    let user_cell = row![
        container(boxed_text(
            initial,
            12.0,
            style::BOLD,
            lh::TEXT_XS,
            style::text_inherit,
        ))
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0))
        .center_x(Length::Fixed(24.0))
        .center_y(Length::Fixed(24.0))
        .style(move |theme: &iced::Theme| style::user_avatar(theme, is_system)),
        column![name_line, sub_line].spacing(0),
    ]
    .spacing(8)
    .align_y(iced::Top)
    .width(Fill);

    let collscan_cell: Element<'static, Message> = if user.collscan_count > 0 {
        container(
            row![
                icons::icon("flame", 10.0, style::AMBER_500),
                boxed_text(
                    format_num(user.collscan_count),
                    12.0,
                    style::SEMIBOLD,
                    lh::TEXT_XS,
                    style::text_inherit,
                ),
            ]
            .spacing(2)
            .align_y(Center),
        )
        .padding([2, 6])
        .style(style::user_collscan_chip)
        .into()
    } else {
        boxed_text("0", 12.0, style::REGULAR, lh::TEXT_XS, style::text_faint).into()
    };

    let auth_cell: Element<'static, Message> = if user.auth_fail_count > 0 {
        container(
            row![
                icons::icon("alert-triangle", 10.0, style::ROSE_600),
                boxed_text(
                    format!("{} fails", user.auth_fail_count),
                    10.0,
                    style::BOLD,
                    lh::TEXT_10,
                    style::text_inherit,
                ),
            ]
            .spacing(4)
            .align_y(Center),
        )
        .padding([2, 6])
        .style(style::user_fail_chip)
        .into()
    } else if user.auth_success_count > 0 {
        container(
            row![
                icons::icon("circle-check", 10.0, style::EMERALD_600),
                boxed_text(
                    format!("{} auth", user.auth_success_count),
                    10.0,
                    style::REGULAR,
                    lh::TEXT_10,
                    style::text_inherit,
                ),
            ]
            .spacing(4)
            .align_y(Center),
        )
        .padding([2, 6])
        .style(style::user_ok_chip)
        .into()
    } else {
        boxed_text("-", 12.0, style::REGULAR, lh::TEXT_XS, style::text_faint).into()
    };

    let user_for_open = user.clone();
    let user_for_filter = user.user_name.clone();
    let filter_active = is_filtered;
    let actions = row![
        button(
            boxed_text(
                if filter_active { "Active" } else { "Filter" },
                11.0,
                style::MEDIUM,
                lh::TEXT_11,
                if filter_active {
                    style::text_white
                } else {
                    style::text_button
                },
            ),
        )
        .on_press(Message::MongoUserChanged(if filter_active {
            "all".to_string()
        } else {
            user_for_filter
        }))
        .padding([4, 8])
        .style(style::user_filter_button(filter_active)),
        button(icons::icon("arrow-right", 14.0, style::SLATE_400))
            .on_press(Message::MongoOpenUser(Some(Box::new(user_for_open.clone()))))
            .padding(4)
            .style(style::transparent_button),
    ]
    .spacing(4)
    .align_y(Center);

    let row_content = container(
        row![
            container(user_cell).width(Fill).padding(Padding {
                top: 10.0,
                right: 12.0,
                bottom: 10.0,
                left: 16.0,
            }),
            container(numeric_cell(format_num(user.slow_query_count), style::text_heading, style::MEDIUM))
                .width(Length::Fixed(COL_QUERIES))
                .align_x(Right)
                .padding(Padding {
                    top: 10.0,
                    right: 12.0,
                    bottom: 10.0,
                    left: 12.0,
                }),
            container(collscan_cell)
                .width(Length::Fixed(COL_COLLSCANS))
                .align_x(Right)
                .padding(Padding {
                    top: 10.0,
                    right: 12.0,
                    bottom: 10.0,
                    left: 12.0,
                }),
            container(numeric_cell(
                format!("{:.1}s", user.total_duration_ms as f64 / 1000.0),
                style::text_body,
                style::REGULAR,
            ))
            .width(Length::Fixed(COL_TOTAL))
            .align_x(Right)
            .padding(Padding {
                top: 10.0,
                right: 12.0,
                bottom: 10.0,
                left: 12.0,
            }),
            container(wrapped_cell(
                format!(
                    "{} ({})",
                    format_ms(user.avg_duration_ms),
                    format_ms(user.p95_duration_ms as f64)
                ),
                11.0,
                style::REGULAR,
                lh::TEXT_11,
                style::text_muted,
            ))
            .width(Length::Fixed(COL_AVG))
            .align_x(Right)
            .padding(Padding {
                top: 10.0,
                right: 12.0,
                bottom: 10.0,
                left: 12.0,
            }),
            container(auth_cell)
                .width(Length::Fixed(COL_SECURITY))
                .align_x(Right)
                .padding(Padding {
                    top: 10.0,
                    right: 12.0,
                    bottom: 10.0,
                    left: 12.0,
                }),
            container(actions)
                .width(Length::Fixed(COL_ACTIONS))
                .align_x(Right)
                .padding(Padding {
                    top: 10.0,
                    right: 16.0,
                    bottom: 10.0,
                    left: 12.0,
                }),
        ]
        .spacing(0)
        .align_y(Center)
        .width(Fill),
    )
    .width(Fill)
    .style(move |theme| style::user_row(theme, selected || index.is_multiple_of(2) && false));

    mouse_area(row_content)
        .on_press(Message::MongoOpenUser(Some(Box::new(user.clone()))))
        .into()
}

fn numeric_cell(
    value: String,
    text_style: fn(&iced::Theme) -> iced::widget::text::Style,
    font: iced::Font,
) -> Element<'static, Message> {
    boxed_text(value, 12.0, font, lh::TEXT_XS, text_style).into()
}

/// Like [`numeric_cell`] but allowed to wrap inside its fixed column.
fn wrapped_cell(
    value: String,
    size: f32,
    font: iced::Font,
    _line_height: f32,
    text_style: fn(&iced::Theme) -> iced::widget::text::Style,
) -> Element<'static, Message> {
    container(
        text(value)
            .size(size)
            .font(font)
            .style(text_style)
            .wrapping(iced::widget::text::Wrapping::Word),
    )
    .into()
}

fn placeholder_card() -> Element<'static, Message> {
    container(
        column![
            icons::icon("user", 40.0, style::SLATE_300),
            boxed_text(
                "Select a User to Inspect",
                12.0,
                style::MEDIUM,
                lh::TEXT_XS,
                style::text_body,
            ),
            container(
                text("Click on any user in the table to review their executed queries, affected collections, client applications, and security authorization logs.")
                    .size(12)
                    .font(style::REGULAR)
                    .style(style::text_faint)
                    .wrapping(iced::widget::text::Wrapping::Word),
            )
            .width(Length::Fixed(320.0)),
        ]
        .spacing(6)
        .align_x(Center),
    )
    .width(Fill)
    .height(Length::Fixed(212.0))
    .center_x(Fill)
    .center_y(Length::Fixed(212.0))
    .padding(32)
    .style(style::user_placeholder)
    .into()
}

fn detail_card(user: &MongoUserActivity, active_filter: &str) -> Element<'static, Message> {
    let initial = user
        .user_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    let filtered = active_filter == user.user_name;
    let uname = user.user_name.clone();

    let mut subtitle = row![].spacing(6).align_y(Center);
    if !user.auth_db.is_empty() {
        subtitle = subtitle.push(
            container(boxed_text(
                format!("DB: {}", user.auth_db),
                10.0,
                style::MEDIUM,
                lh::TEXT_10,
                style::text_muted,
            ))
            .padding([2, 6])
            .style(style::diag_ip_count),
        );
    }
    if !user.app_name.is_empty() {
        subtitle = subtitle.push(
            row![
                icons::icon("terminal", 12.0, style::SLATE_400),
                boxed_text(user.app_name.clone(), 11.0, style::REGULAR, lh::TEXT_11, style::text_muted),
            ]
            .spacing(4)
            .align_y(Center),
        );
    }

    let header = container(
        row![
            row![
                container(boxed_text(initial, 14.0, style::BOLD, lh::TEXT_SM, style::text_inherit))
                    .width(Length::Fixed(36.0))
                    .height(Length::Fixed(36.0))
                    .center_x(Length::Fixed(36.0))
                    .center_y(Length::Fixed(36.0))
                    .style(style::user_avatar_square),
                column![
                    boxed_text(user.user_name.clone(), 16.0, style::BOLD, lh::TEXT_BASE, style::text_heading),
                    subtitle,
                ]
                .spacing(0),
            ]
            .spacing(10)
            .align_y(Center),
            space().width(Fill),
            row![
                button(
                    boxed_text(
                        if filtered { "Filtered" } else { "Filter View" },
                        12.0,
                        style::SEMIBOLD,
                        lh::TEXT_XS,
                        if filtered { style::text_white } else { style::text_emerald },
                    ),
                )
                .on_press(Message::MongoUserChanged(if filtered {
                    "all".to_string()
                } else {
                    uname.clone()
                }))
                .padding([4, 10])
                .style(if filtered {
                    style::user_filter_button(true)
                } else {
                    style::user_filter_button(false)
                }),
                button(icons::icon("x", 16.0, style::SLATE_400))
                    .on_press(Message::MongoOpenUser(None))
                    .padding(4)
                    .style(style::btn_ghost),
            ]
            .spacing(4)
            .align_y(Center),
        ]
        .spacing(8)
        .align_y(iced::Top),
    )
    .width(Fill)
    .padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: 12.0 + BORDER,
        left: 0.0,
    })
    .style(style::mongo_filter_divider);

    let collscan_pct = if user.slow_query_count > 0 {
        (user.collscan_count * 100).div_ceil(user.slow_query_count)
    } else {
        0
    };

    let stats = row![
        stat_tile(
            "DB Time",
            format!("{:.2}s", user.total_duration_ms as f64 / 1000.0),
            format!("Avg {}", format_ms(user.avg_duration_ms)),
            style::text_heading,
        ),
        stat_tile(
            "Queries",
            format_num(user.slow_query_count),
            format!("P95 {}", format_ms(user.p95_duration_ms as f64)),
            style::text_heading,
        ),
        stat_tile(
            "COLLSCANs",
            format_num(user.collscan_count),
            format!("{collscan_pct}% unindexed"),
            style::text_amber,
        ),
        stat_tile(
            "Scan Ratio",
            format!("{:.1}x", user.scan_ratio),
            format!("{} docs", format_num(user.total_docs_examined)),
            style::text_heading,
        ),
    ]
    .spacing(10)
    .width(Fill);

    let mut info = column![].spacing(6).width(Fill);
    info = info.push(info_line(
        "globe",
        "Client IP Addresses:",
        if user.client_ips.is_empty() {
            "Unknown".to_string()
        } else {
            user.client_ips.join(", ")
        },
    ));
    if !user.first_active.is_empty() {
        info = info.push(info_line(
            "clock",
            "First Activity:",
            format_date_time(Some(user.first_active.as_str())),
        ));
    }
    if !user.last_active.is_empty() {
        info = info.push(info_line(
            "clock",
            "Last Activity:",
            format_date_time(Some(user.last_active.as_str())),
        ));
    }
    let auth_value = if user.auth_fail_count > 0 {
        format!(
            "{} succeeded ({} authorization failures)",
            user.auth_success_count, user.auth_fail_count
        )
    } else {
        format!("{} succeeded", user.auth_success_count)
    };
    info = info.push(info_line("shield-alert", "Auth Audit:", auth_value));

    let mut operations = row![].spacing(8);
    for (op, count) in user.operations.iter().take(6) {
        let pct = if user.slow_query_count > 0 {
            (count * 100).div_ceil(user.slow_query_count)
        } else {
            0
        };
        operations = operations.push(
            container(
                row![
                    boxed_text(op.clone(), 12.0, style::SEMIBOLD, lh::TEXT_XS, style::text_body),
                    space().width(Fill),
                    boxed_text(
                        format!("{} ({pct}%)", format_num(*count)),
                        12.0,
                        style::REGULAR,
                        lh::TEXT_XS,
                        style::text_muted,
                    ),
                ]
                .spacing(8)
                .width(Fill),
            )
            .width(Length::FillPortion(1))
            .padding(8)
            .style(style::user_info_card),
        );
    }

    let collections = user.top_collections.as_slice();
    let mut collection_table = column![].spacing(0).width(Fill);
    collection_table = collection_table.push(
        container(
            row![
                boxed_text("Collection", 10.0, style::SEMIBOLD, lh::TEXT_10, style::text_muted)
                    .width(Fill),
                small_right("Queries"),
                small_right("Time"),
                small_right("COLLSCAN"),
                small_right("Action"),
            ]
            .spacing(0)
            .width(Fill),
        )
        .width(Fill)
        .padding(Padding {
            top: 6.0,
            right: 10.0,
            bottom: 6.0,
            left: 10.0,
        })
        .style(style::user_table_head),
    );
    for top in collections {
        collection_table = collection_table.push(collection_row(top));
    }

    container(
        column![
            header,
            container(stats)
                .width(Fill)
                .padding(Padding {
                    top: 12.0,
                    right: 0.0,
                    bottom: 12.0,
                    left: 0.0,
                }),
            container(info)
                .width(Fill)
                .padding(12)
                .style(style::user_info_card),
            container(
                column![
                    boxed_text(
                        "EXECUTED OPERATIONS",
                        12.0,
                        style::WIDER_BOLD,
                        lh::TEXT_XS,
                        style::text_faint,
                    ),
                    operations,
                ]
                .spacing(8),
            )
            .width(Fill)
            .padding(Padding {
                top: 12.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            }),
            container(
                column![
                    boxed_text(
                        format!("COLLECTIONS TOUCHED ({})", collections.len()),
                        12.0,
                        style::WIDER_BOLD,
                        lh::TEXT_XS,
                        style::text_faint,
                    ),
                    container(collection_table)
                        .width(Fill)
                        .style(style::user_info_card),
                ]
                .spacing(8),
            )
            .width(Fill)
            .padding(Padding {
                top: 12.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            }),
            container(
                button(
                    row![
                        boxed_text(
                            format!(
                                "View {} queries in Slow Query Log",
                                format_num(user.slow_query_count)
                            ),
                            12.0,
                            style::SEMIBOLD,
                            lh::TEXT_XS,
                            style::text_emerald,
                        ),
                        icons::icon("external-link", 14.0, style::EMERALD_600),
                    ]
                    .spacing(6)
                    .align_y(Center),
                )
                .on_press(Message::MongoUserChanged(user.user_name.clone()))
                .padding(0)
                .style(style::transparent_button),
            )
            .width(Fill)
            .padding(Padding {
                top: 12.0 + BORDER,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .style(style::mongo_filter_divider),
        ]
        .spacing(0)
        .width(Fill),
    )
    .width(Fill)
    .padding(16.0 + BORDER)
    .style(style::mongo_card)
    .into()
}

fn stat_tile(
    label: &'static str,
    value: String,
    caption: String,
    value_style: fn(&iced::Theme) -> iced::widget::text::Style,
) -> Element<'static, Message> {
    container(
        column![
            boxed_text(label, 10.0, style::REGULAR, lh::TEXT_10, style::text_faint),
            boxed_text(value, 14.0, style::BOLD, lh::TEXT_SM, value_style),
            boxed_text(caption, 10.0, style::REGULAR, lh::TEXT_10, style::text_faint),
        ]
        .spacing(2),
    )
    .padding(10)
    .width(Length::FillPortion(1))
    .style(style::user_stat_tile)
    .into()
}

fn info_line(icon: &'static str, label: &'static str, value: String) -> Element<'static, Message> {
    row![
        row![
            icons::icon(icon, 12.0, style::SLATE_400),
            boxed_text(label, 12.0, style::REGULAR, lh::TEXT_XS, style::text_muted),
        ]
        .spacing(4)
        .align_y(Center),
        space().width(Fill),
        boxed_text(value, 12.0, style::MONO, lh::TEXT_XS, style::text_body),
    ]
    .spacing(8)
    .align_y(Center)
    .into()
}

fn small_right(label: &'static str) -> Element<'static, Message> {
    container(boxed_text(label, 10.0, style::SEMIBOLD, lh::TEXT_10, style::text_muted))
        .width(Length::Fixed(64.0))
        .align_x(Right)
        .into()
}

fn collection_row(top: &MongoUserTopCollection) -> Element<'static, Message> {
    let collscan: Element<'static, Message> = if top.collscan_count > 0 {
        boxed_text(
            format_num(top.collscan_count),
            11.0,
            style::SEMIBOLD,
            lh::TEXT_11,
            style::text_amber,
        )
        .into()
    } else {
        boxed_text("0", 11.0, style::REGULAR, lh::TEXT_11, style::text_faint).into()
    };
    container(
        row![
            container(boxed_text(top.ns.clone(), 11.0, style::MONO, lh::TEXT_11, style::text_body))
                .width(Fill),
            container(boxed_text(
                top.count.to_string(),
                11.0,
                style::MEDIUM,
                lh::TEXT_11,
                style::text_heading,
            ))
            .width(Length::Fixed(64.0))
            .align_x(Right),
            container(boxed_text(
                format!("{:.1}s", top.total_duration_ms as f64 / 1000.0),
                11.0,
                style::REGULAR,
                lh::TEXT_11,
                style::text_muted,
            ))
            .width(Length::Fixed(64.0))
            .align_x(Right),
            container(collscan)
                .width(Length::Fixed(64.0))
                .align_x(Right),
            container(
                button(boxed_text(
                    "Filter",
                    10.0,
                    style::REGULAR,
                    lh::TEXT_10,
                    style::text_emerald,
                ))
                .on_press(Message::MongoCollectionChanged(top.ns.clone()))
                .padding(0)
                .style(style::transparent_button),
            )
            .width(Length::Fixed(64.0))
            .align_x(Right),
        ]
        .spacing(0)
        .width(Fill)
        .align_y(Center),
    )
    .width(Fill)
    .padding(Padding {
        top: 6.0,
        right: 10.0,
        bottom: 6.0,
        left: 10.0,
    })
    .style(style::user_table_row)
    .into()
}

/// Scroll helper kept for the detail inspector's long lists.
#[allow(dead_code)]
fn scroll_body(content: Element<'_, Message>) -> Element<'_, Message> {
    scrollable(content).height(Length::Fixed(220.0)).into()
}
