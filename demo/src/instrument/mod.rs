//! The console's instruments: dials, tapes and a scope screen.
//!
//! They are the console's own rather than the toolkit's: no other
//! application draws them yet. Each is a small description of what to show,
//! made a widget by `quadrille::canvas_widget!` and drawn with the toolkit's
//! `Pen`, so every mark is on the pixel grid.
mod dial;
mod plot;
mod tape;

pub use dial::dial;
pub use plot::{Cursors, channel, plot, trace};
pub use tape::{Marker, Side, tape};
