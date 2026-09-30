#!/usr/bin/env python3
"""Extract a square-pixel outline font's glyphs to BDF at its native pixel size.

This is the "get the pixels out" half of a bitmap-first font pipeline: the BDF
can be edited in any bitmap font editor and compiled back with bdf2ttf.py.

usage: ttf2bdf.py in.otf out.bdf [--ppem N]
"""
import argparse
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from pxfont import PxFont  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("src"); ap.add_argument("dst")
    ap.add_argument("--ppem", type=float)
    a = ap.parse_args()
    f = PxFont(a.src)
    if a.ppem:
        f.upp, f.ppem, f.off_x, f.off_y = f.upem / a.ppem, a.ppem, 0, 0
    hh = f.tt["hhea"]
    asc = math.ceil(hh.ascent / f.upp - 1e-6)
    desc = math.ceil(-hh.descent / f.upp - 1e-6)
    fam = f.tt["name"].getBestFamilyName()
    chars = []
    for cp, g in sorted(f.cmap.items()):
        adv = round(f.adv_px(g))
        r = f.raster(g)
        if r is None:
            chars.append((cp, adv, 0, 0, 0, 0, []))
            continue
        bm, left, top = r
        h, w = bm.shape
        rows = []
        nbytes = (w + 7) // 8
        for row in bm:
            v = 0
            for x, on in enumerate(row):
                if on:
                    v |= 1 << (nbytes * 8 - 1 - x)
            rows.append(f"{v:0{nbytes * 2}X}")
        chars.append((cp, adv, w, h, left, top - h, rows))
    maxw = max(c[2] for c in chars)
    with open(a.dst, "w") as out:
        out.write("STARTFONT 2.1\n")
        out.write(f"FONT -misc-{fam.replace(' ', '')}-Medium-R-Normal--{round(f.ppem)}-{round(f.ppem) * 10}-75-75-C-0-ISO10646-1\n")
        out.write(f"SIZE {round(f.ppem)} 75 75\n")
        out.write(f"FONTBOUNDINGBOX {maxw} {asc + desc} 0 {-desc}\n")
        out.write("STARTPROPERTIES 6\n")
        out.write(f'FAMILY_NAME "{fam}"\nPIXEL_SIZE {round(f.ppem)}\nFONT_ASCENT {asc}\nFONT_DESCENT {desc}\n')
        out.write('CHARSET_REGISTRY "ISO10646"\nCHARSET_ENCODING "1"\nENDPROPERTIES\n')
        out.write(f"CHARS {len(chars)}\n")
        for cp, adv, w, h, xo, yo, rows in chars:
            out.write(f"STARTCHAR U+{cp:04X}\nENCODING {cp}\nSWIDTH {round(adv * 1000 / f.ppem)} 0\nDWIDTH {adv} 0\n")
            out.write(f"BBX {w} {h} {xo} {yo}\nBITMAP\n")
            for r in rows:
                out.write(r + "\n")
            out.write("ENDCHAR\n")
        out.write("ENDFONT\n")
    print(f"{a.dst}: {len(chars)} glyphs, {round(f.ppem)}px, ascent {asc} descent {desc}")


if __name__ == "__main__":
    main()
