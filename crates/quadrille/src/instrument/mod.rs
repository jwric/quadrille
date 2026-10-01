//! Instruments: canvases that draw a reading.
//!
//! Each instrument is a small description of what to show, and a widget that
//! draws it as a canvas, filling the space it is given unless sized. They draw with a [`Pen`](crate::draw::Pen), so every
//! mark is on the pixel grid.
mod dial;
mod plot;
mod tape;

pub use dial::{Dial, dial};
pub use plot::{Cursors, Plot, Trace, channel, plot, trace};
pub use tape::{Marker, Side, Tape, tape};
