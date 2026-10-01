# quadrille — design

quadrille is the ground a pixel-style application is built on: an interface
laid out in whole pixels, set in pixel fonts, coloured by role and drawn
without antialiasing. It knows nothing about what an application is about.
This file is its scope, the design language it carries and the architecture
that carries it. The demo in `demo/` is its specification in pixels: when this
text and a rendered page disagree, the page is checked against the rules below
and one of them is fixed.

## 1. Scope

Code lives in one of three places, and depends only downwards:

```
applications   their subjects: a radio's waterfall, a chat's timeline, a console's instruments
kits           a subject two applications share, cut from one of them
quadrille      the grid, type, colour, drawing, scales and the controls of any interface
the fork       virtual pixels, nearest-neighbour upscaling, crisp snapping
```

1. **Dependencies point down.** The toolkit never names a kit, an
   application, a unit or a subject.
2. **The toolkit test.** Something belongs in the toolkit when it would make
   sense, unchanged, in a chat client, a radio and a transit map. The look
   belongs to the toolkit: lamps, hairlines, phosphor. The subject belongs to
   the application. So the toolkit names things by what they do, a
   `button`, a `tab`, a `segmented` control, and leaves the flavour to the
   style.
3. **Born narrow, promoted on second use.** Code starts in the application
   that needs it. It moves to a kit when a second application wants it, and
   into the toolkit when a second subject wants it unchanged. A raster drawn
   pixel for pixel passed: a radio's waterfall and a chat's avatars both built
   one by hand.
4. **A kit is cut from a working application** when a second working
   application needs the same subject, and is named for that subject (radio,
   flight, transit), never for a genre such as "instruments". A kit that
   cannot name its two users does not exist yet.
5. **Kits and applications use only the public API.** Anything they need
   from the toolkit becomes public; that is how the toolkit proves it is
   enough. They extend it without editing it: new verbs for `Pen` come as an
   extension trait, new widgets follow the catalog pattern, colours of their
   own are derived from the palette's roles.

There are no kits yet. The demo's console owns its instruments (dials, tapes,
a scope screen), its drafting (sheets, notes, dimensions, wiring, added to
`Pen` by a `Drafting` trait) and its charts, built the way any application
would build them.

## 2. Principles

1. **A pixel is the unit.** The interface is laid out and drawn in virtual
   pixels and upscaled nearest-neighbour by the fork. Every position, length
   and font size is whole. A half pixel should be unrepresentable, not avoided.
2. **One face family, native sizes only.** Text is a pixel font at its native
   em or a whole multiple. Hierarchy comes from size, colour and inversion,
   never from a synthetic weight.
3. **Colour is a role.** A palette names what a colour is for (ground, ink,
   accent, alarm…); nothing outside a palette names a hex. A theme is a palette.
4. **Marks mean something.** Boxes are for things a hand would find: buttons,
   lamps, the glass of a display. Regions are separated by shade and
   hairlines, groups by a named rule.
5. **Drop, don't cut.** A label that doesn't fit is left out; its tick or rule
   stays. A cut label reads as a different label.
6. **Redraw on events.** Nothing animates for its own sake. Motion moves in
   whole pixels, on the cadence the data changes.

## 3. The grid

- The fork's `PixelScaleMode::Auto(2)`: one virtual pixel is about two logical
  pixels on every display. `quadrille::settings()` sets it with the fonts and
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

## 4. Type

| Face | Font | Size | Cell | Line | Use |
|---|---|---|---|---|---|
| `Face::BODY` | Departure Mono Tight | 11 | 6 × 12 | 12 | labels, readings, buttons |
| `Face::PROSE` | Departure Mono | 11 | 7 × 14 | 14 | running text |
| `Face::DISPLAY` | Departure Mono | 22 | 14 × 24 | 24 | readouts |
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
  everything after it off the grid; a character no face has falls back to
  Departure Mono's missing glyph, the same on every machine.
- A face is a font and its metrics, so an application can bring its own pixel
  font. The type lab (TYPE page) compares candidate faces in use; see §9.

## 5. Palette

| Role | Use |
|---|---|
| `void` | behind everything; the glass of a display |
| `ground` | a panel's face |
| `raised`, `hover` | a control's face, and under the cursor |
| `edge` | the one hairline colour of borders and dividers |
| `ink`, `muted`, `faint` | text and marks, labels, engraving |
| `line` | line work: grids, scales and drawings, recessive against the ground |
| `accent`, `on_accent` | state: selected, engaged, powered; text knocked out of it |
| `highlight` | passive emphasis |
| `live` | live data, and anything on |
| `caution`, `alarm` | worth a look; wrong |

Themes: TERMINAL (grey on charcoal, amber accent), PAPER (graphite on
off-white, pumpkin accent), PHOSPHOR (green tube), AMBER (amber tube), LCD
(reflective grey-green). A control changes state by stepping along the
surfaces (`ground → raised → hover`), not by taking a new colour. Derived
shades go through `theme::mix` and `theme::dim`, which round to 8 bits. A level
drawn as a tone takes the steps of `theme::Ramp`, which runs from the void out
through the palette's signal colours, so a heat map is in the theme's colours
on every theme.

## 6. Line work and marks

- 1 px for rules, borders, line work and traces; 2 px (`Pen::ring`,
  `Pen::thick_arc`) for a primary outline.
- Heavy rings are thickened inwards along each pixel's major axis, the way a
  pixel artist draws them: solid at every bearing, even across every row.
- Dashes (`Dash`): `HIDDEN` 6/6 for hidden edges, `SHORT`, `DOTTED`, `SPARSE`.
  A dashed line is counted from the end that must look right.
- 1-bit patterns (`Pattern`: checker, 4 × 4 Bayer, grid, hatch, rows,
  columns) stand in for tone. They are anchored to what they belong to, so a
  panned pattern moves with its world instead of shimmering.
- A picture computed pixel by pixel is a `draw::Raster`, drawn one of its
  pixels to one pixel, unfiltered. Tone between two of its colours is
  dithered with a pattern, never blended.
- Arrowheads are solid 45° triangles. Selection and focus are corner
  brackets, not boxes.
- Icons (`icon`) are 7 × 7 sprites that fit the capitals of the body face.

## 7. Composition

- Zones, not cards: bars and pages are regions of ground separated by
  hairlines.
- Groups are a rule broken by the group's name, `┌── NAME ──┐`, with no sides
  or bottom (`widget::group`).
- A field is its name over its value; a reading is both on one line.
- Tabs select a page; the tab of the page in use is lit (`widget::tab`).
- Built-in widgets are wrapped (`widget::text_input`, `pick_list`, `toggler`,
  `scroll`) so their heights are sums of tokens; check boxes and radios are
  the toolkit's own, drawn with icons.
- Labels along a scale are placed by `scale::place` and `scale::row`, which
  keep the ends and drop what would collide.

## 8. Motion

- An application ticks on the cadence of its data. Widgets redraw from
  state; nothing tweens.
- What scrolls moves in whole pixels. A strip chart samples on a grid fixed
  in time, so it steps a whole column at a time; a tape puts a value between
  labels that fraction of the pitch from the centre, rounded.
- Hover redraws only when the hovered item changes.

## 9. Architecture

```
crates/quadrille/   the toolkit (depends on iced_widget only)
  px                spacing steps, snapping helpers
  face, fonts       faces and the font files (OFL)
  theme, style      Theme, Palette, Ramp, styles and catalogs for built-ins
  draw              Pen, shapes, polygons, rasters, sprites, patterns
  scale             the arithmetic of scales: steps, labels, columns
  canvas            Memo, and canvas_widget! for canvas programs
  icon              pixel icons
  widget            controls and indicators
demo/               the showcase: a console built on the toolkit (native and wasm)
  kit, lab          the toolkit itself: every widget, and the type lab
  hud, scope,
  drawing           the console's pages
  instrument,
  drafting, trend,
  thermal, attitude the console's own widgets
fonts/, tools/      font sources and the scripts that build the shipped fonts
web/                host page and build script for the browser
```

- The library depends on `iced_widget` alone (and `iced_core`, to turn on
  `crisp`), so windowing, renderers and executors stay the application's
  choice. It pins the `0.15-pixel-scale` fork, which follows iced's master.
- Widgets follow iced's catalog pattern: each has a `Style`, a `Catalog` and a
  `StyleFn` class, implemented for the quadrille `Theme`.
- A visualization is a canvas program behind a builder. `canvas_widget!`
  makes it a widget of its own size, and a `canvas::Memo` keeps what is slow
  to draw (a frame, a scale, a raster) until what it shows changes.
- The demo's `render` example draws every page in every theme headless to
  PNG; `--lab` draws the type lab once per candidate.

## 10. Open questions

- **Fork branch.** Applications on the fork's 0.14 branches
  (`0.14-pixel-scale`, `0.14-mobile`) need one branch with the pixel-scale
  work, the mobile fixes (whole-pixel translation, `opaque` touch capture,
  same-turn messages, clipboard) and the text fixes below.
- **Text in the fork.** Only a paragraph's origin is rounded; centred or
  right-aligned lines get fractional offsets from cosmic-text. Rounding each
  line's offset under `crisp` would make in-paragraph alignment safe.
- **Type.** Departure Tight is the body face; smaller sizes (a 5 × 8 "small",
  a caps-and-figures "micro") and a hand-corrected bold are open. The lab is
  where they are decided.
- **Colours of a subject.** A kit or an application derives the colours it
  adds (a map's land and sea) from the palette's roles. Overriding them per
  theme wants a typed extension on `Theme`, added when the first application
  needs one.
- **The console's spare options.** The instruments and drafting came whole
  from the toolkit, and carry options the console does not use. They are
  kept, and marked, until a kit is cut or they are trimmed.
- **Publishing.** crates.io needs the fork published or upstreamed.
