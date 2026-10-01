use std::cmp::Ordering;

use iced_widget::core::{Color, Point, Rectangle};
use iced_widget::graphics::geometry;

use super::{Dash, Pattern, Pen, rectangle, shape};

/// A shape bounded by straight edges between pixels.
///
/// A polygon has one or more closed contours and is filled even-odd, so a
/// contour inside another cuts a hole in it. Its edges are 1 px lines, and it
/// covers their pixels and every pixel whose centre lies inside it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Polygon {
    contours: Vec<Vec<Point<i32>>>,
}

impl Polygon {
    /// A polygon with one contour through `points`, closed back to the
    /// first.
    pub fn new(points: impl IntoIterator<Item = Point<i32>>) -> Self {
        Self::default().with(points)
    }

    /// A polygon covering `bounds`.
    pub fn rectangle(bounds: Rectangle<i32>) -> Self {
        let (right, bottom) = (bounds.x + bounds.width - 1, bounds.y + bounds.height - 1);

        Self::new([
            Point::new(bounds.x, bounds.y),
            Point::new(right, bounds.y),
            Point::new(right, bottom),
            Point::new(bounds.x, bottom),
        ])
    }

    /// The same polygon with another contour through `points`.
    pub fn with(mut self, points: impl IntoIterator<Item = Point<i32>>) -> Self {
        let mut contour: Vec<Point<i32>> = Vec::new();

        for point in points {
            if contour.last() != Some(&point) {
                contour.push(point);
            }
        }

        while contour.len() > 1 && contour.first() == contour.last() {
            contour.pop();
        }

        if !contour.is_empty() {
            self.contours.push(contour);
        }

        self
    }

    /// The edges of every contour, as pairs of vertices.
    fn edges(&self) -> impl Iterator<Item = (Point<i32>, Point<i32>)> + '_ {
        self.contours.iter().flat_map(|contour| {
            (0..contour.len()).map(|i| (contour[i], contour[(i + 1) % contour.len()]))
        })
    }

    /// The pixels of its edges, each once.
    pub fn outline(&self) -> Vec<Point<i32>> {
        let mut pixels: Vec<_> = self
            .edges()
            .flat_map(|(from, to)| shape::line(from, to))
            .collect();

        pixels.sort_by_key(|pixel| (pixel.y, pixel.x));
        pixels.dedup();
        pixels
    }

    /// The rows it covers, edges included, as `(y, first x, last x)`.
    pub fn rows(&self) -> Vec<(i32, i32, i32)> {
        let outline = self.outline();
        let mut rows = Vec::new();
        let mut start = 0;

        while start < outline.len() {
            let y = outline[start].y;
            let end = start + outline[start..].partition_point(|pixel| pixel.y == y);

            rows.extend(
                self.spans(y, &outline[start..end])
                    .into_iter()
                    .map(|(from, to)| (y, from, to)),
            );

            start = end;
        }

        rows
    }

    /// Whether it covers the pixel at `point`.
    pub fn contains(&self, point: Point<i32>) -> bool {
        let edge: Vec<_> = self
            .edges()
            .flat_map(|(from, to)| shape::line(from, to))
            .filter(|pixel| pixel.y == point.y)
            .collect();

        self.spans(point.y, &edge)
            .iter()
            .any(|&(from, to)| (from..=to).contains(&point.x))
    }

    /// The smallest rectangle holding every vertex.
    pub fn bounds(&self) -> Rectangle<i32> {
        let points = self.contours.iter().flatten();
        let left = points.clone().map(|p| p.x).min().unwrap_or(0);
        let right = points.clone().map(|p| p.x).max().unwrap_or(-1);
        let top = points.clone().map(|p| p.y).min().unwrap_or(0);
        let bottom = points.map(|p| p.y).max().unwrap_or(-1);

        rectangle(left, top, right - left + 1, bottom - top + 1)
    }

    /// The runs of row `y` it covers: the interior between crossings, taken
    /// even-odd, merged with `edge`, the pixels of the edges on that row.
    fn spans(&self, y: i32, edge: &[Point<i32>]) -> Vec<(i32, i32)> {
        let mut crossings: Vec<(i64, i64)> = self
            .edges()
            .filter(|(from, to)| (from.y <= y && y < to.y) || (to.y <= y && y < from.y))
            .map(|(from, to)| {
                let (dy, dx) = (i64::from(to.y - from.y), i64::from(to.x - from.x));
                let numerator = i64::from(from.x) * dy + i64::from(y - from.y) * dx;

                if dy < 0 {
                    (-numerator, -dy)
                } else {
                    (numerator, dy)
                }
            })
            .collect();

        crossings.sort_by(|a, b| compare(*a, *b));

        let mut spans: Vec<(i32, i32)> = crossings
            .chunks_exact(2)
            .filter_map(|pair| {
                let from = pair[0].0.div_euclid(pair[0].1)
                    + i64::from(pair[0].0.rem_euclid(pair[0].1) != 0);
                let to = pair[1].0.div_euclid(pair[1].1);

                (from <= to).then_some((from as i32, to as i32))
            })
            .collect();

        spans.extend(edge.iter().map(|pixel| (pixel.x, pixel.x)));
        spans.sort_unstable();

        let mut merged: Vec<(i32, i32)> = Vec::with_capacity(spans.len());

        for (from, to) in spans {
            match merged.last_mut() {
                Some(last) if from <= last.1 + 1 => last.1 = last.1.max(to),
                _ => merged.push((from, to)),
            }
        }

        merged
    }
}

/// Orders two fractions with positive denominators.
fn compare(a: (i64, i64), b: (i64, i64)) -> Ordering {
    (a.0 * b.1).cmp(&(b.0 * a.1))
}

impl<Renderer> Pen<'_, Renderer>
where
    Renderer: geometry::Renderer,
{
    /// The 1 px edges of `polygon`.
    pub fn polygon(&mut self, polygon: &Polygon, color: Color) {
        for (from, to) in polygon.edges() {
            self.line(from, to, color);
        }
    }

    /// The edges of `polygon` in `dash`, each edge counted from its first
    /// vertex so that every corner starts a dash.
    pub fn dashed_polygon(&mut self, polygon: &Polygon, dash: Dash, color: Color) {
        for (from, to) in polygon.edges() {
            self.dashed(from, to, dash, color);
        }
    }

    /// Fills `polygon`, edges included, with `pattern` counted from `origin`.
    pub fn fill_polygon(
        &mut self,
        polygon: &Polygon,
        pattern: Pattern,
        origin: Point<i32>,
        color: Color,
    ) {
        for (y, from, to) in polygon.rows() {
            self.pattern(rectangle(from, y, to - from + 1, 1), pattern, origin, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Renders the rows of `polygon` as art: `#` for a covered pixel.
    fn art(polygon: &Polygon) -> String {
        let rows = polygon.rows();
        let left = rows.iter().map(|row| row.1).min().unwrap_or(0);
        let right = rows.iter().map(|row| row.2).max().unwrap_or(0);
        let top = rows.iter().map(|row| row.0).min().unwrap_or(0);
        let bottom = rows.iter().map(|row| row.0).max().unwrap_or(0);

        (top..=bottom)
            .map(|y| {
                (left..=right)
                    .map(|x| {
                        let covered = rows
                            .iter()
                            .any(|&(row, from, to)| row == y && (from..=to).contains(&x));

                        if covered { '#' } else { '.' }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn assert_art(polygon: &Polygon, expected: &str) {
        let expected = expected
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let actual = art(polygon);

        assert_eq!(
            actual, expected,
            "\nexpected:\n{expected}\n\nactual:\n{actual}\n"
        );
    }

    fn points(points: &[(i32, i32)]) -> Vec<Point<i32>> {
        points.iter().map(|&(x, y)| Point::new(x, y)).collect()
    }

    #[test]
    fn a_diamond_fills_symmetrically() {
        let diamond = Polygon::new(points(&[(3, 0), (6, 3), (3, 6), (0, 3)]));

        assert_art(
            &diamond,
            "
            ...#...
            ..###..
            .#####.
            #######
            .#####.
            ..###..
            ...#...
            ",
        );
    }

    #[test]
    fn a_fill_covers_its_outline() {
        let wedge = Polygon::new(points(&[(0, 0), (9, 2), (4, 7)]));
        let rows = wedge.rows();

        for pixel in wedge.outline() {
            assert!(
                rows.iter()
                    .any(|&(y, from, to)| y == pixel.y && (from..=to).contains(&pixel.x)),
                "{pixel:?} of the outline is not filled"
            );
        }

        assert!(wedge.contains(Point::new(4, 3)));
        assert!(!wedge.contains(Point::new(8, 6)));
    }

    #[test]
    fn an_inner_contour_cuts_a_hole() {
        let frame = Polygon::new(points(&[(0, 0), (6, 0), (6, 4), (0, 4)])).with(points(&[
            (2, 1),
            (4, 1),
            (4, 3),
            (2, 3),
        ]));

        assert_art(
            &frame,
            "
            #######
            #######
            ###.###
            #######
            #######
            ",
        );
    }

    #[test]
    fn contours_apart_fill_apart() {
        let pair = Polygon::rectangle(rectangle(0, 0, 2, 3)).with(points(&[
            (5, 0),
            (6, 0),
            (6, 2),
            (5, 2),
        ]));

        assert_art(
            &pair,
            "
            ##...##
            ##...##
            ##...##
            ",
        );
        assert_eq!(pair.bounds(), rectangle(0, 0, 7, 3));
    }

    #[test]
    fn a_closing_vertex_is_ignored() {
        let open = Polygon::new(points(&[(0, 0), (4, 0), (4, 4)]));
        let closed = Polygon::new(points(&[(0, 0), (4, 0), (4, 4), (0, 0)]));

        assert_eq!(open, closed);
        assert_eq!(open.edges().count(), 3);
    }
}
