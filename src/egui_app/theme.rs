use egui::epaint::Shadow;
use egui::{Color32, CornerRadius, FontFamily, Stroke, Visuals};
use fastframe_theme::{Base, Palette as FastframePalette};
use std::collections::BTreeSet;

/// The sixteen standard interface colors defined by fastframe / spotifast,
/// mapped onto the PM2 Log Analyzer Tailwind palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub dark: bool,
    pub window: Color32,
    pub panel: Color32,
    pub surface: Color32,
    pub surface_hover: Color32,
    pub surface_active: Color32,
    pub outline: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub on_accent: Color32,
    pub danger: Color32,
    pub warning: Color32,
    pub overlay: Color32,
    pub shadow: Color32,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            dark: true,
            window: TailwindColors::CANVAS_DARK,       // #0b0f19
            panel: TailwindColors::SLATE_900,          // #0f172a
            surface: TailwindColors::SLATE_900,        // #0f172a
            surface_hover: TailwindColors::SLATE_800,  // #1e293b
            surface_active: TailwindColors::SLATE_700, // #334155
            outline: TailwindColors::SLATE_800,        // #1e293b
            text: TailwindColors::SLATE_100,           // #f1f5f9
            secondary: TailwindColors::SLATE_400,      // #94a3b8
            dim: TailwindColors::SLATE_500,            // #64748b
            accent: TailwindColors::BLUE_600,          // #2563eb
            accent_hover: TailwindColors::BLUE_500,    // #3b82f6
            on_accent: Color32::WHITE,
            danger: TailwindColors::ROSE_500,   // #f43f5e
            warning: TailwindColors::AMBER_400, // #fbbf24
            overlay: TailwindColors::SLATE_950, // #020617
            shadow: Color32::from_black_alpha(120),
        }
    }

    pub fn light() -> Self {
        Self {
            dark: false,
            window: TailwindColors::CANVAS_LIGHT,      // #f7f8fa
            panel: Color32::WHITE,                     // #ffffff
            surface: Color32::WHITE,                   // #ffffff
            surface_hover: TailwindColors::SLATE_50,   // #f8fafc
            surface_active: TailwindColors::SLATE_100, // #f1f5f9
            outline: TailwindColors::SLATE_200,        // #e2e8f0
            text: TailwindColors::SLATE_900,           // #0f172a
            secondary: TailwindColors::SLATE_600,      // #475569
            dim: TailwindColors::SLATE_400,            // #94a3b8
            accent: TailwindColors::BLUE_600,          // #2563eb
            accent_hover: TailwindColors::BLUE_700,    // #1d4ed8
            on_accent: Color32::WHITE,
            danger: TailwindColors::ROSE_600,   // #e11d48
            warning: TailwindColors::AMBER_600, // #d97706
            overlay: Color32::WHITE,
            shadow: Color32::from_black_alpha(20),
        }
    }
}

impl FastframePalette for Palette {
    fn base(base: Base) -> Self {
        match base {
            Base::Dark => Self::dark(),
            Base::Light => Self::light(),
        }
    }

    fn set(&mut self, name: &str, color: Color32) -> bool {
        match name {
            "window" => self.window = color,
            "panel" => self.panel = color,
            "surface" => self.surface = color,
            "surface_hover" => self.surface_hover = color,
            "surface_active" => self.surface_active = color,
            "outline" => self.outline = color,
            "text" => self.text = color,
            "secondary" => self.secondary = color,
            "dim" => self.dim = color,
            "accent" => self.accent = color,
            "accent_hover" => self.accent_hover = color,
            "on_accent" => self.on_accent = color,
            "danger" => self.danger = color,
            "warning" => self.warning = color,
            "overlay" => self.overlay = color,
            "shadow" => self.shadow = color,
            _ => return false,
        }
        true
    }

    fn derive(&mut self, _given: &BTreeSet<&str>) {}
}

pub struct TailwindColors;

impl TailwindColors {
    // Slate
    pub const SLATE_50: Color32 = Color32::from_rgb(248, 250, 252);
    pub const SLATE_100: Color32 = Color32::from_rgb(241, 245, 249);
    pub const SLATE_200: Color32 = Color32::from_rgb(226, 232, 240);
    pub const SLATE_300: Color32 = Color32::from_rgb(203, 213, 225);
    pub const SLATE_400: Color32 = Color32::from_rgb(148, 163, 184);
    pub const SLATE_500: Color32 = Color32::from_rgb(100, 116, 139);
    pub const SLATE_600: Color32 = Color32::from_rgb(71, 85, 105);
    pub const SLATE_700: Color32 = Color32::from_rgb(51, 65, 85);
    pub const SLATE_800: Color32 = Color32::from_rgb(30, 41, 59);
    pub const SLATE_900: Color32 = Color32::from_rgb(15, 23, 42);
    pub const SLATE_950: Color32 = Color32::from_rgb(2, 6, 23);

    // Blue
    pub const BLUE_50: Color32 = Color32::from_rgb(239, 246, 255);
    pub const BLUE_100: Color32 = Color32::from_rgb(219, 234, 254);
    pub const BLUE_200: Color32 = Color32::from_rgb(191, 219, 254);
    pub const BLUE_300: Color32 = Color32::from_rgb(147, 197, 253);
    pub const BLUE_400: Color32 = Color32::from_rgb(96, 165, 250);
    pub const BLUE_500: Color32 = Color32::from_rgb(59, 130, 246);
    pub const BLUE_600: Color32 = Color32::from_rgb(37, 99, 235);
    pub const BLUE_700: Color32 = Color32::from_rgb(29, 78, 216);
    pub const BLUE_800: Color32 = Color32::from_rgb(30, 64, 175);
    pub const BLUE_900: Color32 = Color32::from_rgb(30, 58, 138);
    pub const BLUE_950: Color32 = Color32::from_rgb(23, 37, 84);

    // Emerald
    pub const EMERALD_50: Color32 = Color32::from_rgb(236, 253, 245);
    pub const EMERALD_100: Color32 = Color32::from_rgb(209, 250, 229);
    pub const EMERALD_200: Color32 = Color32::from_rgb(167, 243, 208);
    pub const EMERALD_300: Color32 = Color32::from_rgb(110, 231, 183);
    pub const EMERALD_400: Color32 = Color32::from_rgb(52, 211, 153);
    pub const EMERALD_500: Color32 = Color32::from_rgb(16, 185, 129);
    pub const EMERALD_600: Color32 = Color32::from_rgb(5, 150, 105);
    pub const EMERALD_700: Color32 = Color32::from_rgb(4, 120, 87);
    pub const EMERALD_800: Color32 = Color32::from_rgb(6, 95, 70);
    pub const EMERALD_900: Color32 = Color32::from_rgb(6, 78, 59);
    pub const EMERALD_950: Color32 = Color32::from_rgb(2, 44, 34);

    // Amber
    pub const AMBER_50: Color32 = Color32::from_rgb(255, 251, 235);
    pub const AMBER_100: Color32 = Color32::from_rgb(254, 243, 199);
    pub const AMBER_200: Color32 = Color32::from_rgb(253, 230, 138);
    pub const AMBER_300: Color32 = Color32::from_rgb(252, 211, 77);
    pub const AMBER_400: Color32 = Color32::from_rgb(251, 191, 36);
    pub const AMBER_500: Color32 = Color32::from_rgb(245, 158, 11);
    pub const AMBER_600: Color32 = Color32::from_rgb(217, 119, 6);
    pub const AMBER_700: Color32 = Color32::from_rgb(180, 83, 9);
    pub const AMBER_800: Color32 = Color32::from_rgb(146, 64, 14);
    pub const AMBER_950: Color32 = Color32::from_rgb(69, 26, 3);

    // Teal
    pub const TEAL_50: Color32 = Color32::from_rgb(240, 253, 250);
    pub const TEAL_700: Color32 = Color32::from_rgb(15, 118, 110);

    // Rose
    pub const ROSE_50: Color32 = Color32::from_rgb(255, 241, 242);
    pub const ROSE_100: Color32 = Color32::from_rgb(255, 228, 230);
    pub const ROSE_200: Color32 = Color32::from_rgb(254, 205, 211);
    pub const ROSE_300: Color32 = Color32::from_rgb(253, 164, 175);
    pub const ROSE_400: Color32 = Color32::from_rgb(251, 113, 133);
    pub const ROSE_500: Color32 = Color32::from_rgb(244, 63, 94);
    pub const ROSE_600: Color32 = Color32::from_rgb(225, 29, 72);
    pub const ROSE_700: Color32 = Color32::from_rgb(190, 18, 60);
    pub const ROSE_800: Color32 = Color32::from_rgb(159, 18, 57);
    pub const ROSE_900: Color32 = Color32::from_rgb(136, 19, 55);
    pub const ROSE_950: Color32 = Color32::from_rgb(76, 5, 25);

    // Sky
    pub const SKY_50: Color32 = Color32::from_rgb(240, 249, 255);
    pub const SKY_100: Color32 = Color32::from_rgb(224, 242, 254);
    pub const SKY_200: Color32 = Color32::from_rgb(186, 230, 253);
    pub const SKY_400: Color32 = Color32::from_rgb(56, 189, 248);
    pub const SKY_600: Color32 = Color32::from_rgb(2, 132, 199);
    pub const SKY_700: Color32 = Color32::from_rgb(3, 105, 161);
    pub const SKY_800: Color32 = Color32::from_rgb(7, 89, 133);
    pub const SKY_950: Color32 = Color32::from_rgb(8, 47, 73);

    // Purple / Indigo
    pub const INDIGO_50: Color32 = Color32::from_rgb(238, 242, 255);
    pub const INDIGO_600: Color32 = Color32::from_rgb(79, 70, 229);
    pub const INDIGO_400: Color32 = Color32::from_rgb(129, 140, 248);
    pub const PURPLE_50: Color32 = Color32::from_rgb(250, 245, 255);
    pub const PURPLE_100: Color32 = Color32::from_rgb(243, 232, 255);
    pub const PURPLE_200: Color32 = Color32::from_rgb(233, 213, 255);
    pub const PURPLE_600: Color32 = Color32::from_rgb(147, 51, 234);
    pub const PURPLE_700: Color32 = Color32::from_rgb(126, 34, 206);
    pub const PURPLE_800: Color32 = Color32::from_rgb(107, 33, 168);

    // Canvas background
    pub const CANVAS_LIGHT: Color32 = Color32::from_rgb(247, 248, 250);
    pub const CANVAS_DARK: Color32 = Color32::from_rgb(11, 15, 25);
}

/// The font data this app ships: IBM Plex, the reference app's face.
const PLEX_FONTS: &[(&str, &[u8])] = &[
    (
        "plex-regular",
        include_bytes!("../../assets/fonts/IBMPlexSans-Regular.ttf"),
    ),
    (
        "plex-medium",
        include_bytes!("../../assets/fonts/IBMPlexSans-Medium.ttf"),
    ),
    (
        "plex-semibold",
        include_bytes!("../../assets/fonts/IBMPlexSans-SemiBold.ttf"),
    ),
    (
        "plex-bold",
        include_bytes!("../../assets/fonts/IBMPlexSans-Bold.ttf"),
    ),
    (
        "plex-mono",
        include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf"),
    ),
    (
        "plex-mono-medium",
        include_bytes!("../../assets/fonts/IBMPlexMono-Medium.ttf"),
    ),
    (
        "plex-mono-semibold",
        include_bytes!("../../assets/fonts/IBMPlexMono-SemiBold.ttf"),
    ),
];

/// Sets up fonts on top of `fastframe-fonts`: IBM Plex leads every family
/// (this app's interface face, as in the reference app), with the crate's
/// Inter registration and installed system fallbacks behind it, so scripts
/// Plex does not draw (CJK, Arabic, Thai, …) render instead of showing
/// boxes. `fastframe-text` then applies the desktop's font-rendering
/// settings (hinting, antialiasing, sub-pixel positioning).
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = fastframe_fonts::FontSetup::default().definitions();

    for (name, bytes) in PLEX_FONTS {
        fonts.font_data.insert(
            (*name).to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(bytes)),
        );
    }

    // The chains fastframe built: the interface face, egui's own fonts and
    // the system fallbacks, in order.
    let sans_chain: Vec<String> = fonts.families[&FontFamily::Proportional].clone();
    let mono_chain: Vec<String> = fonts.families[&FontFamily::Monospace].clone();

    fonts.families.insert(
        FontFamily::Proportional,
        with_lead("plex-regular", &sans_chain),
    );
    fonts.families.insert(
        FontFamily::Name("plex-medium".into()),
        with_lead("plex-medium", &with_lead("plex-regular", &sans_chain)),
    );
    fonts.families.insert(
        FontFamily::Name("plex-semibold".into()),
        with_lead("plex-semibold", &with_lead("plex-regular", &sans_chain)),
    );
    fonts.families.insert(
        FontFamily::Name("plex-bold".into()),
        with_lead("plex-bold", &with_lead("plex-regular", &sans_chain)),
    );
    fonts
        .families
        .insert(FontFamily::Monospace, with_lead("plex-mono", &mono_chain));
    fonts.families.insert(
        FontFamily::Name("plex-mono-medium".into()),
        with_lead("plex-mono-medium", &with_lead("plex-mono", &mono_chain)),
    );
    fonts.families.insert(
        FontFamily::Name("plex-mono-semibold".into()),
        with_lead("plex-mono-semibold", &with_lead("plex-mono", &mono_chain)),
    );

    // Follow the desktop's font rendering settings.
    let rendering = fastframe_text::detect();
    rendering.apply_to(&mut fonts);

    ctx.set_fonts(fonts);
}

/// A family list that starts with `head` and falls back through `tail`.
fn with_lead(head: &str, tail: &[String]) -> Vec<String> {
    let mut family = Vec::with_capacity(tail.len() + 1);
    family.push(head.to_owned());
    family.extend_from_slice(tail);
    family
}

pub fn font_regular(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Proportional)
}

pub fn font_medium(size: f32) -> egui::FontId {
    egui::FontId::new(size, FontFamily::Name("plex-medium".into()))
}

pub fn font_semibold(size: f32) -> egui::FontId {
    egui::FontId::new(size, FontFamily::Name("plex-semibold".into()))
}

pub fn font_bold(size: f32) -> egui::FontId {
    egui::FontId::new(size, FontFamily::Name("plex-bold".into()))
}

pub fn font_mono(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Monospace)
}

pub fn font_mono_medium(size: f32) -> egui::FontId {
    egui::FontId::new(size, FontFamily::Name("plex-mono-medium".into()))
}

pub fn font_mono_semibold(size: f32) -> egui::FontId {
    egui::FontId::new(size, FontFamily::Name("plex-mono-semibold".into()))
}

/// The palette the app draws with in `dark` mode. Views read it instead of
/// re-deriving colours, so custom palettes flow through one place.
pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette::dark()
    } else {
        Palette::light()
    }
}

/// Tailwind's line box for `text-xs` (12px/18): egui's default row is
/// ~1.3em, which shrinks every card whose height is driven by its labels.
pub const LH_XS: f32 = 18.0;

/// Usable content width for the reference app's 1024px `lg` breakpoint,
/// after page gutters and egui's vertical scroll bar reservation.
pub const LG_CONTENT_WIDTH: f32 = 984.0;

/// Usable content width for the reference app's 640px `sm` breakpoint.
pub const SM_CONTENT_WIDTH: f32 = 600.0;

/// Line box for `text-[10px]` labels (1.5em).
pub const LH_10: f32 = 15.0;

/// Line box for `text-base` (16px/24).
pub const LH_BASE: f32 = 24.0;

pub fn apply_theme(ctx: &egui::Context, dark: bool) {
    let palette = palette(dark);

    let mut style = (*ctx.global_style()).clone();
    let visuals = &mut style.visuals;

    *visuals = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    visuals.dark_mode = dark;

    // Apply desktop text rendering coverage to visuals
    let rendering = fastframe_text::detect();
    rendering.apply_to_visuals(visuals);

    if dark {
        visuals.panel_fill = palette.window;
        visuals.window_fill = palette.panel;
        visuals.faint_bg_color = palette.surface;
        visuals.extreme_bg_color = palette.overlay;

        visuals.widgets.noninteractive.bg_fill = palette.panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.outline);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.secondary);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

        visuals.widgets.inactive.bg_fill = palette.panel;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.outline);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

        visuals.widgets.hovered.bg_fill = palette.surface_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, TailwindColors::SLATE_700);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

        visuals.widgets.active.bg_fill = palette.accent;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent_hover);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.active.corner_radius = CornerRadius::same(4);

        visuals.widgets.open.bg_fill = palette.surface_hover;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, TailwindColors::SLATE_600);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.open.corner_radius = CornerRadius::same(4);

        visuals.selection.bg_fill = palette.accent.gamma_multiply(0.35);
        visuals.selection.stroke = Stroke::new(1.0, TailwindColors::BLUE_400);

        visuals.window_shadow = Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(100),
        };
        visuals.popup_shadow = Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(120),
        };
    } else {
        visuals.panel_fill = palette.window;
        visuals.window_fill = palette.panel;
        visuals.faint_bg_color = palette.surface_hover;
        visuals.extreme_bg_color = palette.panel;

        visuals.widgets.noninteractive.bg_fill = palette.panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.outline);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.secondary);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

        visuals.widgets.inactive.bg_fill = palette.panel;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.outline);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

        visuals.widgets.hovered.bg_fill = palette.surface_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, TailwindColors::SLATE_300);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.text);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

        visuals.widgets.active.bg_fill = TailwindColors::BLUE_50;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.accent);
        visuals.widgets.active.corner_radius = CornerRadius::same(4);

        visuals.widgets.open.bg_fill = palette.surface_hover;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, TailwindColors::SLATE_300);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, palette.text);
        visuals.widgets.open.corner_radius = CornerRadius::same(4);

        visuals.selection.bg_fill = TailwindColors::BLUE_100;
        visuals.selection.stroke = Stroke::new(1.0, palette.accent);

        visuals.window_shadow = Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(20),
        };
        visuals.popup_shadow = Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(25),
        };
    }

    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.bar_inner_margin = 2.0;

    ctx.set_global_style(style);
}
