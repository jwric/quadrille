use iced::advanced::graphics::geometry;
use iced::{Color, Point};

use quadrille::draw::{Axis, Direction, Pen, rectangle, shape};

/// An orthogonal wire: horizontal and vertical runs between corners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wire {
    corners: Vec<Point<i32>>,
}

impl Wire {
    /// A wire that starts, and so far ends, at `at`.
    pub fn start(at: Point<i32>) -> Self {
        Self { corners: vec![at] }
    }

    /// The wire continued along its row to column `x`.
    pub fn horizontal(self, x: i32) -> Self {
        let end = self.last();

        self.extend(Point::new(x, end.y))
    }

    /// The wire continued along its column to row `y`.
    pub fn vertical(self, y: i32) -> Self {
        let end = self.last();

        self.extend(Point::new(end.x, y))
    }

    /// The wire from `from` to `to` with the fewest bends that leaves along
    /// `leave` and arrives along `arrive`.
    ///
    /// Leaving and arriving along the same axis, the wire crosses over
    /// halfway, rounded towards `from`; along different axes, it bends once.
    pub fn route(from: Point<i32>, to: Point<i32>, leave: Axis, arrive: Axis) -> Self {
        let wire = Self::start(from);
        let halfway = |a: i32, b: i32| a + (b - a) / 2;

        match (leave, arrive) {
            (Axis::Horizontal, Axis::Horizontal) => wire
                .horizontal(halfway(from.x, to.x))
                .vertical(to.y)
                .horizontal(to.x),
            (Axis::Vertical, Axis::Vertical) => wire
                .vertical(halfway(from.y, to.y))
                .horizontal(to.x)
                .vertical(to.y),
            (Axis::Horizontal, Axis::Vertical) => wire.horizontal(to.x).vertical(to.y),
            (Axis::Vertical, Axis::Horizontal) => wire.vertical(to.y).horizontal(to.x),
        }
    }

    /// The first pixel.
    pub fn first(&self) -> Point<i32> {
        self.corners[0]
    }

    /// The last pixel.
    pub fn last(&self) -> Point<i32> {
        self.corners[self.corners.len() - 1]
    }

    /// The corners, from the first pixel to the last.
    pub fn corners(&self) -> &[Point<i32>] {
        &self.corners
    }

    /// The way the wire runs into its last pixel, for an arrowhead there.
    pub fn heading(&self) -> Option<Direction> {
        let [.., before, end] = self.corners[..] else {
            return None;
        };

        Some(if end.x > before.x {
            Direction::Right
        } else if end.x < before.x {
            Direction::Left
        } else if end.y > before.y {
            Direction::Down
        } else {
            Direction::Up
        })
    }

    /// The pixels from the first to the last, each once.
    pub fn pixels(&self) -> Vec<Point<i32>> {
        shape::polyline(&self.corners)
    }

    /// Whether `point` lies on the wire.
    pub fn passes(&self, point: Point<i32>) -> bool {
        self.corners
            .windows(2)
            .any(|pair| on_run(pair[0], pair[1], point))
            || self.corners == [point]
    }

    fn extend(mut self, to: Point<i32>) -> Self {
        let end = self.last();

        if to == end {
            return self;
        }

        // A run that carries on the way the last one went only moves its end.
        if let [.., before, _] = self.corners[..] {
            let across = before.y == end.y && end.y == to.y;
            let along = before.x == end.x && end.x == to.x;
            let onwards = (end.x - before.x).signum() == (to.x - end.x).signum()
                && (end.y - before.y).signum() == (to.y - end.y).signum();

            if (across || along) && onwards {
                *self.corners.last_mut().expect("a wire has a corner") = to;
                return self;
            }
        }

        self.corners.push(to);
        self
    }
}

/// Whether `point` lies on the axis-aligned run from `a` to `b`.
fn on_run(a: Point<i32>, b: Point<i32>, point: Point<i32>) -> bool {
    let within = |p: i32, a: i32, b: i32| (a.min(b)..=a.max(b)).contains(&p);

    (a.y == b.y && point.y == a.y && within(point.x, a.x, b.x))
        || (a.x == b.x && point.x == a.x && within(point.y, a.y, b.y))
}

/// The points where `wires` join: an end of one wire on a run of another,
/// short of that wire's ends, or three or more wire ends meeting.
///
/// Wires that cross without either ending there do not join.
pub fn junctions(wires: &[Wire]) -> Vec<Point<i32>> {
    let mut junctions = Vec::new();

    for (i, wire) in wires.iter().enumerate() {
        for end in [wire.first(), wire.last()] {
            let tee = wires.iter().enumerate().any(|(j, other)| {
                j != i && other.passes(end) && end != other.first() && end != other.last()
            });

            let meeting = wires
                .iter()
                .map(|other| usize::from(other.first() == end) + usize::from(other.last() == end))
                .sum::<usize>();

            if tee || meeting >= 3 {
                junctions.push(end);
            }
        }
    }

    junctions.sort_by_key(|point| (point.y, point.x));
    junctions.dedup();
    junctions
}

/// See [`Drafting::wire`](super::Drafting::wire).
pub(super) fn wire<Renderer: geometry::Renderer>(
    pen: &mut Pen<'_, Renderer>,
    wire: &Wire,
    color: Color,
) {
    if let [only] = wire.corners[..] {
        pen.pixel(only, color);
    }

    for pair in wire.corners.windows(2) {
        let (a, b) = (pair[0], pair[1]);

        if a.y == b.y {
            pen.hline(a.x, b.x, a.y, color);
        } else {
            pen.vline(a.x, a.y, b.y, color);
        }
    }
}

/// See [`Drafting::junction`](super::Drafting::junction).
pub(super) fn junction<Renderer: geometry::Renderer>(
    pen: &mut Pen<'_, Renderer>,
    at: Point<i32>,
    color: Color,
) {
    pen.fill(rectangle(at.x - 1, at.y - 1, 3, 3), color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wire_routed_with_one_bend_is_the_wire_drawn_by_hand() {
        let (from, to) = (Point::new(0, 0), Point::new(12, -8));
        let wire = Wire::start(from).horizontal(12).vertical(-8);

        assert_eq!(
            wire,
            Wire::route(from, to, Axis::Horizontal, Axis::Vertical)
        );
        assert_eq!(wire.pixels().len(), 21);
    }

    /// Renders `wires` as art: `#` for a wire, `o` for a junction.
    fn art(wires: &[Wire]) -> String {
        let pixels: Vec<_> = wires.iter().flat_map(Wire::pixels).collect();
        let joins = junctions(wires);
        let left = pixels.iter().map(|p| p.x).min().unwrap_or(0);
        let right = pixels.iter().map(|p| p.x).max().unwrap_or(0);
        let top = pixels.iter().map(|p| p.y).min().unwrap_or(0);
        let bottom = pixels.iter().map(|p| p.y).max().unwrap_or(0);

        (top..=bottom)
            .map(|y| {
                (left..=right)
                    .map(|x| {
                        let point = Point::new(x, y);

                        if joins.contains(&point) {
                            'o'
                        } else if pixels.contains(&point) {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn assert_art(wires: &[Wire], expected: &str) {
        let expected = expected
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let actual = art(wires);

        assert_eq!(
            actual, expected,
            "\nexpected:\n{expected}\n\nactual:\n{actual}\n"
        );
    }

    #[test]
    fn a_route_along_one_axis_crosses_over_halfway() {
        let wire = Wire::route(
            Point::new(0, 0),
            Point::new(7, 3),
            Axis::Horizontal,
            Axis::Horizontal,
        );

        assert_eq!(wire.heading(), Some(Direction::Right));
        assert_art(
            &[wire],
            "
            ####....
            ...#....
            ...#....
            ...#####
            ",
        );
    }

    #[test]
    fn a_route_across_axes_bends_once() {
        let wire = Wire::route(
            Point::new(0, 4),
            Point::new(4, 0),
            Axis::Vertical,
            Axis::Horizontal,
        );

        assert_eq!(
            wire.corners(),
            [Point::new(0, 4), Point::new(0, 0), Point::new(4, 0)]
        );
        assert_eq!(wire.heading(), Some(Direction::Right));
        assert_eq!(wire.pixels().len(), 9);
    }

    #[test]
    fn a_straight_route_has_no_corners() {
        let wire = Wire::route(
            Point::new(0, 2),
            Point::new(9, 2),
            Axis::Horizontal,
            Axis::Horizontal,
        );

        assert_eq!(wire.corners(), [Point::new(0, 2), Point::new(9, 2)]);
    }

    #[test]
    fn a_branch_joins_where_it_ends_on_a_run() {
        let trunk = Wire::start(Point::new(0, 2)).horizontal(8);
        let up = Wire::start(Point::new(4, 2)).vertical(0);
        let down = Wire::start(Point::new(6, 2)).vertical(4);
        let crossing = Wire::start(Point::new(2, 0)).vertical(4);

        assert_art(
            &[trunk, up, down, crossing],
            "
            ..#.#....
            ..#.#....
            ####o#o##
            ..#...#..
            ..#...#..
            ",
        );
    }

    #[test]
    fn wires_meeting_end_to_end_do_not_join() {
        let first = Wire::start(Point::new(0, 0)).horizontal(4);
        let second = Wire::start(Point::new(4, 0)).vertical(3);

        assert!(junctions(&[first, second]).is_empty());
    }

    #[test]
    fn three_ends_meeting_join() {
        let wires = [
            Wire::start(Point::new(0, 2)).horizontal(3),
            Wire::start(Point::new(3, 2)).vertical(0),
            Wire::start(Point::new(3, 2)).horizontal(6),
        ];

        assert_eq!(junctions(&wires), [Point::new(3, 2)]);
    }

    #[test]
    fn a_run_onwards_extends_the_last() {
        let wire = Wire::start(Point::new(0, 0)).horizontal(3).horizontal(6);

        assert_eq!(wire.corners(), [Point::new(0, 0), Point::new(6, 0)]);
    }
}
