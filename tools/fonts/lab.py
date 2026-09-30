#!/usr/bin/env python3
"""Prepares the candidate fonts the showcase's type lab compares.

For each candidate: subset it to the characters an instrument interface uses,
give it the names fontdb needs to load it, put its vertical metrics on whole
pixels, and measure it. The fonts are written to demo/assets/fonts/lab and the
measurements to demo/src/lab/candidates.rs.

usage: lab.py SOURCE_DIR

SOURCE_DIR holds the upstream files at the paths listed in CANDIDATES; see
demo/assets/fonts/lab/SOURCES.md for where each one comes from.
"""
import os
import sys

from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
OUT = os.path.join(ROOT, "demo", "assets", "fonts", "lab")
RUST = os.path.join(ROOT, "demo", "src", "lab", "candidates.rs")

# Latin, Greek, punctuation, super- and subscripts, currency, letterlike
# symbols, arrows, maths, technical, box drawing, blocks, geometric shapes.
UNICODES = (
    list(range(0x20, 0x7F))
    + list(range(0xA0, 0x180))
    + list(range(0x370, 0x400))
    + list(range(0x2000, 0x2150))
    + list(range(0x2190, 0x2400))
    + list(range(0x2500, 0x2600))
)

# (id, native em in px, source path, family, style, weight, name, licence, note)
CANDIDATES = [
    ("departure-bold", 11, "departure-mono-bold.ttf", "Departure Mono", "Bold", 700,
     "DEPARTURE BOLD", "OFL-1.1", "Departure Mono thickened a pixel to the right."),
    ("pixel-code", 9, "pixelcode/ttf/PixelCode.ttf", None, None, None,
     "PIXEL CODE", "OFL-1.1", "The nearest relative of Departure at a smaller size."),
    ("fixed-5x7", 7, "misc-fixed/conv/Fixed-5x7.ttf", None, None, None,
     "FIXED 5×7", "Public domain", "X11 misc-fixed: the workstation's smallest face."),
    ("fixed-6x10", 10, "misc-fixed/conv/Fixed-6x10.ttf", None, None, None,
     "FIXED 6×10", "Public domain", "X11 misc-fixed at a small body size."),
    ("terminus-12", 12, "terminus/conv/Terminus-12n.ttf", None, None, None,
     "TERMINUS 12", "OFL-1.1", "Departure's capitals and x-height in a 6 px cell."),
    ("terminus-12-bold", 12, "terminus/conv/Terminus-12b.ttf", None, "Bold", 700,
     "TERMINUS 12 BOLD", "OFL-1.1", "A true bold at body size."),
    ("greybeard-11", 11, "greybeard/Greybeard-11px.ttf", None, None, None,
     "GREYBEARD 11", "MIT", "A neutral terminal face with a bold at every size."),
    ("greybeard-11-bold", 11, "greybeard/Greybeard-11px-Bold.ttf", None, None, 700,
     "GREYBEARD 11 BOLD", "MIT", "Greybeard's bold."),
    ("scientifica", 11, "scientifica/scientifica/ttf/scientifica.ttf", None, None, None,
     "SCIENTIFICA", "OFL-1.1", "Condensed: five pixels a cell."),
    ("spleen-5x8", 8, "spleen/conv/Spleen-5x8.ttf", None, None, None,
     "SPLEEN 5×8", "BSD-2-Clause", "Squared letterforms, the Portfolio's line."),
    ("spleen-6x12", 12, "spleen/spleen-2.2.0/spleen-6x12.otf", None, None, None,
     "SPLEEN 6×12", "BSD-2-Clause", "Spleen at body size."),
    ("cozette", 13, "cozette/CozetteVector.ttf", None, None, None,
     "COZETTE", "MIT", "A rounded 6×13 with wide coverage."),
    ("gohu-11", 11, "gohufont/conv/Gohu-11.ttf", None, None, None,
     "GOHU 11", "WTFPL", "A compact terminal face."),
    ("portfolio-6x8", 8, "oldschool-px/Px437_Portfolio_6x8.ttf", None, None, None,
     "PORTFOLIO 6×8", "CC BY-SA 4.0", "A plain 6×8 face from the Oldschool PC Font Pack."),
    ("hp100lx-6x8", 8, "oldschool-px/PxPlus_HP_100LX_6x8.ttf", None, None, None,
     "HP 100LX 6×8", "CC BY-SA 4.0", "Portfolio's sibling, with descenders."),
    ("unscii-8", 8, "unscii/unscii-8.ttf", None, None, None,
     "UNSCII 8", "Public domain", "Home-computer 8×8 with two-pixel stems."),
]


def ink(font, char, upp):
    """The ink bounds of `char` in pixels: (left, right, bottom, top)."""
    glyph_set = font.getGlyphSet()
    name = font.getBestCmap().get(ord(char))
    from fontTools.pens.boundsPen import BoundsPen

    pen = BoundsPen(glyph_set)
    glyph_set[name].draw(pen)
    x_min, y_min, x_max, y_max = pen.bounds
    return round(x_min / upp), round(x_max / upp), round(y_min / upp), round(y_max / upp)


def process(source_dir, entry):
    ident, em, path, family, style, weight, name, licence, note = entry
    font = TTFont(os.path.join(source_dir, path))

    options = subset.Options()
    options.layout_features = []
    options.name_IDs = ["*"]
    options.notdef_outline = True
    options.glyph_names = True
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=UNICODES)
    subsetter.subset(font)

    names = font["name"]
    family = family or names.getBestFamilyName()
    # cosmic-text matches a font's weight exactly, so every face is either a
    # Regular at 400 or a Bold at 700, whatever the source calls it.
    weight = weight or 400
    style = style or ("Bold" if weight >= 700 else "Regular")
    ps_name = f"{family}-{style}".replace(" ", "")

    for record, value in ((1, family), (2, style), (4, f"{family} {style}"), (6, ps_name)):
        names.setName(value, record, 3, 1, 0x409)
    names.removeNames(nameID=16)
    names.removeNames(nameID=17)

    os2 = font["OS/2"]
    os2.usWeightClass = weight

    upp = font["head"].unitsPerEm / em

    # Whole-pixel vertical metrics, the same in every table, so the baseline
    # cosmic-text computes lands on a pixel.
    hhea = font["hhea"]
    use_typo = bool(os2.fsSelection & (1 << 7))
    ascent = os2.sTypoAscender if use_typo else hhea.ascent
    descent = -(os2.sTypoDescender if use_typo else hhea.descent)
    ascent_px, descent_px = round(ascent / upp), round(descent / upp)

    hhea.ascent = os2.sTypoAscender = os2.usWinAscent = round(ascent_px * upp)
    hhea.descent = os2.sTypoDescender = -round(descent_px * upp)
    os2.usWinDescent = round(descent_px * upp)
    hhea.lineGap = os2.sTypoLineGap = 0


    extension = "otf" if "CFF " in font else "ttf"
    os.makedirs(OUT, exist_ok=True)
    font.save(os.path.join(OUT, f"{ident}.{extension}"))

    advance = round(font["hmtx"][font.getBestCmap()[ord("0")]][0] / upp)
    h_left, h_right, _, cap = ink(font, "H", upp)
    _, _, _, x_height = ink(font, "x", upp)
    _, _, depth, _ = ink(font, "p", upp)

    return {
        "id": ident,
        "file": f"{ident}.{extension}",
        "name": name,
        "family": family,
        "weight": weight,
        "licence": licence,
        "note": note,
        "em": em,
        "ascent": ascent_px,
        "descent": descent_px,
        "advance": advance,
        "cap": cap,
        "x_height": x_height,
        "depth": -depth,
        "left": h_left,
        "right": advance - h_right,
    }


def line(m):
    """The densest line height whose baseline is whole and whose ink fits."""
    body = m["ascent"] + m["descent"]
    for height in range(m["cap"] + m["depth"], 64):
        if (height - body) % 2:
            continue
        baseline = (height - body) // 2 + m["ascent"]
        if baseline - m["cap"] >= 0 and height - baseline >= m["depth"]:
            return height
    raise SystemExit(f"{m['id']}: no line fits")


def rust(candidates):
    out = [
        "// Generated by tools/fonts/lab.py; do not edit by hand.",
        "use graticule::Face;",
        "use graticule::face::Metrics;",
        "use iced::font::{Font, Weight};",
        "",
        "use super::Candidate;",
        "",
        "/// The candidate fonts, as their files.",
        "pub const FONTS: &[&[u8]] = &[",
    ]
    for m in candidates:
        out.append(f'    include_bytes!("../../assets/fonts/lab/{m["file"]}"),')
    out += ["];", "", "/// The candidates, measured.", "pub const CANDIDATES: &[Candidate] = &["]
    for m in candidates:
        weight = "Weight::Bold" if m["weight"] >= 700 else "Weight::Normal"
        out += [
            "    Candidate {",
            f'        name: "{m["name"]}",',
            f'        licence: "{m["licence"]}",',
            f'        note: "{m["note"]}",',
            "        face: Face::new(",
            "            Font {",
            f'                weight: {weight},',
            f'                ..Font::with_name("{m["family"]}")',
            "            },",
            "            Metrics {",
            f'                em: {m["em"]},',
            f'                ascent: {m["ascent"]},',
            f'                descent: {m["descent"]},',
            f'                advance: {m["advance"]},',
            f'                cap: {m["cap"]},',
            f'                x_height: {m["x_height"]},',
            f'                depth: {m["depth"]},',
            f'                left: {m["left"]},',
            f'                right: {m["right"]},',
            "            },",
            "            1,",
            f'            {line(m)},',
            "        ),",
            "    },",
        ]
    out += ["];", ""]
    os.makedirs(os.path.dirname(RUST), exist_ok=True)
    with open(RUST, "w") as f:
        f.write("\n".join(out))


def main(source_dir):
    measured = [process(source_dir, entry) for entry in CANDIDATES]
    for m in measured:
        print(f'{m["id"]:18} em {m["em"]:2} cell {m["advance"]}x{line(m):2} cap {m["cap"]} x {m["x_height"]} '
              f'depth {m["depth"]} asc {m["ascent"]} desc {m["descent"]} bearings {m["left"]}/{m["right"]}')
    rust(measured)


if __name__ == "__main__":
    main(sys.argv[1])
