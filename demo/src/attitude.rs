//! The attitude globe: a sphere of latitude and longitude lines turned by the
//! craft's roll, pitch and heading, under a fixed reticle.
use graticule::draw::{Anchor, Direction, Pen, rectangle, shape};
use graticule::{Face, Theme, face};
use iced::widget::canvas::{self, Frame, Geometry};
use iced::{Point, Rectangle, Renderer, mouse};

use crate::Telemetry;

/// The globe for one reading.
pub struct Attitude {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Attitude {
    pub fn of(telemetry: &Telemetry) -> Self {
        Self {
            roll: telemetry.roll,
            pitch: telemetry.pitch,
            yaw: telemetry.yaw,
        }
    }

    /// Where the point at `latitude` and `longitude` lands, relative to the
    /// centre of a globe of `radius`, and whether it faces the viewer.
    fn project(&self, latitude: f32, longitude: f32, radius: f32) -> (f32, f32, bool) {
        let (latitude, longitude) = (latitude.to_radians(), (longitude - self.yaw).to_radians());

        let (x, y, z) = (
            latitude.cos() * longitude.sin(),
            latitude.sin(),
            latitude.cos() * longitude.cos(),
        );

        // Pitch tilts the globe about the horizontal axis, roll turns the
        // picture about the line of sight.
        let pitch = (-self.pitch).to_radians();
        let (y, z) = (
            y * pitch.cos() - z * pitch.sin(),
            y * pitch.sin() + z * pitch.cos(),
        );

        let roll = self.roll.to_radians();
        let (x, y) = (
            x * roll.cos() - y * roll.sin(),
            x * roll.sin() + y * roll.cos(),
        );

        (x * radius, -y * radius, z > 0.0)
    }

    /// Draws the curve of `points` (latitude, longitude pairs) where it faces
    /// the viewer.
    fn trace(
        &self,
        pen: &mut Pen<'_, Renderer>,
        centre: Point<i32>,
        radius: f32,
        points: impl Iterator<Item = (f32, f32)>,
        color: iced::Color,
    ) {
        let mut run = Vec::new();

        for (latitude, longitude) in points {
            let (x, y, front) = self.project(latitude, longitude, radius);

            if front {
                run.push(Point::new(centre.x as f32 + x, centre.y as f32 + y));
            } else if !run.is_empty() {
                pen.curve(run.drain(..), color);
            }
        }

        pen.curve(run, color);
    }
}

impl<Message> canvas::Program<Message, Theme> for Attitude {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let palette = theme.palette();
        let mut frame = Frame::new(renderer, bounds.size());

        {
            let (width, height) = (bounds.width as i32, bounds.height as i32);
            let centre = Point::new(width.div_euclid(2), height.div_euclid(2));
            let radius = (width.min(height) / 2 - 16).max(16);
            let globe = (radius - 3) as f32;

            let mut pen = Pen::new(&mut frame);

            // A fixed field of stars, turning with the heading.
            for i in 0..90u32 {
                let hash = i.wrapping_mul(2_654_435_761);
                let bearing = (hash % 3600) as f32 / 10.0 + self.yaw * 0.5;
                let distance = (radius + 6) as f32
                    + ((hash >> 12) % 400) as f32 / 400.0 * (width.max(height) / 2) as f32;
                let star = shape::polar(centre, bearing, distance);

                if (0..width).contains(&star.x) && (0..height).contains(&star.y) {
                    pen.pixel(
                        star,
                        if hash % 7 == 0 {
                            palette.muted
                        } else {
                            palette.faint
                        },
                    );
                }
            }

            // Parallels and meridians every thirty degrees; the equator in
            // the accent, since it is the horizon.
            for latitude in [-60.0, -30.0, 30.0, 60.0] {
                let points = (0..=120).map(move |i| (latitude, -180.0 + 3.0 * i as f32));

                self.trace(&mut pen, centre, globe, points, palette.line);
            }

            for longitude in (0..12).map(|i| i as f32 * 30.0) {
                let points = (0..=60).map(move |i| (-90.0 + 3.0 * i as f32, longitude));

                self.trace(&mut pen, centre, globe, points, palette.line);
            }

            let equator = (0..=180).map(|i| (0.0, -180.0 + 2.0 * i as f32));
            self.trace(&mut pen, centre, globe, equator, palette.accent);

            // The rim, and the roll scale above it.
            pen.ring(centre, radius, 2, palette.accent);

            for tick in [-60, -45, -30, -20, -10, 0, 10, 20, 30, 45, 60] {
                let length = if tick % 30 == 0 { 6 } else { 3 };
                let bearing = tick as f32;
                let inner = shape::polar(centre, bearing, (radius + 3) as f32);
                let outer = shape::polar(centre, bearing, (radius + 2 + length) as f32);

                pen.line(inner, outer, palette.muted);
            }

            let pointer = shape::polar(centre, self.roll, (radius - 4) as f32);
            pen.arrowhead(pointer, Direction::Up, 2, palette.ink);

            // The reticle: a crosshair with a dark centre, and the craft's
            // wings either side of it.
            pen.crosshair(centre, 8, 3, palette.ink);
            pen.pixel(centre, palette.ink);
            pen.hline(centre.x - radius + 8, centre.x - 20, centre.y, palette.ink);
            pen.hline(centre.x + 20, centre.x + radius - 8, centre.y, palette.ink);

            // Chevron brackets out past the rim.
            let reach = radius * 2 / 3;

            for side in [-1, 1] {
                let x = centre.x + side * (radius + 14);
                let inward = -side * 6;

                pen.polyline(
                    &[
                        Point::new(x + inward, centre.y - reach),
                        Point::new(x, centre.y - reach + 6),
                        Point::new(x, centre.y + reach - 6),
                        Point::new(x + inward, centre.y + reach),
                    ],
                    palette.accent,
                );
                pen.hline(x, x + side * 8, centre.y, palette.accent);
            }

            // Attitude in a box, top right.
            let face = Face::BODY;
            let lines = [
                format!("R {:+04.0}", self.roll),
                format!("P {:+04.0}", self.pitch),
                format!("Y {:03.0}", self.yaw),
            ];

            let box_width = i32::from(face.width(&lines[0])) + 10;
            let box_height = 3 * i32::from(face.line()) + 8;
            let corner = Point::new(width - box_width - 1, 1);

            pen.outline(
                rectangle(corner.x, corner.y, box_width, box_height),
                palette.accent,
            );

            for (i, line) in lines.iter().enumerate() {
                let top = corner.y + 4 + i as i32 * i32::from(face.line());

                pen.text(
                    face,
                    line,
                    Point::new(corner.x + 5, top),
                    Anchor::TOP_LEFT,
                    palette.ink,
                );
            }

            // The autopilot's mode, knocked out of the accent.
            let mode = face::spaced("AUTO");
            let mode_width = i32::from(face.width(&mode)) + 6;
            let mode_top = centre.y + radius / 2;

            pen.fill(
                rectangle(
                    centre.x - mode_width / 2,
                    mode_top,
                    mode_width,
                    i32::from(face.line()),
                ),
                palette.accent,
            );
            pen.text(
                face,
                mode,
                Point::new(centre.x, mode_top),
                Anchor::TOP,
                palette.on_accent,
            );
        }

        vec![frame.into_geometry()]
    }
}
