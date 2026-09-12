//! Monospace cell truncation — the iced stand-in for Tailwind's `truncate`
//! (`overflow: hidden; text-overflow: ellipsis; white-space: nowrap`).
//!
//! Every reference cell that truncates is `font-mono-data`, i.e. IBM Plex Mono,
//! where all glyphs share one advance width. That turns "does it fit" into a
//! division against a metric read once from the bundled font: no text shaping,
//! no layout work, and no font system required at call time.

use std::borrow::Cow;

use ttf_parser::{Face, GlyphId};

/// IBM Plex Mono regular, the face all truncated reference cells use.
const MONO_REGULAR: &[u8] = include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf");
const SANS_REGULAR: &[u8] = include_bytes!("../../assets/fonts/IBMPlexSans-Regular.ttf");
const SANS_MEDIUM: &[u8] = include_bytes!("../../assets/fonts/IBMPlexSans-Medium.ttf");
const SANS_SEMIBOLD: &[u8] = include_bytes!("../../assets/fonts/IBMPlexSans-SemiBold.ttf");
const SANS_BOLD: &[u8] = include_bytes!("../../assets/fonts/IBMPlexSans-Bold.ttf");

/// Weights of IBM Plex Sans the reference asks for by name.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SansWeight {
    Regular,
    Medium,
    Semibold,
    Bold,
}

fn face(id: FaceId) -> &'static Face<'static> {
    static FACES: std::sync::OnceLock<[Face<'static>; 5]> = std::sync::OnceLock::new();
    let faces = FACES.get_or_init(|| {
        [MONO_REGULAR, SANS_REGULAR, SANS_MEDIUM, SANS_SEMIBOLD, SANS_BOLD].map(|bytes| {
            Face::parse(bytes, 0).expect("bundled IBM Plex face parses")
        })
    });
    &faces[id as usize]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FaceId {
    Mono = 0,
    SansRegular = 1,
    SansMedium = 2,
    SansSemibold = 3,
    SansBold = 4,
}

fn monospace() -> &'static Face<'static> {
    face(FaceId::Mono)
}

/// Width of one character cell at `size`, in logical pixels.
pub fn mono_advance(size: f32) -> f32 {
    let face = monospace();
    let advance = face
        .glyph_hor_advance(face.glyph_index('M').expect("Plex Mono has M"))
        .expect("Plex Mono M has an advance") as f32;
    advance * size / f32::from(face.units_per_em())
}

/// Width of `text` in the bundled IBM Plex Sans face at `size`.
///
/// Used to size content-hugging badges, so it only needs glyph advances —
/// kerning between uppercase letters is negligible at badge sizes.
pub fn sans_width(text: &str, size: f32, weight: SansWeight) -> f32 {
    let face = face(match weight {
        SansWeight::Regular => FaceId::SansRegular,
        SansWeight::Medium => FaceId::SansMedium,
        SansWeight::Semibold => FaceId::SansSemibold,
        SansWeight::Bold => FaceId::SansBold,
    });
    let mut units = 0u32;
    for character in text.chars() {
        let glyph = face.glyph_index(character).unwrap_or(GlyphId(0));
        units += u32::from(face.glyph_hor_advance(glyph).unwrap_or(0));
    }
    units as f32 * size / f32::from(face.units_per_em())
}

/// Cuts `text` to the widest prefix that fits in `max_width`, appending `…`
/// when anything was dropped. Borrows when the text already fits.
pub fn truncate_mono(text: &str, size: f32, max_width: f32) -> Cow<'_, str> {
    if text.is_empty() || max_width <= 0.0 {
        return Cow::Borrowed("");
    }

    let cells = (max_width / mono_advance(size)).floor().max(0.0) as usize;
    // `count` is O(n) but n is a table cell, not a document.
    let chars = text.chars().count();
    if chars <= cells {
        return Cow::Borrowed(text);
    }
    if cells == 0 {
        return Cow::Borrowed("");
    }

    let mut truncated = String::with_capacity(text.len());
    truncated.extend(text.chars().take(cells - 1));
    truncated.push('…');
    Cow::Owned(truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plex_mono_advance_is_six_tenths_of_em() {
        // The truncation math relies on Plex Mono being a 600/1000 em monospace.
        let face = monospace();
        let expected = 0.6 * f32::from(face.units_per_em());
        let glyph = face.glyph_index('W').expect("has W");
        let advance = face.glyph_hor_advance(glyph).expect("has advance");
        assert_eq!(f32::from(advance), expected);
        assert!((mono_advance(11.0) - 6.6).abs() < f32::EPSILON);
    }

    #[test]
    fn truncates_to_the_last_cell_that_fits() {
        let width = mono_advance(11.0) * 5.0;
        assert_eq!(truncate_mono("abcde", 11.0, width), "abcde");
        assert_eq!(truncate_mono("abcdef", 11.0, width), "abcd…");
        assert_eq!(truncate_mono("abcdefghij", 11.0, mono_advance(11.0)), "…");
        assert_eq!(truncate_mono("abcdef", 11.0, 0.0), "");
    }
}
