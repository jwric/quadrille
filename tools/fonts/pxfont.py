"""Pixel-font measurement and non-antialiased rasterization from outlines.

The rasterizer samples each pixel centre against the glyph outline (nonzero
winding), at the font's detected native pixel grid, so a square-pixel font is
reproduced exactly and anything off-grid shows up as dropped/doubled pixels.
"""
import math
from collections import Counter
from functools import lru_cache

import numpy as np
from fontTools.pens.basePen import BasePen
from fontTools.pens.recordingPen import RecordingPen
from fontTools.ttLib import TTFont


class FlatPen(BasePen):
    """Flattens an outline into closed polygons; records curve/diagonal usage."""

    def __init__(self, glyphSet=None, steps=8):
        super().__init__(glyphSet)
        self.polys, self.cur, self.steps = [], None, steps
        self.curves = 0
        self.diagonals = 0
        self.segments = 0

    def _moveTo(self, p):
        self.cur = [p]

    def _lineTo(self, p):
        a = self.cur[-1]
        self.segments += 1
        if abs(a[0] - p[0]) > 1e-6 and abs(a[1] - p[1]) > 1e-6:
            self.diagonals += 1
        self.cur.append(p)

    def _curveToOne(self, p1, p2, p3):
        self.curves += 1
        self.segments += 1
        p0 = self.cur[-1]
        for i in range(1, self.steps + 1):
            t = i / self.steps
            mt = 1 - t
            self.cur.append((mt ** 3 * p0[0] + 3 * mt * mt * t * p1[0] + 3 * mt * t * t * p2[0] + t ** 3 * p3[0],
                             mt ** 3 * p0[1] + 3 * mt * mt * t * p1[1] + 3 * mt * t * t * p2[1] + t ** 3 * p3[1]))

    def _qCurveToOne(self, p1, p2):
        self.curves += 1
        self.segments += 1
        p0 = self.cur[-1]
        for i in range(1, self.steps + 1):
            t = i / self.steps
            mt = 1 - t
            self.cur.append((mt * mt * p0[0] + 2 * mt * t * p1[0] + t * t * p2[0],
                             mt * mt * p0[1] + 2 * mt * t * p1[1] + t * t * p2[1]))

    def _closePath(self):
        if self.cur and len(self.cur) > 2:
            self.polys.append(self.cur)
        self.cur = None

    _endPath = _closePath


RANGES = {
    "ascii": list(range(0x20, 0x7F)),
    "latin1": list(range(0xA0, 0x100)),
    "latinA": list(range(0x100, 0x180)),
    "greek": [c for c in range(0x391, 0x3CA) if c != 0x3A2 and not (0x3AA <= c <= 0x3B0)],
    "cyrillic": list(range(0x410, 0x450)),
    "box": list(range(0x2500, 0x2580)),
    "block": list(range(0x2580, 0x25A0)),
    "arrows": [0x2190, 0x2191, 0x2192, 0x2193, 0x2194, 0x2195, 0x21B5, 0x21E4, 0x21E5],
    "tech": [ord(c) for c in "°±µΩ×÷·…≤≥≈≠∞√π∆‰€▲▼►◄●○■□◆✓"],
}

SPEC_LINES = [
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789 +-.,:;%°",
    "FREQ 14.074 MHz  SNR -12dB  PWR 100W",
    "The quick brown fox jumps over the lazy dog",
]
BOX_LINES = ["┌─┬─┐ ╔═╦═╗ ▲▼◄► ░▒▓█", "│ │ │ ║ ║ ║ ←↑→↓ ±µΩ", "└─┴─┘ ╚═╩═╝ ≤≥≈° ×÷·"]


class PxFont:
    def __init__(self, path, index=0):
        self.path = path
        self.tt = TTFont(path, fontNumber=index)
        self.upem = self.tt["head"].unitsPerEm
        self.cmap = self.tt.getBestCmap() or {}
        self.gs = self.tt.getGlyphSet()
        self.hmtx = self.tt["hmtx"].metrics
        self._poly = {}
        self.detect_grid()

    # ---------- outlines ----------
    def gname(self, ch):
        return self.cmap.get(ord(ch) if isinstance(ch, str) else ch)

    def poly(self, gname):
        if gname not in self._poly:
            pen = FlatPen(self.gs)
            try:
                self.gs[gname].draw(pen)
            except Exception:
                pass
            self._poly[gname] = pen
        return self._poly[gname]

    def raw_points(self, gname):
        rp = RecordingPen()
        try:
            self.gs[gname].draw(rp)
        except Exception:
            return []
        pts = []
        for op, args in rp.value:
            for a in args:
                if isinstance(a, tuple) and len(a) == 2:
                    pts.append(a)
        return pts

    # ---------- grid detection ----------
    def detect_grid(self):
        chars = [chr(c) for c in range(0x21, 0x7F)]
        glyphs = [g for g in (self.gname(c) for c in chars) if g]
        diffs_x, diffs_y, xs, ys = [], [], [], []
        curves = diag = segs = 0
        for g in glyphs:
            pts = self.raw_points(g)
            fp = self.poly(g)
            curves += fp.curves
            diag += fp.diagonals
            segs += fp.segments
            if not pts:
                continue
            ux = sorted({round(p[0], 2) for p in pts})
            uy = sorted({round(p[1], 2) for p in pts})
            xs += ux
            ys += uy
            diffs_x += [b - a for a, b in zip(ux, ux[1:])]
            diffs_y += [b - a for a, b in zip(uy, uy[1:])]
        advs = [self.hmtx[g][0] for g in glyphs]
        self.curve_frac = curves / max(1, segs)
        self.diag_frac = diag / max(1, segs)

        def best_unit(diffs):
            """Frequent coordinate steps: candidate pixel sizes (or multiples)."""
            if not diffs:
                return []
            c = Counter(round(d * 2) / 2 for d in diffs if d > 0.5)
            if not c:
                return []
            total = sum(c.values())
            freq = [d for d in sorted(c) if c[d] >= max(3, 0.02 * total)]
            return freq[:8] or [min(c)]

        def score(vals, p):
            if not vals:
                return 0.0, 0.0
            v = np.array(vals, dtype=float)
            r = np.mod(v, p)
            # offset = most common residue
            rr = np.round(r / p * 20) / 20 % 1.0
            off = Counter(rr.tolist()).most_common(1)[0][0] * p
            q = (v - off) / p
            ok = np.abs(q - np.round(q)) < 0.06
            return float(ok.mean()), off

        bases = set(best_unit(diffs_x)) | set(best_unit(diffs_y))
        cand = set()
        for d in bases:
            for b in (d, d / 2, d / 3):
                for q in (b, self.upem / max(1, round(self.upem / b))):
                    for k in (1, 2, 3, 4):
                        if q * k <= self.upem / 3:
                            cand.add(round(q * k, 4))
        best = None
        scored = []
        for p in cand:
            sx, ox = score(xs + advs, p)
            sy, oy = score(ys, p)
            scored.append((p, sx, sy, ox, oy))
        if scored:
            top = max(min(s[1], s[2]) for s in scored)
            if top > 0.97:
                # Highest alignment wins; among (near-)ties take the coarsest grid.
                best = max((s for s in scored if min(s[1], s[2]) >= top - 0.002), key=lambda s: s[0])
        if not best and bases:
            # No clean grid: report the dominant step as the "pixel" and flag it.
            p = min(bases)
            sx, ox = score(xs + advs, p)
            sy, oy = score(ys, p)
            best = (p, sx, sy, ox, oy)
        if not best:
            best = (self.upem / 16, 0, 0, 0, 0)
        self.upp, self.align_x, self.align_y, self.off_x, self.off_y = best
        # Snap upp so that ppem is a clean number when very close.
        ppem = self.upem / self.upp
        if abs(ppem - round(ppem)) < 0.05 and abs(ppem - round(ppem)) > 1e-9:
            ppem = round(ppem)
            p = self.upem / ppem
            sx, ox = score(xs + advs, p)
            sy, oy = score(ys, p)
            if min(sx, sy) >= min(self.align_x, self.align_y) - 0.01:
                self.upp, self.align_x, self.align_y, self.off_x, self.off_y = p, sx, sy, ox, oy
            else:
                ppem = self.upem / self.upp
        self.ppem = ppem

    # ---------- rasterization ----------
    @lru_cache(maxsize=None)
    def raster(self, gname):
        """Returns (bitmap[rows, cols] bool with row 0 = top, left_px, top_px)."""
        fp = self.poly(gname)
        if not fp.polys:
            return None
        p, ox, oy = self.upp, self.off_x, self.off_y
        allpts = np.concatenate([np.array(pl, dtype=float) for pl in fp.polys])
        xmin, ymin = allpts.min(0)
        xmax, ymax = allpts.max(0)
        i0 = math.floor((xmin - ox) / p + 1e-6)
        i1 = math.ceil((xmax - ox) / p - 1e-6)
        j0 = math.floor((ymin - oy) / p + 1e-6)
        j1 = math.ceil((ymax - oy) / p - 1e-6)
        if i1 <= i0 or j1 <= j0:
            return None
        cx = ox + (np.arange(i0, i1) + 0.5) * p
        cy = oy + (np.arange(j0, j1) + 0.5) * p
        X, Y = np.meshgrid(cx, cy)
        X = X.ravel()[:, None]
        Y = Y.ravel()[:, None]
        edges = []
        for pl in fp.polys:
            a = np.array(pl, dtype=float)
            b = np.roll(a, -1, axis=0)
            edges.append(np.hstack([a, b]))
        E = np.concatenate(edges)
        x0, y0, x1, y1 = E[:, 0], E[:, 1], E[:, 2], E[:, 3]
        isleft = (x1 - x0) * (Y - y0) - (X - x0) * (y1 - y0)
        up = (y0 <= Y) & (y1 > Y) & (isleft > 0)
        down = (y0 > Y) & (y1 <= Y) & (isleft < 0)
        wn = up.sum(1) - down.sum(1)
        bm = (wn != 0).reshape(len(cy), len(cx))[::-1]
        return bm, i0, j1  # top row corresponds to pixel row j1-1 → top edge y=j1

    def adv_px(self, gname):
        return self.hmtx[gname][0] / self.upp

    # ---------- metrics ----------
    def ink_top(self, chars):
        v = []
        for c in chars:
            g = self.gname(c)
            r = g and self.raster(g)
            if r:
                v.append(r[2])
        return max(v) if v else None

    def ink_bottom(self, chars):
        v = []
        for c in chars:
            g = self.gname(c)
            r = g and self.raster(g)
            if r:
                v.append(r[2] - r[0].shape[0])
        return min(v) if v else None

    def metrics(self):
        t = self.tt
        hh = t["hhea"]
        os2 = t["OS/2"] if "OS/2" in t else None
        p = self.upp
        advs = {}
        for c in range(0x20, 0x7F):
            g = self.gname(chr(c))
            if g:
                advs[chr(c)] = round(self.adv_px(g), 2)
        adv_set = sorted(set(advs.values()))
        m = {
            "upem": self.upem,
            "units_per_px": round(p, 3),
            "ppem": round(self.ppem, 3),
            "ppem_integral": abs(self.ppem - round(self.ppem)) < 1e-6,
            "grid_align": round(min(self.align_x, self.align_y), 3),
            "grid_offset_px": (round(self.off_x / p, 2), round(self.off_y / p, 2)),
            "curve_frac": round(self.curve_frac, 3),
            "diag_frac": round(self.diag_frac, 3),
            "mono": len(adv_set) == 1,
            "adv": adv_set[0] if len(adv_set) == 1 else None,
            "adv_n": advs.get("n"), "adv_M": advs.get("M"), "adv_i": advs.get("i"), "adv_0": advs.get("0"),
            "adv_range": (adv_set[0], adv_set[-1]) if adv_set else None,
            "cap": self.ink_top("H"),
            "xh": self.ink_top("x"),
            "asc": self.ink_top("bdhkl"),
            "desc": self.ink_bottom("gjpqy"),
            "hhea": (round(hh.ascent / p, 2), round(hh.descent / p, 2), round(hh.lineGap / p, 2)),
            "typo": (round(os2.sTypoAscender / p, 2), round(os2.sTypoDescender / p, 2), round(os2.sTypoLineGap / p, 2)) if os2 else None,
            "win": (round(os2.usWinAscent / p, 2), round(os2.usWinDescent / p, 2)) if os2 else None,
            "glyphs": len(t.getGlyphOrder()),
            "kern": "GPOS" in t or "kern" in t,
            "embedded_bitmaps": "EBDT" in t,
        }
        m["line_hhea"] = round(m["hhea"][0] - m["hhea"][1] + m["hhea"][2], 2)
        # What cosmic-text (via swash) will use: OS/2 typo metrics when
        # USE_TYPO_METRICS is set (and ascender != 0), else hhea. The baseline
        # lands at line_top + (L - (asc + desc)) / 2 + asc, so it is on the pixel
        # grid only if asc and desc are whole pixels and L - (asc + desc) is even.
        use_typo = bool(os2 is not None and os2.fsSelection & (1 << 7) and os2.sTypoAscender != 0)
        a = (os2.sTypoAscender if use_typo else hh.ascent) / p
        d = -(os2.sTypoDescender if use_typo else hh.descent) / p
        m["ct_metrics"] = "typo" if use_typo else "hhea"
        m["ct_asc"], m["ct_desc"] = round(a, 3), round(d, 3)
        # baseline = top + (L + asc - desc) / 2  -> whole pixel iff L + asc - desc is even.
        k = a - d
        m["ct_crisp"] = abs(k - round(k)) < 0.02
        m["ct_line_parity"] = ("even" if round(k) % 2 == 0 else "odd") if m["ct_crisp"] else None
        # Glyphs that sit on a raised baseline (8x8 ROM style): measure relative
        # to the bottom of 'H' so cap/x/asc/desc are true ink heights.
        hb = self.ink_bottom("H") or 0
        m["base_shift"] = hb
        for k in ("cap", "xh", "asc", "desc"):
            if m[k] is not None:
                m[k] -= hb
        cov = {}
        for k, rng in RANGES.items():
            # Count only code points whose glyph has ink (some BDF-derived
            # fonts map Latin-1 to blank placeholders).
            have = sum(1 for c in rng if c in self.cmap and (c in (0x20, 0xA0) or self.poly(self.cmap[c]).polys))
            if k == "tech" and 0x2126 in self.cmap and 0x3A9 not in self.cmap:
                have += 0  # Ω counted via U+03A9 check below
            cov[k] = (have, len(rng))
        m["cov"] = cov
        name = t["name"]
        m["family"] = name.getBestFamilyName()
        m["subfamily"] = name.getBestSubFamilyName()
        lic = name.getDebugName(13) or ""
        m["license_name"] = lic.strip().replace("\n", " ")[:140]
        m["copyright"] = (name.getDebugName(0) or "").strip().replace("\n", " ")[:140]
        return m

    def line_metrics(self):
        hh = self.tt["hhea"]
        asc = math.ceil(hh.ascent / self.upp - 1e-6)
        desc = math.ceil(-hh.descent / self.upp - 1e-6)
        gap = max(0, round(hh.lineGap / self.upp))
        return asc, desc, gap

    # ---------- text rendering ----------
    def render_line(self, text, missing_box=True):
        """Render one line: returns (canvas uint8 [h,w] with 0/1/2, baseline_row).
        value 1 = ink, 2 = missing-glyph marker."""
        items, pen = [], 0.0
        cap = self.ink_top("H") or max(3, int(self.ppem * 0.6))
        nadv = self.adv_px(self.gname("n")) if self.gname("n") else self.ppem / 2
        for ch in text:
            g = self.gname(ch)
            if g is None:
                w = max(3, round(nadv))
                items.append(("miss", round(pen), w, cap))
                pen += w
                continue
            r = self.raster(g)
            if r:
                items.append(("g", round(pen), r))
            pen += self.adv_px(g)
        tops = [cap] + [it[2][2] for it in items if it[0] == "g"]
        bots = [0] + [it[2][2] - it[2][0].shape[0] for it in items if it[0] == "g"]
        top, bot = max(tops), min(bots)
        lefts = [0] + [it[1] + it[2][1] for it in items if it[0] == "g"]
        rights = [math.ceil(pen)] + [it[1] + it[2][1] + it[2][0].shape[1] for it in items if it[0] == "g"]
        left = min(lefts)
        W = max(rights) - left
        H = top - bot
        canvas = np.zeros((H, W), dtype=np.uint8)
        for it in items:
            if it[0] == "g":
                _, x, (bm, l, t) = it
                r0 = top - t
                c0 = x + l - left
                sub = canvas[r0:r0 + bm.shape[0], c0:c0 + bm.shape[1]]
                sub[bm[: sub.shape[0], : sub.shape[1]]] = 1
            elif missing_box:
                _, x, w, h = it
                r0, c0 = top - h, x - left
                canvas[r0, c0:c0 + w - 1] = 2
                canvas[top - 1, c0:c0 + w - 1] = 2
                canvas[r0:top, c0] = 2
                canvas[r0:top, c0 + w - 2] = 2
        return canvas, top, -left

    def covers(self, text):
        return all(ord(c) in self.cmap or c == " " for c in text)
