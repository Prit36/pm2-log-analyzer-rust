#!/usr/bin/env python3
"""Patch mongo/chart.rs for the Axis color argument (UTF-8 safe)."""
import io
import sys

path = "src/ui/mongo/chart.rs"
with io.open(path, "r", encoding="utf-8") as f:
    src = f.read()

def sub(old, new, count=1):
    global src
    if old not in src:
        print("MISSING:", old[:80].replace("\n", "\\n"))
        sys.exit(1)
    src = src.replace(old, new, count)

# Time-bucket chart: two build_numeric_axis calls.
sub(
    """        &build_numeric_axis(
            &left_ticks,
            AxisKind::NumberCount,
            Orientation::Left,
            plot,
            plot.x - 8.0,
            plot.x,
        )""",
    """        &build_numeric_axis(
            &left_ticks,
            AxisKind::NumberCount,
            Orientation::Left,
            plot,
            plot.x - 8.0,
            plot.x,
            tick_color(is_dark),
        )""",
)
sub(
    """        &build_numeric_axis(
            &left_ticks,
            AxisKind::NumberCount,
            Orientation::Left,
            plot,
            plot.x - 8.0,
            plot.x,
        ),""",
    """        &build_numeric_axis(
            &left_ticks,
            AxisKind::NumberCount,
            Orientation::Left,
            plot,
            plot.x - 8.0,
            plot.x,
            tick_color(is_dark),
        ),""",
)

# Slow-query timeline.
sub(
    """    let axis = build_numeric_axis(
        &ticks,
        AxisKind::NumberCount,
        Orientation::Left,
        plot,
        plot.x - 8.0,
        plot.x,
    );""",
    """    let axis = build_numeric_axis(
        &ticks,
        AxisKind::NumberCount,
        Orientation::Left,
        plot,
        plot.x - 8.0,
        plot.x,
        tick_color(is_dark),
    );""",
)

# Unit-suffixed axis helper: thread the label color through.
sub(
    """fn right_axis_with_unit(ticks: &[f64], plot: Rect) -> crate::ui::charts::Axis {
    numeric_axis_with_unit(
        ticks,
        AxisKind::NumberPlain,
        Orientation::Right,
        plot,
        plot.x + plot.w + 8.0,
        plot.x + plot.w,
        "ms",
    )
}""",
    """fn right_axis_with_unit(ticks: &[f64], plot: Rect, color: &'static str) -> crate::ui::charts::Axis {
    numeric_axis_with_unit(
        ticks,
        AxisKind::NumberPlain,
        Orientation::Right,
        plot,
        plot.x + plot.w + 8.0,
        plot.x + plot.w,
        "ms",
        color,
    )
}""",
)
sub(
    """fn numeric_axis_with_unit(
    ticks: &[f64],
    kind: AxisKind,
    orientation: Orientation,
    plot: Rect,
    label_offset: f64,
    axis_line: f64,
    unit: &str,
) -> crate::ui::charts::Axis {
    let mut axis = build_numeric_axis(ticks, kind, orientation, plot, label_offset, axis_line);""",
    """fn numeric_axis_with_unit(
    ticks: &[f64],
    kind: AxisKind,
    orientation: Orientation,
    plot: Rect,
    label_offset: f64,
    axis_line: f64,
    unit: &str,
    color: &'static str,
) -> crate::ui::charts::Axis {
    let mut axis =
        build_numeric_axis(ticks, kind, orientation, plot, label_offset, axis_line, color);""",
)
sub("        &right_axis_with_unit(&right_ticks, plot),", "        &right_axis_with_unit(&right_ticks, plot, tick_color(is_dark)),")
sub(
    """    let axis = numeric_axis_with_unit(
        &ticks,
        AxisKind::NumberPlain,
        Orientation::Bottom,
        plot,
        plot.y + plot.h + 16.0,
        plot.y + plot.h,
        "s",
    );""",
    """    let axis = numeric_axis_with_unit(
        &ticks,
        AxisKind::NumberPlain,
        Orientation::Bottom,
        plot,
        plot.y + plot.h + 16.0,
        plot.y + plot.h,
        "s",
        tick_color(is_dark),
    );""",
)

with io.open(path, "w", encoding="utf-8", newline="\n") as f:
    f.write(src)
print("mongo/chart.rs patched")
