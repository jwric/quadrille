//! The spacing scale and the pixel grid.
//!
//! Every length in a quadrille interface is a whole number of virtual pixels.
//! Spacing comes from the steps below; anything else is derived from a
//! [`Face`](crate::Face)'s cell. The steps are `f32` so they can be handed
//! straight to iced, but every one of them is whole.

use iced_widget::core::{Point, Rectangle, Size};

/// Borders, rules, and a gap that only separates.
pub const HAIR: f32 = 1.0;
/// Inside a cluster: a label and its value, a lamp and its legend.
pub const TIGHT: f32 = 2.0;
/// Between siblings.
pub const GAP: f32 = 4.0;
/// Between groups.
pub const WIDE: f32 = 8.0;
/// Between regions.
pub const FAR: f32 = 16.0;

/// The nearest whole pixel to `value`.
pub fn snap(value: f32) -> f32 {
    value.round()
}

/// `point` moved to the nearest whole pixel.
pub fn snap_point(point: Point) -> Point {
    Point::new(point.x.round(), point.y.round())
}

/// The whole pixels inside `bounds`: its origin rounded, its size floored.
///
/// Layout can hand a widget fractional bounds when it centres an odd size in
/// an even one; drawing into the floored interior keeps every edge crisp.
pub fn interior(bounds: Rectangle) -> Rectangle<i32> {
    Rectangle {
        x: bounds.x.round() as i32,
        y: bounds.y.round() as i32,
        width: bounds.width.floor() as i32,
        height: bounds.height.floor() as i32,
    }
}

/// `size` floored to whole pixels.
pub fn floor(size: Size) -> Size<i32> {
    Size::new(size.width.floor() as i32, size.height.floor() as i32)
}

/// The offset that centres `inner` pixels in `outer`, rounded down.
///
/// Rounding the same way everywhere means an odd leftover always falls on the
/// same side, so two things centred in the same space stay aligned.
pub const fn centre(outer: i32, inner: i32) -> i32 {
    (outer - inner).div_euclid(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_odd_leftover_falls_below_and_right() {
        assert_eq!(centre(12, 8), 2);
        assert_eq!(centre(12, 7), 2);
        assert_eq!(centre(7, 12), -3);
    }

    #[test]
    fn the_interior_keeps_whole_pixels() {
        let bounds = Rectangle::new(Point::new(10.5, -3.4), Size::new(20.7, 9.99));

        assert_eq!(
            interior(bounds),
            Rectangle {
                x: 11,
                y: -3,
                width: 20,
                height: 9
            }
        );
    }
}
