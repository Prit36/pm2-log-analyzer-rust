#!/usr/bin/env python3
"""Re-apply the chart parity edits to src/ui/charts.rs (UTF-8 safe)."""
import io
import sys

path = "src/ui/charts.rs"
with io.open(path, "r", encoding="utf-8") as f:
    src = f.read()

def sub(old, new, count=1):
    global src
    if old not in src:
        print("MISSING:", old[:90].replace("\n", "\\n"))
        sys.exit(1)
    src = src.replace(old, new, count)

# 1. Axis struct gains a label color.
sub(
    """pub(crate) struct Axis {
    pub(crate) orientation: Orientation,
    pub(crate) len: f64,
    pub(crate) ticks: Vec<Tick>,
    pub(crate) axis_line: f64,
    label_offset: f64,
    font_family: Option<&'static str>,
    font_size: f64,
}""",
    """pub(crate) struct Axis {
    pub(crate) orientation: Orientation,
    pub(crate) len: f64,
    pub(crate) ticks: Vec<Tick>,
    pub(crate) axis_line: f64,
    label_offset: f64,
    font_family: Option<&'static str>,
    font_size: f64,
    color: &'static str,
}""",
)

# 2. build_numeric_axis gains the color parameter.
sub(
    """pub(crate) fn build_numeric_axis(
    tick_values: &[f64],
    kind: AxisKind,
    orientation: Orientation,
    plot: Rect,
    label_offset: f64,
    axis_line: f64,
) -> Axis {""",
    """pub(crate) fn build_numeric_axis(
    tick_values: &[f64],
    kind: AxisKind,
    orientation: Orientation,
    plot: Rect,
    label_offset: f64,
    axis_line: f64,
    color: &'static str,
) -> Axis {""",
)
sub(
    """        label_offset,
        font_family: None,
        font_size: FONT_SIZE,
    }
}

pub(crate) fn x_axis(""",
    """        label_offset,
        font_family: None,
        font_size: FONT_SIZE,
        color,
    }
}

pub(crate) fn x_axis(""",
)

# 3. x_axis tick color.
sub(
    """pub(crate) fn x_axis(
    ticks: &[Tick],
    _x: f64,
    axis_line: f64,
    w: f64,
    _end: f64,
    _tick_color: &str,
) -> Axis {""",
    """pub(crate) fn x_axis(
    ticks: &[Tick],
    _x: f64,
    axis_line: f64,
    w: f64,
    _end: f64,
    tick_color: &'static str,
) -> Axis {""",
)
sub(
    """        label_offset: axis_line + 8.0,
        font_family: None,
        font_size: FONT_SIZE,
    }
}""",
    """        label_offset: axis_line + 8.0,
        font_family: None,
        font_size: FONT_SIZE,
        color: tick_color,
    }
}""",
)

# 4. Margins tuples: (top, right, bottom, left) = (10, 16, 4, 0).
src = src.replace(
    "let margins = (10.0, 16.0, 0.0, 4.0);",
    "let margins = (10.0, 16.0, 4.0, 0.0);",
)

# 5. build_numeric_axis call sites gain tick_color.
for old, new in [
    (
        """                plot.x - 8.0,
                plot.x,
            );""",
        """                plot.x - 8.0,
                plot.x,
                tick_color,
            );""",
    ),
    (
        """                plot.x + plot.w + 8.0,
                plot.x + plot.w,
            );""",
        """                plot.x + plot.w + 8.0,
                plot.x + plot.w,
                tick_color,
            );""",
    ),
]:
    count = src.count(old)
    if count == 0:
        print("MISSING call-site pattern", old[:60])
        sys.exit(1)
    src = src.replace(old, new)
    print(f"patched {count} axis call site(s)")

# 6. topP95 direct Axis constructions.
sub(
    """                font_family: None,
                font_size: FONT_SIZE,
            };
            emit_axis(&mut svg, &x_axis, plot);

            let y_axis = Axis {
                orientation: Orientation::Left,
                len: plot.h,
                ticks: y_ticks,
                axis_line: plot.x,
                label_offset: plot.x - 8.0,
                font_family: Some("IBM Plex Mono, monospace"),
                font_size: 9.0,
            };
            emit_axis(&mut svg, &y_axis, plot);
            let _ = category_tick;""",
    """                font_family: None,
                font_size: FONT_SIZE,
                color: tick_color,
            };
            emit_axis(&mut svg, &x_axis, plot);

            let y_axis = Axis {
                orientation: Orientation::Left,
                len: plot.h,
                ticks: y_ticks,
                axis_line: plot.x,
                label_offset: plot.x - 8.0,
                font_family: Some("IBM Plex Mono, monospace"),
                font_size: 9.0,
                color: category_tick,
            };
            emit_axis(&mut svg, &y_axis, plot);""",
)

# 7. emit_axis labels become glyph paths.
sub(
    """        let (tx, ty, anchor, dy) = match axis.orientation {
            Orientation::Bottom => (tick.coord, axis.label_offset, "middle", "0.71em"),
            Orientation::Left => (axis.label_offset, tick.coord, "end", "0.355em"),
            Orientation::Right => (axis.label_offset, tick.coord, "start", "0.355em"),
        };
        let font_family = match axis.font_family {
            Some(family) => format!(" font-family=\\"{family}\\""),
            None => String::new(),
        };
        svg.out.push_str(&format!(
            "<g class=\\"recharts-layer recharts-cartesian-axis-tick-label\\"><text width=\\"{}\\" orientation=\\"{orientation}\\" height=\\"{}\\" stroke=\\"none\\" font-size=\\"{}\\"{font_family} x=\\"{}\\" y=\\"{}\\" class=\\"recharts-text recharts-cartesian-axis-tick-value\\" text-anchor=\\"{anchor}\\" fill=\\"#64748b\\"><tspan x=\\"{}\\" dy=\\"{dy}\\">{}</tspan></text></g>",
            trim_num(axis_w), trim_num(axis_h), trim_num(axis.font_size),
            trim_num(tx), trim_num(ty), trim_num(tx), tick.value
        ));""",
    """        let (tx, ty, anchor, dy_em) = match axis.orientation {
            Orientation::Bottom => (tick.coord, axis.label_offset, "middle", 0.71),
            Orientation::Left => (axis.label_offset, tick.coord, "end", 0.355),
            Orientation::Right => (axis.label_offset, tick.coord, "start", 0.355),
        };
        // `resvg` cannot see the page webfont, so labels are emitted as the
        // bundled face's outlines instead of `<text>` elements.
        let face_kind = if axis.font_family.is_some() {
            crate::utils::text_path::FaceKind::MonoRegular
        } else {
            crate::utils::text_path::FaceKind::SansRegular
        };
        let anchor_kind = match anchor {
            "middle" => crate::utils::text_path::Anchor::Middle,
            "end" => crate::utils::text_path::Anchor::End,
            _ => crate::utils::text_path::Anchor::Start,
        };
        let baseline = ty + dy_em * axis.font_size;
        let d = crate::utils::text_path::path_data(
            &tick.value,
            face_kind,
            axis.font_size,
            tx,
            baseline,
            anchor_kind,
        );
        svg.out.push_str(&format!(
            "<g class=\\"recharts-layer recharts-cartesian-axis-tick-label\\"><path d=\\"{d}\\" fill=\\"{}\\" stroke=\\"none\\" class=\\"recharts-text recharts-cartesian-axis-tick-value\\"></path></g>",
            axis.color
        ));""",
)

# 8. Chart heading uses tracking-wide.
sub(
    """    let mut heading = row![text("API VISUAL ANALYTICS")
        .size(12)
        .font(style::SEMIBOLD)
        .style(style::text_body)];""",
    """    let mut heading = row![text("API VISUAL ANALYTICS")
        .size(12)
        .font(style::WIDE_SEMIBOLD)
        .style(style::text_body)];""",
)

# 9. Layout button: 22.5px box (py-0.5 + border + 11px/16.5 label).
sub(
    """    let layout_button = button(
        row![
            icons::icon(if wide { "columns2" } else { "rows2" }, 12.0, muted),
            text(if wide { "Split View" } else { "Wide View" }).size(11),
        ]
        .spacing(4)
        .align_y(Center),
    )
    .on_press(Message::ToggleChartLayout)
    .padding([4, 8])
    .style(style::btn_secondary);""",
    """    let layout_button = button(
        row![
            icons::icon(if wide { "columns2" } else { "rows2" }, 12.0, muted),
            crate::ui::boxed_text(
                if wide { "Split View" } else { "Wide View" },
                11.0,
                style::REGULAR,
                crate::ui::lh::TEXT_11,
                style::text_muted,
            ),
        ]
        .spacing(4)
        .align_y(Center),
    )
    .on_press(Message::ToggleChartLayout)
    .padding([3, 9])
    .style(style::btn_secondary);""",
)

# 10. Header: row1 + tab strip in one column with gap-2.
sub(
    """    let header = container(
        row![heading, space().width(Fill), layout_button]
            .spacing(8)
            .align_y(Center),
    )
    .width(Fill)
    .padding(Padding {
        top: 10.0,
        right: 14.0,
        bottom: 10.0,
        left: 14.0,
    });

    let mut tabs = row![].spacing(4).width(Fill);
    if multi_day {
        tabs = tabs.push(tab(app, ChartMode::DailyTrend, multi_day));
    }
    for mode in [
        ChartMode::TimeOfDay,
        ChartMode::Throughput,
        ChartMode::Distribution,
        ChartMode::TopP95,
    ] {
        tabs = tabs.push(tab(app, mode, multi_day));
    }""",
    """    let mut tabs = row![].spacing(4).width(Fill);
    if multi_day {
        tabs = tabs.push(tab(app, ChartMode::DailyTrend, multi_day));
    }
    for mode in [
        ChartMode::TimeOfDay,
        ChartMode::Throughput,
        ChartMode::Distribution,
        ChartMode::TopP95,
    ] {
        tabs = tabs.push(tab(app, mode, multi_day));
    }

    let header = container(
        column![
            row![heading, space().width(Fill), layout_button]
                .spacing(8)
                .align_y(Center),
            // The reference tab strip is flex-1 with `whitespace-nowrap`, so the
            // labels keep their natural width and the strip clips at the card.
            container(container(tabs).padding(2).style(style::segmented).clip(true),)
                .width(Fill),
        ]
        .spacing(8),
    )
    .width(Fill)
    .padding(Padding {
        top: 10.0,
        right: 14.0,
        bottom: 10.0,
        left: 14.0,
    });""",
)

# 11. Plot: legend overlays the SVG's reserved bottom strip; host pads 12 all around.
sub(
    """            column![
                stack![chart, hover_layer(app, svg_width, svg_height)],
                legend(app, is_dark),
            ]
            .spacing(4)
            .into()""",
    """            // Recharts renders the legend as HTML inside the chart host; it
            // overlays the empty strip at the bottom of the 316px SVG.
            stack![
                chart,
                hover_layer(app, svg_width, svg_height),
                container(legend(app, is_dark))
                    .width(Fill)
                    .height(Fill)
                    .align_y(iced::Bottom)
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 4.0,
                        left: 0.0,
                    }),
            ]
            .into()""",
)
sub(
    """    container(
        column![
            header,
            // The reference tab strip is flex-1 with `whitespace-nowrap`, so the
            // labels keep their natural width and the strip clips at the card.
            container(
                container(tabs)
                    .padding(2)
                    .style(style::segmented)
                    .clip(true),
            )
            .padding(Padding {
                top: 0.0,
                right: 14.0,
                bottom: 10.0,
                left: 14.0,
            }),
            container(plot)
                .width(Fill)
                .height(Length::Fixed(host_height as f32))
                .center_x(Fill)
                .padding(Padding {
                    top: 0.0,
                    right: 12.0,
                    bottom: 12.0,
                    left: 12.0,
                }),
        ]
        .spacing(0),
    )
    .width(Fill)
    .style(style::card)
    .into()
}""",
    """    container(
        column![
            header,
            container(plot)
                .width(Fill)
                .height(Length::Fixed(host_height as f32))
                .padding(Padding {
                    top: 12.0,
                    right: 12.0,
                    bottom: 12.0,
                    left: 12.0,
                }),
        ]
        .spacing(0)
        .width(Fill),
    )
    .width(Fill)
    .padding(crate::ui::BORDER)
    .style(style::card)
    .into()
}""",
)

# 12. Tab labels use the boxed line height.
sub(
    """            text(mode.label(multi_day))
                .size(12)
                .font(style::MEDIUM)
                .wrapping(iced::widget::text::Wrapping::None),""",
    """            crate::ui::boxed_text(
                mode.label(multi_day),
                12.0,
                style::MEDIUM,
                crate::ui::lh::TEXT_XS,
                style::text_chart_tab(active),
            ),""",
)

# 13. Legend dots are 14px circles with Recharts' trailing item margin.
sub(
    """fn legend(app: &App, is_dark: bool) -> Element<'_, Message> {
    let mut items = row![].spacing(10).align_y(Center);
    for (name, color) in legend_items(app.chart_mode, is_dark) {
        let color = style::from_hex(color);
        items = items.push(
            row![
                container(space().width(8.0).height(8.0))
                    .width(Length::Fixed(8.0))
                    .height(Length::Fixed(8.0))
                    .style(style::dot(color)),
                text(name).size(11).style(move |_theme| {
                    iced::widget::text::Style {
                        color: Some(color),
                    }
                }),
            ]
            .spacing(4)
            .align_y(Center),
        );
    }
    container(items).width(Fill).center_x(Fill).into()
}""",
    """fn legend(app: &App, is_dark: bool) -> Element<'_, Message> {
    let mut items = row![].spacing(10).align_y(Center);
    for (name, color) in legend_items(app.chart_mode, is_dark) {
        let color = style::from_hex(color);
        items = items.push(
            row![
                // Recharts' default `iconSize` is 14 for every `iconType`.
                container(space().width(14.0).height(14.0))
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(14.0))
                    .style(style::dot(color)),
                crate::ui::boxed_text(
                    name,
                    11.0,
                    style::REGULAR,
                    crate::ui::lh::TEXT_11,
                    move |_theme| iced::widget::text::Style { color: Some(color) },
                ),
            ]
            .spacing(4)
            .align_y(Center),
        );
    }
    // Recharts' legend items keep a trailing `margin-right: 10px`, which shifts
    // the centered group left by half of it.
    items = items.push(space().width(10.0));
    container(items)
        .width(Fill)
        .center_x(Fill)
        .padding(Padding {
            top: 0.0,
            right: 8.0,
            bottom: 0.0,
            left: 8.0,
        })
        .into()
}""",
)

with io.open(path, "w", encoding="utf-8", newline="\n") as f:
    f.write(src)
print("charts.rs patched")
