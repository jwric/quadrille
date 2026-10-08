# quadrille

Pixel-perfect type, drawing and widgets for [iced]: interfaces laid out in
whole pixels, set in pixel fonts and drawn without antialiasing.

It targets the [`0.15-pixel-scale` iced fork][fork], which lays an interface
out in virtual pixels and upscales it nearest-neighbour.

- **Grid**: every length is a whole number of virtual pixels, from a small
  spacing scale, and centring rounds the same way everywhere.
- **Type**: Departure Mono at its native sizes, with a tighter 6 × 12 cell for
  dense panels, or a pixel font of your own. Baselines land on whole pixels.
- **Colour**: a small palette named for roles, five themes, mixes rounded to
  8 bits, and a stepped ramp for drawing a level as a tone.
- **Drawing**: a `Pen` for canvases with integer coordinates only: lines,
  circles, heavy rings, arcs, ellipses, polygons, dashes, dither patterns,
  sprites, rasters, and text anchored by its baseline or capitals.
- **Visualization**: the arithmetic of scales (round steps, labels that give
  way rather than being cut, samples folded into columns), drawings kept
  until what they show changes, and a macro that makes a canvas program a
  widget.
- **Widgets**: buttons, tabs, segmented controls, check boxes, radios, lamps,
  bar gauges, knobs, groups, fields, and flat, square styles for every
  built-in iced widget.

quadrille knows nothing about what an application is about. A radio's
waterfall, a chat's timeline and a console's instruments are their
applications' own, built from the parts above; a subject two applications
share can become a kit of its own. [DESIGN.md](DESIGN.md) sets out what
belongs where.

## Using it

```rust
use quadrille::{Element, Theme, widget};

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .settings(quadrille::settings())
        .theme(|_: &App| Theme::TERMINAL)
        .run()
}

#[derive(Default)]
struct App {
    armed: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Armed(bool),
}

impl App {
    fn update(&mut self, Message::Armed(armed): Message) {
        self.armed = armed;
    }

    fn view(&self) -> Element<'_, Message> {
        widget::group(
            "SAFETY",
            widget::checkbox("MASTER ARM", self.armed, Message::Armed),
        )
        .into()
    }
}
```

## The showcase

`demo/` is the console of a spacecraft that does not exist, built on the
toolkit the way any application would be: flight instruments, a bench with a
scope, a strip chart and a heat map, and a technical drawing, all the
console's own. Two pages show the toolkit itself: every widget in every
state, and a type lab comparing candidate fonts.

It needs a recent stable Rust (the crates use edition 2024, so 1.85 or newer)
and a Wayland or X11 session for the window; the first build fetches the
[iced fork][fork] from GitHub.

```sh
cargo run -p quadrille-demo --release
```

F1–F5 choose the page and F6 changes the theme.

In a browser (this needs `wasm-pack`):

```sh
web/build.sh
python3 -m http.server 8080 --directory web
```

Every page can be rendered headless to PNG, one pixel per virtual pixel:

```sh
cargo run -p quadrille-demo --example render -- target/render
cargo run -p quadrille-demo --example render -- target/lab 42 --lab
```

## Fonts

Departure Mono is by Helena Zhang and licensed under the SIL Open Font License
1.1. Departure Mono Tight is derived from it by `tools/fonts/build.sh` (python3 with
fontTools). The
type lab's candidate fonts, their sources and licences are listed in
`demo/assets/fonts/lab/SOURCES.md`.

## Licence

The code is licensed under either of MIT or Apache-2.0, at your option. The
fonts keep their own licences.

[iced]: https://github.com/iced-rs/iced
[fork]: https://github.com/jwric/iced/tree/0.15-pixel-scale
