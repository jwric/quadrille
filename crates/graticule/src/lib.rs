//! A pixel-perfect retro UI toolkit for [iced].
//!
//! Graticule targets the `0.15-pixel-scale` iced fork, which lays an interface
//! out in virtual pixels and upscales it nearest-neighbour. On top of that it
//! provides:
//!
//! - a [`Theme`] with a small, role-named [`Palette`] and styles for iced's
//!   built-in widgets;
//! - pixel fonts at their native sizes, described by [`Face`];
//! - integer drawing primitives for canvases, in [`draw`];
//! - instrument-panel widgets, in [`widget`].
//!
//! Start an application with [`settings`], so that fonts, the default face,
//! the pixel scale and antialiasing match what the toolkit is drawn for:
//!
//! ```no_run
//! # use graticule::Theme;
//! # #[derive(Default)] struct App;
//! # #[derive(Debug, Clone)] enum Message {}
//! # impl App {
//! #     fn update(&mut self, _message: Message) {}
//! #     fn view(&self) -> impl iced::Widget<Message, Theme> { graticule::widget::label("hello") }
//! # }
//! pub fn main() -> iced::Result {
//!     iced::application(App::default, App::update, App::view)
//!         .settings(graticule::settings())
//!         .theme(|_: &App| Theme::TERMINAL)
//!         .run()
//! }
//! ```
//!
//! [iced]: https://github.com/iced-rs/iced
pub mod draw;
pub mod face;
pub mod fonts;
pub mod icon;
pub mod instrument;
pub mod px;
pub mod style;
pub mod theme;
pub mod widget;

pub use face::Face;
pub use theme::{Palette, Theme};

/// An [`iced::Element`] drawn with the graticule [`Theme`].
pub type Element<'a, Message, Renderer = iced_widget::Renderer> =
    iced_widget::core::Element<'a, Message, Theme, Renderer>;

/// The virtual-pixel size a [`PixelScaleMode::Auto`] targets: one virtual
/// pixel is about this many logical pixels on every display.
///
/// [`PixelScaleMode::Auto`]: iced::PixelScaleMode::Auto
pub const LOGICAL_PER_VIRTUAL: u32 = 2;

/// The [`iced::Settings`] a graticule application runs with.
///
/// It loads the toolkit's fonts, makes [`Face::BODY`] the default face, turns
/// antialiasing off, and scales the interface by [`LOGICAL_PER_VIRTUAL`].
/// Tests should build their simulators from the same settings, so that what is
/// measured is what runs.
pub fn settings() -> iced_widget::core::Settings {
    iced_widget::core::Settings {
        fonts: fonts::ALL.iter().map(|font| (*font).into()).collect(),
        font: Face::BODY.font,
        text_size: f32::from(Face::BODY.size()).into(),
        antialiasing: false,
        pixel_scale: iced_widget::core::PixelScaleMode::Auto(LOGICAL_PER_VIRTUAL),
        ..iced_widget::core::Settings::default()
    }
}
