# graticule — design

The toolkit draws instrument panels: the screens of test equipment, avionics
and early embedded GUIs, and the technical drawings that document them. This
file is the design language and the architecture that carries it. The demo in
`demo/` is its specification in pixels: when this text and a rendered page
disagree, the page is checked against the rules below and one of them is
fixed.

## 1. Principles

1. **A pixel is the unit.** The interface is laid out and drawn in virtual
   pixels and upscaled nearest-neighbour by the fork. Every position, length
   and font size is whole. A half pixel should be unrepresentable, not avoided.
2. **One face family, native sizes only.** Text is a pixel font at its native
   em or a whole multiple. Hierarchy comes from size, colour and inversion,
   never from a synthetic weight.
3. **Colour is a role.** A palette names what a colour is for (ground, ink,
   accent, alarm…); nothing outside a palette names a hex. A theme is a palette.
4. **Marks mean something.** Boxes are for things a hand would find on a
   panel: keys, lamps, the glass of a display. Regions are separated by
   shade and hairlines, groups by a named rule.
5. **Drop, don't cut.** A label that doesn't fit is left out; its tick or rule
   stays. A cut label reads as a different label.
6. **Redraw on events.** Nothing animates for its own sake. Motion moves in
   whole pixels, on the cadence the data changes.

## 2. The grid

- The fork's `PixelScaleMode::Auto(2)`: one virtual pixel is about two logical
  pixels on every display. `graticule::settings()` sets it with the fonts and
  antialiasing off, and tests run with the same settings.
- Spacing steps (`px`): `HAIR` 1, `TIGHT` 2, `GAP` 4, `WIDE` 8, `FAR` 16.
  Everything else is derived from a face's cell.
- Odd sizes where a centre is needed: icons, lamps that sit on a centre line,
  circles drawn around a pixel (`2r + 1` across).
- Centring rounds down (`px::centre`), so an odd leftover always falls on the
  same side and two things centred in the same space stay aligned.
- Drawing goes through `draw::Pen` with integer coordinates. Shapes are
  rasterized by the toolkit (`draw::shape`) and handed to the renderer as
  axis-aligned rectangles, so they light the same pixels on wgpu, tiny-skia and
  WebGL.

## 3. Type

| Face | Font | Size | Cell | Line | Use |
|---|---|---|---|---|---|
| `Face::BODY` | Departure Mono Tight | 11 | 6 × 12 | 12 | labels, readings, keys |
| `Face::PROSE` | Departure Mono | 11 | 7 × 14 | 14 | running text |
| `Face::DISPLAY` | Departure Mono | 22 | 14 × 24 | 24 | readouts, tapes |
| `Face::HERO` | Departure Mono | 33 | 21 × 36 | 36 | one number per screen |

- Departure Mono draws a 5 px glyph body in a 7 px cell; Tight narrows the
  cell to 6 px. At a 12 px line the capitals have two pixels above them and
  descenders two below.
- cosmic-text puts the baseline at `(line + ascent − descent) / 2`, so a face's
  line height must have the parity that makes it whole. `Face::new` checks it
  at compile time.
- Box-drawing tables use `Face::with_line_box`, whose line is the font's own
  box height, so vertical rules join.
- Emphasis is an inverse block (`widget::inverse`): the accent behind text in
  `on_accent`. Wide tracking is done with whole cells (`face::spaced`).
- Only characters the face has. A fallback glyph has a foreign advance and puts
  everything after it off the grid.
- The type lab (TYPE page) compares candidate faces in use; see §8.

## 4. Palette

| Role | Use |
|---|---|
| `void` | behind everything; the glass of a display |
| `ground` | a panel's face |
| `raised`, `hover` | a control's face, and under the cursor |
| `edge` | the one hairline colour of borders and dividers |
| `ink`, `muted`, `faint` | text and marks, labels, engraving |
| `line` | schematic line work, recessive against the ground |
| `accent`, `on_accent` | state: selected, engaged, powered; text knocked out of it |
| `highlight` | passive emphasis |
| `live` | live data |
| `caution`, `alarm` | worth a look; wrong |

Themes: TERMINAL (grey on charcoal, amber accent), PAPER (graphite on
off-white, pumpkin accent), PHOSPHOR (green tube), AMBER (amber tube), LCD
(reflective grey-green). A control changes state by stepping along the
surfaces (`ground → raised → hover`), not by taking a new colour. Derived
shades go through `theme::mix` and `theme::dim`, which round to 8 bits.

## 5. Line work and marks

- 1 px for rules, borders, schematics and traces; 2 px (`Pen::ring`,
  `Pen::thick_arc`) for the primary outline of an instrument.
- Heavy rings are thickened inwards along each pixel's major axis, the way a
  pixel artist draws them: solid at every bearing, even across every row.
- Dashes (`Dash`): `HIDDEN` 6/6 for hidden edges, `SHORT`, `DOTTED`, `SPARSE`.
  A dashed line is counted from the end that must look right.
- 1-bit patterns (`Pattern`: checker, 4 × 4 Bayer, grid, hatch, rows,
  columns) stand in for tone. They are anchored to what they belong to, so a
  panned pattern moves with its world instead of shimmering.
- Arrowheads are solid 45° triangles. Callouts are a diagonal leader to the
  end of a shelf the label sits on, with no arrowhead. Selection and focus are
  corner brackets, not boxes.
- Icons (`icon`) are 7 × 7 sprites that fit the capitals of the body face.

## 6. Composition

- Zones, not cards: the status bar, the page and the soft keys are regions of
  ground separated by hairlines.
- Groups are a rule broken by the group's name, `┌── NAME ──┐`, with no sides
  or bottom (`widget::group`).
- A field is its name over its value; a reading is both on one line.
- Soft keys along the bottom select the page; the key of the page in use is
  lit.
- Built-in widgets are wrapped (`widget::text_input`, `pick_list`, `toggler`,
  `scroll`) so their heights are sums of tokens; check boxes and radios are
  the toolkit's own, drawn with icons.

## 7. Motion

- The demo ticks at 15 Hz while an animated page is shown and slower
  otherwise. Instruments redraw from state; nothing tweens.
- Tapes scroll in whole pixels: a value between labels puts them that fraction
  of the pitch from the centre, rounded.
- Hover redraws only when the hovered item changes.

## 8. Architecture

```
crates/graticule/   the library (depends on iced_widget only)
  face, fonts       faces and the font files (OFL)
  px                spacing steps, snapping helpers
  theme, style      Theme, Palette, styles and catalogs for built-ins
  draw              Pen, shapes, sprites, patterns, annotation marks
  icon              pixel icons
  widget            controls and indicators
  instrument        canvas instruments: tape, dial, ...
demo/               the showcase (native and wasm)
fonts/, tools/      font sources and the scripts that build the shipped fonts
web/                host page and build script for the browser
```

- The library depends on `iced_widget` alone, so windowing, renderers and
  executors stay the application's choice. It pins the `0.14-pixel-scale` fork.
- Widgets follow iced's catalog pattern: each has a `Style`, a `Catalog` and a
  `StyleFn` class, implemented for the graticule `Theme`.
- Instruments are canvas programs with builder APIs that turn into `Element`s
  filling their space unless sized.
- The demo's `render` example draws every page in every theme headless to
  PNG; `--lab` draws the type lab once per candidate.

## 9. Open questions

- **Fork branch.** Applications pin `0.14-pixel-scale` or `0.14-mobile`. A
  toolkit both can use needs one branch with the pixel-scale work, the mobile
  fixes (whole-pixel translation, `opaque` touch capture, same-turn messages,
  clipboard) and the text fixes below.
- **Text in the fork.** Only a paragraph's origin is rounded; centred or
  right-aligned lines get fractional offsets from cosmic-text. Rounding each
  line's offset under `crisp` would make in-paragraph alignment safe.
- **Type.** Departure Tight is the body face; smaller sizes (a 5 × 8 "small",
  a caps-and-figures "micro") and a hand-corrected bold are open. The lab is
  where they are decided.
- **Publishing.** crates.io needs the fork published or upstreamed.
