//! Instruments: canvases that draw a reading.
//!
//! Each instrument is a small description of what to show, turned into a
//! canvas [`Element`](iced_widget::core::Element) that fills the space it is
//! given unless sized. They draw with a [`Pen`](crate::draw::Pen), so every
//! mark is on the pixel grid.
mod dial;
mod plot;
mod scale;
mod spectrum;
mod tape;
mod waterfall;

pub use dial::{Dial, dial};
pub use plot::{Cursors, Plot, Trace, channel, plot, trace};
pub use scale::engineering;
pub use spectrum::{Spectrum, spectrum};
pub use tape::{Marker, Side, Tape, tape};
pub use waterfall::{History, Ramp, Waterfall, waterfall};

use std::cell::RefCell;

use iced_widget::canvas::Cache;
use iced_widget::graphics::geometry;

/// Clears `cache` unless `key` is what it was last drawn from.
fn keep<K: PartialEq, Renderer: geometry::Renderer>(
    cache: &Cache<Renderer>,
    drawn: &RefCell<Option<K>>,
    key: K,
) {
    let mut drawn = drawn.borrow_mut();

    if drawn.as_ref() != Some(&key) {
        cache.clear();
        *drawn = Some(key);
    }
}
