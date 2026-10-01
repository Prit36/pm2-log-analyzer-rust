use egui::{
    Align, Color32, CornerRadius, Frame, Layout, Margin, Rect, Response, Sense, Stroke, Ui,
    UiBuilder, Vec2, pos2, vec2,
};

use super::icons::render_icon;
use super::theme::{
    Palette, TailwindColors, font_bold, font_medium, font_mono, font_regular, font_semibold,
};

/// The reference app's three button sizes (`rounded-lg px-3 py-1.5`,
/// `rounded px-3 py-1.5`, `rounded px-2.5 py-1`): height, horizontal
/// padding and corner radius in one place.
#[derive(Clone, Copy)]
pub struct BtnStyle {
    pub height: f32,
    pub pad_x: f32,
    pub radius: u8,
}

/// Compact bar actions (ingest has-data panel).
pub const SMALL: BtnStyle = BtnStyle {
    height: 24.0,
    pad_x: 10.0,
    radius: 4,
};
/// Content actions (dropzone, paste, mongo).
pub const PAD: BtnStyle = BtnStyle {
    height: 28.0,
    pad_x: 12.0,
    radius: 4,
};
/// Header actions (`rounded-lg`).
pub const HDR: BtnStyle = BtnStyle {
    height: 28.0,
    pad_x: 12.0,
    radius: 8,
};

/// The palette's disabled treatment, for any button drawn with `Sense::hover()`.
struct Disabled {
    text: Color32,
    bg: Color32,
    border: Color32,
}

impl Disabled {
    fn of(p: &Palette) -> Self {
        Self {
            text: p.dim,
            bg: p.surface,
            border: p.outline,
        }
    }
}

/// Pressed state shared by the hand-drawn buttons: pointer down on the
/// button, or Space/Enter held while the button holds keyboard focus.
fn pressed(resp: &Response) -> bool {
    resp.is_pointer_button_down_on()
        || (resp.has_focus()
            && resp
                .ctx
                .input(|i| i.key_down(egui::Key::Space) || i.key_down(egui::Key::Enter)))
}

/// The one-pixel inset focus outline ZapFast draws on every focused control.
fn paint_focus_ring(ui: &Ui, rect: Rect, corner_radius: u8) {
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(corner_radius),
        Stroke::new(1.0, Color32::WHITE.gamma_multiply(0.8)),
        egui::StrokeKind::Inside,
    );
}

/// Primary solid blue action button (matches reference `rounded bg-blue-600 px-2.5 py-1 text-xs font-medium text-white hover:bg-blue-700`).
/// Disable it with `Sense::hover()` before calling; the disabled treatment follows.
pub fn primary_button(ui: &mut Ui, text: &str, icon: Option<&str>, style: BtnStyle) -> Response {
    let enabled = ui.is_enabled();
    let font = font_medium(12.0);
    let palette = super::theme::palette(ui.visuals().dark_mode);
    let disabled = Disabled::of(&palette);
    let text_color = if enabled {
        palette.on_accent
    } else {
        disabled.text
    };
    let text_galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font, text_color);
    let text_size = text_galley.size();

    let icon_size = if icon.is_some() { 14.0 } else { 0.0 };
    let gap = if icon.is_some() { 6.0 } else { 0.0 };
    let pad_x = style.pad_x;

    let width = pad_x * 2.0 + icon_size + gap + text_size.x;
    let height = style.height;

    let (rect, response) = ui.allocate_exact_size(
        vec2(width, height),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );

    if ui.is_rect_visible(rect) {
        let fill = if !enabled {
            disabled.bg
        } else if pressed(&response) {
            TailwindColors::BLUE_800
        } else if response.hovered() {
            TailwindColors::BLUE_700
        } else {
            TailwindColors::BLUE_600
        };

        ui.painter()
            .rect_filled(rect, CornerRadius::same(style.radius), fill);

        if enabled && response.has_focus() {
            paint_focus_ring(ui, rect, style.radius);
        }

        let mut current_x = rect.left() + pad_x;
        if let Some(icon_name) = icon {
            let icon_rect = Rect::from_center_size(
                pos2(current_x + icon_size / 2.0, rect.center().y),
                Vec2::splat(icon_size),
            );
            render_icon(icon_name, text_color, icon_size).paint_at(ui, icon_rect);
            current_x += icon_size + gap;
        }

        ui.painter().galley(
            pos2(current_x, rect.center().y - text_size.y / 2.0),
            text_galley,
            text_color,
        );
    }

    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// Secondary outline button (matches reference `border border-slate-200 bg-white hover:bg-slate-50 text-slate-700 rounded px-2.5 py-1 text-xs font-medium dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700`)
pub fn secondary_button(
    ui: &mut Ui,
    text: &str,
    icon: Option<&str>,
    dark: bool,
    style: BtnStyle,
) -> Response {
    let enabled = ui.is_enabled();
    let font = font_medium(12.0);
    let palette = super::theme::palette(dark);
    let disabled = Disabled::of(&palette);
    let text_color = if enabled {
        if dark {
            TailwindColors::SLATE_200
        } else {
            TailwindColors::SLATE_700
        }
    } else {
        disabled.text
    };
    let text_galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font, text_color);
    let text_size = text_galley.size();

    let icon_size = if icon.is_some() { 14.0 } else { 0.0 };
    let gap = if icon.is_some() { 6.0 } else { 0.0 };
    let pad_x = style.pad_x;

    let width = pad_x * 2.0 + icon_size + gap + text_size.x;
    let height = style.height;

    let (rect, response) = ui.allocate_exact_size(
        vec2(width, height),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );

    if ui.is_rect_visible(rect) {
        let (bg, border) = if !enabled {
            (disabled.bg, disabled.border)
        } else if dark {
            if pressed(&response) {
                (TailwindColors::SLATE_700, TailwindColors::SLATE_600)
            } else if response.hovered() {
                (TailwindColors::SLATE_700, TailwindColors::SLATE_600)
            } else {
                (TailwindColors::SLATE_800, TailwindColors::SLATE_700)
            }
        } else {
            if pressed(&response) {
                (TailwindColors::SLATE_100, TailwindColors::SLATE_300)
            } else if response.hovered() {
                (TailwindColors::SLATE_50, TailwindColors::SLATE_300)
            } else {
                (Color32::WHITE, TailwindColors::SLATE_200)
            }
        };

        ui.painter()
            .rect_filled(rect, CornerRadius::same(style.radius), bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(style.radius),
            Stroke::new(1.0, border),
            egui::StrokeKind::Inside,
        );

        if enabled && response.has_focus() {
            paint_focus_ring(ui, rect, style.radius);
        }

        let icon_color = if !enabled {
            disabled.text
        } else if dark {
            TailwindColors::SLATE_300
        } else {
            TailwindColors::SLATE_600
        };

        let mut current_x = rect.left() + pad_x;
        if let Some(icon_name) = icon {
            let icon_rect = Rect::from_center_size(
                pos2(current_x + icon_size / 2.0, rect.center().y),
                Vec2::splat(icon_size),
            );
            render_icon(icon_name, icon_color, icon_size).paint_at(ui, icon_rect);
            current_x += icon_size + gap;
        }

        ui.painter().galley(
            pos2(current_x, rect.center().y - text_size.y / 2.0),
            text_galley,
            text_color,
        );
    }

    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// Danger/Cancel button (matches reference `border border-rose-200 bg-rose-50 px-2.5 py-1 text-xs font-medium text-rose-700 hover:bg-rose-100 dark:border-rose-900/50 dark:bg-rose-950/40 dark:text-rose-300 dark:hover:bg-rose-900/60`)
pub fn cancel_button(ui: &mut Ui, text: &str, dark: bool, style: BtnStyle) -> Response {
    let font = font_medium(12.0);
    let text_color = if dark {
        TailwindColors::ROSE_300
    } else {
        TailwindColors::ROSE_700
    };
    let text_galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font, text_color);
    let text_size = text_galley.size();

    let width = style.pad_x * 2.0 + text_size.x;
    let height = style.height;

    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());

    if ui.is_rect_visible(rect) {
        let (bg, border) = if dark {
            if pressed(&response) {
                (
                    TailwindColors::ROSE_900.gamma_multiply(0.6),
                    TailwindColors::ROSE_800,
                )
            } else if response.hovered() {
                (
                    TailwindColors::ROSE_900.gamma_multiply(0.6),
                    TailwindColors::ROSE_800,
                )
            } else {
                (
                    TailwindColors::ROSE_950.gamma_multiply(0.5),
                    TailwindColors::ROSE_900,
                )
            }
        } else {
            if pressed(&response) {
                (TailwindColors::ROSE_200, TailwindColors::ROSE_400)
            } else if response.hovered() {
                (TailwindColors::ROSE_100, TailwindColors::ROSE_300)
            } else {
                (TailwindColors::ROSE_50, TailwindColors::ROSE_200)
            }
        };

        ui.painter()
            .rect_filled(rect, CornerRadius::same(style.radius), bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(style.radius),
            Stroke::new(1.0, border),
            egui::StrokeKind::Inside,
        );

        if response.has_focus() {
            paint_focus_ring(ui, rect, style.radius);
        }

        ui.painter().galley(
            pos2(
                rect.center().x - text_size.x / 2.0,
                rect.center().y - text_size.y / 2.0,
            ),
            text_galley,
            text_color,
        );
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// Square icon button (for theme toggle, clear, etc.): reference `rounded-lg p-1.5`.
pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str, dark: bool) -> Response {
    let size = 28.0;
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::click());

    if ui.is_rect_visible(rect) {
        let (bg, border) = if dark {
            if pressed(&response) {
                (TailwindColors::SLATE_700, TailwindColors::SLATE_600)
            } else if response.hovered() {
                (TailwindColors::SLATE_700, TailwindColors::SLATE_600)
            } else {
                (TailwindColors::SLATE_800, TailwindColors::SLATE_700)
            }
        } else {
            if pressed(&response) {
                (TailwindColors::SLATE_100, TailwindColors::SLATE_300)
            } else if response.hovered() {
                (TailwindColors::SLATE_50, TailwindColors::SLATE_300)
            } else {
                (Color32::WHITE, TailwindColors::SLATE_200)
            }
        };

        ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(8),
            Stroke::new(1.0, border),
            egui::StrokeKind::Inside,
        );

        if response.has_focus() {
            paint_focus_ring(ui, rect, 8);
        }

        let icon_color = if icon == "sun" {
            TailwindColors::AMBER_400
        } else if dark {
            TailwindColors::SLATE_200
        } else {
            TailwindColors::SLATE_600
        };

        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(16.0));
        render_icon(icon, icon_color, 16.0).paint_at(ui, icon_rect);
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.on_hover_text(tooltip)
}

/// Segmented mode tabs (matches reference `rounded-xl bg-slate-100 p-1 dark:bg-slate-800`)
pub struct SegmentedItem<'a> {
    pub label: &'a str,
    pub icon: Option<&'a str>,
    pub badge: Option<&'a str>,
    pub has_dot: bool,
    pub dot_color: Color32,
    pub active_color: Option<Color32>,
}

pub fn segmented_control(
    ui: &mut Ui,
    items: &[SegmentedItem<'_>],
    selected_idx: usize,
    dark: bool,
) -> Option<usize> {
    let container_bg = if dark {
        TailwindColors::SLATE_800
    } else {
        TailwindColors::SLATE_100
    };

    let mut clicked = None;

    Frame::NONE
        .fill(container_bg)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 0.0);

                for (idx, item) in items.iter().enumerate() {
                    let is_active = idx == selected_idx;

                    let font = font_semibold(12.0);

                    let active_color = item.active_color.unwrap_or(TailwindColors::BLUE_600);
                    let active_dark_color = if active_color == TailwindColors::EMERALD_600 {
                        TailwindColors::EMERALD_400
                    } else {
                        TailwindColors::BLUE_400
                    };

                    let text_color = if is_active {
                        if dark {
                            active_dark_color
                        } else {
                            active_color
                        }
                    } else {
                        if dark {
                            TailwindColors::SLATE_400
                        } else {
                            TailwindColors::SLATE_600
                        }
                    };

                    let text_galley =
                        ui.painter()
                            .layout_no_wrap(item.label.to_string(), font, text_color);
                    let text_size = text_galley.size();

                    let icon_size = if item.icon.is_some() { 14.0 } else { 0.0 };
                    let icon_gap = if item.icon.is_some() { 5.0 } else { 0.0 };

                    let badge_width = if let Some(b) = item.badge {
                        let b_galley = ui.painter().layout_no_wrap(
                            b.to_string(),
                            font_bold(10.0),
                            Color32::WHITE,
                        );
                        b_galley.size().x + 10.0
                    } else {
                        0.0
                    };
                    let badge_gap = if item.badge.is_some() { 5.0 } else { 0.0 };
                    let dot_width = if item.has_dot { 10.0 } else { 0.0 };

                    let pad_x = 12.0;
                    let width = pad_x * 2.0
                        + icon_size
                        + icon_gap
                        + text_size.x
                        + badge_gap
                        + badge_width
                        + dot_width;
                    let height = 28.0;

                    let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());

                    if ui.is_rect_visible(rect) {
                        if is_active {
                            let fill = if dark {
                                TailwindColors::SLATE_900
                            } else {
                                Color32::WHITE
                            };
                            ui.painter().rect_filled(rect, CornerRadius::same(8), fill);
                            // Subtle shadow
                            ui.painter().rect_stroke(
                                rect,
                                CornerRadius::same(8),
                                Stroke::new(
                                    1.0,
                                    if dark {
                                        TailwindColors::SLATE_700
                                    } else {
                                        TailwindColors::SLATE_200.gamma_multiply(0.7)
                                    },
                                ),
                                egui::StrokeKind::Inside,
                            );
                        } else if pressed(&resp) || resp.hovered() {
                            let hover_fill = if dark {
                                TailwindColors::SLATE_700.gamma_multiply(if pressed(&resp) {
                                    0.8
                                } else {
                                    0.5
                                })
                            } else {
                                Color32::WHITE.gamma_multiply(if pressed(&resp) {
                                    0.9
                                } else {
                                    0.6
                                })
                            };
                            ui.painter()
                                .rect_filled(rect, CornerRadius::same(8), hover_fill);
                        }

                        let mut current_x = rect.left() + pad_x;
                        if let Some(icon_name) = item.icon {
                            let icon_color = if is_active {
                                if dark {
                                    active_dark_color
                                } else {
                                    active_color
                                }
                            } else if dark {
                                TailwindColors::SLATE_400
                            } else {
                                TailwindColors::SLATE_500
                            };
                            let icon_rect = Rect::from_center_size(
                                pos2(current_x + icon_size / 2.0, rect.center().y),
                                Vec2::splat(icon_size),
                            );
                            render_icon(icon_name, icon_color, icon_size).paint_at(ui, icon_rect);
                            current_x += icon_size + icon_gap;
                        }

                        ui.painter().galley(
                            pos2(current_x, rect.center().y - text_size.y / 2.0),
                            text_galley,
                            text_color,
                        );
                        current_x += text_size.x;

                        if let Some(b) = item.badge {
                            current_x += badge_gap;
                            let badge_font = font_bold(10.0);
                            let badge_galley = ui.painter().layout_no_wrap(
                                b.to_string(),
                                badge_font,
                                if is_active {
                                    if dark {
                                        active_dark_color
                                    } else {
                                        active_color
                                    }
                                } else {
                                    TailwindColors::SLATE_500
                                },
                            );
                            let b_rect = Rect::from_min_size(
                                pos2(current_x, rect.center().y - 8.0),
                                vec2(badge_galley.size().x + 8.0, 16.0),
                            );
                            let b_fill = if dark {
                                TailwindColors::SLATE_800
                            } else {
                                TailwindColors::SLATE_100
                            };
                            ui.painter()
                                .rect_filled(b_rect, CornerRadius::same(8), b_fill);
                            ui.painter().galley(
                                pos2(
                                    b_rect.left() + 4.0,
                                    b_rect.center().y - badge_galley.size().y / 2.0,
                                ),
                                badge_galley,
                                text_color,
                            );
                            current_x += b_rect.width();
                        }

                        if item.has_dot {
                            let dot_center = pos2(current_x + 6.0, rect.center().y);
                            ui.painter().circle_filled(dot_center, 3.5, item.dot_color);
                        }
                    }

                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        clicked = Some(idx);
                    }
                }
            });
        });

    clicked
}

/// HTTP Method Badge (matches reference `inline-block shrink-0 rounded px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide ring-1`)
pub fn method_badge(ui: &mut Ui, method: &str, dark: bool) {
    let font = font_bold(10.0);

    let (bg, text_color, border) = match method {
        "GET" => {
            if dark {
                (
                    Color32::from_rgb(0x06, 0x22, 0x38),
                    Color32::from_rgb(0x38, 0xbd, 0xf8),
                    Color32::from_rgba_premultiplied(0x02, 0x84, 0xc7, 0x99),
                )
            } else {
                (
                    TailwindColors::SKY_50,
                    TailwindColors::SKY_700,
                    TailwindColors::SKY_200,
                )
            }
        }
        "POST" => {
            if dark {
                (
                    Color32::from_rgb(0x06, 0x26, 0x1c),
                    Color32::from_rgb(0x34, 0xd3, 0x99),
                    Color32::from_rgba_premultiplied(0x05, 0x96, 0x69, 0x99),
                )
            } else {
                (
                    TailwindColors::EMERALD_50,
                    TailwindColors::EMERALD_700,
                    TailwindColors::EMERALD_200,
                )
            }
        }
        "PUT" | "PATCH" => {
            if dark {
                (
                    Color32::from_rgb(0x38, 0x1a, 0x06),
                    Color32::from_rgb(0xfb, 0xbf, 0x24),
                    Color32::from_rgba_premultiplied(0xd9, 0x77, 0x06, 0x99),
                )
            } else {
                (
                    TailwindColors::AMBER_50,
                    TailwindColors::AMBER_700,
                    TailwindColors::AMBER_200,
                )
            }
        }
        "DELETE" => {
            if dark {
                (
                    Color32::from_rgb(0x3d, 0x08, 0x18),
                    Color32::from_rgb(0xfb, 0x71, 0x85),
                    Color32::from_rgba_premultiplied(0xe1, 0x1d, 0x48, 0x99),
                )
            } else {
                (
                    TailwindColors::ROSE_50,
                    TailwindColors::ROSE_700,
                    TailwindColors::ROSE_200,
                )
            }
        }
        _ => {
            if dark {
                (
                    TailwindColors::SLATE_900,
                    TailwindColors::SLATE_400,
                    TailwindColors::SLATE_700,
                )
            } else {
                (
                    TailwindColors::SLATE_50,
                    TailwindColors::SLATE_600,
                    TailwindColors::SLATE_200,
                )
            }
        }
    };

    let text_galley = ui
        .painter()
        .layout_no_wrap(method.to_string(), font, text_color);
    let size = text_galley.size();
    let width = size.x + 12.0;
    let height = 19.0;

    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, CornerRadius::same(4), bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.0, border),
            egui::StrokeKind::Inside,
        );
        ui.painter().galley(
            pos2(
                rect.center().x - size.x / 2.0,
                rect.center().y - size.y / 2.0,
            ),
            text_galley,
            text_color,
        );
    }
}

/// Filter chip button (matches reference `rounded px-2 py-0.5 text-[10px] font-medium tracking-wide ring-1 transition-colors`; date chips use the mono face).
pub fn filter_chip(ui: &mut Ui, label: &str, active: bool, dark: bool) -> Response {
    chip(ui, label, active, dark, false)
}

/// Method chip: `text-[10px] font-bold uppercase tracking-wide ring-1`.
pub fn method_chip(ui: &mut Ui, label: &str, active: bool, dark: bool) -> Response {
    chip(ui, label, active, dark, true)
}

fn chip(ui: &mut Ui, label: &str, active: bool, dark: bool, bold_upper: bool) -> Response {
    let font = if bold_upper {
        font_bold(10.0)
    } else {
        font_mono(10.0)
    };

    let (bg, text_color, border) = if active {
        if dark {
            (
                TailwindColors::BLUE_950.gamma_multiply(0.6),
                TailwindColors::BLUE_300,
                TailwindColors::BLUE_800,
            )
        } else {
            (
                TailwindColors::BLUE_50,
                TailwindColors::BLUE_700,
                TailwindColors::BLUE_200,
            )
        }
    } else {
        if dark {
            (
                TailwindColors::SLATE_800.gamma_multiply(0.6),
                TailwindColors::SLATE_400,
                TailwindColors::SLATE_700,
            )
        } else {
            (
                TailwindColors::SLATE_50,
                TailwindColors::SLATE_500,
                TailwindColors::SLATE_200,
            )
        }
    };

    let text_galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font, text_color);
    let text_size = text_galley.size();
    let width = text_size.x + 16.0;
    let height = 20.0;

    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());

    if ui.is_rect_visible(rect) {
        let hover_bg = if response.hovered() && !active {
            if dark {
                TailwindColors::SLATE_800
            } else {
                TailwindColors::SLATE_100
            }
        } else {
            bg
        };

        ui.painter()
            .rect_filled(rect, CornerRadius::same(4), hover_bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.0, border),
            egui::StrokeKind::Inside,
        );

        ui.painter().galley(
            pos2(
                rect.center().x - text_size.x / 2.0,
                rect.center().y - text_size.y / 2.0,
            ),
            text_galley,
            text_color,
        );
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// Search input field with search icon and clear 'X' button
pub fn search_input(
    ui: &mut Ui,
    id_str: &str,
    text: &mut String,
    placeholder: &str,
    width: f32,
    dark: bool,
) -> Response {
    let height = 30.0;
    let (rect, _response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

    let (bg, border, text_color, placeholder_color) = if dark {
        (
            TailwindColors::SLATE_950,
            TailwindColors::SLATE_700,
            TailwindColors::SLATE_100,
            TailwindColors::SLATE_500,
        )
    } else {
        (
            Color32::WHITE,
            TailwindColors::SLATE_200,
            TailwindColors::SLATE_800,
            TailwindColors::SLATE_400,
        )
    };

    let id = egui::Id::new(id_str);
    let has_focus = ui.memory(|m| m.has_focus(id));
    let active_border = if has_focus {
        if dark {
            TailwindColors::BLUE_400
        } else {
            TailwindColors::BLUE_500
        }
    } else {
        border
    };

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, CornerRadius::same(4), bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(if has_focus { 1.5 } else { 1.0 }, active_border),
            egui::StrokeKind::Inside,
        );

        // Search icon
        let icon_rect =
            Rect::from_center_size(pos2(rect.left() + 14.0, rect.center().y), Vec2::splat(13.0));
        render_icon("search", placeholder_color, 13.0).paint_at(ui, icon_rect);
    }

    let edit_rect = Rect::from_min_max(
        pos2(rect.left() + 26.0, rect.top() + 1.0),
        pos2(
            rect.right() - if text.is_empty() { 6.0 } else { 22.0 },
            rect.bottom() - 1.0,
        ),
    );

    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(edit_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );

    let edit_resp = child.add(
        egui::TextEdit::singleline(text)
            .id(id)
            .hint_text(
                egui::RichText::new(placeholder)
                    .color(placeholder_color)
                    .font(font_regular(12.0)),
            )
            .font(font_regular(12.0))
            .text_color(text_color)
            .frame(egui::Frame::NONE)
            .desired_width(edit_rect.width()),
    );

    if !text.is_empty() {
        let clear_rect = Rect::from_center_size(
            pos2(rect.right() - 12.0, rect.center().y),
            Vec2::splat(16.0),
        );
        let clear_resp = ui.interact(clear_rect, ui.id().with((id_str, "clear")), Sense::click());
        if ui.is_rect_visible(clear_rect) {
            let col = if clear_resp.hovered() {
                text_color
            } else {
                placeholder_color
            };
            ui.painter().line_segment(
                [
                    pos2(clear_rect.left() + 4.0, clear_rect.top() + 4.0),
                    pos2(clear_rect.right() - 4.0, clear_rect.bottom() - 4.0),
                ],
                Stroke::new(1.2, col),
            );
            ui.painter().line_segment(
                [
                    pos2(clear_rect.left() + 4.0, clear_rect.bottom() - 4.0),
                    pos2(clear_rect.right() - 4.0, clear_rect.top() + 4.0),
                ],
                Stroke::new(1.2, col),
            );
        }
        if clear_resp.clicked() {
            text.clear();
            ui.memory_mut(|m| m.request_focus(id));
        }
        if clear_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    edit_resp
}

/// Sleek select dropdown matching reference app fieldClass
pub fn select_dropdown<T: Clone + PartialEq>(
    ui: &mut Ui,
    id_str: &str,
    selected: &T,
    options: &[(T, &str)],
    dark: bool,
) -> Option<T> {
    let mut chosen = None;
    let current_label = options
        .iter()
        .find(|(val, _)| val == selected)
        .map(|(_, l)| *l)
        .unwrap_or("Select");

    let combo = egui::ComboBox::from_id_salt(id_str)
        .selected_text(
            egui::RichText::new(current_label)
                .font(font_regular(12.0))
                .color(if dark {
                    TailwindColors::SLATE_100
                } else {
                    TailwindColors::SLATE_800
                }),
        )
        .height(240.0);

    combo.show_ui(ui, |ui| {
        for (val, opt_label) in options {
            let is_sel = val == selected;
            let text = egui::RichText::new(*opt_label)
                .font(if is_sel {
                    font_semibold(12.0)
                } else {
                    font_regular(12.0)
                })
                .color(if is_sel {
                    if dark {
                        TailwindColors::BLUE_400
                    } else {
                        TailwindColors::BLUE_600
                    }
                } else if dark {
                    TailwindColors::SLATE_200
                } else {
                    TailwindColors::SLATE_700
                });

            if ui.selectable_label(is_sel, text).clicked() {
                chosen = Some(val.clone());
            }
        }
    });

    chosen
}

/// Sleek number input (e.g. Min ms, Top N)
pub fn number_field(ui: &mut Ui, id_str: &str, value: &mut usize, width: f32, dark: bool) -> bool {
    let mut val_str = value.to_string();
    let mut changed = false;

    let height = 30.0;
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

    let (bg, border, text_color) = if dark {
        (
            TailwindColors::SLATE_950,
            TailwindColors::SLATE_700,
            TailwindColors::SLATE_100,
        )
    } else {
        (
            Color32::WHITE,
            TailwindColors::SLATE_200,
            TailwindColors::SLATE_800,
        )
    };

    let id = egui::Id::new(id_str);
    let has_focus = ui.memory(|m| m.has_focus(id));
    let active_border = if has_focus {
        if dark {
            TailwindColors::BLUE_400
        } else {
            TailwindColors::BLUE_500
        }
    } else {
        border
    };

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, CornerRadius::same(4), bg);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(if has_focus { 1.5 } else { 1.0 }, active_border),
            egui::StrokeKind::Inside,
        );
    }

    let edit_rect = Rect::from_min_max(
        pos2(rect.left() + 8.0, rect.top() + 1.0),
        pos2(rect.right() - 8.0, rect.bottom() - 1.0),
    );

    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(edit_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );

    let resp = child.add(
        egui::TextEdit::singleline(&mut val_str)
            .id(id)
            .font(font_regular(12.0))
            .text_color(text_color)
            .frame(egui::Frame::NONE)
            .desired_width(edit_rect.width()),
    );

    if resp.changed() {
        if let Ok(num) = val_str.trim().parse::<usize>() {
            *value = num;
            changed = true;
        } else if val_str.trim().is_empty() {
            *value = 0;
            changed = true;
        }
    }

    changed
}

/// Reset filters button with active filter counter badge
pub fn reset_filters_button(ui: &mut Ui, active_count: usize, dark: bool) -> Response {
    let font = font_semibold(12.0);
    let has_active = active_count > 0;

    let (bg, border, text_color) = if has_active {
        if dark {
            (
                TailwindColors::ROSE_950.gamma_multiply(0.4),
                TailwindColors::ROSE_900.gamma_multiply(0.6),
                TailwindColors::ROSE_300,
            )
        } else {
            (
                TailwindColors::ROSE_50,
                TailwindColors::ROSE_200,
                TailwindColors::ROSE_700,
            )
        }
    } else {
        (
            Color32::TRANSPARENT,
            Color32::TRANSPARENT,
            if dark {
                TailwindColors::SLATE_500
            } else {
                TailwindColors::SLATE_400
            },
        )
    };

    let text_galley =
        ui.painter()
            .layout_no_wrap("Reset all filters".to_string(), font, text_color);
    let text_size = text_galley.size();

    let icon_size = 12.0;
    let gap = 6.0;
    let badge_width = if has_active { 20.0 } else { 0.0 };
    let width = 20.0 + icon_size + gap + text_size.x + badge_width;
    let height = 28.0;

    let (rect, resp) = ui.allocate_exact_size(
        vec2(width, height),
        if has_active {
            Sense::click()
        } else {
            Sense::hover()
        },
    );

    if ui.is_rect_visible(rect) {
        if has_active {
            let hover_bg = if resp.hovered() {
                if dark {
                    TailwindColors::ROSE_900.gamma_multiply(0.6)
                } else {
                    TailwindColors::ROSE_100
                }
            } else {
                bg
            };
            ui.painter()
                .rect_filled(rect, CornerRadius::same(6), hover_bg);
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(6),
                Stroke::new(1.0, border),
                egui::StrokeKind::Inside,
            );
        }

        let mut current_x = rect.left() + 10.0;
        let icon_rect = Rect::from_center_size(
            pos2(current_x + icon_size / 2.0, rect.center().y),
            Vec2::splat(icon_size),
        );
        render_icon("rotate-ccw", text_color, icon_size).paint_at(ui, icon_rect);
        current_x += icon_size + gap;

        ui.painter().galley(
            pos2(current_x, rect.center().y - text_size.y / 2.0),
            text_galley,
            text_color,
        );
        current_x += text_size.x + 6.0;

        if has_active {
            let b_count = active_count.to_string();
            let b_galley = ui.painter().layout_no_wrap(
                b_count,
                font_bold(10.0),
                if dark {
                    TailwindColors::ROSE_200
                } else {
                    TailwindColors::ROSE_800
                },
            );
            let b_rect =
                Rect::from_center_size(pos2(current_x + 7.0, rect.center().y), vec2(16.0, 16.0));
            let b_bg = if dark {
                TailwindColors::ROSE_900
            } else {
                TailwindColors::ROSE_200.gamma_multiply(0.8)
            };
            ui.painter()
                .rect_filled(b_rect, CornerRadius::same(8), b_bg);
            ui.painter().galley(
                pos2(
                    b_rect.center().x - b_galley.size().x / 2.0,
                    b_rect.center().y - b_galley.size().y / 2.0,
                ),
                b_galley,
                text_color,
            );
        }
    }

    if has_active && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Standard Card Container (matches reference `rounded border border-slate-200 bg-white shadow-xs dark:border-slate-800 dark:bg-slate-900`)
pub fn card_frame(dark: bool) -> Frame {
    let (bg, border) = if dark {
        (TailwindColors::SLATE_900, TailwindColors::SLATE_800)
    } else {
        (Color32::WHITE, TailwindColors::SLATE_200)
    };

    Frame::NONE
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::ZERO)
}
