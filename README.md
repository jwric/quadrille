# quadrille

A pixel-perfect retro UI toolkit for [iced]: instrument panels, technical
drawings and pixel type, drawn in whole virtual pixels.

It targets the [`0.15-pixel-scale` iced fork][fork], which lays an interface
out in virtual pixels and upscales it nearest-neighbour.

- **Theme**: a small palette named for roles, five themes, and flat, square
  styles for every built-in iced widget.
- **Type**: Departure Mono at its native sizes, with a tighter 6 × 12 cell for
  dense panels. Line heights keep the baseline on a whole pixel.
- **Drawing**: a `Pen` for canvases with integer coordinates only: lines,
  circles, heavy rings, arcs, ellipses, dashes, dither patterns, sprites,
  callouts, dimension lines and text anchored by its baseline or capitals.
- **Widgets**: keys, soft keys, selectors, check boxes, radios, lamps, bar
  gauges, groups, fields, and canvas instruments (value tapes, dials).

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

`demo/` is the console of a spacecraft that does not exist: flight
instruments, signal instruments, a technical drawing, every widget in every
state, and a type lab comparing candidate fonts.

```sh
cargo run -p quadrille-demo --release
```

F1–F5 choose the page and F6 changes the theme.

In a browser:

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
1.1. Departure Mono Tight is derived from it by `tools/fonts/build.sh`. The
type lab's candidate fonts, their sources and licences are listed in
`demo/assets/fonts/lab/SOURCES.md`.

## Licence

The code is licensed under either of MIT or Apache-2.0, at your option. The
fonts keep their own licences.

[iced]: https://github.com/iced-rs/iced
[fork]: https://github.com/jwric/iced/tree/0.15-pixel-scale
