# Pixel fonts

A survey of freely licensed pixel fonts for the toolkit, 2026-09-30: about 40 families, 147 files, each measured from its outlines and rendered at its native size. The table at the end has every one. The candidates the TYPE page compares were chosen from it; see `demo/assets/fonts/lab/SOURCES.md`.

## How a pixel font renders in the fork

cosmic-text 0.15 and swash, as the fork uses them:

1. **Outlines only.** Embedded bitmap strikes (EBDT) are never used. A font must be a TTF or OTF whose outlines sit on its pixel grid. TerminusTTF's outlines are its 32 px design, so Terminus comes from its BDF instead.
2. **Native em.** The font size must be the em at which a font pixel is one pixel, or a whole multiple of it.
3. **Baseline.** `line_y = top + (L − (ascent + descent)) / 2 + ascent` (`buffer.rs`). Ascent and descent come from the OS/2 typo metrics when USE_TYPO_METRICS is set, and from hhea otherwise. The baseline is on a whole pixel only if both are whole and `L + ascent − descent` is even.
4. **No default line height is safe.** iced's `LineHeight::Relative(1.3)` is never pixel-safe; always set an absolute integer.
5. **No OpenType features.** cosmic-text supports them but iced doesn't pass them through. Departure's small caps, superscripts, old-style figures and fractions are only reachable by baking them into a derived font.
6. **Loading.** fontdb silently drops a face with no PostScript name (name ID 6), and cosmic-text matches weight exactly: a Medium (500) face doesn't answer a Normal query.

## What made Departure Mono big

Its spacing, not its glyphs:
- Departure Mono draws a 5 px glyph body in a 7 px cell, with a 14 px line.
- 89 of the 94 printable ASCII glyphs keep their ink in columns 1–5; only `# % & { }` reach column 6.
- Narrowing the advance to 6 px, with every glyph left where it is, gives **Departure Mono Tight**: a 6 × 12 cell as dense across a line as Portfolio 6×8 or Terminus 12, still reading as Departure.
- Box-drawing and block glyphs are cropped to the narrower cell so they still tile.

## Families

- **Departure Mono** (OFL, 11 px, Regular only): the most refined, with full box drawing (128) and blocks (32). The 7 px pitch and 14 px line are what cost density.
- **Pixel Code** (OFL): Regular at 9 px sits on the grid; other weights are fractional and only suit large sizes. Departure's nearest relative at a smaller size: 6 × 12 cell, cap 7, x-height 5, but shallow 1 px descenders.
- **Terminus** from BDF (OFL, reserved name "Terminus Font"): 12–32 px with a true bold at every size, 1357 glyphs. At 12 px it has Departure's cap and x-height (8 / 6). Nothing below 12 px.
- **X11 misc-fixed** (public domain): 4×6 to 10×20, with bold at 13–15 px and up to 5206 glyphs. The workstation look; 5×7 and 6×10 are fine small faces.
- **Greybeard** / **ttyp0** (MIT, ttyp0-derived): 11–22 px with bold at every size, italic at 15–18, 3077 glyphs, exactly on the grid. A neutral terminal face.
- **Spleen** (BSD 2-Clause): 5×8 up to 32×64, squared forms, no bold. 5×8 and 6×12 have little box drawing and 5×8 lacks the Latin-1 symbols.
- **Galmuri** (OFL): 7, 9, 11 (with bold and condensed) and 14 px, plus mono 7, 9, 11. The one full system with a proportional face, but less of an instrument feel, and large files (CJK).
- **Ark Pixel / Fusion Pixel** (OFL): 8, 10, 12 px, mono and proportional, neutral. The 8 px mono has a cramped 4 px pitch.
- **Scientifica** (OFL): 11 px em with a 5 px pitch and cap 7, in regular, bold and italic. Slightly quirky letterforms.
- **Cozette** (MIT): 13 px, 6047 glyphs, one size.
- **Gohu** (WTFPL): 11 and 14 px, regular and bold.
- **Tamzen** (Tamsyn licence): a tidy family but only 191 glyphs and no box drawing.
- **unscii** (public domain): 8 × 8 home-computer style with 2 px stems.
- **Pixel Operator** (CC0): 8 and 16 px with mono, small caps and bold; Latin only, no box drawing.
- **Mx / Px Oldschool PC pack** (CC BY-SA 4.0): the HP 100LX 6x8, 8x8, 10x11 and 16x12 form a real family; HP 100LX 6x8 is Portfolio with descenders and 782 glyphs.
- **Tiny5** (OFL): the best of the very small fonts (cap 5, with Greek and Cyrillic).
- **Off the grid:** Pixelify Sans, VT323, DotGothic16 and Boxy Bold don't sit on a pixel grid. Jersey 10's em is 18.667 px and Picopixel's 7.667 px. monogram, Linssen's fonts, Silkscreen, Micro 5, LanaPixel and PixelMplus12 have fractional metrics and need patching before use.

## Directions

**A. Departure first** (chosen for now)

| Role | Font | Em | Cell | Cap / x |
|---|---|---|---|---|
| Body | Departure Mono Tight | 11 | 6 × 12 | 8 / 6 |
| Prose | Departure Mono | 11 | 7 × 14 | 8 / 6 |
| Small | Pixel Code Regular, until an in-house Small exists | 9 | 6 × 10 | 7 / 5 |
| Tiny | X11 Fixed 5×7, until an in-house Micro exists | 7 | 5 × 7 | 6 / 4 |
| Bold | Departure smear-bold, hand-corrected | 11 | 7 × 14 | 8 / 6 |
| Display | Departure at 2× and 3× | 22 / 33 | | |

**B. Terminus and Fixed** (off the shelf): Fixed 5×7, Fixed 6×10, Terminus 12 and 12 Bold, Terminus 14–16 Bold, Terminus 20–32 Bold for display. True bolds at body size and up, excellent coverage; no bold at the small sizes.

**C. Scientifica and Greybeard:** Scientifica 11 (regular, bold, italic), Greybeard 11 and 11 Bold on an odd 13 px line, Greybeard 16–22 Bold for display. A neutral terminal look, with bold everywhere.

**D. Galmuri:** 7 / 9 / 11 / 14 with mono companions and a proportional face. Its 9 and 11 mono faces sit a pixel off their baseline and need patching.

## Licences

- **Safe:** OFL (without a reserved name, or unmodified), MIT, BSD, CC0 and public domain.
- **Modifications:** subsetting, renaming and format conversion count as modifying a font. A font with a Reserved Font Name (Terminus) can't keep that name modified, and ttyp0 derivatives must be renamed.
- **Share-alike:** CC BY-SA 4.0 (the whole Oldschool PC pack, Portfolio included) keeps its licence on derivatives.
- **Unclear or excluded:** Daniel Linssen's m3x6, m5x7 and m6x11 say "free with attribution" with no licence text; 04b is freeware without a licence; Picopixel's name table says GPL. Zpix charges for commercial use and forbids modification; tewi was withdrawn by its author.
- **Departure Mono:** OFL with no Reserved Font Name, so derivatives are allowed. Renaming them is still polite.

## Making our own

Proven in `tools/fonts/`:
1. **Extract** an outline pixel font to BDF at its native size. Departure round-trips through extract and compile with no difference in pixels or advances over 1079 glyphs; its OpenType features are not carried.
2. **Edit** the bitmaps.
3. **Compile** back to TTF: merged square contours, em × 64 units, whole-pixel hhea and OS/2 metrics, name and post tables with a PostScript name, `--embolden` for a smear bold.

Still to do for production:
- carry OpenType features through a `.fea` file;
- generate box-drawing, block, braille and arrow glyphs for each cell size;
- whole-pixel kerning if a proportional face is added;
- a test that renders through swash and compares with the source bitmaps;
- move the scripts to a Rust compiler that can also emit bitmaps for rasters.

The planned sizes, in Departure's style:
- **Departure Small:** em 9, 6 × 12 cell, cap 7, x-height 5, descender 2. Pixel Code has these metrics and is the reference.
- **Departure Micro:** 5 × 8 or 5 × 9, cap 5 or 6, x-height 4. X11 Fixed 5×7, Tamzen 5×9 and Tiny5 are the references.
- **Bold:** smear, then about ten hand fixes per size (m, w, M, W among them).

Effort: about one to two days per size for ASCII, and one to two more for Latin-1 and symbols.

## Every font measured

Columns:
- **em px:** the native size.
- **cap / x / asc / desc:** ink heights in pixels, measured against the bottom of `H`.
- **hhea line:** the line the font's own metrics give.
- **cosmic-text baseline:** whether the baseline lands on a pixel, and the line-height parity it needs.
- **grid:** whether the outlines sit exactly on the pixel grid.
- **Coverage:** glyphs with ink, per block.

| Font | Licence | em px | advance px | cap / x / asc / desc | hhea line | cosmic-text baseline | grid | glyphs | ASCII · L1 · LatA · Greek · Cyr · Box · Block · Arrows · Tech |
|---|---|---|---|---|---|---|---|---|---|
| Departure Mono | OFL-1.1 | 11 | 7 mono | 8 / 6 / 8 / 2 | 14 | ok, L even | exact | 1186 | 95 · 96 · 127 · 49 · 64 · 128 · 32 · 6 · 24 |
| Px437 Portfolio 6x8 (current) | CC BY-SA 4.0 ⚠ | 8 | 6 mono | 7 / 5 / 7 / 0 | 8 | ok, L even | exact | 289 | 95 · 55 · 0 · 12 · 0 · 40 · 8 · 6 · 18 |
| PxPlus HP 100LX 6x8 | CC BY-SA 4.0 ⚠ | 8 | 6 mono | 7 / 5 / 7 / 1 | 8 | ok, L even | exact | 782 | 95 · 96 · 128 · 49 · 64 · 40 · 9 · 6 · 27 |
| PxPlus HP 100LX 8x8 | CC BY-SA 4.0 ⚠ | 8 | 8 mono | 7 / 5 / 7 / 1 | 8 | ok, L even | exact | 782 | 95 · 96 · 128 · 49 · 64 · 40 · 9 · 6 · 27 |
| PxPlus HP 100LX 10x11 | CC BY-SA 4.0 ⚠ | 12 | 10 mono | 9 / 6 / 9 / 1 | 11 | ok, L odd | exact | 782 | 95 · 96 · 128 · 49 · 64 · 40 · 9 · 6 · 27 |
| Px437 EverexME 5x8 | CC BY-SA 4.0 ⚠ | 8 | 5 mono | 7 / 5 / 7 / 1 | 8 | ok, L even | exact | 289 | 95 · 55 · 0 · 12 · 0 · 40 · 8 · 6 · 18 |
| Px437 ATI SmallW 6x8 | CC BY-SA 4.0 ⚠ | 8 | 6 mono | 7 / 5 / 7 / 1 | 8 | ok, L even | exact | 289 | 95 · 55 · 0 · 12 · 0 · 40 · 8 · 6 · 18 |
| Px437 TsengEVA 132 6x8 | CC BY-SA 4.0 ⚠ | 8 | 6 mono | 7 / 5 / 7 / 1 | 8 | ok, L even | exact | 289 | 95 · 55 · 0 · 12 · 0 · 40 · 8 · 6 · 18 |
| Px437 Paradise132 7x9 | CC BY-SA 4.0 ⚠ | 12 | 7 mono | 7 / 5 / 7 / 2 | 9 | ok, L odd | exact | 289 | 95 · 55 · 0 · 12 · 0 · 40 · 8 · 6 · 18 |
| Px437 TridentEarly 8x11 | CC BY-SA 4.0 ⚠ | 12 | 8 mono | 8 / 6 / 8 / 1 | 11 | ok, L odd | exact | 289 | 95 · 55 · 0 · 12 · 0 · 40 · 8 · 6 · 18 |
| Px437 IBM XGA-AI 7x15 | CC BY-SA 4.0 ⚠ | 16 | 7 mono | 9 / 7 / 10 / 3 | 15 | ok, L odd | exact | 289 | 95 · 55 · 0 · 12 · 0 · 40 · 8 · 6 · 18 |
| PxPlus Rainbow100 re.132 (DEC) | CC BY-SA 4.0 ⚠ | 24 | 9 mono | 14 / 10 / 14 / 4 | 20 | ok, L even | exact | 782 | 95 · 96 · 128 · 49 · 64 · 40 · 9 · 6 · 27 |
| Px437 IBM 3270pc | CC BY-SA 4.0 ⚠ | 16 | 9 mono | 9 / 6 / 9 / 2 | 14 | ok, L even | exact | 289 | 95 · 55 · 0 · 12 · 0 · 40 · 8 · 6 · 18 |
| Px437 ToshibaSat 8x8 | CC BY-SA 4.0 ⚠ | 8 | 8 mono | 7 / 5 / 7 / 1 | 8 | ok, L even | exact | 289 | 95 · 55 · 0 · 12 · 0 · 40 · 8 · 6 · 18 |
| Spleen 5x8 | BSD-2-Clause | 8 | 5 mono | 6 / 5 / 7 / 1 | 8 | ok, L even | exact | 473 | 95 · 1 · 0 · 0 · 0 · 11 · 0 · 0 · 1 |
| Spleen 6x12 | BSD-2-Clause | 12 | 6 mono | 8 / 6 / 8 / 3 | 12.84 | ok, L even | exact | 549 | 95 · 96 · 0 · 0 · 64 · 11 · 0 · 0 · 7 |
| Spleen 8x16 | BSD-2-Clause | 16 | 8 mono | 10 / 7 / 10 / 3 | 17.12 | ok, L even | exact | 979 | 95 · 96 · 128 · 12 · 64 · 128 · 32 · 6 · 22 |
| Cozette (Vector) | MIT | 13 | 6 mono | 8 / 6 / 10 / 4 | 13 | ok, L odd | exact | 6047 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 28 |
| Cozette (Vector) Bold | MIT | 13 | 7 mono | 8 / 6 / 10 / 4 | 13 | ok, L odd | exact | 6047 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 28 |
| Tamzen 5x9 | Tamsyn (permissive, free) | 9 | 5 mono | 5 / 4 / 6 / 2 | 9 | ok, L odd | exact | 191 | 95 · 80 · 0 · 0 · 0 · 0 · 0 · 0 · 3 |
| Tamzen 5x9 Bold | Tamsyn (permissive, free) | 9 | 5 mono | 5 / 4 / 6 / 2 | 9 | ok, L odd | exact | 191 | 95 · 80 · 0 · 0 · 0 · 0 · 0 · 0 · 3 |
| Tamzen 6x12 | Tamsyn (permissive, free) | 12 | 6 mono | 7 / 5 / 7 / 2 | 12 | ok, L even | exact, H sits +1px | 191 | 95 · 80 · 0 · 0 · 0 · 0 · 0 · 0 · 3 |
| Tamzen 6x12 Bold | Tamsyn (permissive, free) | 12 | 6 mono | 7 / 5 / 7 / 2 | 12 | ok, L even | exact, H sits +1px | 191 | 95 · 80 · 0 · 0 · 0 · 0 · 0 · 0 · 3 |
| Tamzen 7x13 | Tamsyn (permissive, free) | 13 | 7 mono | 7 / 5 / 7 / 2 | 13 | ok, L odd | exact, H sits +1px | 191 | 95 · 80 · 0 · 0 · 0 · 0 · 0 · 0 · 3 |
| Tamzen 7x13 Bold | Tamsyn (permissive, free) | 13 | 7 mono | 7 / 5 / 7 / 2 | 13 | ok, L odd | exact, H sits +1px | 191 | 95 · 80 · 0 · 0 · 0 · 0 · 0 · 0 · 3 |
| Tamzen 7x14 | Tamsyn (permissive, free) | 14 | 7 mono | 8 / 6 / 9 / 3 | 14 | ok, L even | exact | 190 | 95 · 79 · 0 · 0 · 0 · 0 · 0 · 0 · 3 |
| Tamzen 8x15 | Tamsyn (permissive, free) | 15 | 8 mono | 8 / 6 / 9 / 3 | 15 | ok, L odd | exact | 190 | 95 · 79 · 0 · 0 · 0 · 0 · 0 · 0 · 3 |
| Tamzen 8x16 | Tamsyn (permissive, free) | 16 | 8 mono | 9 / 7 / 10 / 3 | 16 | ok, L even | exact | 190 | 95 · 79 · 0 · 0 · 0 · 0 · 0 · 0 · 3 |
| Terminus 12 | OFL-1.1 | 12 | 6 mono | 8 / 6 / 8 / 2 | 12 | ok, L even | exact | 1357 | 95 · 96 · 128 · 49 · 64 · 120 · 30 · 7 · 27 |
| Terminus 12 Bold | OFL-1.1 | 12 | 6 mono | 8 / 6 / 8 / 2 | 12 | ok, L even | exact | 1357 | 95 · 96 · 128 · 49 · 64 · 120 · 30 · 7 · 27 |
| Terminus 14 | OFL-1.1 | 14 | 8 mono | 10 / 7 / 10 / 2 | 14 | ok, L even | exact | 1357 | 95 · 96 · 128 · 49 · 64 · 120 · 30 · 7 · 27 |
| Terminus 16 | OFL-1.1 | 16 | 8 mono | 10 / 7 / 10 / 3 | 16 | ok, L even | exact | 1357 | 95 · 96 · 128 · 49 · 64 · 120 · 30 · 7 · 27 |
| TerminusTTF (outline) | OFL-1.1 | 32 | 16 mono | 22 / 16 / 22 / 5 | 34.88 | ✗ asc 26.66 desc 5.34 | offset (0.0, 0.65), H sits -1px | 1359 | 95 · 96 · 128 · 49 · 64 · 120 · 30 · 7 · 27 |
| Scientifica | OFL-1.1 | 11 | 5 mono | 7 / 5 / 7 / 2 | 11 | ok, L odd | exact | 1088 | 95 · 96 · 128 · 49 · 0 · 82 · 14 · 7 · 23 |
| Scientifica Bold | OFL-1.1 | 11 | 5 mono | 7 / 5 / 7 / 2 | 11 | ok, L odd | exact | 985 | 95 · 96 · 128 · 49 · 0 · 77 · 14 · 6 · 21 |
| Scientifica Italic | OFL-1.1 | 11 | 5 mono | 7 / 5 / 7 / 2 | 11 | ok, L odd | exact | 1076 | 95 · 96 · 128 · 49 · 0 · 82 · 14 · 7 · 23 |
| Creep | MIT | 11 | 5 mono | 7 / 5 / 7 / 2 | 11 | ok, L odd | exact | 830 | 95 · 93 · 0 · 49 · 0 · 128 · 31 · 6 · 16 |
| GohuFont 11 | WTFPL | 11 | 6 mono | 8 / 5 / 8 / 2 | 11 | ok, L odd | exact | 1598 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 27 |
| GohuFont 11 Bold | WTFPL | 11 | 6 mono | 8 / 5 / 8 / 2 | 11 | ok, L odd | exact | 1598 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 27 |
| GohuFont 14 | WTFPL | 14 | 8 mono | 9 / 7 / 10 / 3 | 14 | ok, L even | exact | 850 | 95 · 96 · 126 · 49 · 64 · 101 · 24 · 7 · 23 |
| ProggyTiny | MIT | 16 | 6 mono | 7 / 5 / 8 / 2 | 10 | ok, L odd | exact | 257 | 95 · 95 · 0 · 0 · 0 · 0 · 0 · 0 · 7 |
| ProggySmall | MIT | 16 | 7 mono | 7 / 5 / 8 / 2 | 10 | ok, L odd | exact | 257 | 95 · 93 · 0 · 0 · 0 · 0 · 0 · 0 · 7 |
| ProggyClean | MIT | 16 | 7 mono | 8 / 6 / 9 / 3 | 13 | ok, L even | exact | 257 | 95 · 95 · 0 · 0 · 0 · 0 · 0 · 0 · 7 |
| ProggySquare | MIT | 16 | 7 mono | 8 / 6 / 8 / 2 | 11 | ok, L even | exact | 257 | 95 · 96 · 0 · 0 · 0 · 0 · 0 · 0 · 7 |
| Unscii 8 | Public domain | 8 | 8 mono | 7 / 5 / 7 / 1 | 8.75 | ok, L even | exact, H sits +1px | 3163 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 26 |
| Unscii 8 thin | Public domain | 8 | 8 mono | 7 / 5 / 7 / 1 | 8.75 | ok, L even | exact, H sits +1px | 3163 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 26 |
| Unscii 16 | Public domain | 16 | 8 mono | 11 / 7 / 11 / 3 | 17.5 | ok, L even | exact, H sits +3px | 3212 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 26 |
| Monogram | CC0 | 16 | 6 mono | 7 / 5 / 7 / 2 | 12.66 | ✗ asc 10.66 desc 2 | exact | 121 | 95 · 10 · 0 · 0 · 0 · 0 · 0 · 0 · 1 |
| Monogram Extended | CC0 | 16 | 6 mono | 7 / 5 / 7 / 2 | 12.66 | ✗ asc 10.66 desc 2 | exact | 580 | 95 · 95 · 127 · 49 · 64 · 0 · 0 · 6 · 19 |
| Monogram Extended Italic | CC0 | 16 | 6 mono | 7 / 5 / 7 / 2 | 13 | ok, L odd | exact | 580 | 95 · 95 · 127 · 49 · 64 · 0 · 0 · 6 · 19 |
| m3x6 | 'free to use with attribution' (no formal licence) ⚠ | 16 | 2–6 (n=4) | 6 / 5 / 6 / 2 | 14.1 | ✗ asc 10.66 desc 2 | exact | 132 | 95 · 32 · 0 · 0 · 0 · 0 · 0 · 0 · 7 |
| m5x7 | 'free to use, attribution appreciated' (no formal licence) ⚠ | 16 | 2–8 (n=6) | 7 / 5 / 7 / 2 | 14.1 | ✗ asc 10.66 desc 2 | exact | 322 | 95 · 95 · 127 · 0 · 0 · 0 · 0 · 0 · 7 |
| m6x11 | 'free to use with attribution' (no formal licence) ⚠ | 16 | 3–10 (n=7) | 11 / 8 / 11 / 3 | 15.44 | ok, L even | exact | 127 | 95 · 27 · 0 · 0 · 0 · 0 · 0 · 0 · 6 |
| m6x11plus | 'free to use with attribution' (no formal licence) ⚠ | 18 | 3–10 (n=7) | 11 / 8 / 11 / 3 | 18 | ok, L even | exact | 228 | 94 · 81 · 47 · 1 · 0 · 0 · 0 · 0 · 4 |
| Pixel Operator 8 | CC0 | 8 | 4–9 (n=7) | 7 / 5 / 7 / 1 | 8.72 | ok, L even | exact | 241 | 95 · 84 · 34 · 0 · 0 · 0 · 0 · 0 · 9 |
| Pixel Operator 8 Bold | CC0 | 8 | 4–14 (n=8) | 7 / 5 / 7 / 1 | 8.72 | ok, L even | exact | 241 | 95 · 84 · 34 · 0 · 0 · 0 · 0 · 0 · 9 |
| Pixel Operator Mono 8 | CC0 | 8 | 8 mono | 7 / 5 / 7 / 1 | 8.72 | ok, L even | exact | 241 | 95 · 84 · 34 · 0 · 0 · 0 · 0 · 0 · 9 |
| Pixel Operator | CC0 | 16 | 4–9 (n=7) | 9 / 7 / 9 / 2 | 16.72 | ok, L even | exact | 241 | 95 · 84 · 34 · 0 · 0 · 0 · 0 · 0 · 9 |
| Pixel Operator Bold | CC0 | 16 | 4–12 (n=8) | 9 / 7 / 9 / 2 | 16.72 | ok, L even | exact | 241 | 95 · 84 · 34 · 0 · 0 · 0 · 0 · 0 · 9 |
| Pixel Operator Mono | CC0 | 16 | 8 mono | 9 / 7 / 9 / 2 | 16.72 | ok, L even | exact | 241 | 95 · 84 · 34 · 0 · 0 · 0 · 0 · 0 · 9 |
| Pixel Operator SC | CC0 | 16 | 4–9 (n=7) | 9 / 7 / 7 / 0 | 16.72 | ok, L even | exact | 241 | 95 · 84 · 34 · 0 · 0 · 0 · 0 · 0 · 9 |
| Pixel Operator HB | CC0 | 16 | 4–12 (n=8) | 9 / 7 / 9 / 2 | 17.72 | ok, L even | exact | 241 | 95 · 84 · 34 · 0 · 0 · 0 · 0 · 0 · 9 |
| Ark Pixel 10px Mono | OFL-1.1 | 10 | 5 mono | 7 / 5 / 7 / 1 | 10 | ok, L even | exact | 3921 | 95 · 95 · 126 · 0 · 0 · 128 · 32 · 9 · 17 |
| Ark Pixel 10px Prop | OFL-1.1 | 10 | 4–8 (n=5) | 7 / 5 / 7 / 2 | 14 | ok, L even | exact | 4190 | 95 · 95 · 128 · 49 · 64 · 128 · 32 · 9 · 19 |
| Ark Pixel 12px Mono | OFL-1.1 | 12 | 6 mono | 8 / 6 / 8 / 2 | 12 | ok, L even | exact | 24260 | 95 · 95 · 126 · 49 · 64 · 128 · 32 · 9 · 19 |
| Ark Pixel 12px Prop | OFL-1.1 | 12 | 4–10 (n=6) | 9 / 6 / 9 / 2 | 16 | ok, L even | exact | 24279 | 95 · 95 · 128 · 49 · 64 · 128 · 32 · 9 · 19 |
| Fusion Pixel 8px Mono | OFL-1.1 | 8 | 4 mono | 6 / 4 / 6 / 1 | 8 | ok, L even | exact | 28048 | 95 · 94 · 128 · 0 · 0 · 128 · 32 · 8 · 20 |
| Fusion Pixel 8px Prop | OFL-1.1 | 8 | 4–6 (n=4) | 6 / 4 / 6 / 2 | 12 | ok, L even | exact | 28048 | 95 · 94 · 128 · 0 · 0 · 128 · 32 · 8 · 20 |
| Galmuri7 | OFL-1.1 | 8 | 2–7 (n=5) | 7 / 5 / 7 / 1 | 11 | ok, L odd | exact | 20151 | 95 · 96 · 128 · 0 · 0 · 128 · 32 · 9 · 22 |
| Galmuri Mono7 | OFL-1.1 | 8 | 4 mono | 7 / 5 / 7 / 1 | 11 | ok, L odd | exact | 19550 | 95 · 89 · 124 · 0 · 0 · 128 · 32 · 9 · 22 |
| Galmuri9 | OFL-1.1 | 10 | 2–8 (n=6) | 9 / 6 / 9 / 1 | 13 | ok, L odd | exact | 20714 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 27 |
| Galmuri Mono9 | OFL-1.1 | 10 | 5 mono | 8 / 5 / 8 / 1 | 13 | ok, L odd | exact, H sits +1px | 19856 | 95 · 95 · 128 · 49 · 64 · 128 · 32 · 9 · 27 |
| Galmuri11 | OFL-1.1 | 12 | 4–10 (n=7) | 11 / 8 / 11 / 2 | 16 | ok, L even | exact | 20968 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 27 |
| Galmuri11 Bold | OFL-1.1 | 12 | 3–13 (n=8) | 11 / 8 / 11 / 2 | 16 | ok, L even | exact | 12698 | 95 · 96 · 0 · 0 · 0 · 128 · 32 · 4 · 18 |
| Galmuri11 Condensed | OFL-1.1 | 12 | 2–8 (n=5) | 10 / 7 / 10 / 2 | 16 | ok, L even | exact, H sits +1px | 13146 | 95 · 96 · 128 · 0 · 64 · 109 · 32 · 6 · 16 |
| Galmuri Mono11 | OFL-1.1 | 12 | 6 mono | 10 / 7 / 10 / 2 | 16 | ok, L even | exact, H sits +1px | 20327 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 27 |
| Galmuri14 | OFL-1.1 | 15 | 5–15 (n=9) | 14 / 10 / 14 / 3 | 21 | ok, L even | exact | 19225 | 95 · 96 · 128 · 0 · 0 · 76 · 5 · 6 · 16 |
| Zpix | Proprietary: free personal/edu, USD 1000 commercial; no modification ✗ | 12 | 3–8 (n=7) | 9 / 7 / 9 / 1 | 12 | ok, L even | exact, H sits -1px | 22241 | 95 · 95 · 128 · 49 · 64 · 112 · 18 · 6 · 25 |
| Silkscreen | OFL-1.1 | 8 | 3–7 (n=7) | 5 / 5 / 5 / 1 | 10.24 | ✗ asc 8.24 desc 2 | exact | 228 | 95 · 96 · 7 · 0 · 0 · 0 · 0 · 0 · 9 |
| Silkscreen Bold | OFL-1.1 | 8 | 4–8 (n=8) | 5 / 5 / 5 / 1 | 10.24 | ✗ asc 8.24 desc 2 | exact | 228 | 95 · 96 · 7 · 0 · 0 · 0 · 0 · 0 · 9 |
| Tiny5 | OFL-1.1 | 8 | 2–6 (n=4) | 5 / 4 / 5 / 1 | 9 | ok, L odd | exact | 1155 | 95 · 95 · 128 · 49 · 64 · 0 · 0 · 6 · 27 |
| Micro 5 | OFL-1.1 | 11 | 2–7 (n=4) | 5 / 4 / 5 / 1 | 11 | ✗ asc 7.93 desc 3.07 | exact | 360 | 95 · 84 · 99 · 0 · 0 · 0 · 0 · 0 · 6 |
| Bytesized | OFL-1.1 | 8 | 4 mono | 4 / 3 / 4 / 0 | 10 | ok, L even | exact, H sits -1px | 328 | 95 · 88 · 100 · 0 · 0 · 0 · 0 · 0 · 12 |
| Pixelify Sans (VF) | OFL-1.1 | 11* | 2.2–9.54 (n=6.45) | 8 / 6 / 8 / 2 | 13.2 | ✗ asc 10.12 desc 3.08 | OFF (0.21), H sits -1px | 576 | 95 · 94 · 126 · 46 · 62 · 0 · 0 · 0 · 16 |
| Jersey 10 | OFL-1.1 | 18.67 | 3–12 (n=8) | 10 / 8 / 10 / 2 | 20 | ok, L even | exact | 363 | 95 · 84 · 99 · 0 · 0 · 0 · 0 · 0 · 6 |
| Press Start 2P | OFL-1.1 | 8 | 8 mono | 7 / 5 / 7 / 1 | 8 | ok, L even | exact, H sits +1px | 658 | 95 · 96 · 125 · 49 · 64 · 0 · 0 · 4 · 20 |
| VT323 | OFL-1.1 | 25* | 10 mono | 14 / 10 / 14 / 4 | 25 | ok, L odd | OFF (0.69) | 587 | 95 · 96 · 127 · 4 · 0 · 0 · 0 · 0 · 17 |
| DotGothic16 | OFL-1.1 | 16* | 8 mono | 14 / 10 / 13 / 1 | 23.17 | ✗ asc 18.56 desc 4.61 | OFF (0.19), H sits -1px | 9362 | 95 · 96 · 10 · 48 · 64 · 87 · 17 · 4 · 21 |
| Kirsch | OFL-1.1 | 16 | 6 mono | 9 / 5 / 9 / 3 | 16 | ok, L even | exact | 8291 | 95 · 95 · 128 · 49 · 64 · 128 · 32 · 9 · 28 |
| Kirsch Propo | OFL-1.1 | 16 | 6 mono | 9 / 5 / 9 / 3 | 16 | ok, L even | exact | 8291 | 95 · 95 · 128 · 49 · 64 · 128 · 32 · 9 · 28 |
| Cherry 10 | ISC-style (permissive) | 10 | 6 mono | 7 / 5 / 7 / 2 | 10 | ok, L even | exact | 193 | 95 · 96 · 0 · 0 · 0 · 0 · 0 · 0 · 6 |
| Cherry 10 Bold | ISC-style (permissive) | 10 | 6 mono | 7 / 5 / 7 / 2 | 10 | ok, L even | exact | 193 | 95 · 96 · 0 · 0 · 0 · 0 · 0 · 0 · 6 |
| Cherry 13 | ISC-style (permissive) | 13 | 7 mono | 9 / 7 / 9 / 2 | 13 | ok, L odd | exact | 193 | 95 · 96 · 0 · 0 · 0 · 0 · 0 · 0 · 6 |
| Tom Thumb | MIT / CC0 (Robey Pointer) | 6 | 4 mono | 5 / 4 / 5 / 1 | 6 | ok, L even | exact | 204 | 95 · 95 · 7 · 0 · 0 · 0 · 0 · 0 · 8 |
| Monocraft | OFL-1.1 | 18 | 12 mono | 14 / 10 / 14 / 2 | 25.62 | ok, L even | exact | 1700 | 95 · 95 · 109 · 49 · 64 · 0 · 0 · 5 · 19 |
| Monocraft Bold | OFL-1.1 | 22.5 | 15 mono | 19 / 14 / 19 / 3 | 33.02 | ✗ asc 20 desc 2.5 | OFF (0.50), H sits -1px | 1700 | 95 · 95 · 109 · 49 · 64 · 0 · 0 · 5 · 19 |
| Pixel Code | OFL-1.1 | 9 | 6 mono | 7 / 5 / 7 / 1 | 11.98 | ok, L even | exact | 1928 | 95 · 92 · 127 · 49 · 64 · 126 · 31 · 6 · 19 |
| Pixel Code Bold | OFL-1.1 | 9* | 6 mono | 8 / 6 / 8 / 1 | 11.98 | ok, L even | OFF (0.15) | 1928 | 95 · 92 · 127 · 49 · 64 · 126 · 31 · 6 · 19 |
| Greybeard 11px | MIT (ttyp0-derived) | 11 | 6 mono | 8 / 6 / 8 / 2 | 11 | ok, L odd | exact | 3077 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| Greybeard 11px Bold | MIT (ttyp0-derived) | 11 | 6 mono | 8 / 6 / 8 / 2 | 11 | ok, L odd | exact | 3077 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| Greybeard 12px | MIT (ttyp0-derived) | 12 | 6 mono | 8 / 6 / 8 / 2 | 12 | ok, L even | exact | 3077 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| Greybeard 13px | MIT (ttyp0-derived) | 13 | 7 mono | 9 / 7 / 10 / 2 | 13 | ok, L odd | exact | 3077 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| Greybeard 14px | MIT (ttyp0-derived) | 14 | 7 mono | 9 / 7 / 10 / 2 | 14 | ok, L even | exact | 3077 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| Greybeard 16px | MIT (ttyp0-derived) | 16 | 8 mono | 10 / 7 / 10 / 3 | 16 | ok, L even | exact | 3077 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| Greybeard 16px Bold | MIT (ttyp0-derived) | 16 | 8 mono | 10 / 7 / 10 / 3 | 16 | ok, L even | exact | 3077 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| Dogica (mono) | OFL-1.1 (RFN 'Dogica') | 8 | 8 mono | 7 / 6 / 7 / 1 | 6 | ok, L even | exact | 262 | 95 · 96 · 12 · 0 · 0 · 0 · 0 · 0 · 9 |
| Dogica Pixel (kerned) | OFL-1.1 (RFN 'Dogica') | 8 | 3–9 (n=7) | 7 / 6 / 7 / 1 | 6 | ok, L even | exact | 262 | 95 · 96 · 12 · 0 · 0 · 0 · 0 · 0 · 9 |
| Dogica Pixel (kerned) Bold | OFL-1.1 (RFN 'Dogica') | 8 | 4–10 (n=8) | 7 / 6 / 7 / 1 | 6 | ok, L even | exact | 262 | 95 · 96 · 12 · 0 · 0 · 0 · 0 · 0 · 9 |
| LanaPixel | OFL-1.1 or CC-BY 4.0 (dual) | 11 | 2–8.01 (n=5) | 8 / 6 / 9 / 4 | 14 | ✗ asc 7.97 desc 3.02 | exact | 19519 | 95 · 96 · 128 · 49 · 64 · 0 · 0 · 6 · 9 |
| Picopixel | GPL (per name table; unclear font exception) ⚠ | 7.67 | 1.99–5.97 (n=3.98) | 5 / 3 / 5 / 1 | 6.98 | ok, L odd | exact | 105 | 95 · 9 · 0 · 0 · 0 · 0 · 0 · 0 · 0 |
| 04b03 | Freeware, no licence text ⚠ | 8 | 2–6 (n=5) | 5 / 4 / 5 / 2 | 8 | ok, L even | exact | 99 | 95 · 1 · 0 · 0 · 0 · 0 · 0 · 0 · 0 |
| 04b08 | Freeware, no licence text ⚠ | 8 | 2–7 (n=6) | 5 / 5 / 5 / 0 | 8 | ok, L even | exact | 99 | 95 · 1 · 0 · 0 · 0 · 0 · 0 · 0 · 0 |
| Kenney Mini | CC0 | 8 | 2–6 (n=5) | 5 / 4 / 5 / 1 | 10 | ok, L even | exact | 212 | 95 · 95 · 1 · 0 · 0 · 0 · 0 · 0 · 8 |
| Kenney Mini Square Mono | CC0 | 8 | 6 mono | 5 / 5 / 5 / 0 | 10 | ok, L even | exact | 210 | 95 · 95 · 1 · 0 · 0 · 0 · 0 · 0 · 8 |
| Kenney Pixel | CC0 | 16 | 2–8 (n=6) | 7 / 6 / 8 / 1 | 12 | ok, L even | exact | 212 | 95 · 95 · 1 · 0 · 0 · 0 · 0 · 0 · 8 |
| Kenney High | CC0 | 16 | 2–8 (n=6) | 9 / 9 / 9 / 1 | 14 | ok, L even | exact | 212 | 95 · 95 · 1 · 0 · 0 · 0 · 0 · 0 · 8 |
| PixelMplus10 | M+ FONT LICENSE (permissive) | 10 | 5 mono | 7 / 5 / 7 / 2 | 11 | ok, L odd | exact | 7253 | 95 · 96 · 0 · 48 · 64 · 32 · 0 · 4 · 20 |
| PixelMplus10 Bold | M+ FONT LICENSE (permissive) | 10 | 5 mono | 7 / 5 / 7 / 2 | 11 | ok, L odd | exact | 7253 | 95 · 96 · 0 · 48 · 64 · 32 · 0 · 4 · 20 |
| PixelMplus12 | M+ FONT LICENSE (permissive) | 12 | 6 mono | 10 / 7 / 10 / 2 | 13.01 | ✗ asc 10.61 desc 2.4 | offset (0.0, 0.6), H sits -1px | 7253 | 95 · 96 · 0 · 48 · 64 · 32 · 0 · 4 · 20 |
| PixelMplus12 Bold | M+ FONT LICENSE (permissive) | 12 | 6 mono | 10 / 7 / 10 / 2 | 13.01 | ✗ asc 10.61 desc 2.4 | offset (0.0, 0.6), H sits -1px | 7253 | 95 · 96 · 0 · 48 · 64 · 32 · 0 · 4 · 20 |
| X11 Fixed 4x6 | Public domain | 6 | 4 mono | 5 / 4 / 5 / 1 | 6 | ok, L even | exact | 920 | 95 · 96 · 128 · 49 · 64 · 128 · 22 · 6 · 21 |
| X11 Fixed 5x7 | Public domain | 7 | 5 mono | 6 / 4 / 6 / 1 | 7 | ok, L odd | exact | 1849 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 27 |
| X11 Fixed 5x8 | Public domain | 8 | 5 mono | 6 / 4 / 6 / 1 | 8 | ok, L even | exact | 1427 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 27 |
| X11 Fixed 6x9 | Public domain | 9 | 6 mono | 6 / 4 / 6 / 2 | 9 | ok, L odd | exact | 1297 | 95 · 96 · 128 · 49 · 64 · 53 · 32 · 6 · 27 |
| X11 Fixed 6x10 | Public domain | 10 | 6 mono | 7 / 5 / 7 / 2 | 10 | ok, L even | exact | 1598 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 27 |
| X11 Fixed 6x12 | Public domain | 12 | 6 mono | 7 / 5 / 7 / 2 | 12 | ok, L even | exact | 4532 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 28 |
| X11 Fixed 6x13 | Public domain | 13 | 6 mono | 9 / 6 / 9 / 2 | 13 | ok, L odd | exact | 4122 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 28 |
| X11 Fixed 6x13 Bold | Public domain | 13 | 6 mono | 9 / 6 / 9 / 2 | 13 | ok, L odd | exact | 1283 | 95 · 96 · 128 · 49 · 64 · 22 · 1 · 4 · 15 |
| X11 Fixed 7x13 | Public domain | 13 | 7 mono | 9 / 6 / 9 / 2 | 13 | ok, L odd | exact | 3227 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 27 |
| X11 Fixed 7x13 Bold | Public domain | 13 | 7 mono | 9 / 6 / 9 / 2 | 13 | ok, L odd | exact | 1004 | 95 · 96 · 128 · 49 · 64 · 18 · 1 · 4 · 15 |
| X11 Fixed 7x14 | Public domain | 14 | 7 mono | 10 / 7 / 10 / 2 | 14 | ok, L even | exact | 2577 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 27 |
| X11 Fixed 7x14 Bold | Public domain | 14 | 7 mono | 10 / 7 / 10 / 2 | 14 | ok, L even | exact | 1010 | 95 · 96 · 128 · 49 · 64 · 18 · 1 · 4 · 15 |
| X11 Fixed 8x13 | Public domain | 13 | 8 mono | 9 / 6 / 9 / 2 | 13 | ok, L odd | exact | 3704 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 27 |
| X11 Fixed 8x13 Bold | Public domain | 13 | 8 mono | 10 / 7 / 10 / 2 | 13 | ok, L odd | exact | 1142 | 95 · 96 · 128 · 49 · 64 · 24 · 1 · 4 · 15 |
| X11 Fixed 9x15 | Public domain | 15 | 9 mono | 10 / 7 / 10 / 3 | 15 | ok, L odd | exact | 4778 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 9 · 28 |
| X11 Fixed clR6x12 | Public domain | 12 | 6 mono | 7 / 5 / 8 / 3 | 12 | ok, L even | exact | 1195 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 27 |
| UW ttyp0 11 | ttyp0 (MIT + rename-if-modified) | 11 | 6 mono | 8 / 6 / 8 / 2 | 11 | ok, L odd | exact | 3771 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| UW ttyp0 11 Bold | ttyp0 (MIT + rename-if-modified) | 11 | 6 mono | 8 / 6 / 8 / 2 | 11 | ok, L odd | exact | 3771 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| UW ttyp0 12 | ttyp0 (MIT + rename-if-modified) | 12 | 6 mono | 8 / 6 / 8 / 2 | 12 | ok, L even | exact | 3771 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| UW ttyp0 13 | ttyp0 (MIT + rename-if-modified) | 13 | 7 mono | 9 / 7 / 10 / 2 | 13 | ok, L odd | exact | 3771 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| UW ttyp0 13 Bold | ttyp0 (MIT + rename-if-modified) | 13 | 7 mono | 9 / 7 / 10 / 2 | 13 | ok, L odd | exact | 3771 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| UW ttyp0 14 | ttyp0 (MIT + rename-if-modified) | 14 | 7 mono | 9 / 7 / 10 / 2 | 14 | ok, L even | exact | 3771 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| UW ttyp0 16 | ttyp0 (MIT + rename-if-modified) | 16 | 8 mono | 10 / 7 / 10 / 3 | 16 | ok, L even | exact | 3771 | 95 · 96 · 128 · 49 · 64 · 128 · 32 · 6 · 28 |
| Departure Mono (TTF->BDF->TTF roundtrip) | OFL-1.1 | 11 | 7 mono | 8 / 6 / 8 / 2 | 14 | ok, L even | exact | 1080 | 95 · 96 · 127 · 49 · 64 · 128 · 32 · 6 · 24 |
| Departure Mono (derived smear bold) Bold | OFL-1.1 | 11 | 7 mono | 8 / 6 / 8 / 2 | 14 | ok, L even | exact | 1080 | 95 · 96 · 127 · 49 · 64 · 128 · 32 · 6 · 24 |
| Departure Mono Tight (derived, adv 6) | OFL-1.1 | 11 | 6 mono | 8 / 6 / 8 / 2 | 14 | ok, L even | exact | 1080 | 95 · 96 · 127 · 49 · 64 · 128 · 32 · 6 · 24 |
| Boxy Bold Bold | CC0 (per OpenGameArt listing) | 8 | 2.75–9.75 (n=6.75) | 8 / 7 / 8 / 1 | 9 | ok, L odd | OFF (0.86), H sits -1px | 102 | 95 · 0 · 0 · 0 · 0 · 0 · 0 · 0 · 0 |
