//! The pixels of shapes, computed with integer rasterization.
//!
//! Every function here decides which whole pixels a shape lights, so a shape
//! lights the same pixels on every renderer and at every position. A pixel is
//! named by its top-left corner; `(x, y)` covers `[x, x + 1) × [y, y + 1)`.
use iced_widget::core::{Point, Rectangle};

use super::Dash;

/// The pixels of the line from `from` to `to`, both included, in order.
///
/// One pixel per step along the major axis, so the line is a pixel thin at
/// every angle and a 45° line is a perfect stair.
pub fn line(from: Point<i32>, to: Point<i32>) -> Vec<Point<i32>> {
    let dx = (to.x - from.x).abs();
    let dy = -(to.y - from.y).abs();
    let step_x = if from.x < to.x { 1 } else { -1 };
    let step_y = if from.y < to.y { 1 } else { -1 };

    let mut pixels = Vec::with_capacity((dx - dy + 1) as usize);
    let mut error = dx + dy;
    let (mut x, mut y) = (from.x, from.y);

    loop {
        pixels.push(Point::new(x, y));

        if x == to.x && y == to.y {
            return pixels;
        }

        let doubled = 2 * error;

        if doubled >= dy {
            error += dy;
            x += step_x;
        }

        if doubled <= dx {
            error += dx;
            y += step_y;
        }
    }
}

/// The pixels of a 1 px circle of `radius` around the centre pixel, each once.
///
/// A circle drawn around a pixel is `2 × radius + 1` pixels across, so it has
/// a centre pixel and its axes are whole columns and rows.
pub fn circle(centre: Point<i32>, radius: i32) -> Vec<Point<i32>> {
    let mut pixels = Vec::new();

    octants(radius, |x, y| {
        for (px, py) in [
            (x, y),
            (y, x),
            (-y, x),
            (-x, y),
            (-x, -y),
            (-y, -x),
            (y, -x),
            (x, -y),
        ] {
            pixels.push(Point::new(centre.x + px, centre.y + py));
        }
    });

    dedup(pixels)
}

/// The pixels of a [`circle`] broken into `dash`, with a dash centred
/// straight up.
///
/// The pattern is stretched to repeat a whole number of times around the
/// circle, so the last dash meets the first evenly.
pub fn dashed_circle(centre: Point<i32>, radius: i32, dash: Dash) -> Vec<Point<i32>> {
    let period = i32::from(dash.on) + i32::from(dash.off);

    if period == 0 {
        return circle(centre, radius);
    }

    let circumference = std::f32::consts::TAU * radius as f32;
    let periods = (circumference / period as f32).round().max(1.0);

    circle(centre, radius)
        .into_iter()
        .filter(|pixel| {
            let bearing = bearing(pixel.x - centre.x, pixel.y - centre.y);
            let along = bearing / 360.0 * periods * period as f32;
            let index = (along + f32::from(dash.on) / 2.0).floor() as i32;

            dash.lights(index)
        })
        .collect()
}

/// The rows of a filled disc of `radius` around the centre pixel, as
/// `(y, first x, last x)`.
///
/// The disc covers exactly the pixels inside the [`circle`] of the same
/// radius, ring included.
pub fn disc(centre: Point<i32>, radius: i32) -> Vec<(i32, i32, i32)> {
    if radius < 0 {
        return Vec::new();
    }

    let mut half = vec![0; radius as usize + 1];

    octants(radius, |x, y| {
        half[y as usize] = half[y as usize].max(x);
        half[x as usize] = half[x as usize].max(y);
    });

    (-radius..=radius)
        .map(|dy| {
            let span = half[dy.unsigned_abs() as usize];

            (centre.y + dy, centre.x - span, centre.x + span)
        })
        .collect()
}

/// The pixels of a ring `weight` pixels wide, its outer edge on the [`circle`]
/// of `radius`, each once.
///
/// Each pixel of the circle is thickened inwards along its major axis, the way
/// a pixel artist draws a heavy circle: the ring is `weight` pixels across
/// every row and column it crosses, and solid at every bearing.
pub fn ring(centre: Point<i32>, radius: i32, weight: i32) -> Vec<Point<i32>> {
    let mut pixels = Vec::new();

    for pixel in circle(centre, radius) {
        let (dx, dy) = (pixel.x - centre.x, pixel.y - centre.y);
        let (step_x, step_y) = if dx.abs() >= dy.abs() {
            (-dx.signum(), 0)
        } else {
            (0, -dy.signum())
        };

        for i in 0..weight.max(1) {
            pixels.push(Point::new(pixel.x + i * step_x, pixel.y + i * step_y));
        }
    }

    dedup(pixels)
}

/// The pixels of a [`ring`] between two bearings, clockwise from `from`.
pub fn thick_arc(
    centre: Point<i32>,
    radius: i32,
    weight: i32,
    from: f32,
    to: f32,
) -> Vec<Point<i32>> {
    let sweep = sweep(from, to);

    ring(centre, radius, weight)
        .into_iter()
        .filter(|pixel| within(bearing(pixel.x - centre.x, pixel.y - centre.y), from, sweep))
        .collect()
}

/// The pixels of the arc of a 1 px circle between two bearings.
///
/// Bearings are in degrees clockwise from straight up, as on a dial; the arc
/// runs clockwise from `from` to `to`.
pub fn arc(centre: Point<i32>, radius: i32, from: f32, to: f32) -> Vec<Point<i32>> {
    let sweep = sweep(from, to);

    circle(centre, radius)
        .into_iter()
        .filter(|pixel| within(bearing(pixel.x - centre.x, pixel.y - centre.y), from, sweep))
        .collect()
}

/// The clockwise sweep from `from` to `to`, in degrees; a full turn when they
/// differ by a whole number of turns.
fn sweep(from: f32, to: f32) -> f32 {
    let sweep = (to - from).rem_euclid(360.0);

    if sweep == 0.0 && to != from {
        360.0
    } else {
        sweep
    }
}

fn within(bearing: f32, from: f32, sweep: f32) -> bool {
    (bearing - from).rem_euclid(360.0) <= sweep
}

/// The bearing of the offset `(dx, dy)`: degrees clockwise from straight up.
pub fn bearing(dx: i32, dy: i32) -> f32 {
    (dx as f32).atan2(-dy as f32).to_degrees().rem_euclid(360.0)
}

/// The pixel at `distance` from `centre` along `bearing`, rounded.
pub fn polar(centre: Point<i32>, bearing: f32, distance: f32) -> Point<i32> {
    let radians = bearing.to_radians();

    Point::new(
        centre.x + (distance * radians.sin()).round() as i32,
        centre.y - (distance * radians.cos()).round() as i32,
    )
}

/// The pixels of a 1 px axis-aligned ellipse around the centre pixel, each
/// once.
pub fn ellipse(centre: Point<i32>, radius_x: i32, radius_y: i32) -> Vec<Point<i32>> {
    if radius_x <= 0 || radius_y <= 0 {
        return line(
            Point::new(centre.x - radius_x.max(0), centre.y - radius_y.max(0)),
            Point::new(centre.x + radius_x.max(0), centre.y + radius_y.max(0)),
        );
    }

    let (rx, ry) = (f64::from(radius_x), f64::from(radius_y));
    let (rx2, ry2) = (rx * rx, ry * ry);

    let mut pixels = Vec::new();
    let mut quadrants = |x: i32, y: i32| {
        for (px, py) in [(x, y), (-x, y), (x, -y), (-x, -y)] {
            pixels.push(Point::new(centre.x + px, centre.y + py));
        }
    };

    let (mut x, mut y) = (0, radius_y);
    let mut dx = 0.0;
    let mut dy = 2.0 * rx2 * ry;
    let mut decision = ry2 - rx2 * ry + 0.25 * rx2;

    while dx < dy {
        quadrants(x, y);
        x += 1;
        dx += 2.0 * ry2;

        if decision < 0.0 {
            decision += dx + ry2;
        } else {
            y -= 1;
            dy -= 2.0 * rx2;
            decision += dx - dy + ry2;
        }
    }

    let (fx, fy) = (f64::from(x), f64::from(y));
    let mut decision = ry2 * (fx + 0.5).powi(2) + rx2 * (fy - 1.0).powi(2) - rx2 * ry2;

    while y >= 0 {
        quadrants(x, y);
        y -= 1;
        dy -= 2.0 * rx2;

        if decision > 0.0 {
            decision += rx2 - dy;
        } else {
            x += 1;
            dx += 2.0 * ry2;
            decision += dx - dy + rx2;
        }
    }

    dedup(pixels)
}

/// The pixels of the polyline through `points`, with no pixel lit twice where
/// two segments meet.
pub fn polyline(points: &[Point<i32>]) -> Vec<Point<i32>> {
    let mut pixels: Vec<Point<i32>> = Vec::new();

    for pair in points.windows(2) {
        let segment = line(pair[0], pair[1]);
        let skip = usize::from(!pixels.is_empty());

        pixels.extend(segment.into_iter().skip(skip));
    }

    if pixels.is_empty() {
        pixels.extend(points.first());
    }

    pixels
}

/// The strips of a solid 45° triangle pointing `towards`, with its tip at
/// `tip` and `size` pixels from the tip to its base.
///
/// The base is `2 × size + 1` pixels long, so the tip is its middle pixel.
pub fn triangle(tip: Point<i32>, towards: Direction, size: i32) -> Vec<Rectangle<i32>> {
    (0..=size.max(0))
        .map(|i| match towards {
            Direction::Right => Rectangle {
                x: tip.x - i,
                y: tip.y - i,
                width: 1,
                height: 2 * i + 1,
            },
            Direction::Left => Rectangle {
                x: tip.x + i,
                y: tip.y - i,
                width: 1,
                height: 2 * i + 1,
            },
            Direction::Down => Rectangle {
                x: tip.x - i,
                y: tip.y - i,
                width: 2 * i + 1,
                height: 1,
            },
            Direction::Up => Rectangle {
                x: tip.x - i,
                y: tip.y + i,
                width: 2 * i + 1,
                height: 1,
            },
        })
        .collect()
}

/// One of the four directions of the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Towards the top.
    Up,
    /// Towards the right.
    Right,
    /// Towards the bottom.
    Down,
    /// Towards the left.
    Left,
}

/// Merges pixels into runs: horizontal runs along rows, or vertical runs down
/// columns when `vertical` is set.
///
/// Adjacent pixels in the order given become one rectangle, so a shallow line
/// costs a few rectangles instead of one per pixel.
pub fn runs(pixels: &[Point<i32>], vertical: bool) -> Vec<Rectangle<i32>> {
    let mut runs: Vec<Rectangle<i32>> = Vec::new();

    for pixel in pixels {
        if let Some(run) = runs.last_mut() {
            if !vertical && run.y == pixel.y {
                if pixel.x == run.x + run.width {
                    run.width += 1;
                    continue;
                }

                if pixel.x == run.x - 1 {
                    run.x -= 1;
                    run.width += 1;
                    continue;
                }
            }

            if vertical && run.x == pixel.x {
                if pixel.y == run.y + run.height {
                    run.height += 1;
                    continue;
                }

                if pixel.y == run.y - 1 {
                    run.y -= 1;
                    run.height += 1;
                    continue;
                }
            }
        }

        runs.push(Rectangle {
            x: pixel.x,
            y: pixel.y,
            width: 1,
            height: 1,
        });
    }

    runs
}

/// Walks the first octant of a midpoint circle, from the top down to the 45°
/// diagonal, calling `plot(x, y)` with `x >= y >= 0`.
fn octants(radius: i32, mut plot: impl FnMut(i32, i32)) {
    if radius < 0 {
        return;
    }

    let (mut x, mut y) = (radius, 0);
    let mut error = 1 - radius;

    while x >= y {
        plot(x, y);
        y += 1;

        if error < 0 {
            error += 2 * y + 1;
        } else {
            x -= 1;
            error += 2 * (y - x) + 1;
        }
    }
}

fn dedup(mut pixels: Vec<Point<i32>>) -> Vec<Point<i32>> {
    pixels.sort_by_key(|pixel| (pixel.y, pixel.x));
    pixels.dedup();
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dashed_circle_repeats_its_dash_evenly() {
        let full = circle(Point::new(0, 0), 10);
        let dashed = dashed_circle(Point::new(0, 0), 10, Dash::new(4, 4));

        assert!(dashed.iter().all(|pixel| full.contains(pixel)));
        assert!(dashed.len() * 3 > full.len());
        assert!(dashed.len() * 3 < full.len() * 2);

        // Mirror images across the vertical axis are lit alike, bar the
        // pixels where a dash ends.
        let symmetric = dashed
            .iter()
            .filter(|pixel| dashed.contains(&Point::new(-pixel.x, pixel.y)))
            .count();

        assert!(symmetric * 4 > dashed.len() * 3);
    }

    /// Renders `pixels` as art: `#` for a lit pixel, `.` otherwise.
    fn art(pixels: &[Point<i32>]) -> String {
        let left = pixels.iter().map(|p| p.x).min().unwrap_or(0);
        let right = pixels.iter().map(|p| p.x).max().unwrap_or(0);
        let top = pixels.iter().map(|p| p.y).min().unwrap_or(0);
        let bottom = pixels.iter().map(|p| p.y).max().unwrap_or(0);

        (top..=bottom)
            .map(|y| {
                (left..=right)
                    .map(|x| {
                        if pixels.contains(&Point::new(x, y)) {
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

    fn assert_art(pixels: &[Point<i32>], expected: &str) {
        let expected = expected
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let actual = art(pixels);

        assert_eq!(
            actual, expected,
            "\nexpected:\n{expected}\n\nactual:\n{actual}\n"
        );
    }

    #[test]
    fn a_shallow_line_is_one_pixel_per_column() {
        assert_art(
            &line(Point::new(0, 0), Point::new(7, 2)),
            "
            ##......
            ..####..
            ......##
            ",
        );
    }

    #[test]
    fn a_diagonal_is_a_perfect_stair() {
        assert_art(
            &line(Point::new(0, 3), Point::new(3, 0)),
            "
            ...#
            ..#.
            .#..
            #...
            ",
        );
    }

    #[test]
    fn a_circle_has_a_centre_pixel_and_whole_axes() {
        assert_art(
            &circle(Point::new(0, 0), 4),
            "
            ...###...
            .##...##.
            .#.....#.
            #.......#
            #.......#
            #.......#
            .#.....#.
            .##...##.
            ...###...
            ",
        );
    }

    #[test]
    fn a_disc_fills_its_circle() {
        let centre = Point::new(10, -3);
        let ring = circle(centre, 6);
        let rows = disc(centre, 6);

        for pixel in &ring {
            assert!(
                rows.iter()
                    .any(|&(y, from, to)| y == pixel.y && (from..=to).contains(&pixel.x)),
                "{pixel:?} of the ring is outside the disc"
            );
        }

        for &(y, from, to) in &rows {
            assert!(ring.contains(&Point::new(from, y)));
            assert!(ring.contains(&Point::new(to, y)));
        }
    }

    #[test]
    fn an_arc_keeps_the_pixels_between_its_bearings() {
        let centre = Point::new(0, 0);
        let upper = arc(centre, 5, 270.0, 90.0);

        assert!(upper.iter().all(|pixel| pixel.y <= 0));
        assert!(upper.contains(&Point::new(0, -5)));
        assert!(upper.contains(&Point::new(-5, 0)));
        assert!(upper.contains(&Point::new(5, 0)));
    }

    #[test]
    fn an_ellipse_is_a_flattened_ring() {
        let ellipse = ellipse(Point::new(0, 0), 6, 3);

        assert_art(
            &ellipse,
            "
            ...#######...
            .##.......##.
            #...........#
            #...........#
            #...........#
            .##.......##.
            ...#######...
            ",
        );
    }

    #[test]
    fn a_ring_is_solid_at_every_bearing() {
        let pixels = ring(Point::new(0, 0), 5, 2);

        assert_art(
            &pixels,
            "
            ...#####...
            ..#######..
            .##.....##.
            ##.......##
            ##.......##
            ##.......##
            ##.......##
            ##.......##
            .##.....##.
            ..#######..
            ...#####...
            ",
        );
    }

    #[test]
    fn a_polyline_lights_its_corners_once() {
        let pixels = polyline(&[Point::new(0, 0), Point::new(3, 0), Point::new(3, 2)]);

        assert_eq!(pixels.len(), 6);
        assert_art(
            &pixels,
            "
            ####
            ...#
            ...#
            ",
        );
    }

    #[test]
    fn runs_merge_neighbours_along_the_major_axis() {
        let shallow = line(Point::new(0, 0), Point::new(7, 2));
        let steep = line(Point::new(0, 0), Point::new(1, 9));

        assert_eq!(runs(&shallow, false).len(), 3);
        assert_eq!(runs(&steep, true).len(), 2);
    }

    #[test]
    fn bearings_run_clockwise_from_up() {
        assert_eq!(bearing(0, -1), 0.0);
        assert_eq!(bearing(1, 0), 90.0);
        assert_eq!(bearing(0, 1), 180.0);
        assert_eq!(bearing(-1, 0), 270.0);
        assert_eq!(polar(Point::new(0, 0), 90.0, 4.0), Point::new(4, 0));
    }
}
