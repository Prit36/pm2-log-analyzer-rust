#!/usr/bin/env python3
"""Generate letter-spaced IBM Plex faces.

Tailwind's `tracking-wide` / `tracking-tight` add +/-0.025em to every glyph
advance. iced's text stack (cosmic-text) has no letter-spacing API, so the
spacing is baked into patched copies of the bundled faces instead — the
advance delta scales with the font size exactly like an `em`-based CSS value.

Outputs:
  IBMPlexSansTight-{SemiBold,Bold}.ttf   family "IBM Plex Sans Tight"
  IBMPlexSansWide-{Medium,SemiBold,Bold}.ttf  family "IBM Plex Sans Wide"
  IBMPlexSansWider-Bold.ttf      family "IBM Plex Sans Wider"
  IBMPlexMonoWide-Regular.ttf    family "IBM Plex Mono Wide"
"""
import os
from fontTools.ttLib import TTFont

HERE = os.path.dirname(os.path.abspath(__file__))
FONTS = os.path.join(HERE, "..", "assets", "fonts")

VARIANTS = [
    ("IBMPlexSans-SemiBold.ttf", "IBMPlexSansTight-SemiBold.ttf", "IBM Plex Sans Tight", "SemiBold", -0.025),
    ("IBMPlexSans-Bold.ttf", "IBMPlexSansTight-Bold.ttf", "IBM Plex Sans Tight", "Bold", -0.025),
    ("IBMPlexSans-Medium.ttf", "IBMPlexSansWide-Medium.ttf", "IBM Plex Sans Wide", "Medium", 0.025),
    ("IBMPlexSans-SemiBold.ttf", "IBMPlexSansWide-SemiBold.ttf", "IBM Plex Sans Wide", "SemiBold", 0.025),
    ("IBMPlexSans-Bold.ttf", "IBMPlexSansWide-Bold.ttf", "IBM Plex Sans Wide", "Bold", 0.025),
    ("IBMPlexSans-Bold.ttf", "IBMPlexSansWider-Bold.ttf", "IBM Plex Sans Wider", "Bold", 0.05),
    ("IBMPlexMono-Regular.ttf", "IBMPlexMonoWide-Regular.ttf", "IBM Plex Mono Wide", "Regular", 0.025),
]


def set_names(font, family, subfamily):
    name = font["name"]
    full = f"{family} {subfamily}" if subfamily != "Regular" else family
    ps = (family + "-" + subfamily).replace(" ", "")
    for record in name.names:
        if record.nameID == 1:
            record.string = family
        elif record.nameID == 2:
            record.string = subfamily
        elif record.nameID == 4:
            record.string = full
        elif record.nameID == 6:
            record.string = ps
        elif record.nameID == 16:
            record.string = family
        elif record.nameID == 17:
            record.string = subfamily


def main():
    for src, dst, family, subfamily, em in VARIANTS:
        path = os.path.join(FONTS, src)
        font = TTFont(path)
        upm = font["head"].unitsPerEm
        delta = round(upm * em)
        hmtx = font["hmtx"].metrics
        for glyph in list(hmtx.keys()):
            advance, lsb = hmtx[glyph]
            hmtx[glyph] = (max(0, advance + delta), lsb)
        set_names(font, family, subfamily)
        out = os.path.join(FONTS, dst)
        font.save(out)
        print(f"{dst}: {family} {subfamily} (delta {delta} units/em)")


if __name__ == "__main__":
    main()
