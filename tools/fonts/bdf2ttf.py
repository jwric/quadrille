#!/usr/bin/env python3
"""Convert a BDF bitmap font into a TrueType font with square-pixel outlines.

Each glyph's pixels are traced into closed, axis-aligned contours (outer
contours clockwise, holes counter-clockwise, collinear points removed), so the
result renders pixel-exact at exactly its native size (em = FONT_ASCENT +
FONT_DESCENT pixels) in any outline rasterizer, including swash.

usage: bdf2ttf.py in.bdf out.ttf [--family NAME] [--style STYLE] [--upp 64]
"""
import argparse
import codecs

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen


def parse_bdf(path):
    props, glyphs = {}, []
    with open(path, encoding="latin-1") as f:
        lines = iter(f.read().splitlines())
    cur = None
    for line in lines:
        parts = line.split()
        if not parts:
            continue
        key = parts[0]
        if key == "FONTBOUNDINGBOX":
            props["FONTBOUNDINGBOX"] = tuple(int(v) for v in parts[1:5])
        elif key == "STARTPROPERTIES":
            for pl in lines:
                if pl.startswith("ENDPROPERTIES"):
                    break
                k, _, v = pl.partition(" ")
                props[k] = v.strip().strip('"')
        elif key == "FONT":
            props["FONT"] = line[5:].strip()
        elif key == "STARTCHAR":
            cur = {"name": line[10:].strip(), "enc": -1, "dwidth": None, "bbx": None, "rows": []}
        elif key == "ENCODING" and cur is not None:
            cur["enc"] = int(parts[1])
            if cur["enc"] == -1 and len(parts) > 2:
                cur["enc"] = -1
        elif key == "DWIDTH" and cur is not None:
            cur["dwidth"] = int(parts[1])
        elif key == "BBX" and cur is not None:
            cur["bbx"] = tuple(int(v) for v in parts[1:5])
        elif key == "BITMAP" and cur is not None:
            for bl in lines:
                if bl.startswith("ENDCHAR"):
                    break
                cur["rows"].append(bl.strip())
            glyphs.append(cur)
            cur = None
    return props, glyphs


def glyph_pixels(g):
    """Return the set of lit pixels (x, y) with y up, baseline at y=0."""
    w, h, xo, yo = g["bbx"]
    px = set()
    for r, hexrow in enumerate(g["rows"]):
        if not hexrow:
            continue
        bits = int(hexrow, 16)
        nbits = len(hexrow) * 4
        y = yo + h - 1 - r
        for x in range(w):
            if bits >> (nbits - 1 - x) & 1:
                px.add((xo + x, y))
    return px


def trace(pixels):
    """Trace lit pixels into closed contours (lists of integer corner points)."""
    out = {}
    for (x, y) in pixels:
        # Clockwise in y-up space: up the left side, right along the top,
        # down the right side, left along the bottom. Shared edges cancel.
        for a, b, nb in (
            ((x, y), (x, y + 1), (x - 1, y)),
            ((x, y + 1), (x + 1, y + 1), (x, y + 1)),
            ((x + 1, y + 1), (x + 1, y), (x + 1, y)),
            ((x + 1, y), (x, y), (x, y - 1)),
        ):
            if nb not in pixels:
                out.setdefault(a, []).append(b)
    contours = []
    while out:
        start = next(iter(out))
        pts = [start]
        prev = None
        p = start
        while True:
            cands = out[p]
            if len(cands) == 1 or prev is None:
                nxt = cands[0]
            else:
                # Two outgoing edges at a diagonal pinch: prefer the right turn
                # so pixels touching only at a corner stay separate contours.
                dx, dy = p[0] - prev[0], p[1] - prev[1]
                right = (dy, -dx)
                nxt = next((c for c in cands if (c[0] - p[0], c[1] - p[1]) == right), cands[0])
            cands.remove(nxt)
            if not cands:
                del out[p]
            prev, p = p, nxt
            if p == start:
                break
            pts.append(p)
        # Drop collinear points.
        simp = []
        n = len(pts)
        for i in range(n):
            a, b, c = pts[i - 1], pts[i], pts[(i + 1) % n]
            if (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]) != 0:
                simp.append(b)
        contours.append(simp)
    return contours


def enc_to_unicode(enc, props):
    reg = props.get("CHARSET_REGISTRY", "ISO10646").upper()
    cse = props.get("CHARSET_ENCODING", "1").upper()
    if enc < 0:
        return None
    if reg.startswith("ISO10646") or (reg == "ISO8859" and cse == "1"):
        return enc
    if reg.startswith("ISO8859"):
        try:
            return ord(bytes([enc]).decode(f"iso8859-{cse}"))
        except Exception:
            return None
    if "437" in reg + cse or reg.startswith("IBM"):
        return ord(codecs.decode(bytes([enc]), "cp437")) if enc < 256 else None
    return enc


def convert(src, dst, family=None, style=None, upp=64, em=None, embolden=False):
    props, glyphs = parse_bdf(src)
    asc = int(props.get("FONT_ASCENT", props["FONTBOUNDINGBOX"][1] + props["FONTBOUNDINGBOX"][3]))
    desc = int(props.get("FONT_DESCENT", -props["FONTBOUNDINGBOX"][3]))
    em = em or asc + desc  # native size = the number passed as font size
    upem = em * upp
    family = family or props.get("FAMILY_NAME", "BDF Font")
    style = style or ("Bold" if props.get("WEIGHT_NAME", "").lower() == "bold" else "Regular")

    order, glyf, hmtx, cmap = [".notdef"], {}, {}, {}
    # .notdef: hollow box
    pen = TTGlyphPen(None)
    w = max(1, (props["FONTBOUNDINGBOX"][0])) * upp
    for c in ([(0, 0), (0, asc * upp), (w - upp, asc * upp), (w - upp, 0)],
              [(upp, upp), (w - 2 * upp, upp), (w - 2 * upp, (asc - 1) * upp), (upp, (asc - 1) * upp)]):
        pen.moveTo(c[0]); [pen.lineTo(p) for p in c[1:]]; pen.closePath()
    glyf[".notdef"] = pen.glyph(); hmtx[".notdef"] = (w, 0)

    seen = set()
    for g in glyphs:
        u = enc_to_unicode(g["enc"], props)
        if u is None or u in seen or g["bbx"] is None:
            continue
        seen.add(u)
        name = f"uni{u:04X}" if u <= 0xFFFF else f"u{u:05X}"
        pixels = glyph_pixels(g)
        if embolden:  # classic bitmap "smear" bold: OR each glyph with itself shifted 1px right
            pixels |= {(x + 1, y) for (x, y) in pixels}
        pen = TTGlyphPen(None)
        xmin = 0
        if pixels:
            xmin = min(p[0] for p in pixels)
            for c in trace(pixels):
                pen.moveTo((c[0][0] * upp, c[0][1] * upp))
                for p in c[1:]:
                    pen.lineTo((p[0] * upp, p[1] * upp))
                pen.closePath()
        glyf[name] = pen.glyph()
        adv = (g["dwidth"] if g["dwidth"] is not None else g["bbx"][0]) * upp
        hmtx[name] = (adv, xmin * upp if pixels else 0)
        order.append(name)
        cmap[u] = name

    def top(ch):
        n = cmap.get(ord(ch))
        g = glyf.get(n)
        if g is None or not hasattr(g, "coordinates") or not len(g.coordinates):
            return 0
        return max(y for _, y in g.coordinates)

    fb = FontBuilder(upem, isTTF=True)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap(cmap)
    fb.setupGlyf(glyf)
    fb.setupHorizontalMetrics(hmtx)
    fb.setupHorizontalHeader(ascent=asc * upp, descent=-desc * upp, lineGap=0)
    # fontdb (cosmic-text's font database) drops a face without a PostScript
    # name, so the table carries the full set of identifying names.
    ps_name = f"{family}-{style}".replace(" ", "")
    fb.setupNameTable({"familyName": family, "styleName": style,
                       "fullName": f"{family} {style}", "psName": ps_name,
                       "uniqueFontIdentifier": ps_name, "version": "Version 1.000",
                       "copyright": props.get("COPYRIGHT", ""),
                       "licenseDescription": props.get("NOTICE", ""),
                       "description": f"Converted from {src.split('/')[-1]} by bdf2ttf.py; native size {em}px"})
    fb.setupOS2(sTypoAscender=asc * upp, sTypoDescender=-desc * upp, sTypoLineGap=0,
                usWinAscent=asc * upp, usWinDescent=desc * upp,
                sxHeight=top("x"), sCapHeight=top("H"),
                fsSelection=0x20 if style == "Bold" else 0x40)
    fb.updateHead(macStyle=1 if style == "Bold" else 0)
    fb.setupPost(isFixedPitch=int(len({v[0] for k, v in hmtx.items() if k != '.notdef'}) == 1))
    fb.save(dst)
    return em, len(order)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("src"); ap.add_argument("dst")
    ap.add_argument("--family"); ap.add_argument("--style")
    ap.add_argument("--upp", type=int, default=64)
    ap.add_argument("--em", type=int, help="em size in px (default FONT_ASCENT + FONT_DESCENT)")
    ap.add_argument("--embolden", action="store_true", help="derive a smear-bold (+1px horizontal)")
    a = ap.parse_args()
    em, n = convert(a.src, a.dst, a.family, a.style, a.upp, a.em, a.embolden)
    print(f"{a.dst}: em={em}px glyphs={n}")
