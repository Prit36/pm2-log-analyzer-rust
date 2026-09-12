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

/// Every face the app can ask for, in the order `iced` should register them.
pub const FILES: [&[u8]; 7] = [
    include_bytes!("../../assets/fonts/IBMPlexSans-Regular.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSans-Medium.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSans-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexSans-Bold.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexMono-Regular.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexMono-Medium.ttf"),
    include_bytes!("../../assets/fonts/IBMPlexMono-SemiBold.ttf"),
];
