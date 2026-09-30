#!/usr/bin/env python3
"""Derive Departure Mono Tight from Departure Mono's 11 px bitmaps.

Departure Mono draws a 5 px glyph body in a 7 px cell. Tight keeps every glyph
where it is and narrows the cell to 6 px, so a line of text is a seventh
shorter at the same cap height.

Box-drawing, block and legacy-computing glyphs tile their cell, so they are
cropped to the new width instead: a rule still meets its neighbour and a
vertical still sits on the same column.

usage: derive_tight.py departure-mono-11.bdf departure-mono-tight-11.bdf
"""
import re
import sys

WIDTH = 6

# Glyphs that tile the cell and must be cropped to it.
TILING = [(0x2500, 0x259F), (0x1FB00, 0x1FBFF)]


def tiles(cp):
    return any(lo <= cp <= hi for lo, hi in TILING)


def crop(rows, w, xo):
    """Crops a glyph's hex rows so that no pixel lands at or past WIDTH."""
    keep = max(0, min(w, WIDTH - xo))
    if keep == w:
        return rows, w
    nbytes = (keep + 7) // 8
    out = []
    for row in rows:
        bits = int(row, 16) >> (len(row) * 4 - w)
        bits >>= w - keep
        out.append(f"{bits << (nbytes * 8 - keep):0{nbytes * 2}X}" if keep else "")
    return out, keep


def main(src, dst):
    text = open(src, encoding="latin-1").read()
    text = text.replace('FAMILY_NAME "Departure Mono"', 'FAMILY_NAME "Departure Mono Tight"')
    text = re.sub(r"FONTBOUNDINGBOX \d+", f"FONTBOUNDINGBOX {WIDTH}", text, count=1)

    def glyph(m):
        head, enc, body = m.group(1), int(m.group(2)), m.group(3)
        body = re.sub(r"DWIDTH \d+ 0", f"DWIDTH {WIDTH} 0", body)
        body = re.sub(r"SWIDTH \d+ 0", f"SWIDTH {round(WIDTH * 1000 / 11)} 0", body)
        if tiles(enc):
            bbx = re.search(r"BBX (-?\d+) (-?\d+) (-?\d+) (-?\d+)\nBITMAP\n(.*)", body, re.S)
            w, h, xo, yo = (int(v) for v in bbx.groups()[:4])
            rows = bbx.group(5).split()
            rows, w = crop(rows, w, xo)
            if w == 0:
                h = xo = yo = 0
                rows = []
            body = body[: bbx.start()] + f"BBX {w} {h} {xo} {yo}\nBITMAP\n" + "".join(r + "\n" for r in rows)
        return f"{head}ENCODING {enc}\n{body}ENDCHAR"

    text = re.sub(r"(STARTCHAR [^\n]*\n)ENCODING (-?\d+)\n(.*?)ENDCHAR", glyph, text, flags=re.S)
    open(dst, "w", encoding="latin-1").write(text)


if __name__ == "__main__":
    main(*sys.argv[1:3])
