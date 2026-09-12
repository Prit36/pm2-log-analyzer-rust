//! SVG glyph outlines for the bundled IBM Plex faces.
//!
//! The in-app chart is an SVG document rendered by iced through `resvg`,
//! whose font database only knows system fonts — the reference's HTML relies
//! on the page webfont. Converting every `<text>` element to a `<path>` keeps
//! the *bundled* IBM Plex outlines (the exact faces the reference loads) and
//! makes the chart independent of whatever fonts a machine has installed.

use std::sync::OnceLock;

use ttf_parser::{Face, OutlineBuilder};

const SANS_REGULAR: &[u8] = include_bytes!("../../assets/fonts/IBMPlexSans-Regular.ttf");
const SANS_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/IBMPlexSans-Medium.ttf");
const SANS_SEMIBOLD: &[u8] = include_bytes!("../../assets/fonts/IBMPlexSans-SemiBold.ttf");
const MONO_REGULAR: &[u8] = include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf");
const MONO_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/IBMPlexMono-Medium.ttf");

/// The face used for chart labels (`IBM Plex Sans` at the reference weights).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FaceKind {
    SansRegular,
    SansMedium,
    SansSemiBold,
    MonoRegular,
    MonoMedium,
}

fn face(kind: FaceKind) -> &'static Face<'static> {
    static SANS_R: OnceLock<Face<'static>> = OnceLock::new();
    static SANS_M: OnceLock<Face<'static>> = OnceLock::new();
    static SANS_SB: OnceLock<Face<'static>> = OnceLock::new();
    static MONO_R: OnceLock<Face<'static>> = OnceLock::new();
    static MONO_M: OnceLock<Face<'static>> = OnceLock::new();
    match kind {
        FaceKind::SansRegular => SANS_R.get_or_init(|| Face::parse(SANS_REGULAR, 0).expect("sans")),
        FaceKind::SansMedium => SANS_M.get_or_init(|| Face::parse(SANS_MEDIUM, 0).expect("sans")),
        FaceKind::SansSemiBold => {
            SANS_SB.get_or_init(|| Face::parse(SANS_SEMIBOLD, 0).expect("sans"))
        }
        FaceKind::MonoRegular => {
            MONO_R.get_or_init(|| Face::parse(MONO_REGULAR, 0).expect("mono"))
        }
        FaceKind::MonoMedium => MONO_M.get_or_init(|| Face::parse(MONO_MEDIUM, 0).expect("mono")),
    }
}

/// Horizontal alignment of the laid-out text at the anchor point.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

fn fmt(number: f64) -> String {
    let rounded = (number * 100.0).round() / 100.0;
    let mut out = format!("{rounded:.2}");
    while out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    if out == "-0" {
        out = "0".to_string();
    }
    out
}

struct PathBuilder {
    d: String,
    scale: f64,
    pen_x: f64,
    baseline_y: f64,
}

impl OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = self.pen_x + f64::from(x) * self.scale;
        let y = self.baseline_y - f64::from(y) * self.scale;
        self.d.push_str(&format!("M{} {}", fmt(x), fmt(y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = self.pen_x + f64::from(x) * self.scale;
        let y = self.baseline_y - f64::from(y) * self.scale;
        self.d.push_str(&format!("L{} {}", fmt(x), fmt(y)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let x1 = self.pen_x + f64::from(x1) * self.scale;
        let y1 = self.baseline_y - f64::from(y1) * self.scale;
        let x = self.pen_x + f64::from(x) * self.scale;
        let y = self.baseline_y - f64::from(y) * self.scale;
        self.d
            .push_str(&format!("Q{} {} {} {}", fmt(x1), fmt(y1), fmt(x), fmt(y)));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let x1 = self.pen_x + f64::from(x1) * self.scale;
        let y1 = self.baseline_y - f64::from(y1) * self.scale;
        let x2 = self.pen_x + f64::from(x2) * self.scale;
        let y2 = self.baseline_y - f64::from(y2) * self.scale;
        let x = self.pen_x + f64::from(x) * self.scale;
        let y = self.baseline_y - f64::from(y) * self.scale;
        self.d.push_str(&format!(
            "C{} {} {} {} {} {}",
            fmt(x1),
            fmt(y1),
            fmt(x2),
            fmt(y2),
            fmt(x),
            fmt(y)
        ));
    }

    fn close(&mut self) {
        self.d.push('Z');
    }
}

fn glyph_advance(face: &Face<'_>, glyph: u16) -> f64 {
    f64::from(face.glyph_hor_advance(ttf_parser::GlyphId(glyph)).unwrap_or(0))
}

fn kerning(face: &Face<'_>, left: u16, right: u16) -> f64 {
    let Some(kern) = face.tables().kern else {
        return 0.0;
    };
    for subtable in kern.subtables {
        if !subtable.horizontal || subtable.variable {
            continue;
        }
        if let Some(value) = subtable.glyphs_kerning(ttf_parser::GlyphId(left), ttf_parser::GlyphId(right)) {
            return f64::from(value);
        }
    }
    0.0
}

/// Total advance width of `text` at `size` pixels.
pub fn measure(text: &str, kind: FaceKind, size: f64) -> f64 {
    let face = face(kind);
    let scale = size / f64::from(face.units_per_em());
    let mut width = 0.0;
    let mut previous: Option<u16> = None;
    for ch in text.chars() {
        let Some(gid) = face.glyph_index(ch) else {
            continue;
        };
        if let Some(left) = previous {
            width += kerning(face, left, gid.0) * scale;
        }
        width += glyph_advance(face, gid.0) * scale;
        previous = Some(gid.0);
    }
    width
}

/// SVG path data for `text`, with the alphabetic baseline at `baseline_y`.
pub fn path_data(text: &str, kind: FaceKind, size: f64, x: f64, baseline_y: f64, anchor: Anchor) -> String {
    let face = face(kind);
    let scale = size / f64::from(face.units_per_em());
    let width = measure(text, kind, size);
    let start_x = match anchor {
        Anchor::Start => x,
        Anchor::Middle => x - width / 2.0,
        Anchor::End => x - width,
    };

    let mut builder = PathBuilder {
        d: String::new(),
        scale,
        pen_x: start_x,
        baseline_y,
    };
    let mut previous: Option<u16> = None;
    for ch in text.chars() {
        let Some(gid) = face.glyph_index(ch) else {
            continue;
        };
        if let Some(left) = previous {
            builder.pen_x += kerning(face, left, gid.0) * scale;
        }
        if let Some(_outline) = face.outline_glyph(gid, &mut builder) {
            // The builder wrote the glyph commands.
        }
        builder.pen_x += glyph_advance(face, gid.0) * scale;
        previous = Some(gid.0);
    }
    builder.d
}
