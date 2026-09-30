use iced_widget::core::{Color, Point};
use iced_widget::graphics::geometry;

use super::{Dash, Pen, shape};

/// A chain line: long dashes parted by short ones, the line type of
/// centrelines.
///
/// A period is a long dash, a gap, a short dash and another gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chain {
    /// Pixels in a long dash.
    pub long: u8,
    /// Pixels in a short dash.
    pub short: u8,
    /// Dark pixels either side of a short dash.
    pub gap: u8,
}

impl Chain {
    /// A centreline: long dashes of eleven pixels, and a dot between them
    /// with two pixels either side.
    pub const CENTRE: Self = Self::new(11, 1, 2);

    /// A chain of `long` and `short` dashes with `gap` dark pixels between
    /// them.
    pub const fn new(long: u8, short: u8, gap: u8) -> Self {
        Self { long, short, gap }
    }

    /// The pixels in one period.
    pub fn period(self) -> i32 {
        i32::from(self.long) + i32::from(self.short) + 2 * i32::from(self.gap)
    }

    /// Whether the `index`th pixel of a line is lit, counting from the first
    /// pixel of a long dash.
    pub fn lights(self, index: i32) -> bool {
        let period = self.period();

        if period == 0 {
            return true;
        }

        let index = index.rem_euclid(period);
        let (long, short, gap) = (
            i32::from(self.long),
            i32::from(self.short),
            i32::from(self.gap),
        );

        index < long || (long + gap..long + gap + short).contains(&index)
    }
}

impl<Renderer> Pen<'_, Renderer>
where
    Renderer: geometry::Renderer,
{
    /// A 1 px chain line from `from` to `to`, its pattern centred on the
    /// middle pixel so that both ends look alike.
    ///
    /// The middle pixel is the middle of a long dash; a line an odd number of
    /// pixels long with an odd `long` is exactly symmetric.
    pub fn chain(&mut self, from: Point<i32>, to: Point<i32>, chain: Chain, color: Color) {
        let line = shape::line(from, to);
        let middle = (line.len() as i32 - 1).div_euclid(2);
        let shift = i32::from(chain.long).div_euclid(2) - middle;

        let pixels: Vec<_> = line
            .into_iter()
            .enumerate()
            .filter(|(i, _)| chain.lights(*i as i32 + shift))
            .map(|(_, pixel)| pixel)
            .collect();

        let steep = (to.y - from.y).abs() > (to.x - from.x).abs();

        for run in shape::runs(&pixels, steep) {
            self.fill(run, color);
        }
    }

    /// A 1 px circle broken into `dash`, with a dash centred straight up.
    ///
    /// The pattern is stretched to repeat a whole number of times around the
    /// circle, so the last dash meets the first evenly.
    pub fn dashed_circle(&mut self, centre: Point<i32>, radius: i32, dash: Dash, color: Color) {
        let pixels = dashed_circle(centre, radius, dash);

        self.pixels(&pixels, color);
    }
}

/// The lit pixels of a [`shape::circle`] broken into `dash`.
fn dashed_circle(centre: Point<i32>, radius: i32, dash: Dash) -> Vec<Point<i32>> {
    let period = i32::from(dash.on) + i32::from(dash.off);

    if period == 0 {
        return shape::circle(centre, radius);
    }

    let circumference = std::f32::consts::TAU * radius as f32;
    let periods = (circumference / period as f32).round().max(1.0);

    shape::circle(centre, radius)
        .into_iter()
        .filter(|pixel| {
            let bearing = shape::bearing(pixel.x - centre.x, pixel.y - centre.y);
            let along = bearing / 360.0 * periods * period as f32;
            let index = (along + f32::from(dash.on) / 2.0).floor() as i32;

            dash.lights(index)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern(chain: Chain, length: i32) -> String {
        (0..length)
            .map(|i| if chain.lights(i) { '#' } else { '.' })
            .collect()
    }

    #[test]
    fn a_chain_alternates_long_and_short_dashes() {
        assert_eq!(pattern(Chain::new(5, 1, 2), 20), "#####..#..#####..#..");
        assert_eq!(pattern(Chain::CENTRE, 16), "###########..#..");
    }

    #[test]
    fn a_chain_without_a_period_is_solid() {
        assert!(Chain::new(0, 0, 0).lights(7));
    }

    #[test]
    fn a_dashed_circle_repeats_its_dash_evenly() {
        let circle = shape::circle(Point::new(0, 0), 10);
        let dashed = dashed_circle(Point::new(0, 0), 10, Dash::new(4, 4));

        assert!(dashed.iter().all(|pixel| circle.contains(pixel)));
        assert!(dashed.len() * 3 > circle.len());
        assert!(dashed.len() * 3 < circle.len() * 2);

        // Mirror images across the vertical axis are lit alike, bar the
        // pixels where a dash ends.
        let symmetric = dashed
            .iter()
            .filter(|pixel| dashed.contains(&Point::new(-pixel.x, pixel.y)))
            .count();

        assert!(symmetric * 4 > dashed.len() * 3);
    }
}
