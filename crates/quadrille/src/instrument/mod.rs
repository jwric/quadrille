//! Instruments: canvases that draw a reading.
//!
//! Each instrument is a small description of what to show, and a widget that
//! draws it as a canvas, filling the space it is given unless sized. They draw with a [`Pen`](crate::draw::Pen), so every
//! mark is on the pixel grid.
mod dial;
mod plot;
mod spectrum;
mod tape;
mod waterfall;

pub use dial::{Dial, dial};
pub use plot::{Cursors, Plot, Trace, channel, plot, trace};
pub use spectrum::{Spectrum, spectrum};
pub use tape::{Marker, Side, Tape, tape};
pub use waterfall::{History, Waterfall, waterfall};

/// The default margin left of a spectrum or a waterfall: room for `-100`
/// and a tick.
const GUTTER: u16 = 28;
