# departuremono.com

An analysis of the Departure Mono site (Helena Zhang), 2026-09-30, from its live DOM, computed styles, CSS, script bundle and SVG assets, at 1440 px and 390 px wide. The site's assets are its author's and are not kept here.

Page facts: it is a SolidJS single-page app (`#root`), 17,621 px tall at 1440 wide. It uses one font, Departure Mono v1.500 (1,186 glyphs). The CSS is about 15 KB. Nearly all artwork is vector SVG. The only raster elements are the game's canvas and its ball and paddle PNGs.

---

## 0. The core idea in one paragraph

The page tells one story in two acts, and the whole thing is a world built from fictional props. Act 1 is **daylight or paper**: a desk collage in `#eeeeee` and `#444444`. It holds a mission letter to "Dr. E. Kerning", a newspaper clipping, an ID badge, a boarding pass, a diner receipt and a luggage tag. Between the acts, a **spacecraft HUD** sticks to the screen while the page background fades from light to near-black, and the HUD turns from grey to **amber**. Act 2 is **night or terminal** in `#222222` and `#c0c0c0`. It holds box-drawing diagrams, a glyph inspector, a NASA-style engineering cutaway with dozens of callouts, amber syntax-highlighted code, a tractor-feed printout, and a playable Arkanoid.

Every prop uses only the one monospaced pixel font, a restrained grey scale, olive-grey hairlines, and a single warm accent. Almost every dimension is a multiple of 11 px, because the font's pixel is 1/11 of its size.

---

## 1. Section-by-section walkthrough

### 1.1 Header (light, y 0 to 410)
- **Title block**: `DEPARTURE_MONO` at 88 px in `--soot #333` on an `--aluminum #ccc` background. It reads as a highlighted selection or LCD segment. Line-height is 1, so the grey box hugs the glyph pixels exactly.
- **Title glitch**: every 400 to 2000 ms, random letters swap to look-alike glyphs (E becomes 3/Σ/Ξ/€/Ǝ, A becomes Λ, R becomes 2/₹, T becomes 7, U becomes Ʉ, O becomes 0, N becomes Ɲ, space becomes _). About 10% of the time it does a 4-frame flicker at 30 ms per frame. Captured frames include `DEPAR7U2E M0N0` and `DEPΛRTURE`.
- **Version tag** `v1.500` at 11 px, set as a superscript next to the title (top-aligned, 11 px offset).
- **Menu** in the top right, 16.5 px, one link per line, each with a glyph prefix used as an icon: `↓ DOWNLOAD`, `> GITHUB`, `♥ DONATE`. On hover the link gets a `--foam #bccabb` background block, like a highlighter or selection.
- **Layout**: 137 px top padding, max-width 1440, 44 px side gutters. The layout is asymmetric: the title is on the left and the menu hangs right.

### 1.2 Letter collage (`#letter`, light)
- **Margin comment**: a gutter of `░` light-shade glyphs forms a dithered vertical bar. Beside it is 11 px uppercase clay text: "DEPARTURE MONO IS A / MONOSPACED PIXEL FONT WITH / A LO-FI TECHNICAL VIBE". This comment device repeats in every section.
- **Mission brief** (`brief.svg`): white paper with a column of punched holes (light-grey circles every 66 px) and a faint vertical margin rule. It is a typewritten letter at 16.5 px on a 24.75 px leading. It carries two address blocks: "MERCURY RESEARCH LABS / 3572 WILSHIRE BLVD #9732 / LOS ANGELES, CA 90010" and "FARNSWORTH INSTITUTE / 340 WEST 77TH ST / NEW NEW YORK, NY 10024". It opens "Dear Dr. Kerning," and has key phrases **highlighted in sage** (`#8AA788` at 50% opacity).
- **Newspaper clipping** in newsprint tan `#D9C9B6`. The headline "EVIDENCE OF GEOTHERMAL ACTIVITY ON NEWLY DISCOVERED KBOS" is set big and ragged. A **dithered halftone "photo"** is made of blocky pixel clusters. A hairline rule sits above 11 px body text, and the bottom edge is **torn**.
- **Paperclip**: a single 2 px `#333` stroke path that overlaps the clipping.
- **ID badge** in `#444`, with rounded corners and a lanyard slot bar. It reads "TIER 1 ACCESS", with an **ordered-dither photo placeholder**, a white square chip, a vertical stacked number `2 / 0 / 7`, the name "Dr. E. Kerning" and "Mercury Research Labs". It sits mostly off-screen at left: -250 px. On hover it slides in 100 px and rotates 3°.
- **Planet construction drawing** (`planet.svg`): 1 px clay strokes. It is a 992 px circle plus a flattened ellipse (an orbit or globe), horizontal and vertical axes, and a few chords and diagonals. It reads like a geometry construction sheet and sits behind everything, cropped by the viewport.
- **Highlighter** (`highlighter-outline.svg`): a patent-style **outline drawing** of a highlighter pen in 1 px clay with round joins. Every face and bevel is drawn as a separate closed path.

### 1.3 Departure board (`#departures`)
- A plain `<pre>` at 44 px, line-height 1.27, drawn **entirely with box-drawing characters** (`│ ├─┼─┤`). Its columns are Flight / Destination ↑ / Departing / Gate / Status. The rows use codes: `LH789 EUR Europa 1 13:45 Z23 Delayed`, `XX123 KBA Kuiper Alpha 08:00 22 On Time`, `AF321 MAR Mars Landing`, `UA567 NTK New Tokyo`, `QF678 ZMB Zvezda Moonbase`.
- `↑` in the header acts as a **sort indicator**.
- The **XX123 row blinks** with a foam-green background: `steps(1)` at 1 s, a hard on/off with no fade.
- The table is shifted **-290 px, so it bleeds off the left edge**: the Flight column is cut off. The bleed is deliberate.
- Comment: "IT'S GREAT FOR WORKING WITH TABULAR DATA".

### 1.4 Travel ephemera (`#ephemera`)
- **Boarding pass** (`#444` card, 1352 × 466):
  - A slot marker (a white bar) sits beside "KÁRMÁN SPACELINES / BOARDING PASS".
  - A **dithered-noise "barcode"** with white end blocks sits above the serial `XX-2488319840`.
  - Tiny 11 px caps field labels (PASSENGER, ORIGIN, DESTINATION, FLIGHT, DEPARTURE, GATE, BOARDING, SEAT) sit above larger values.
  - Huge IATA codes `LAX` and `KBA`, with `LOS ANGELES ▶▶ KUIPER ALPHA`.
  - An asterisk frame `****** FIRST CLASS ******`.
  - **Dashed perforation** tear lines, and a stub on the right that repeats the key data.
  - **Vertical fine print**: "DO NOT EXPOSE TO EXCESSIVE HEAT OR DIRECT SUNLIGHT".
- **Receipt** (white thermal paper with a torn bottom edge):
  - "Double R Diner / Terminal 9 / Los Angeles Intl Airport / 1 World Way…", dated "Thu April 15, 2049 10:29 AM".
  - `Merchant ID: ***3047`, `Terminal ID: ***6832`, `Transaction ID: #E6D598EF`, `Type: CREDIT`.
  - "Cherry pie 39.75 / Damn good coffee 25.50", "Subtotal 65.25". Amounts are right-aligned in a monospace column.
- **Bag tag** (sage `#8AA788`, rotated 270°):
  - A chamfered corner and a punched hole with a darker ring.
  - A grid of double hairline rules.
  - `WEIGHT 45 LBS` (LBS in small caps) and `ORIG. FLIGHT XX789→`.
  - A **rotated vertical label** "DESTINATION" and a letterspaced `K U I P E R / A L P H A`.
  - A letterspaced serial strip `X X - 2 4 8 8 3 1 9 8 4 0`.
  - A **dither-gradient block** in the corner.
- Hovering the group fans the items apart: translate plus rotate, over 1 s with a snappy ease.

### 1.5 Type samples
Each sample has a **spec header** in 11 px clay: `DEPARTURE MONO    121PX    -11PX TRACK`. The samples are:
- "ATTN: PASSENGERS QUIET IN THE CABIN" at 121 px, -11 px tracking.
- "FLIGHT ATTENDANTS, PREPARE FOR TAKEOFF" at 55 px, -5 px tracking.
- A caps paragraph at 22 px.
- A sentence-case paragraph at 16.5 px.

All samples are `contenteditable`, so the section doubles as a type tester. Focus inverts the sample: light text on a dark block.

### 1.6 Cockpit HUD transition (`#transition`, 1800 px tall; the SVG is sticky)
This is the centrepiece. An `IntersectionObserver` on a sentinel sets `--t` to either 0% or 100%. The background uses `color-mix(in lch, enamel, carbon var(--t))` and the ink uses `color-mix(in lch, smoke, amber var(--t))`, both with a 0.4 s transition. The HUD therefore switches from **grey on light** to **amber on black**, and the rest of the page stays dark afterwards.

The HUD is a 1218 × 580 SVG. Its text is **outlined pixel-font glyph paths**, so letters are drawn with rectangles on a 1, 1.25, 1.5 or 2 px grid. All colour is `currentColor`.
- **Top strip**: `EARTH TIME:` over `15:20:30:85` in the top-left and top-right corners. It ticks every frame (HH:MM:SS:centiseconds). In the centre is `|||||||| D E P A R T U R E   M O N O ||||||||`: letters spaced out with literal spaces and flanked by tick bars.
- **Left column**:
  - Section headers use **box-drawing bracket frames**: `┌──── OMS ────┐`, `┌─ OS AUDIO ─┐`, `┌─ VID INPUT ─┐`. They show the top edge only, with corner drops.
  - Gauge labels `HE TANK P` and `N₂ TANK P` use a real subscript.
  - **Horizontal bar gauges**: an 87 × 15 1 px outline with a 16 px solid fill and an 11 px numeric readout (`3480`, `3540`, `3520`).
  - A **semicircular dial**: a 2 px arc with a 1 px needle.
  - Three **33 × 33 solid key-caps** `A B C` with knocked-out letters.
- **Glyph tape**: a vertical numeric tape `1180…1192` at 22 px. The current value, **1186**, is the font's real glyph count, framed by **four L-shaped corner brackets** (11 × 2 px bars). The tape fades out at top and bottom with a gradient mask (hidden 0 to 15%, fully visible 35 to 75%, hidden 95 to 100%).
- **Centre reticle**:
  - A 2 px circle (r 220).
  - Inside it, two masked 1 px ellipses (rx 160 / ry 360 and rx 360 / ry 160) form a **curved globe grid**.
  - Crosshair lines that **stop short of the centre**, leaving a gap, and a small 44 px plus sign in the middle.
  - 22 px ticks at the four cardinal points outside the ring, plus four diagonal ticks.
  - **Chevron side brackets** `⟨ ⟩`: 2 px strokes with a centre tick.
  - A floating 2 px box reading `R 003 / P 031 / Y 133` (roll, pitch, yaw; zero-padded, random-walking).
  - An **inverse label** `A U T O`.
  - A **warp starfield**: small squares spawn at the centre, accelerate outward, grow as `sqrt(distance)/2`, and are clipped to the circle.
- **Right column**:
  - An `ACCEL FPS²` tape `30 25 … 00 -01 -02 -03` with a solid **◀ triangle pointer**. It jitters every second with a 500 ms ease-out.
  - `INT 68°F 20°C / EXT -456°F -271°C`, padded into columns. EXT jitters between -273 and -270.
  - Framed groups `┌── APU ──┐` (FUEL %, H₂O %) and `┌─ HYDRAULIC ─┐` (QTY %), with zero-padded readouts `090`, `095`, `071`.
- **Bottom spec plate**: an inverse two-line tag `DEPARTURE / MONO`, stepped so the second line is narrower, like a folder tab. Next to it are key/value pairs `INITIATED: 2022`, `LAUNCHED: 2024`, `CLASSIFICATION: PIXEL MONOSPACED`, `LICENSE: SIL OFL`.
- On mobile the fixed-size SVG is simply centre-cropped, so only the reticle shows.

### 1.7 Box drawing (`#diagram1`, dark)
- Comment: "INCLUDED: BOX DRAWING CHARACTERS FOR YOUR PSEUDO-GRAPHICAL NEEDS".
- **TCP header diagram** (RFC style) at 11 px. It has a **two-row bit ruler**: tens `0 … 1 … 2 … 3` above ones `0 1 2 … 9 0 1`. Below that is a tick row `├─┴─┴─┴─┼─┴─…─┤`, then fields such as Source Port / Destination Port / Sequence Number / Offset / Res. / Flags / Window / Checksum / Urgent Pointer / Options / Padding.
- **Keyboard layout** drawn in box drawing at 16.5 px, cropped by the right edge.
- Colour: `--clay #6c6c58` on carbon. It is deliberately low-contrast (2.97:1) and reads like a blueprint underlay.

### 1.8 Glyph inspector (`#tester`, dark, `cursor: cell`)
- **Specimen card**, sticky at 44 px from the top: a `--cement #c0c0c0` card with a **48 px chamfered top-right corner** made with `clip-path`.
  - Top row: glyph name "LATIN CAPITAL LETTER Q" on the left and `U+0051` on the right.
  - The glyph is drawn at 440 px, so each font pixel is a crisp 40 px square.
  - **Metric bands** separated by 1 px rules, labelled on the left with values on the right: `ASCENDER / CAP HEIGHT 400`, `X-HEIGHT 300`, `BASELINE 0`, `DESCENDER -100`. Row heights are multiples of 40 (237/80/240/80).
  - Two small **6 × 22 rounded "pill" markers** at the left edge, like caret or registration marks.
- **Glyph grid**: glyphs at 44 px, 11 columns, with a segment header in 11 px mud caps above each group (BASIC LATIN, EXTENDED LATIN, CYRILLIC, GREEK, PUNCTUATION, SYMBOLS, NUMERALS, MATH, CURRENCY, GRAPHICAL). Small caps appear as a second A to Z row. The Graphical set shows block elements, shades, arrows, triangles, suits, notes and box-drawing pieces.
  - **Hover**: four amber **corner brackets**, each 12 px long and 2 px thick, drawn with conic gradients. The native title tooltip shows `U+0041`.
  - **Selected**: a solid amber cell with the glyph in carbon.
  - Clicking a glyph updates the specimen card.
- **Apollo Guidance Computer block diagram** (`#apollo1`): box-drawing ASCII art rotated 90°, sitting in the left margin behind the specimen card. It has boxes like CLOCK / OSCILLATOR AND TIMING PULSES / SEQUENCE GENERATOR / ARITHMETIC UNIT / MEMORY ADDRESS REGISTER. It uses `►◄▲▼` arrowheads and `┼` junctions, pseudo-3D memory stacks drawn with `/` diagonals and `++++` fill, and overbar notation (`S` over `───`).

### 1.9 Code and engineering drawing (`#code`, dark)
- **Mercury capsule cutaway** (`mercury-diagram.svg`, 1136 × 1523): 393 paths, all 1 px `#6C6C58`.
  - Hidden internal parts are **dashed `6 6`** (88 paths), with `stroke-linecap: square`.
  - A horizontal **centreline axis** runs across the whole width.
  - More than 30 **callouts**: HF COMMUNICATIONS REC/TRANS, ATTITUDE GYRO, 250 VA INVERTER, ROLL CONTROL VALVE - MANUAL, S BAND BEACON, TORUS TANK - AUTO CONTROL, 3000 WATT/HR BATTERY, RETROGRADE PACKAGE, BELL CRANK PITCH CONTROL, ACCELEROMETER, COMMAND REG (A)&(B) RELAY UNITS, RATE GYRO ×3, UHF RESCUE COMMUNICATIONS REC/TRANS, and more.
  - A credit **"DIAGRAM REDRAWN FROM: NASA.GOV/IMAGE-DETAIL/MERCURY1"** is set at an angle along the hull edge, with small perpendicular tick marks.
  - A **struck-through "CONFIDENTIAL"** crossed by a thick 6 px bar, and a caption "Figure 49".
- **Scroll countdown numeral** `#sensor`: 220 px amber digits showing `100 - scroll%`, zero-padded (`14`, `08`). They overlay the diagram like an altimeter.
- **Rust code** (SVG): keywords in `#FFA133`, types in `#FFC24D`, identifiers in `#C0C0C0`, punctuation in `#8E8E8E`, so syntax colouring is monochrome plus amber. The content is `use crate::biometrics::Monitor; … pub struct LifeSupport<'t> … const UPSILON: u64 = 41;`.
- **SQL**: `SELECT` and `FROM` in amber; `mission_name, launch_date, spacecraft_name, astronaut_count … gamma_bloc_missions;` in grey.
- **Mission report printout**: **continuous tractor-feed paper** in cement grey.
  - Sprocket holes are 22 px circles every 66 px, centred 33 px from each edge, with 1 px carbon perforation rules 66 px in from each side. Everything is on the 11 px grid.
  - It is an editable `<pre>` at 13.75 px containing a Markdown-ish document: `# MISSION REPORT 75X9389 … 1/17`, `SUBJECT: ANOMALOUS ENERGY SIGNATURES FROM KERBEROS 5`, `EARTH DATE: NOV 20, 2057`, `---`, `## OVERVIEW`, `### OBSERVATIONS`.
  - It contains a **box-drawing table** with a right-aligned numeric column and chemical subscripts (CH₄, C₂H₆, H₂O).
- Comment: "ADD A RETRO FLAVOR TO YOUR CODE AND TECHNICAL DOCUMENTATION".

### 1.10 Deparkanoid (p5.js canvas, 880 × 600; hidden on mobile)
- The bricks are 80 × 22 px (22 = 2 × 11) and are arranged in the **shape of a space invader**.
- Each brick has a 4 px carbon stroke and a **3 px bevel**: white at 80 alpha on the top and left, `#333` on the bottom and right. This is a classic Win95/NES bevel.
- Brick fills are `--dark` for the "eyes" and `--smoke` for the body.
- The ball is an amber pixel PNG. The paddle is grey with **amber end caps**.
- `SCORE 00000` is zero-padded to 5 digits at 16 px, top left.
- The win screen shows `YOU WIN` at 88 px in amber and a **blinking** (500 ms) `RIGHT CLICK TO INSERT COIN` in pumpkin.
- Controls: mouse, touch, or arrow keys. Easter eggs: Alt+≥ turns on autoplay, Alt+÷ turns on sound (three mp3 blips).
- Comment: "AND OF COURSE, DEPARTURE MONO IS PERFECT FOR YOUR 8-BIT VIDEO GAME OR ASCII ART".

### 1.11 Footer
- `DEPARTURE MONO` at 44 px, `--soot` on an `--ash #8e8e8e` block. This mirrors the header's inverse title block.
- A **callout** with a 7 px `--mud` left bar and 11/14 px text: "DEPARTURE MONO IS A MONOSPACED PIXEL FONT BY HELENA ZHANG, LICENSED UNDER THE SIL OFL. WEBSITE BUILT BY TOBIAS FRIED." The links are underlined and turn amber on hover.

---

## 2. Palette

The tokens are defined on `:root`. They are named after materials, which is a nice touch in itself.

| Token | Hex | Where it is used |
|---|---|---|
| `--enamel` | `#eeeeee` | Light page background (act 1); text on dark cards (boarding pass, badge) |
| `--aluminum` | `#cccccc` | Header title highlight block |
| `--cement` | `#c0c0c0` | **Dark-mode ink** (the main foreground); specimen card; printout paper; code identifiers |
| `--ash` | `#8e8e8e` | Footer title block; code punctuation |
| `--smoke` | `#666666` | HUD ink in the light state; brick fill |
| `--dark` | `#444444` | **Light-mode ink**; boarding pass and badge card fill; dark bricks |
| `--soot` | `#333333` | Title glyphs; paperclip; newsprint ink |
| `--carbon` | `#222222` | **Dark background** (main, footer); specimen ink; printout ink; `::selection` background |
| `--black` | `#141414` | Defined, but rarely visible |
| `--clay` | `#6c6c58` | **Olive-grey line work**: every schematic, box-drawing diagram and comment on light |
| `--mud` | `#8a8a6f` | Lighter olive for comments, segment headers and callout bars on dark |
| `--foam` | `#bccabb` | Pale sage **highlight**: menu hover, blinking row, focus |
| `--amber` (= `--accent`) | `#ffa133` | **The only accent** |
| `--pumpkin` | `#e47b1a` | Deeper amber; game text only |
| `--flux` | `#c8be50` | Olive yellow; defined but not visibly used |
| SVG-only colours | `#8AA788` sage (bag tag, text highlight at 50%), `#D9C9B6` newsprint tan, `#B5B58B` khaki, `#999999` badge dither, `#FFC24D` light amber (code types), `#FFFFFF` paper | Props |

**Contrast strategy (measured WCAG ratios):**
- Content text has strong contrast: `#444` on `#eee` is 8.4:1 and `#c0c0c0` on `#222` is 8.7:1.
- **Background schematics are deliberately recessive**: clay on carbon is 2.97:1 and clay on enamel is 4.6:1. This gives the "blueprint underlay" feeling. Diagrams never compete with content.
- Two grey ramps plus one chromatic temperature. The greys are perfectly neutral. The warmth comes from an olive-tinted line colour (clay and mud) plus sage and tan props, which feels like aged paper and military or aerospace documentation.
- **Accent budget**: amber is used only for (a) *state*: the selected glyph fill, hover corner brackets and link hover; (b) *the "powered-on" instrument*: the whole HUD after the transition, the sensor numeral, and the game ball and paddle caps; (c) *syntax keywords*. It never appears as decoration in act 1. The page stays greyscale until the craft "powers on".
- **Inversion as emphasis**: the title, footer title, AUTO label, VID INPUT keys, the stepped DEPARTURE/MONO tag, the selected glyph, and `::selection` are all solid blocks with knocked-out or reversed text instead of bold. There is no bold weight anywhere.
- Section tone changes by **swapping `--fg` and `--bg` on the container** (`main { --fg: cement; --bg: carbon }`).

---

## 3. Typography system

- **One font, one weight**: Departure Mono Regular, with UPM 550. One font pixel is 50 units, so **at 11 px one font pixel equals one CSS pixel**. The character cell is 7 × 11 px: advance 350, cap height 400 (8 px), x-height 300 (6 px), descender 100 (2 px), ascent 550, and 150 below the baseline. `:root { font-size: 11px }`.
- **Sizes are integer or quarter multiples of 11**, so glyph pixels land on whole device pixels:

  | Size | Multiple of 11 | Used for |
  |---|---|---|
  | 11 px | 1× | labels, comments, metadata |
  | 13.75 px | 1.25× | printout |
  | 16.5 px | 1.5× | body, menu, letter |
  | 22 px | 2× | lead paragraph, HUD tape |
  | 33 px | 3× | mobile h2, game prompt |
  | 44 px | 4× | departure board, glyph grid, footer h2 |
  | 55 px | 5× | subhead sample |
  | 88 px | 8× | title, "YOU WIN" |
  | 121 px | 11× | hero sample |
  | 220 px | 20× | sensor numeral |
  | 440 px | 40× | specimen glyph |

  The only exceptions are 14 and 16 px inside the HUD and the game (SVG and canvas contexts).
- **Line height**: `normal` gives 14 px at 11 px (a 3 px gap). Body at 16.5 px uses 24.75 px (1.5×). Display text uses **1.0**, so highlight blocks hug the pixels. The departure board uses 1.27, box drawing 1.2 (the lines must touch), and the HUD 1.1.
- **Tracking is quantised to font pixels**: -11 px at 121 px and -5 px at 55 px each remove exactly one font pixel. Otherwise tracking is 0 and the monospace grid is sacred. **Wide letterspacing is done with literal spaces**: `D E P A R T U R E`, `A U T O`, `K U I P E R`, `X X - 2 4 8 8`. The spacing stays on the character grid.
- **Hierarchy through size and inversion, never weight**:
  - *Labels, micro-copy*: 11 px, ALL CAPS, clay or mud, often in a 2 to 4 column spec row (`DEPARTURE MONO   55PX   -5PX TRACK`) or placed above a value (boarding pass).
  - *Body*: 16.5 px, sentence case, full-strength ink.
  - *Display*: 44 to 121 px, ALL CAPS, sometimes on a grey or inverse block.
  - *Data*: zero-padded and fixed width (`00000`, `003`, `090`, `1186`, `15:20:30:85`), right-aligned in columns, with `°F/°C` units and `***` masking.
- **Casing**: caps for anything "system" (labels, headers, codes), sentence case for anything "human" (the letter, receipt items, the badge name). Small caps via `font-variant: small-caps` appear in the glyph set and in "LBS".
- **Symbols used as UI**: `↓ > ♥` as menu icons; `░` as a comment gutter; `▶▶`, `→` and `↑` as flow and sort indicators; `◀` as a pointer; `┌─ ─┐` as group frames; `│├┼┤─` as tables; `►◄▲▼` as arrowheads; subscripts and superscripts (`N₂`, `H₂O`, `FPS²`, `CH₄`).
- Rendering: `-webkit-font-smoothing: antialiased`, `text-rendering: optimizeLegibility`, `font-synthesis: none`. There is no `image-rendering: pixelated` anywhere: crispness comes entirely from **grid-aligned sizing and vector geometry**.

---

## 4. Line work

- **Weights**:
  - **1 px** is the default for all schematics: planet, highlighter, Mercury cutaway, HUD inner grid, specimen metric rules and box-drawing strokes.
  - **2 px** is for primary instrument outlines: the HUD reticle circle, chevron brackets, the R/P/Y box, the dial arc, the corner-bracket markers, the glyph hover brackets and the paperclip.
  - **Heavy accents**: a 7 px callout bar, the 6 px CONFIDENTIAL strike bar, the 4 px brick stroke.
- **Dashes**: `6 6` (on = off) for hidden or internal lines in the engineering drawing. Boarding pass perforations are short dash segments. There are no dotted lines; dither patterns do that job.
- **Caps and joins**: `stroke-linecap: square` for technical linework; `stroke-linejoin: round` only on the "object" drawings (planet, highlighter).
- **Corners**: square almost everywhere. The exceptions are all **physical-object cues**:
  - a 48 px **chamfer** on the specimen card (via `clip-path`)
  - a chamfer on the bag tag
  - a small radius on the badge and the pill markers (2 px radius)
  - **torn edges** (irregular polylines) on the receipt and the clipping
  - **punched holes** on the letter, the printout and the bag tag
- **Arrowheads**: always **solid filled triangles** or glyph arrows (`◀`, `►◄▲▼`, `→`, `▶▶`). Callout leader lines have **no arrowheads and no dots**; they simply end at the part.
- **Leader-line construction** (Mercury): a straight diagonal runs from the part to the *start* of a horizontal **underline shelf**. The label sits on the shelf, and the shelf overhangs the text by a few pixels on each end. Multi-line labels (e.g. "HF RESCUE COMMUNICATIONS / REC/TRANS") hang a second, shorter shelf off the first through a vertical drop, forming a `├` bracket. Leaders fan out roughly in parallel and never cross the labels.
- **Diagram vocabulary**: centrelines through the axis of symmetry; dashed hidden lines; circle-plus-ellipse "globe" grids clipped by a mask; crosshairs with a central gap; **corner brackets** (four L-shapes) as selection or focus frames; box-drawing top-only group frames `┌── LABEL ──┐`; tick marks perpendicular to an edge; rotated credit text running along an edge; and **dither fields** (ordered or random 1-bit noise) as stand-ins for photos, barcodes and QR codes.
- **Construction honesty**: every prop is drawn as outlines of its real faces (the highlighter has separate bevel faces; the capsule has overlapping hidden and visible parts). There is no shading, gradient or shadow anywhere, except the game-brick bevel and the HUD fade mask.

---

## 5. Little details

1. `v1.500` version superscript beside the title.
2. The title glitch with look-alike glyph substitutions and occasional 30 ms flicker bursts.
3. Menu icons made from glyphs: `↓` `>` `♥`.
4. `░` dithered comment gutters in every section: "DEPARTURE MONO IS A MONOSPACED PIXEL FONT WITH A LO-FI TECHNICAL VIBE", "IT'S GREAT FOR WORKING WITH TABULAR DATA", "INCLUDED: BOX DRAWING CHARACTERS…", "ADD A RETRO FLAVOR…", "AND OF COURSE…".
5. "Dear Dr. **Kerning**", "E. KERNING", Mission Director: typography puns throughout.
6. "Farnsworth Institute, NEW NEW YORK, NY 10024" and "Professor Hubert J. Farnsworth" (Futurama).
7. "Double R Diner / Cherry pie / Damn good coffee" (Twin Peaks). A 2049 timestamp and masked IDs `***3047`, `***6832`, and `#E6D598EF`.
8. "KÁRMÁN SPACELINES" (the Kármán line), with diacritics shown off in the name.
9. Serial `XX-2488319840`, repeated on the pass, the bag tag (letterspaced) and the barcode.
10. Flight `XX789`, gate `97A`, seat `4C`, boarding `11:01`, `****** FIRST CLASS ******`.
11. Vertical fine print "DO NOT EXPOSE TO EXCESSIVE HEAT OR DIRECT SUNLIGHT".
12. Bag tag: `WEIGHT 45 LBS`, `ORIG. FLIGHT XX789→`, rotated `DESTINATION`.
13. Badge: `TIER 1 ACCESS`, room or ID `207`, white chip square, lanyard slot.
14. Destinations: KBA Kuiper Alpha, EUR Europa 1, MAR Mars Landing, NTK New Tokyo, ZMB Zvezda Moonbase.
15. The blinking "your flight" row (XX123).
16. The `↑` sort caret in the table header.
17. Type spec headers `121PX  -11PX TRACK`.
18. HUD: `EARTH TIME` with centiseconds ×2, `INT 68°F 20°C / EXT -456°F -271°C` with jittering temperature, `HE TANK P`, `N₂ TANK P`, `OMS`, `APU`, `FUEL %`, `H₂O %`, `HYDRAULIC QTY %`, `OS AUDIO MASTER VOLUME`, `VID INPUT A B C`, `ACCEL FPS²`, `R/P/Y` attitude, `AUTO` mode badge.
19. The HUD **GLYPHS tape centred on 1186**, the real glyph count of the font (loaded at runtime from the OTF).
20. HUD spec plate: `INITIATED: 2022`, `LAUNCHED: 2024`, `CLASSIFICATION: PIXEL MONOSPACED`, `LICENSE: SIL OFL`.
21. TCP header diagram with an RFC-style bit ruler.
22. Keyboard with `s p a c e` letterspaced on the spacebar.
23. Apollo Guidance Computer block diagram hidden in the margin with overbar register notation.
24. Specimen metrics `400 / 300 / 0 / -100` in font units, plus Unicode names and code points.
25. `U+XXXX` tooltips on every glyph.
26. Mercury cutaway credit "DIAGRAM REDRAWN FROM: NASA.GOV/IMAGE-DETAIL/MERCURY1", `~~CONFIDENTIAL~~`, "Figure 49".
27. The scroll countdown numeral (100 down to 00) in amber.
28. Rust `LifeSupport<'t>`, `eva_remaining`, `const UPSILON: u64 = 41`, and SQL `gamma_bloc_missions`.
29. Mission report `75X9389`, page `1/17`, `KERBEROS 5`, `NOV 20, 2057`, a chemical table with Unicode subscripts.
30. Tractor-feed sprocket holes and perforation lines on the printout; punched binder holes on the letter.
31. Game: invader-shaped brick wall, `SCORE 00000`, `YOU WIN`, blinking `RIGHT CLICK TO INSERT COIN`, and hidden Alt-key autoplay and sound toggles.
32. Footer credits in a 7 px bar callout.
33. `cursor: cell` (the plus cursor) over the glyph lab.
34. Inverted `::selection` colours that differ per section.

---

## 6. Motion and interaction

| Element | Behaviour | Timing / easing |
|---|---|---|
| Title | random glyph substitution glitch | 400–2000 ms random; 30 ms flicker frames |
| Departure row | background on/off blink | `1s steps(1) infinite` (hard cut) |
| Menu link hover | foam background block | 75 ms `--flick` = `cubic-bezier(.36,2.09,.07,-1.52)` (overshoot; feels near-instant, "electrical") |
| Links | colour goes to amber | 150 ms `--flick` |
| Badge hover | slides out 100 px, rotates 3° | 0.6 s `cubic-bezier(1,.05,.48,.99)` (slow start, hard landing) |
| Ephemera hover | pass, receipt and tag fan apart | 1 s, same easing |
| HUD section | sticky SVG; theme flip light to dark via `--t` and LCH `color-mix` | 0.4 s colour transition, triggered by an IntersectionObserver |
| HUD clock | EARTH TIME ticks | every animation frame |
| HUD telemetry | EXT temp, R/P/Y random walk, ACCEL tape offset | 1 Hz; tape moves with 500 ms ease-out |
| HUD radar | starfield squares spawn at 3%/frame, accelerate out, grow | per frame, with a 500 ms ease-out transform |
| Glyph item | hover: amber corner brackets; click: amber fill and specimen update; keyboard: tabindex plus Enter | instant brackets; 150 ms flick fill |
| Specimen | sticky while the list scrolls | — |
| Sensor numeral | 100 − scroll% | on scroll |
| Type samples, report | `contenteditable`; focus inverts the sample | — |
| Game | p5.js, 60 fps, mouse, touch or keys | "insert coin" blinks at 500 ms |

The motion philosophy: **no fades, no soft easing on UI state**. State changes are either instant, stepped (blink), or snap with an overshoot curve so fast it reads as a switch. The only "smooth" motion is physical (props sliding, instrument tapes settling) or the one big day-to-night colour transition.

The cursor is `cell` over the glyph lab and `pointer` on links; everywhere else it is the default. Selection colours are inverted blocks.

---

## 7. Reusable design rules and primitives for a pixel UI toolkit

1. **11-px unit grid**: every size, spacing and stroke position is a multiple of a base "font pixel" (11 px at 1×). The toolkit should expose `unit(n)` and snap all layout to it. Text sizes come only in integer or quarter multiples.
2. **Inverse label block**: text on a solid block that hugs the glyph pixels (line-height 1), used for titles, mode badges (`A U T O`) and selected items. This replaces bold.
3. **Stepped inverse tag**: a two-line inverse label whose second line is narrower, forming a folder-tab or plate silhouette (`DEPARTURE / MONO`).
4. **Top-bracket group frame** `┌──── LABEL ────┐`: a hairline top edge with short corner drops and a centred caps label that breaks the line. It has no side or bottom borders.
5. **Corner-bracket focus frame**: four L-shaped marks (length ≈ 1 unit, 2 px thick) at the corners of a cell. Used for hover or focus (accent) and for "current value" on tapes.
6. **Bar gauge with readout**: a 1 px outline track with a solid fill inset and a zero-padded numeric readout to the right in 1× text. Stack them at a 24 px pitch under a caps label (with sub/superscripts).
7. **Vertical value tape**: a column of numbers with a gradient fade mask top and bottom, a centred current value marked by corner brackets or a solid `◀` pointer, and eased re-centring.
8. **Reticle**: a 2 px circle, a masked 1 px lat/long ellipse grid, cardinal and diagonal ticks outside the ring, a crosshair with a central gap plus a small centre plus, and chevron side brackets with a centre tick.
9. **Semicircle dial**: a 2 px 180° arc with a 1 px needle from the hub.
10. **Leader-line callout**: a diagonal hairline from a target point to the end of a horizontal underline shelf. The caps label sits on the shelf, which overhangs the text by about 2 px. There is no arrowhead. Multi-line labels use a `├`-bracket drop to a second shelf.
11. **Technical drawing strokes**: 1 px solid for visible edges, `6 6` dashed for hidden edges, a long centreline through the symmetry axis, square caps, all in a low-contrast line colour.
12. **Bit / tick ruler**: a two-row index (tens over ones), then a tick row (`├─┴─┴─┼─┤`) with major ticks every 8 or 16, sitting directly on a table.
13. **Box-drawing table**: `│ ├─┼─┤` borders, a header row with a sort caret `↑`, left-aligned text columns, right-aligned zero-padded numeric columns, and an optional blinking highlighted row.
14. **Spec row / key-value plate**: 2 to 5 columns of `KEY:` over `value` (or `LABEL  VALUE  UNIT` inline) in 1× caps, used as footers of instruments and headers of samples.
15. **Field label over value**: a tiny caps label (1×) directly above a large value (2× to 4×), the boarding-pass layout. Includes an arrow joiner between paired values (`LAX ▶▶ KBA`).
16. **Margin comment with dither gutter**: a column of `░` (or a 1-bit checker) as a vertical bar beside 1× caps annotation text in the line colour.
17. **Dither fill**: 1-bit ordered or random noise fields used as image, barcode, QR or gradient stand-ins, with a density ramp at the edges.
18. **Chamfered panel**: a rectangle with one 45° cut corner (about 4 units), for cards and tags. Pair it with 1 px internal metric rules that carry a label on the left and a value on the right.
19. **Perforation / sprocket strip**: punched circles (2 units in diameter at a 6-unit pitch, 3 units from the edge) plus a 1 px perforation rule, for "printout" and "paper" containers. Also dashed tear lines between a ticket body and its stub.
20. **Torn edge**: an irregular polyline bottom edge for paper props (receipt, clipping).
21. **Glyph cell grid**: fixed-size cells in N columns with a caps segment header. Hover shows corner brackets, selection is an accent fill with inverted glyph, and each cell has a code-point tooltip.
22. **Glyph / pixel blow-up**: a character rendered at 40× so each pixel is a hard square, overlaid on horizontal metric bands (ascender, x-height, baseline, descender) with values.
23. **Letterspaced-by-space headings**: spacing done with whole character cells (`D E P A R T U R E`), flanked by tick-bar runs `|||||||`, so everything stays on the monospace grid.
24. **Telemetry text widgets**: a live clock `HH:MM:SS:CC`, temperatures padded to fixed width with units, attitude `R 003 / P 031 / Y 133`, and 1 Hz jitter for liveliness.
25. **Theme flip as an event**: two palettes (paper and terminal) with the accent held back until a "power-on" moment, interpolated in LCH over about 0.4 s. After that, the instruments render in the accent.
26. **Win95 / NES bevel block**: a 4 px dark outline with a 3 px light top-left and a dark bottom-right inset, for game-like buttons and bricks. Use it sparingly, for "toy" moments only.
27. **Rotated edge text with ticks**: a small caps credit or annotation set along an arbitrary edge angle, with perpendicular tick marks crossing the edge.
28. **Stepped motion**: blink with `steps(1)`, flicker bursts of about 30 ms, and overshoot-snap easing (`cubic-bezier(.36,2.09,.07,-1.52)`) for state changes. No soft fades on UI state.
