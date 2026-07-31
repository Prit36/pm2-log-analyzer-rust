use egui::{Color32, Context, CornerRadius, Stroke, Style, Visuals};

pub fn apply_ops_theme(ctx: &Context) {
    let mut style: Style = (*ctx.style()).clone();
    let mut visuals = Visuals::dark();

    // Dark sleek Ops palette
    visuals.window_fill = Color32::from_rgb(15, 23, 42); // slate-900
    visuals.panel_fill = Color32::from_rgb(15, 23, 42);
    visuals.faint_bg_color = Color32::from_rgb(30, 41, 59); // slate-800
    visuals.extreme_bg_color = Color32::from_rgb(2, 6, 23); // slate-950

    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 41, 59);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0f32, Color32::from_rgb(226, 232, 240));
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 41, 59);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0f32, Color32::from_rgb(203, 213, 225));
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(51, 65, 85);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0f32, Color32::from_rgb(248, 250, 252));
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

    visuals.widgets.active.bg_fill = Color32::from_rgb(99, 102, 241); // indigo accent
    visuals.widgets.active.fg_stroke = Stroke::new(1.0f32, Color32::WHITE);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);

    visuals.selection.bg_fill = Color32::from_rgb(79, 70, 229);
    visuals.selection.stroke = Stroke::new(1.0f32, Color32::WHITE);

    style.visuals = visuals;
    ctx.set_style(style);
}

pub fn get_method_color(method_str: &str) -> Color32 {
    match method_str {
        "GET" => Color32::from_rgb(59, 130, 246),     // blue-500
        "POST" => Color32::from_rgb(34, 197, 94),    // green-500
        "PUT" => Color32::from_rgb(245, 158, 11),    // amber-500
        "DELETE" => Color32::from_rgb(239, 68, 68),   // red-500
        "PATCH" => Color32::from_rgb(168, 85, 247),  // purple-500
        "OPTIONS" => Color32::from_rgb(100, 116, 139),// slate-500
        _ => Color32::from_rgb(148, 163, 184),
    }
}
