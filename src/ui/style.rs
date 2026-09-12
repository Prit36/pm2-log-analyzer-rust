//! iced style functions and the Tailwind-derived palette used by every view.
//!
//! The reference app is styled with Tailwind's slate/blue/rose ramps; iced has
//! no CSS, so each utility class used by the reference maps to one function
//! here. Every function switches on the theme so light/dark stay symmetric.

use iced::widget::{
    button, checkbox, container, pick_list, progress_bar, text, text_editor, text_input,
};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// Font families and weights used across the reference's Tailwind classes.
/// The faces are bundled, so `font-medium`/`font-semibold`/`font-bold` and the
/// `font-mono-data` family all resolve to the exact reference typography.
pub use crate::ui::fonts::{
    BOLD, MEDIUM, MONO, MONO_MEDIUM, MONO_SEMIBOLD, REGULAR, SEMIBOLD,
};

pub fn color(hex: u32) -> Color {
    Color::from_rgb8(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

/// Parses a `#rrggbb` string (chart palette constants).
pub fn from_hex(hex: &str) -> Color {
    let digits = hex.trim_start_matches('#');
    color(u32::from_str_radix(digits, 16).unwrap_or(0))
}

// Tailwind slate.
pub const SLATE_50: Color = Color::from_rgb8(0xf8, 0xfa, 0xfc);
pub const SLATE_100: Color = Color::from_rgb8(0xf1, 0xf5, 0xf9);
pub const SLATE_200: Color = Color::from_rgb8(0xe2, 0xe8, 0xf0);
pub const SLATE_300: Color = Color::from_rgb8(0xcb, 0xd5, 0xe1);
pub const SLATE_400: Color = Color::from_rgb8(0x94, 0xa3, 0xb8);
pub const SLATE_500: Color = Color::from_rgb8(0x64, 0x74, 0x8b);
pub const SLATE_600: Color = Color::from_rgb8(0x47, 0x55, 0x69);
pub const SLATE_700: Color = Color::from_rgb8(0x33, 0x41, 0x55);
pub const SLATE_800: Color = Color::from_rgb8(0x1e, 0x29, 0x3b);
pub const SLATE_900: Color = Color::from_rgb8(0x0f, 0x17, 0x2a);
pub const SLATE_950: Color = Color::from_rgb8(0x02, 0x06, 0x17);

// Tailwind blue.
pub const BLUE_50: Color = Color::from_rgb8(0xef, 0xf6, 0xff);
pub const BLUE_100: Color = Color::from_rgb8(0xdb, 0xea, 0xfe);
pub const BLUE_200: Color = Color::from_rgb8(0xbf, 0xdb, 0xfe);
pub const BLUE_400: Color = Color::from_rgb8(0x60, 0xa5, 0xfa);
pub const BLUE_500: Color = Color::from_rgb8(0x3b, 0x82, 0xf6);
pub const BLUE_600: Color = Color::from_rgb8(0x25, 0x63, 0xeb);
pub const BLUE_700: Color = Color::from_rgb8(0x1d, 0x4e, 0xd8);
pub const BLUE_800: Color = Color::from_rgb8(0x1e, 0x40, 0xaf);
pub const BLUE_950: Color = Color::from_rgb8(0x17, 0x25, 0x54);

// Tailwind rose.
pub const ROSE_50: Color = Color::from_rgb8(0xff, 0xf1, 0xf2);
pub const ROSE_100: Color = Color::from_rgb8(0xff, 0xe4, 0xe6);
pub const ROSE_200: Color = Color::from_rgb8(0xfe, 0xcd, 0xd3);
pub const ROSE_300: Color = Color::from_rgb8(0xfd, 0xa4, 0xaf);
pub const ROSE_400: Color = Color::from_rgb8(0xfb, 0x71, 0x85);
pub const ROSE_600: Color = Color::from_rgb8(0xe1, 0x1d, 0x48);
pub const ROSE_700: Color = Color::from_rgb8(0xbe, 0x12, 0x3c);
pub const ROSE_800: Color = Color::from_rgb8(0x9f, 0x12, 0x39);
pub const ROSE_900: Color = Color::from_rgb8(0x88, 0x13, 0x37);
pub const ROSE_950: Color = Color::from_rgb8(0x4c, 0x05, 0x19);

// Amber / emerald / sky accents from the reference badges.
pub const AMBER_50: Color = Color::from_rgb8(0xff, 0xfb, 0xeb);
pub const AMBER_200: Color = Color::from_rgb8(0xfd, 0xe6, 0x8a);
pub const AMBER_400: Color = Color::from_rgb8(0xfb, 0xbf, 0x24);
pub const AMBER_700: Color = Color::from_rgb8(0xb4, 0x53, 0x09);
pub const EMERALD_50: Color = Color::from_rgb8(0xec, 0xfd, 0xf5);
pub const EMERALD_200: Color = Color::from_rgb8(0xa7, 0xf3, 0xd0);
pub const EMERALD_400: Color = Color::from_rgb8(0x34, 0xd3, 0x99);
pub const EMERALD_700: Color = Color::from_rgb8(0x04, 0x78, 0x57);
pub const SKY_50: Color = Color::from_rgb8(0xf0, 0xf9, 0xff);
pub const SKY_200: Color = Color::from_rgb8(0xba, 0xe6, 0xfd);
pub const SKY_700: Color = Color::from_rgb8(0x03, 0x69, 0xa1);

// Tailwind emerald / amber extras used by the MongoDB views.
pub const EMERALD_100: Color = Color::from_rgb8(0xd1, 0xfa, 0xe5);
pub const EMERALD_300: Color = Color::from_rgb8(0x6e, 0xe7, 0xb7);
pub const EMERALD_500: Color = Color::from_rgb8(0x10, 0xb9, 0x81);
pub const EMERALD_600: Color = Color::from_rgb8(0x05, 0x96, 0x69);
pub const EMERALD_800: Color = Color::from_rgb8(0x06, 0x5f, 0x46);
pub const AMBER_100: Color = Color::from_rgb8(0xfe, 0xf3, 0xc7);
pub const AMBER_300: Color = Color::from_rgb8(0xfc, 0xd3, 0x4d);
pub const AMBER_500: Color = Color::from_rgb8(0xf5, 0x9e, 0x0b);
pub const AMBER_600: Color = Color::from_rgb8(0xd9, 0x77, 0x06);
pub const AMBER_800: Color = Color::from_rgb8(0x92, 0x40, 0x0e);
pub const PURPLE_50: Color = Color::from_rgb8(0xfa, 0xf5, 0xff);
pub const PURPLE_300: Color = Color::from_rgb8(0xd8, 0xb4, 0xfe);
pub const PURPLE_700: Color = Color::from_rgb8(0x7e, 0x22, 0xce);
pub const INDIGO_50: Color = Color::from_rgb8(0xee, 0xf2, 0xff);
pub const INDIGO_300: Color = Color::from_rgb8(0xa5, 0xb4, 0xfc);
pub const INDIGO_700: Color = Color::from_rgb8(0x43, 0x38, 0xca);
pub const TEAL_50: Color = Color::from_rgb8(0xf0, 0xfd, 0xfa);
pub const TEAL_300: Color = Color::from_rgb8(0x5e, 0xea, 0xd4);
pub const TEAL_700: Color = Color::from_rgb8(0x0f, 0x76, 0x6e);
/// Dark-mode badge fills (`dark:bg-sky-950/60` and friends).
pub const SKY_950: Color = Color::from_rgb8(0x08, 0x2f, 0x49);
pub const PURPLE_950: Color = Color::from_rgb8(0x3b, 0x07, 0x64);
pub const INDIGO_950: Color = Color::from_rgb8(0x1e, 0x1b, 0x4b);
pub const TEAL_950: Color = Color::from_rgb8(0x04, 0x2f, 0x2e);
pub const EMERALD_950: Color = Color::from_rgb8(0x02, 0x2c, 0x22);

pub fn dark(theme: &Theme) -> bool {
    matches!(theme, Theme::Dark)
}

fn rounded(radius: f32, color: Color, width: f32) -> Border {
    Border {
        color,
        width,
        radius: radius.into(),
    }
}

// ── App chrome ──────────────────────────────────────────────────────────────

/// Window background (`bg-slate-50 dark:bg-slate-950`).
pub fn app_style(theme: &Theme) -> iced::theme::Style {
    let (background, text) = if dark(theme) {
        (SLATE_950, SLATE_100)
    } else {
        (SLATE_50, SLATE_900)
    };
    iced::theme::Style {
        background_color: background,
        text_color: text,
    }
}

/// White card with a hairline border (`rounded border border-slate-200 bg-white`).
pub fn card(theme: &Theme) -> container::Style {
    let (background, border) = if dark(theme) {
        (SLATE_900, SLATE_800)
    } else {
        (Color::WHITE, SLATE_200)
    };
    container::Style {
        background: Some(background.into()),
        border: rounded(4.0, border, 1.0),
        ..container::Style::default()
    }
}

/// Card with a blue tint while a file drag hovers the window.
pub fn drop_zone(theme: &Theme, hovered: bool) -> container::Style {
    if !hovered {
        return card(theme);
    }
    let background = if dark(theme) { BLUE_950 } else { BLUE_50 };
    container::Style {
        background: Some(background.into()),
        border: rounded(4.0, BLUE_500, 1.0),
        ..container::Style::default()
    }
}

/// Flat panel used for the chart host (`bg-white dark:bg-slate-900/80`).
pub fn panel(theme: &Theme) -> container::Style {
    card(theme)
}

/// Page header strip (`border-b bg-white/95 dark:bg-slate-900/95`).
pub fn header(theme: &Theme) -> container::Style {
    let (background, border) = if dark(theme) {
        (SLATE_900, SLATE_800)
    } else {
        (Color::WHITE, SLATE_200)
    };
    container::Style {
        background: Some(background.into()),
        border: Border {
            color: border,
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.06),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 0.0,
        },
        ..container::Style::default()
    }
}

/// Bottom-right toast (`bg-slate-900 text-white dark:bg-slate-800`).
pub fn toast(theme: &Theme) -> container::Style {
    let background = if dark(theme) { SLATE_800 } else { SLATE_900 };
    container::Style {
        text_color: Some(if dark(theme) { SLATE_100 } else { Color::WHITE }),
        background: Some(background.into()),
        border: rounded(8.0, if dark(theme) { SLATE_700 } else { SLATE_200 }, 1.0),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 18.0,
        },
        ..container::Style::default()
    }
}

/// KPI tile background (`bg-white dark:bg-slate-900` with `gap-px` hairlines).
pub fn kpi_tile(theme: &Theme) -> container::Style {
    let background = if dark(theme) { SLATE_900 } else { Color::WHITE };
    container::Style {
        background: Some(background.into()),
        ..container::Style::default()
    }
}

/// Segmented control background (`bg-slate-100 dark:bg-slate-800`).
pub fn segmented(theme: &Theme) -> container::Style {
    let background = if dark(theme) { SLATE_800 } else { SLATE_100 };
    container::Style {
        background: Some(background.into()),
        border: rounded(12.0, Color::TRANSPARENT, 0.0),
        ..container::Style::default()
    }
}

/// Table header strip (`bg-slate-50 dark:bg-slate-950`).
pub fn table_header(theme: &Theme) -> container::Style {
    let (background, bottom) = if dark(theme) {
        (SLATE_950, SLATE_800)
    } else {
        (SLATE_50, SLATE_200)
    };
    container::Style {
        background: Some(background.into()),
        border: Border {
            color: bottom,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// Zebra-striped table row background (`border-b border-slate-100`).
pub fn table_row(theme: &Theme, even: bool) -> container::Style {
    let background = if even {
        if dark(theme) { SLATE_900 } else { Color::WHITE }
    } else if dark(theme) {
        color(0x0a0f1d)
    } else {
        color(0xfafbfc)
    };
    container::Style {
        background: Some(background.into()),
        ..container::Style::default()
    }
}

/// The `border-b` hairline the reference draws under every table row.
pub fn row_divider(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(if dark(theme) { SLATE_800 } else { SLATE_100 }.into()),
        ..container::Style::default()
    }
}

/// Blue information pill (`bg-blue-50 text-blue-700`).
pub fn info_pill(theme: &Theme) -> container::Style {
    let (background, text_color) = if dark(theme) {
        (BLUE_950, BLUE_400)
    } else {
        (BLUE_50, BLUE_700)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(999.0, Color::TRANSPARENT, 0.0),
        ..container::Style::default()
    }
}

/// Empty-state / muted text block.
pub fn empty_state(theme: &Theme) -> container::Style {
    let background = if dark(theme) { SLATE_900 } else { Color::WHITE };
    container::Style {
        text_color: Some(if dark(theme) { SLATE_500 } else { SLATE_400 }),
        background: Some(background.into()),
        border: rounded(4.0, if dark(theme) { SLATE_800 } else { SLATE_200 }, 1.0),
        ..container::Style::default()
    }
}

pub fn progress(theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: (if dark(theme) { SLATE_800 } else { SLATE_100 }).into(),
        bar: if dark(theme) { BLUE_500 } else { BLUE_600 }.into(),
        border: rounded(999.0, Color::TRANSPARENT, 0.0),
    }
}

// ── Text ────────────────────────────────────────────────────────────────────

pub fn text_primary(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { SLATE_100 } else { SLATE_900 }),
    }
}

pub fn text_heading(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { SLATE_100 } else { SLATE_900 }),
    }
}

pub fn text_muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { SLATE_400 } else { SLATE_500 }),
    }
}

pub fn text_subheading(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { SLATE_300 } else { SLATE_600 }),
    }
}

/// Default table cell copy (`text-slate-700 dark:text-slate-300`).
pub fn text_body(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { SLATE_300 } else { SLATE_700 }),
    }
}

/// Emphasised cell copy (`text-slate-800 dark:text-slate-200`).
pub fn text_strong(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { SLATE_200 } else { SLATE_800 }),
    }
}

pub fn text_faint(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { SLATE_500 } else { SLATE_400 }),
    }
}

/// Zero-count cells (`text-slate-400 dark:text-slate-600`).
pub fn text_fainter(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { SLATE_600 } else { SLATE_400 }),
    }
}

pub fn text_accent(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { BLUE_400 } else { BLUE_600 }),
    }
}

pub fn text_danger(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { ROSE_400 } else { ROSE_600 }),
    }
}

pub fn text_amber(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { AMBER_400 } else { AMBER_700 }),
    }
}

pub fn text_success(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { EMERALD_400 } else { EMERALD_700 }),
    }
}

pub fn text_white(_theme: &Theme) -> text::Style {
    text::Style {
        color: Some(Color::WHITE),
    }
}

// ── MongoDB view styles ────────────────────────────────────────────────────

/// `text-emerald-600 dark:text-emerald-400`.
pub fn text_emerald(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if dark(theme) { EMERALD_400 } else { EMERALD_600 }),
    }
}

/// `bg-emerald-50 text-emerald-700` pill (`MongoHeaderInfo` date badge).
pub fn emerald_pill(theme: &Theme) -> container::Style {
    let (background, text_color) = if dark(theme) {
        (EMERALD_950, EMERALD_300)
    } else {
        (EMERALD_50, EMERALD_700)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(999.0, Color::TRANSPARENT, 0.0),
        ..container::Style::default()
    }
}

/// Mongo KPI/diag card (`rounded-xl border ... p-3.5 shadow-xs`).
pub fn mongo_card(theme: &Theme) -> container::Style {
    let (background, border) = if dark(theme) {
        (SLATE_900, SLATE_800)
    } else {
        (Color::WHITE, SLATE_200)
    };
    container::Style {
        background: Some(background.into()),
        border: rounded(12.0, border, 1.0),
        ..container::Style::default()
    }
}

/// Amber warning card for COLLSCAN-heavy state (`bg-amber-50/50 border-amber-300`).
pub fn amber_card(theme: &Theme) -> container::Style {
    let (background, border) = if dark(theme) {
        (Color::from_rgb8(0x2b, 0x1a, 0x05), AMBER_600)
    } else {
        (AMBER_50, AMBER_300)
    };
    container::Style {
        background: Some(background.into()),
        border: rounded(12.0, border, 1.0),
        ..container::Style::default()
    }
}

/// MongoDB operation badge (`MongoPatternTable::getOpBadge`).
pub fn mongo_op_badge(op: &str) -> impl Fn(&Theme) -> container::Style + 'static {
    let op = op.to_string();
    move |theme| {
        let is_dark = dark(theme);
        let (background, text_color, ring) = match op.as_str() {
            "find" => (
                if is_dark { SKY_950 } else { SKY_50 },
                if is_dark { SKY_200 } else { SKY_700 },
                if is_dark { SKY_700 } else { SKY_200 },
            ),
            "aggregate" => (
                if is_dark { PURPLE_950 } else { PURPLE_50 },
                if is_dark { PURPLE_300 } else { PURPLE_700 },
                if is_dark { PURPLE_700 } else { PURPLE_300 },
            ),
            "distinct" => (
                if is_dark { INDIGO_950 } else { INDIGO_50 },
                if is_dark { INDIGO_300 } else { INDIGO_700 },
                if is_dark { INDIGO_700 } else { INDIGO_300 },
            ),
            "getMore" => (
                if is_dark { TEAL_950 } else { TEAL_50 },
                if is_dark { TEAL_300 } else { TEAL_700 },
                if is_dark { TEAL_700 } else { TEAL_300 },
            ),
            "update" | "findAndModify" => (
                if is_dark { color(0x451a03) } else { AMBER_50 },
                if is_dark { AMBER_300 } else { AMBER_700 },
                if is_dark { AMBER_600 } else { AMBER_300 },
            ),
            "delete" => (
                if is_dark { ROSE_950 } else { ROSE_50 },
                if is_dark { ROSE_300 } else { ROSE_700 },
                if is_dark { ROSE_700 } else { ROSE_300 },
            ),
            _ => (
                if is_dark { SLATE_800 } else { SLATE_100 },
                if is_dark { SLATE_300 } else { SLATE_700 },
                if is_dark { SLATE_700 } else { SLATE_300 },
            ),
        };
        container::Style {
            text_color: Some(text_color),
            background: Some(background.into()),
            border: rounded(4.0, ring, 1.0),
            ..container::Style::default()
        }
    }
}

/// COLLSCAN flame chip (`bg-amber-100 text-amber-800`).
pub fn collscan_chip(theme: &Theme) -> container::Style {
    let (background, text_color) = if dark(theme) {
        (color(0x451a03), AMBER_300)
    } else {
        (AMBER_100, AMBER_800)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        ..container::Style::default()
    }
}

/// Index-suggestion button (`border-emerald-200 bg-emerald-50/70`).
pub fn index_chip(theme: &Theme) -> container::Style {
    let (background, border, text_color) = if dark(theme) {
        (EMERALD_950, EMERALD_700, EMERALD_300)
    } else {
        (EMERALD_50, EMERALD_200, EMERALD_800)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(4.0, border, 1.0),
        ..container::Style::default()
    }
}

/// Button variant of [`index_chip`] with a hover edge.
pub fn index_chip_button(
    theme: &Theme,
    status: button::Status,
) -> button::Style {
    let is_dark = dark(theme);
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let (background, border, text_color) = if is_dark {
        (EMERALD_950, if hovered { EMERALD_400 } else { EMERALD_700 }, EMERALD_300)
    } else {
        (EMERALD_50, if hovered { EMERALD_400 } else { EMERALD_200 }, EMERALD_800)
    };
    button_style(Some(background), text_color, rounded(4.0, border, 1.0))
}

/// Recharts-style hover tooltip card.
pub fn chart_tooltip(theme: &Theme) -> container::Style {
    let (background, border) = if dark(theme) {
        (SLATE_800, SLATE_700)
    } else {
        (Color::WHITE, SLATE_200)
    };
    container::Style {
        background: Some(background.into()),
        border: rounded(6.0, border, 1.0),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..container::Style::default()
    }
}

/// Preformatted JSON block (`bg-slate-50 font-mono text-[11px]`).
pub fn code_block(theme: &Theme) -> container::Style {
    let (background, border) = if dark(theme) {
        (SLATE_950, SLATE_800)
    } else {
        (SLATE_50, SLATE_200)
    };
    container::Style {
        background: Some(background.into()),
        border: rounded(6.0, border, 1.0),
        ..container::Style::default()
    }
}

/// Filter chip used across the Mongo filter bar (`bg-slate-100` / active dark).
pub fn mongo_chip(theme: &Theme, active: bool) -> container::Style {
    let is_dark = dark(theme);
    let (background, text_color) = if active {
        if is_dark {
            (SLATE_200, SLATE_900)
        } else {
            (SLATE_800, Color::WHITE)
        }
    } else if is_dark {
        (SLATE_800, SLATE_400)
    } else {
        (SLATE_100, SLATE_600)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(4.0, Color::TRANSPARENT, 0.0),
        ..container::Style::default()
    }
}

/// COLLSCAN-only plan chip (`bg-amber-50 ring-amber-300`, solid when active).
pub fn collscan_filter_chip(theme: &Theme, active: bool) -> container::Style {
    let is_dark = dark(theme);
    let (background, text_color, border) = if active {
        if is_dark {
            (AMBER_500, SLATE_950, AMBER_500)
        } else {
            (AMBER_600, Color::WHITE, AMBER_600)
        }
    } else if is_dark {
        (Color::from_rgb8(0x2b, 0x1a, 0x05), AMBER_400, AMBER_600)
    } else {
        (AMBER_50, AMBER_700, AMBER_300)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(4.0, border, 1.0),
        ..container::Style::default()
    }
}

/// IXSCAN-only plan chip (`bg-emerald-600` when active).
pub fn ixscan_filter_chip(theme: &Theme, active: bool) -> container::Style {
    let is_dark = dark(theme);
    let (background, text_color) = if active {
        if is_dark {
            (EMERALD_500, SLATE_950)
        } else {
            (EMERALD_600, Color::WHITE)
        }
    } else if is_dark {
        (SLATE_800, SLATE_400)
    } else {
        (SLATE_100, SLATE_600)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(4.0, Color::TRANSPARENT, 0.0),
        ..container::Style::default()
    }
}

/// Scan-ratio toggle (`border-rose-300 bg-rose-50 text-rose-700`).
pub fn scan_ratio_chip(theme: &Theme, active: bool) -> container::Style {
    let is_dark = dark(theme);
    let (background, text_color, border) = if active {
        if is_dark {
            (ROSE_950, ROSE_300, ROSE_600)
        } else {
            (ROSE_50, ROSE_700, ROSE_300)
        }
    } else if is_dark {
        (SLATE_900, SLATE_400, SLATE_700)
    } else {
        (Color::WHITE, SLATE_600, SLATE_200)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(4.0, border, 1.0),
        ..container::Style::default()
    }
}

/// View-switcher tab (`MongoFilterBar`): white/emerald when active.
pub fn mongo_tab_button(
    active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let is_dark = dark(theme);
        let (background, text_color) = if active {
            if is_dark {
                (SLATE_900, EMERALD_400)
            } else {
                (Color::WHITE, EMERALD_700)
            }
        } else {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            if is_dark {
                (
                    if hovered { SLATE_800 } else { Color::TRANSPARENT },
                    SLATE_400,
                )
            } else {
                (
                    if hovered { SLATE_50 } else { Color::TRANSPARENT },
                    SLATE_600,
                )
            }
        };
        button_style(Some(background), text_color, rounded(6.0, Color::TRANSPARENT, 0.0))
    }
}

/// Tab count badge (`bg-emerald-100 text-emerald-800`).
pub fn tab_badge(theme: &Theme, active: bool) -> container::Style {
    let is_dark = dark(theme);
    let (background, text_color) = if active {
        if is_dark {
            (EMERALD_950, EMERALD_300)
        } else {
            (EMERALD_100, EMERALD_800)
        }
    } else if is_dark {
        (SLATE_700, SLATE_300)
    } else {
        (SLATE_200, SLATE_700)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        ..container::Style::default()
    }
}

/// Module level of a `MongoUserActivityPanel` user card.
pub fn user_card(theme: &Theme, selected: bool) -> container::Style {
    let is_dark = dark(theme);
    let (background, border) = if selected {
        if is_dark {
            (EMERALD_950, EMERALD_700)
        } else {
            (EMERALD_50, EMERALD_300)
        }
    } else if is_dark {
        (SLATE_900, SLATE_800)
    } else {
        (Color::WHITE, SLATE_200)
    };
    container::Style {
        background: Some(background.into()),
        border: rounded(12.0, border, 1.0),
        ..container::Style::default()
    }
}

/// Severity badge for diagnostics errors (`MongoDiagnosticsPanel`).
pub fn severity_badge(theme: &Theme, severity: &str) -> container::Style {
    let is_dark = dark(theme);
    let (background, text_color) = match severity {
        "F" | "E" => (
            if is_dark { ROSE_950 } else { ROSE_50 },
            if is_dark { ROSE_300 } else { ROSE_700 },
        ),
        "W" => (
            if is_dark { Color::from_rgb8(0x2b, 0x1a, 0x05) } else { AMBER_50 },
            if is_dark { AMBER_400 } else { AMBER_700 },
        ),
        _ => (
            if is_dark { SLATE_800 } else { SLATE_100 },
            if is_dark { SLATE_400 } else { SLATE_600 },
        ),
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(4.0, Color::TRANSPARENT, 0.0),
        ..container::Style::default()
    }
}

// ── Buttons ─────────────────────────────────────────────────────────────────

fn button_style(
    background: Option<Color>,
    text_color: Color,
    border: Border,
) -> button::Style {
    button::Style {
        background: background.map(Background::from),
        text_color,
        border,
        shadow: Shadow::default(),
        snap: false,
    }
}

fn hover_tint(theme: &Theme, light: Color, dark_color: Color) -> Color {
    if dark(theme) { dark_color } else { light }
}

/// `bg-blue-600 text-white hover:bg-blue-700`.
pub fn btn_primary(theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => BLUE_700,
        _ => BLUE_600,
    };
    let _ = theme;
    button_style(Some(background), Color::WHITE, Border::default())
}

/// `border-slate-200 bg-white text-slate-700 hover:bg-slate-50`.
pub fn btn_secondary(theme: &Theme, status: button::Status) -> button::Style {
    let is_dark = dark(theme);
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => {
            hover_tint(theme, SLATE_50, SLATE_700)
        }
        button::Status::Disabled => hover_tint(theme, Color::WHITE, SLATE_800),
        _ => {
            if is_dark {
                SLATE_800
            } else {
                Color::WHITE
            }
        }
    };
    let border = if is_dark { SLATE_700 } else { SLATE_200 };
    let text_color = if is_dark { SLATE_200 } else { SLATE_700 };
    button_style(Some(background), text_color, rounded(8.0, border, 1.0))
}

/// `border-rose-200 bg-rose-50 text-rose-700 hover:bg-rose-100`.
pub fn btn_danger(theme: &Theme, status: button::Status) -> button::Style {
    let is_dark = dark(theme);
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => {
            hover_tint(theme, ROSE_100, ROSE_900)
        }
        _ => {
            if is_dark {
                ROSE_950
            } else {
                ROSE_50
            }
        }
    };
    let border = if is_dark { ROSE_900 } else { ROSE_200 };
    let text_color = if is_dark { ROSE_300 } else { ROSE_700 };
    button_style(Some(background), text_color, rounded(8.0, border, 1.0))
}

/// Transparent text button (`text-slate-500 hover:text-slate-800`).
pub fn btn_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let is_dark = dark(theme);
    let text_color = match status {
        button::Status::Hovered | button::Status::Pressed => {
            if is_dark {
                SLATE_200
            } else {
                SLATE_800
            }
        }
        _ => {
            if is_dark {
                SLATE_400
            } else {
                SLATE_500
            }
        }
    };
    button_style(None, text_color, Border::default())
}

/// Hairline between KPI tiles (`gap-px bg-slate-200`).
pub fn divider(theme: &Theme) -> container::Style {
    let color = if dark(theme) { SLATE_800 } else { SLATE_200 };
    container::Style {
        background: Some(color.into()),
        ..container::Style::default()
    }
}

/// Active-filter counter pill inside the reset button.
pub fn reset_count(theme: &Theme) -> container::Style {
    let (background, text_color) = if dark(theme) {
        (ROSE_900, ROSE_200)
    } else {
        (ROSE_200, ROSE_800)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(999.0, Color::TRANSPARENT, 0.0),
        ..container::Style::default()
    }
}

/// Colored status dot (`size-1.5 rounded-full`).
pub fn dot(color: Color) -> impl Fn(&Theme) -> container::Style + 'static {
    move |_theme| container::Style {
        background: Some(color.into()),
        border: rounded(999.0, Color::TRANSPARENT, 0.0),
        ..container::Style::default()
    }
}

/// A button whose visual treatment lives entirely in its child container.
pub fn transparent_button(_theme: &Theme, _status: button::Status) -> button::Style {
    button_style(None, Color::TRANSPARENT, Border::default())
}

/// Solid blue action button without a border (`rounded bg-blue-600 text-white`).
/// Solid emerald button (`bg-emerald-600 hover:bg-emerald-500 text-white`).
pub fn btn_emerald(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => EMERALD_500,
        button::Status::Disabled => EMERALD_300,
        _ => EMERALD_600,
    };
    button_style(Some(background), Color::WHITE, Border::default())
}

/// Emerald icon plate on the Mongo drop zone (`bg-emerald-50 ring-emerald-200`).
pub fn ingest_badge(theme: &Theme) -> container::Style {
    let (background, border) = if dark(theme) {
        (EMERALD_950, EMERALD_700)
    } else {
        (EMERALD_50, EMERALD_200)
    };
    container::Style {
        background: Some(background.into()),
        border: rounded(16.0, border, 1.0),
        ..container::Style::default()
    }
}

/// Inline error strip (`border-rose-200 bg-rose-50 text-rose-700`).
pub fn danger_surface(theme: &Theme) -> container::Style {
    let (background, border, text_color) = if dark(theme) {
        (ROSE_950, ROSE_900, ROSE_300)
    } else {
        (ROSE_50, ROSE_200, ROSE_700)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(4.0, border, 1.0),
        ..container::Style::default()
    }
}

pub fn btn_solid(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => BLUE_700,
        _ => BLUE_600,
    };
    button_style(Some(background), Color::WHITE, Border::default())
}

/// App switcher tab (`rounded-lg px-3 py-1.5` with active pill).
pub fn nav_tab(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, _status| {
        let is_dark = dark(theme);
        let (background, text_color) = if active {
            (
                if is_dark { SLATE_900 } else { Color::WHITE },
                if is_dark { BLUE_400 } else { BLUE_600 },
            )
        } else {
            (
                Color::TRANSPARENT,
                if is_dark { SLATE_400 } else { SLATE_600 },
            )
        };
        button_style(Some(background), text_color, Border::default())
    }
}

/// Mongo variant of [`nav_tab`] (emerald accent).
pub fn nav_tab_mongo(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, _status| {
        let is_dark = dark(theme);
        let (background, text_color) = if active {
            (
                if is_dark { SLATE_900 } else { Color::WHITE },
                if is_dark { EMERALD_400 } else { EMERALD_700 },
            )
        } else {
            (
                Color::TRANSPARENT,
                if is_dark { SLATE_400 } else { SLATE_600 },
            )
        };
        button_style(Some(background), text_color, Border::default())
    }
}

/// Chart mode tab (`bg-white text-blue-600` when active).
pub fn chart_tab(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let is_dark = dark(theme);
        if active {
            let background = if is_dark { BLUE_600 } else { Color::WHITE };
            let text_color = if is_dark { Color::WHITE } else { BLUE_600 };
            button_style(Some(background), text_color, Border::default())
        } else {
            let text_color = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    if is_dark {
                        SLATE_200
                    } else {
                        SLATE_900
                    }
                }
                _ => {
                    if is_dark {
                        SLATE_400
                    } else {
                        SLATE_600
                    }
                }
            };
            button_style(None, text_color, Border::default())
        }
    }
}

/// Filter / method chip (`ring-1` pill).
pub fn chip(active: bool) -> impl Fn(&Theme) -> container::Style + 'static {
    move |theme| {
        let is_dark = dark(theme);
        let (background, text_color, ring) = if active {
            (
                if is_dark { BLUE_950 } else { BLUE_50 },
                if is_dark { BLUE_400 } else { BLUE_700 },
                if is_dark { BLUE_800 } else { BLUE_200 },
            )
        } else {
            (
                if is_dark { SLATE_800 } else { SLATE_50 },
                if is_dark { SLATE_400 } else { SLATE_500 },
                if is_dark { SLATE_700 } else { SLATE_200 },
            )
        };
        container::Style {
            text_color: Some(text_color),
            background: Some(background.into()),
            border: rounded(4.0, ring, 1.0),
            ..container::Style::default()
        }
    }
}

/// Date chip uses a mono font but the same ring treatment as [`chip`].
pub fn day_chip(active: bool) -> impl Fn(&Theme) -> container::Style + 'static {
    chip(active)
}

/// Reset-filters action chip.
pub fn reset_chip(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let is_dark = dark(theme);
        if !active {
            let text_color = if is_dark { SLATE_500 } else { SLATE_400 };
            return button_style(None, text_color, Border::default());
        }
        let background = match status {
            button::Status::Hovered | button::Status::Pressed => {
                if is_dark { ROSE_900 } else { ROSE_100 }
            }
            _ => {
                if is_dark {
                    ROSE_950
                } else {
                    ROSE_50
                }
            }
        };
        let border = if is_dark { ROSE_900 } else { ROSE_200 };
        let text_color = if is_dark { ROSE_300 } else { ROSE_700 };
        button_style(Some(background), text_color, rounded(6.0, border, 1.0))
    }
}

/// Sortable table header button.
pub fn sort_header(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let is_dark = dark(theme);
        let text_color = if active {
            if is_dark { BLUE_400 } else { BLUE_600 }
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => {
                    if is_dark {
                        SLATE_200
                    } else {
                        SLATE_900
                    }
                }
                _ => {
                    if is_dark {
                        SLATE_400
                    } else {
                        SLATE_500
                    }
                }
            }
        };
        button_style(None, text_color, Border::default())
    }
}

/// Endpoint path button (`font-mono hover:text-blue-600`).
/// Endpoint path link — blue on row hover (`group-hover:text-blue-600`).
pub fn path_button(row_hovered: bool) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let is_dark = dark(theme);
        let blue = if is_dark { BLUE_400 } else { BLUE_600 };
        let text_color = match status {
            button::Status::Hovered | button::Status::Pressed => blue,
            _ if row_hovered => blue,
            _ => {
                if is_dark {
                    SLATE_200
                } else {
                    SLATE_800
                }
            }
        };
        button_style(None, text_color, Border::default())
    }
}

/// HTTP method badge (`ring-1` colored label).
pub fn method_badge(method: &str) -> impl Fn(&Theme) -> container::Style + 'static {
    let method = method.to_string();
    move |theme| {
        let is_dark = dark(theme);
        let (background, text_color, ring) = match method.as_str() {
            "GET" => (
                if is_dark { color(0x062238) } else { SKY_50 },
                if is_dark { color(0x38bdf8) } else { SKY_700 },
                if is_dark { color(0x0284c7) } else { SKY_200 },
            ),
            "POST" => (
                if is_dark { color(0x06261c) } else { EMERALD_50 },
                if is_dark { EMERALD_400 } else { EMERALD_700 },
                if is_dark { color(0x059669) } else { EMERALD_200 },
            ),
            "PUT" | "PATCH" => (
                if is_dark { color(0x381a06) } else { AMBER_50 },
                if is_dark { AMBER_400 } else { AMBER_700 },
                if is_dark { color(0xd97706) } else { AMBER_200 },
            ),
            "DELETE" => (
                if is_dark { color(0x3d0818) } else { ROSE_50 },
                if is_dark { ROSE_400 } else { ROSE_700 },
                if is_dark { color(0xe11d48) } else { ROSE_200 },
            ),
            _ => (
                if is_dark { SLATE_900 } else { SLATE_50 },
                if is_dark { SLATE_400 } else { SLATE_600 },
                if is_dark { SLATE_700 } else { SLATE_200 },
            ),
        };
        container::Style {
            text_color: Some(text_color),
            background: Some(background.into()),
            border: rounded(4.0, ring, 1.0),
            ..container::Style::default()
        }
    }
}

/// File chip in the ingest panel (`bg-slate-100 dark:bg-slate-800`).
pub fn file_chip(theme: &Theme) -> container::Style {
    let (background, text_color) = if dark(theme) {
        (SLATE_800, SLATE_300)
    } else {
        (SLATE_100, SLATE_700)
    };
    container::Style {
        text_color: Some(text_color),
        background: Some(background.into()),
        border: rounded(4.0, Color::TRANSPARENT, 0.0),
        ..container::Style::default()
    }
}

/// Pending drop banner (`border-blue-200 bg-blue-50 dark:bg-slate-800`).
pub fn pending_banner(theme: &Theme) -> container::Style {
    let (background, border) = if dark(theme) {
        (SLATE_800, BLUE_950)
    } else {
        (color(0xeff6ff), BLUE_200)
    };
    container::Style {
        background: Some(background.into()),
        border: rounded(4.0, border, 1.0),
        ..container::Style::default()
    }
}

// ── Inputs ──────────────────────────────────────────────────────────────────

/// Text field (`rounded border border-slate-200 bg-white focus:border-blue-500`).
pub fn field(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let is_dark = dark(theme);
    let border_color = match status {
        text_input::Status::Focused { .. } => {
            if is_dark {
                BLUE_400
            } else {
                BLUE_500
            }
        }
        text_input::Status::Hovered => {
            if is_dark {
                SLATE_600
            } else {
                SLATE_300
            }
        }
        _ => {
            if is_dark {
                SLATE_700
            } else {
                SLATE_200
            }
        }
    };
    text_input::Style {
        background: (if is_dark { SLATE_950 } else { Color::WHITE }).into(),
        border: rounded(4.0, border_color, 1.0),
        icon: if is_dark { SLATE_400 } else { SLATE_500 },
        placeholder: if is_dark { SLATE_500 } else { SLATE_400 },
        value: if is_dark { SLATE_100 } else { SLATE_800 },
        selection: if is_dark { BLUE_800 } else { BLUE_200 },
    }
}

/// Multi-line paste editor (`bg-slate-50 dark:bg-slate-950`).
pub fn editor(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let is_dark = dark(theme);
    let border_color = match status {
        text_editor::Status::Focused { .. } => {
            if is_dark {
                BLUE_400
            } else {
                BLUE_500
            }
        }
        text_editor::Status::Hovered => {
            if is_dark {
                SLATE_600
            } else {
                SLATE_300
            }
        }
        _ => {
            if is_dark {
                SLATE_700
            } else {
                SLATE_200
            }
        }
    };
    text_editor::Style {
        background: (if is_dark { SLATE_950 } else { SLATE_50 }).into(),
        border: rounded(4.0, border_color, 1.0),
        placeholder: if is_dark { SLATE_500 } else { SLATE_400 },
        value: if is_dark { SLATE_100 } else { SLATE_800 },
        selection: if is_dark { BLUE_800 } else { BLUE_200 },
    }
}

/// Pick list (`rounded border bg-white`).
pub fn picker(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let is_dark = dark(theme);
    let border_color = match status {
        pick_list::Status::Opened { .. } => {
            if is_dark {
                BLUE_400
            } else {
                BLUE_500
            }
        }
        pick_list::Status::Hovered => {
            if is_dark {
                SLATE_600
            } else {
                SLATE_300
            }
        }
        _ => {
            if is_dark {
                SLATE_700
            } else {
                SLATE_200
            }
        }
    };
    pick_list::Style {
        text_color: if is_dark { SLATE_100 } else { SLATE_800 },
        placeholder_color: if is_dark { SLATE_500 } else { SLATE_400 },
        handle_color: if is_dark { SLATE_400 } else { SLATE_500 },
        background: (if is_dark { SLATE_950 } else { Color::WHITE }).into(),
        border: rounded(4.0, border_color, 1.0),
    }
}

/// Checkbox (`rounded border-slate-200` with a blue check).
pub fn check(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let is_dark = dark(theme);
    let is_hovered = matches!(status, checkbox::Status::Hovered { .. });
    let background = if is_dark { SLATE_950 } else { Color::WHITE };
    let border_color = if is_hovered {
        if is_dark {
            SLATE_500
        } else {
            SLATE_400
        }
    } else if is_dark {
        SLATE_600
    } else {
        SLATE_300
    };
    checkbox::Style {
        background: background.into(),
        icon_color: if is_dark { BLUE_400 } else { BLUE_600 },
        border: rounded(4.0, border_color, 1.0),
        text_color: Some(if is_dark { SLATE_300 } else { SLATE_600 }),
    }
}

/// Chart mode tab on the Mongo chart card (`MongoLatencyChart`).
pub fn mongo_chart_tab(
    theme: &Theme,
    mode: crate::store::mongo_store::MongoChartMode,
    active: bool,
    status: button::Status,
) -> button::Style {
    use crate::store::mongo_store::MongoChartMode;

    let is_dark = dark(theme);
    let accent = match mode {
        MongoChartMode::ThroughputLatency => {
            if is_dark {
                EMERALD_400
            } else {
                EMERALD_700
            }
        }
        MongoChartMode::Plans => {
            if is_dark {
                AMBER_400
            } else {
                AMBER_700
            }
        }
        MongoChartMode::TopCollections => {
            if is_dark {
                PURPLE_300
            } else {
                PURPLE_700
            }
        }
    };
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let (background, text_color) = if active {
        if is_dark {
            (SLATE_900, accent)
        } else {
            (Color::WHITE, accent)
        }
    } else if is_dark {
        (if hovered { SLATE_800 } else { Color::TRANSPARENT }, SLATE_300)
    } else {
        (if hovered { SLATE_50 } else { Color::TRANSPARENT }, SLATE_900)
    };
    button_style(Some(background), text_color, rounded(4.0, Color::TRANSPARENT, 0.0))
}
