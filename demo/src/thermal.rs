//! The hull's temperatures: its skin unrolled, the way round it from left to
//! right and the nose at the top, in the steps of the palette's ramp.
//!
//! A temperature between two steps is dithered between them with the 4 × 4
//! ordered pattern, anchored to the skin, so a gradient reads as a gradient
//! and the picture is still made of the ramp's colours alone.
//!
//! The picture is a `quadrille::draw::Raster`, kept in a `Memo` until the
//! readings change; the scales round it come from `quadrille::scale`.
use std::f32::consts::TAU;
use std::ops::RangeInclusive;

use iced::widget::canvas::{self, Frame, Geometry};
use iced::{Length, Point, Rectangle, Renderer, mouse};
use quadrille::canvas::Memo;
use quadrille::draw::{Anchor, Direction, Horizontal, Pattern, Pen, Raster, Vertical, rectangle};
use quadrille::theme::Ramp;
use quadrille::{Face, Palette, Theme, px, scale};

use crate::Telemetry;

/// The temperatures the ramp spans, in °C.
pub const RANGE: RangeInclusive<f32> = -150.0..=150.0;

/// The width of the gutter left of the map, which holds `FWD` and `AFT`.
pub const GUTTER: u16 = 22;

/// Where the equipment bay sits on the skin: its bearing in degrees and its
/// station from the nose, as a fraction of the length.
const BAY: (f32, f32) = (200.0, 0.4);

/// The radiators: two panels a quarter turn either side of the bay's
/// opposite, their bearings and the stations they run between.
const RADIATORS: [f32; 2] = [110.0, 290.0];
const PANEL: (f32, f32) = (0.18, 0.72);

/// The temperature of the skin at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Field {
    /// The bearing round the hull the sun stands over, in degrees.
    pub sun: f32,
    /// The temperature of the skin in shadow, in °C.
    pub shade: f32,
    /// The main engine's thrust, as a fraction of rated.
    pub thrust: f32,
}

impl Field {
    /// The skin as the telemetry finds it: the craft turns under the sun as
    /// it yaws.
    pub fn of(telemetry: &Telemetry) -> Self {
        Self {
            sun: (360.0 - telemetry.yaw).rem_euclid(360.0),
            shade: telemetry.temperature.1,
            thrust: telemetry.thrust,
        }
    }

    /// The temperature at `bearing` degrees round the hull and `station`,
    /// from 0 at the nose to 1 at the engine.
    pub fn at(&self, bearing: f32, station: f32) -> f32 {
        let off = |a: f32, b: f32| {
            let d = (a - b).rem_euclid(360.0);

            d.min(360.0 - d)
        };

        let lit = ((bearing - self.sun) / 360.0 * TAU).cos().max(0.0);
        let sun = 160.0 * lit;
        let engine = 140.0 * self.thrust * (-(1.0 - station) / 0.14).exp();
        let bay = 45.0
            * (-(off(bearing, BAY.0) / 26.0).powi(2)).exp()
            * (-((station - BAY.1) / 0.12).powi(2)).exp();
        let radiator = RADIATORS
            .iter()
            .any(|centre| off(bearing, *centre) < 14.0)
            .then_some(-35.0)
            .filter(|_| (PANEL.0..=PANEL.1).contains(&station))
            .unwrap_or(0.0);

        self.shade + sun + engine + bay + radiator
    }

    /// The temperature at the middle of the equipment bay.
    pub fn bay(&self) -> f32 {
        self.at(BAY.0, BAY.1)
    }

    /// The mean temperature of the skin round the engine.
    pub fn aft(&self) -> f32 {
        const BEARINGS: usize = 72;

        (0..BEARINGS)
            .map(|i| self.at(360.0 * (i as f32 + 0.5) / BEARINGS as f32, 1.0))
            .sum::<f32>()
            / BEARINGS as f32
    }

    /// The hottest, the coldest and the mean temperature over the skin, and
    /// the bearing of the hottest.
    pub fn survey(&self) -> Survey {
        const BEARINGS: usize = 120;
        const STATIONS: usize = 40;

        let mut survey = Survey {
            hottest: f32::NEG_INFINITY,
            bearing: 0.0,
            coldest: f32::INFINITY,
            mean: 0.0,
        };

        for i in 0..BEARINGS {
            let bearing = 360.0 * (i as f32 + 0.5) / BEARINGS as f32;

            for j in 0..STATIONS {
                let temperature = self.at(bearing, (j as f32 + 0.5) / STATIONS as f32);

                if temperature > survey.hottest {
                    survey.hottest = temperature;
                    survey.bearing = bearing;
                }

                survey.coldest = survey.coldest.min(temperature);
                survey.mean += temperature;
            }
        }

        survey.mean /= (BEARINGS * STATIONS) as f32;
        survey
    }
}

/// A [`Field`] summed up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Survey {
    pub hottest: f32,
    pub bearing: f32,
    pub coldest: f32,
    pub mean: f32,
}

/// A map of the hull's temperatures.
pub struct Thermal {
    field: Field,
    width: Length,
    height: Length,
}

/// The map of `field`.
pub fn thermal(field: Field) -> Thermal {
    Thermal {
        field,
        width: Length::Fill,
        height: Length::Fill,
    }
}

/// Where the parts of a [`Thermal`] map go in its bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    /// The frame round the picture.
    frame: Rectangle<i32>,
    /// The frame round the ramp's scale, right of the picture and as tall.
    key: Rectangle<i32>,
}

impl Plan {
    fn fit(width: i32, height: i32) -> Option<Self> {
        let face = Face::BODY;
        let strip = i32::from(face.line()) + px::TIGHT as i32;
        let gutter = i32::from(GUTTER);
        let labels = i32::from(face.width("-150"));
        let key = rectangle(width - labels - 4 - 8, strip, 8, height - 2 * strip);
        let frame = rectangle(gutter, strip, key.x - px::WIDE as i32 - gutter, key.height);

        (frame.width >= 3 && frame.height >= 3).then_some(Self { frame, key })
    }

    /// The pixels of the picture inside the frame.
    fn inside(&self) -> Rectangle<i32> {
        rectangle(
            self.frame.x + 1,
            self.frame.y + 1,
            self.frame.width - 2,
            self.frame.height - 2,
        )
    }

    /// The column of `bearing`.
    fn column(&self, bearing: f32) -> i32 {
        let inside = self.inside();

        inside.x + ((bearing / 360.0) * inside.width as f32).floor() as i32
    }
}

impl Thermal {
    /// The picture: a pixel for each bearing and station, dithered between
    /// the two steps its temperature falls between.
    fn picture(&self, plan: &Plan, ramp: &Ramp) -> Raster {
        let inside = plan.inside();
        let (low, high) = (*RANGE.start(), *RANGE.end());
        let (width, height) = (inside.width as u32, inside.height as u32);
        let steps = ramp.steps();
        let last = steps.len() - 1;

        Raster::from_fn(width, height, |x, y| {
            let bearing = 360.0 * (x as f32 + 0.5) / width as f32;
            let station = (y as f32 + 0.5) / height as f32;
            let level = ((self.field.at(bearing, station) - low) / (high - low)).clamp(0.0, 1.0);

            let position = level * last as f32;
            let below = position.floor();
            let share = ((position - below) * 16.0).round() as u8;
            let up = Pattern::Bayer(share).lights(x as i32, y as i32);

            steps[(below as usize + usize::from(up)).min(last)]
        })
    }

    fn chrome(&self, pen: &mut Pen<'_, Renderer>, plan: &Plan, palette: &Palette) {
        let face = Face::BODY;
        let frame = plan.frame;
        let inside = plan.inside();
        let bottom = frame.y + frame.height - 1;

        pen.outline(frame, palette.edge);

        // The ends of the hull, in the gutter.
        pen.text(
            face,
            "FWD",
            Point::new(frame.x - 4, inside.y),
            Anchor::new(Horizontal::Right, Vertical::CapTop),
            palette.muted,
        );
        pen.text(
            face,
            "AFT",
            Point::new(frame.x - 4, bottom),
            Anchor::new(Horizontal::Right, Vertical::Baseline),
            palette.muted,
        );

        // Bearings: a tick below the frame every quarter turn, labelled
        // where the labels do not collide.
        let mut labels = Vec::new();

        for quarter in 0..=4 {
            let bearing = 90.0 * quarter as f32;
            let x = plan.column(bearing).min(inside.x + inside.width - 1);

            pen.vline(x, bottom + 1, bottom + 2, palette.line);
            labels.push((x, format!("{}°", bearing as i32)));
        }

        let widths: Vec<_> = labels
            .iter()
            .map(|(x, text)| (*x, i32::from(face.width(text))))
            .collect();
        let placed = scale::place(
            &widths,
            frame.x..=frame.x + frame.width - 1,
            i32::from(face.advance()),
        );

        for ((_, text), left) in labels.iter().zip(placed) {
            if let Some(left) = left {
                pen.text(
                    face,
                    text,
                    Point::new(left, bottom + 1 + px::TIGHT as i32),
                    Anchor::TOP_LEFT,
                    palette.muted,
                );
            }
        }

        self.key(pen, plan, palette);
    }

    /// The ramp's scale: a block of each step from the coldest at the
    /// bottom, and the temperatures of a few of the steps' edges beside it.
    fn key(&self, pen: &mut Pen<'_, Renderer>, plan: &Plan, palette: &Palette) {
        let face = Face::BODY;
        let key = plan.key;
        let blocks = rectangle(key.x + 1, key.y + 1, key.width - 2, key.height - 2);
        let ramp = Ramp::of(palette);
        let steps = ramp.steps();
        let (low, high) = (*RANGE.start(), *RANGE.end());
        let edge = |fraction: f32| {
            blocks.y + blocks.height - (fraction * blocks.height as f32).round() as i32
        };

        for (index, color) in steps.iter().enumerate() {
            let below = edge(index as f32 / steps.len() as f32);
            let above = edge((index + 1) as f32 / steps.len() as f32);

            pen.fill(
                rectangle(blocks.x, above, blocks.width, below - above),
                *color,
            );
        }

        pen.outline(key, palette.edge);

        let cap = i32::from(face.cap());
        let step = scale::step(high - low, (key.height / (2 * cap)).max(1) as u32);
        let first = (low / step).ceil() as i32;
        let last = (high / step).floor() as i32;

        // From the top down, the order `place` takes them along the column.
        let marks: Vec<(i32, String)> = (first..=last)
            .rev()
            .map(|k| {
                let value = k as f32 * step;

                (edge((value - low) / (high - low)), format!("{value:+.0}"))
            })
            .collect();

        let heights: Vec<_> = marks.iter().map(|(y, _)| (*y, cap)).collect();
        let placed = scale::place(&heights, key.y..=key.y + key.height - 1, px::GAP as i32);

        for ((y, text), top) in marks.into_iter().zip(placed) {
            if let Some(top) = top {
                pen.hline(key.x + key.width, key.x + key.width + 1, y, palette.line);
                pen.text(
                    face,
                    if text == "+0" { "0".to_owned() } else { text },
                    Point::new(key.x + key.width + 4, top),
                    Anchor::new(Horizontal::Left, Vertical::CapTop),
                    palette.muted,
                );
            }
        }
    }

    /// The sun's bearing, marked above the frame.
    fn sun(&self, pen: &mut Pen<'_, Renderer>, plan: &Plan, palette: &Palette) {
        let face = Face::BODY;
        let x = plan.column(self.field.sun);
        let top = plan.frame.y - 1;
        let label = "SUN";
        let half = i32::from(face.width(label)).div_euclid(2);
        let left = (x - half).clamp(plan.frame.x, plan.frame.x + plan.frame.width - 2 * half);

        pen.arrowhead(Point::new(x, top), Direction::Down, 2, palette.accent);
        pen.text(
            face,
            label,
            Point::new(left, top - 3),
            Anchor::new(Horizontal::Left, Vertical::Baseline),
            palette.accent,
        );
    }
}

/// What the picture of a [`Thermal`] map is drawn from.
type Picture = (Field, Palette, Plan);

/// The state of a [`Thermal`] map: its frame and its picture, each drawn
/// again only when what it shows has changed.
pub struct State {
    chrome: Memo<(Palette, Plan), Renderer>,
    picture: Memo<Picture, Renderer>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            chrome: Memo::new(),
            picture: Memo::new(),
        }
    }
}

impl<Message> canvas::Program<Message, Theme> for Thermal {
    type State = State;

    fn draw(
        &self,
        state: &State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let palette = *theme.palette();
        let size = px::floor(bounds.size());

        let Some(plan) = Plan::fit(size.width, size.height) else {
            return Vec::new();
        };

        let picture = state.picture.draw(
            renderer,
            bounds.size(),
            (self.field, palette, plan),
            |frame| {
                let inside = plan.inside();

                Pen::new(frame).raster(
                    &self.picture(&plan, &Ramp::of(&palette)),
                    Point::new(inside.x, inside.y),
                );
            },
        );

        let chrome = state
            .chrome
            .draw(renderer, bounds.size(), (palette, plan), |frame| {
                self.chrome(&mut Pen::new(frame), &plan, &palette);
            });

        let mut frame = Frame::new(renderer, bounds.size());

        self.sun(&mut Pen::new(&mut frame), &plan, &palette);

        vec![picture, chrome, frame.into_geometry()]
    }
}

quadrille::canvas_widget!(Thermal);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sunlit_side_is_warmer_than_the_shade() {
        let field = Field {
            sun: 90.0,
            shade: -120.0,
            thrust: 0.0,
        };

        assert!(field.at(90.0, 0.2) > field.at(270.0, 0.2) + 100.0);
        assert!((field.at(270.0, 0.2) + 120.0).abs() < 0.5);
    }

    #[test]
    fn the_engine_heats_the_aft_end() {
        let field = Field {
            sun: 0.0,
            shade: -120.0,
            thrust: 0.8,
        };

        // Opposite the equipment bay, so that only the engine differs.
        assert!(field.at(20.0, 1.0) > field.at(20.0, 0.5) + 100.0);
    }

    #[test]
    fn the_same_second_draws_the_same_skin() {
        let a = Field::of(&Telemetry::at(42.0)).survey();
        let b = Field::of(&Telemetry::at(42.0)).survey();

        assert_eq!(a, b);
        assert!(a.coldest < a.mean && a.mean < a.hottest);
    }
}
