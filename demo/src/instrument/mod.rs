//! The console's instruments: dials, tapes, a scope screen and a tuning
//! readout.
//!
//! They are the console's own rather than the toolkit's: no other
//! application draws them yet. Each is a small description of what to show,
//! made a widget by `quadrille::canvas_widget!` (or, for the readout, a widget
//! of its own) and drawn with the toolkit's `Pen`, so every mark is on the
//! pixel grid.
pub mod digits;

mod dial;
mod plot;
mod tape;

pub use dial::dial;
pub use digits::Digits;
pub use plot::{Cursors, channel, plot, trace};
pub use tape::{Marker, Side, tape};

/// A whole number shown to `places` digits, each of which the wheel steps
/// by its place value once [`Digits::on_change`] is set.
pub fn digits<'a, Message>(value: i64, places: u8) -> Digits<'a, Message> {
    Digits::new(value, places)
}
