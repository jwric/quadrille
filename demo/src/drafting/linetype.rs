use iced::advanced::graphics::geometry;
use iced::{Color, Point};

use quadrille::draw::{Pen, shape};

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

/// See [`Drafting::chain`](super::Drafting::chain).
pub(super) fn chain<Renderer: geometry::Renderer>(
    pen: &mut Pen<'_, Renderer>,
    from: Point<i32>,
    to: Point<i32>,
    chain: Chain,
    color: Color,
) {
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
        pen.fill(run, color);
    }
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
}
