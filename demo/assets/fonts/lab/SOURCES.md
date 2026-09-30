# Type lab candidates

The fonts in this folder exist only so the showcase's type lab can compare
them with the toolkit's own. Each was subset to Latin, Greek, symbols and box
drawing, given a PostScript name, and had its vertical metrics rounded to
whole pixels by `tools/fonts/lab.py`; none of them is used by the library.

| File | Font | Source | Licence |
|---|---|---|---|
| `departure-bold.ttf` | Departure Mono, thickened a pixel | derived from Departure Mono 1.500 (Helena Zhang) | SIL OFL 1.1 |
| `pixel-code.ttf` | Pixel Code Regular, 9 px | github.com/qwerasd205/PixelCode v2.2 (Qwerasd) | SIL OFL 1.1 |
| `fixed-5x7.ttf`, `fixed-6x10.ttf` | X11 misc-fixed 5×7, 6×10 | Markus Kuhn's ucs-fonts, BDF converted | Public domain |
| `terminus-12.ttf`, `terminus-12-bold.ttf` | Terminus 12 | terminus-font 4.49.1 (Dimitar Toshkov Zhekov), BDF converted | SIL OFL 1.1, Reserved Font Name "Terminus Font" |
| `greybeard-11.ttf`, `greybeard-11-bold.ttf` | Greybeard 11px | github.com/flowchartsman/greybeard 1.0.0 (Uwe Waldmann, Andy Walker) | MIT, ttyp0-derived |
| `scientifica.ttf` | scientifica | github.com/nerdypepper/scientifica 2.3 (Akshay Oppiliappan) | SIL OFL 1.1 |
| `spleen-5x8.ttf`, `spleen-6x12.otf` | Spleen 5×8, 6×12 | github.com/fcambus/spleen 2.2.0 (Frederic Cambus) | BSD 2-Clause |
| `cozette.ttf` | CozetteVector | github.com/slavfox/Cozette 1.30.0 | MIT |
| `gohu-11.ttf` | GohuFont 11 | github.com/hchargois/gohufont | WTFPL |
| `portfolio-6x8.ttf`, `hp100lx-6x8.ttf` | Px437 Portfolio 6x8, PxPlus HP 100LX 6x8 | The Ultimate Oldschool PC Font Pack 2.2 (VileR, int10h.org) | CC BY-SA 4.0 |
| `unscii-8.ttf` | unscii 8 | viznut.fi/unscii | Public domain |

Licence texts are in `licenses/`.

Before this repository is published, the candidates need another look:
subsetting and renaming are modifications, so Terminus (a Reserved Font Name)
and any other font whose licence restricts modified versions should be renamed
or fetched unmodified at build time, and the CC BY-SA files keep their licence.
