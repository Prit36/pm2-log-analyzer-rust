//! Bundled IBM Plex faces — the reference loads the same families from Google
//! Fonts (`index.html`), so shipping them keeps text metrics identical offline.
//!
//! The whole app is unicode-safe without a system font because every weight the
//! reference uses is embedded; `style::REGULAR` / `style::MONO` are the defaults
//! for proportional and tabular text respectively.

use iced::font::{Family, Weight};
use iced::Font;

pub const SANS_FAMILY: &str = "IBM Plex Sans";
pub const MONO_FAMILY: &str = "IBM Plex Mono";

pub const REGULAR: Font = Font {
    family: Family::Name(SANS_FAMILY),
    weight: Weight::Normal,
    ..Font::DEFAULT
};
pub const MEDIUM: Font = Font {
    family: Family::Name(SANS_FAMILY),
    weight: Weight::Medium,
    ..Font::DEFAULT
};
pub const SEMIBOLD: Font = Font {
    family: Family::Name(SANS_FAMILY),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};
pub const BOLD: Font = Font {
    family: Family::Name(SANS_FAMILY),
    weight: Weight::Bold,
    ..Font::DEFAULT
};

pub const MONO: Font = Font {
    family: Family::Name(MONO_FAMILY),
    weight: Weight::Normal,
    ..Font::DEFAULT
};
pub const MONO_MEDIUM: Font = Font {
    family: Family::Name(MONO_FAMILY),
    weight: Weight::Medium,
    ..Font::DEFAULT
};
pub const MONO_SEMIBOLD: Font = Font {
    family: Family::Name(MONO_FAMILY),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};

/// Letter-spaced faces (`scripts/gen-tracked-fonts.py`) that reproduce
/// Tailwind's `tracking-wide` (+0.025em), `tracking-wider` (+0.05em) and
/// `tracking-tight` (-0.025em) because iced's text stack has no letter-spacing
/// knob.
pub const WIDE_FAMILY: &str = "IBM Plex Sans Wide";
pub const WIDER_FAMILY: &str = "IBM Plex Sans Wider";
pub const TIGHT_FAMILY: &str = "IBM Plex Sans Tight";
pub const MONO_WIDE_FAMILY: &str = "IBM Plex Mono Wide";

pub const WIDE_MEDIUM: Font = Font {
    family: Family::Name(WIDE_FAMILY),
    weight: Weight::Medium,
    ..Font::DEFAULT
};
pub const WIDE_SEMIBOLD: Font = Font {
    family: Family::Name(WIDE_FAMILY),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};
pub const WIDE_BOLD: Font = Font {
    family: Family::Name(WIDE_FAMILY),
    weight: Weight::Bold,
    ..Font::DEFAULT
};
pub const WIDER_BOLD: Font = Font {
    family: Family::Name(WIDER_FAMILY),
    weight: Weight::Bold,
    ..Font::DEFAULT
};
pub const TIGHT_SEMIBOLD: Font = Font {
    family: Family::Name(TIGHT_FAMILY),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};
pub const TIGHT_BOLD: Font = Font {
    family: Family::Name(TIGHT_FAMILY),
    weight: Weight::Bold,
    ..Font::DEFAULT
};
pub const MONO_WIDE: Font = Font {
    family: Family::Name(MONO_WIDE_FAMILY),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

/// Every face the app can ask for, in the order `iced` should register them.
pub const FILES: [&[u8]; 14] = [
    include_bytes!("../../assets/fonts/IBMPlexSans-Regular.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSans-Medium.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSans-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSans-Bold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexMono-Medium.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexMono-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSansTight-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSansTight-Bold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSansWide-Medium.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSansWide-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSansWide-Bold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSansWider-Bold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexMonoWide-Regular.ttf"),
];
