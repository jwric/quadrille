//! A strip chart: a few readings over the last minute, the newest at the
//! right edge.
//!
//! The chart takes a reading once a column, at moments counted from a fixed
//! epoch rather than from now, so as time passes the traces step left a
//! whole column at a time instead of shimmering.
//!
//! It is the console's own, drawn from the toolkit's parts: its scales come
//! from `quadrille::scale`, its frame is kept in a `Memo`, and
//! `canvas_widget!` makes it a widget.
use std::ops::RangeInclusive;

use iced::widget::canvas::{self, Frame, Geometry};
use iced::{Length, Point, Rectangle, Renderer, mouse};
use quadrille::canvas::Memo;
use quadrille::draw::{Anchor, Dash, Pen, rectangle};
use quadrille::instrument::channel;
use quadrille::scale::{self, End};
use quadrille::{Face, Palette, Theme, px};

/// A reading the chart takes at a moment, in seconds.
type Reading<'a> = Box<dyn Fn(f32) -> f32 + 'a>;

/// A strip chart of readings over the `span` seconds up to `now`.
pub struct Trend<'a> {
    series: Vec<(&'static str, Reading<'a>)>,
    now: f32,
    range: RangeInclusive<f32>,
    unit: &'static str,
    span: f32,
    gutter: u16,
    width: Length,
    height: Length,
}

/// A strip chart of the `span` seconds up to `now`, of readings in `range`.
pub fn trend<'a>(now: f32, span: f32, range: RangeInclusive<f32>, unit: &'static str) -> Trend<'a> {
    Trend {
        series: Vec::new(),
        now,
        range,
        unit,
        span,
        gutter: 0,
        width: Length::Fill,
        height: Length::Fill,
    }
}

impl<'a> Trend<'a> {
    /// Adds a reading, taken at a moment given in seconds.
    pub fn series(mut self, name: &'static str, reading: impl Fn(f32) -> f32 + 'a) -> Self {
        self.series.push((name, Box::new(reading)));
        self
    }

    /// Sets the least width of the margin left of the screen, which holds
    /// the value labels: charts stacked with the same gutter line up.
    pub fn gutter(mut self, gutter: u16) -> Self {
        self.gutter = gutter;
        self
    }

    /// Sets the height of the chart.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Where the parts of the chart go in `width` × `height` pixels.
    fn plan(&self, width: i32, height: i32) -> Option<Plan> {
        let face = Face::BODY;
        let strip = i32::from(face.line()) + px::TIGHT as i32;
        let screen = height - 2 * strip;

        // A label every two lines at most, so they never crowd.
        let step = scale::step(
            self.range.end() - self.range.start(),
            (screen / (2 * i32::from(face.line()))).max(1) as u32,
        );
        let widest = divisions(&self.range, step)
            .iter()
            .map(|value| i32::from(face.width(&self.label(*value, step))))
            .max()
            .unwrap_or(0);
        let gutter = (widest + 4).max(i32::from(self.gutter));
        let frame = rectangle(gutter, strip, width - gutter, screen);

        (frame.width >= 3 && frame.height >= 3).then_some(Plan { frame, step })
    }

    /// The label of the value division at `value`.
    fn label(&self, value: f32, step: f32) -> String {
        let magnitude = self.range.start().abs().max(self.range.end().abs());

        scale::tick(value, step, magnitude)
    }

    /// The row of `value`, held inside the screen.
    fn row(&self, plan: &Plan, value: f32) -> i32 {
        let (low, high) = (*self.range.start(), *self.range.end());
        let fraction = if high == low {
            0.0
        } else {
            ((value - low) / (high - low)).clamp(0.0, 1.0)
        };

        plan.bottom() - (fraction * (plan.bottom() - plan.top()) as f32).round() as i32
    }

    /// The column of the moment `age` seconds ago.
    fn column(&self, plan: &Plan, age: f32) -> i32 {
        let last = plan.columns() - 1;

        plan.left() + last - ((age / self.span) * last as f32).round() as i32
    }

    /// The moment each column shows, the oldest first: one column apart,
    /// counted from a fixed epoch so that a moment keeps its value as the
    /// chart moves on.
    fn moments(&self, plan: &Plan) -> Vec<f32> {
        let last = plan.columns() - 1;
        let pitch = self.span / last.max(1) as f32;
        let newest = (self.now / pitch).floor();

        (0..=last)
            .map(|column| (newest - (last - column) as f32) * pitch)
            .collect()
    }

    fn chrome(&self, pen: &mut Pen<'_, Renderer>, plan: &Plan, width: i32, palette: &Palette) {
        let face = Face::BODY;
        let frame = plan.frame;
        let (left, right) = (plan.left(), frame.x + frame.width - 2);
        let cap = i32::from(face.cap());

        pen.fill(frame, palette.void);

        // Value divisions: a dotted rule each, labelled in the gutter where
        // the label clears the one below it.
        let mut clear = i32::MAX;

        for value in divisions(&self.range, plan.step) {
            let y = self.row(plan, value);

            if y > plan.top() && y < plan.bottom() {
                pen.dashed(
                    Point::new(left, y),
                    Point::new(right, y),
                    Dash::SPARSE,
                    palette.faint,
                );
            }

            let top = y - cap.div_euclid(2);

            if top >= 0 && top + cap < clear {
                pen.text(
                    face,
                    self.label(value, plan.step),
                    Point::new(frame.x - 4, y),
                    Anchor::RIGHT,
                    palette.muted,
                );
                clear = top - px::TIGHT as i32;
            }
        }

        // Time divisions: a dotted rule and a tick each, counted back from
        // the newest, and labelled below where the labels do not collide.
        let pitch = 8 * i32::from(face.advance());
        let every = scale::step(self.span, (frame.width / pitch).max(1) as u32);
        let mut labels = Vec::new();
        let mut age = 0.0;

        while age <= self.span + every * 1e-3 {
            let x = self.column(plan, age);

            if x > left && x < right {
                pen.dashed(
                    Point::new(x, plan.top()),
                    Point::new(x, plan.bottom() - 3),
                    Dash::SPARSE,
                    palette.faint,
                );
            }

            pen.vline(x, plan.bottom() - 2, plan.bottom(), palette.line);

            let text = if age == 0.0 {
                "NOW".to_owned()
            } else {
                format!("-{}s", scale::tick(age, every, self.span))
            };

            labels.push((x, text));
            age += every;
        }

        pen.outline(frame, palette.edge);

        // `place` keeps the last label first: hand them over oldest first,
        // so the newest is the one that is always kept.
        labels.reverse();

        let widths: Vec<_> = labels
            .iter()
            .map(|(x, text)| (*x, i32::from(face.width(text))))
            .collect();
        let placed = scale::place(&widths, frame.x..=width - 1, i32::from(face.advance()));
        let top = frame.y + frame.height + px::TIGHT as i32;

        for ((_, text), left) in labels.iter().zip(placed) {
            if let Some(left) = left {
                pen.text(
                    face,
                    text,
                    Point::new(left, top),
                    Anchor::TOP_LEFT,
                    palette.muted,
                );
            }
        }
    }

    /// Each reading as a trace of one run a column, the first on top.
    fn traces(&self, pen: &mut Pen<'_, Renderer>, plan: &Plan, palette: &Palette) {
        let moments = self.moments(plan);

        for (index, (_, reading)) in self.series.iter().enumerate().rev() {
            let color = channel(palette, index);
            let mut runs: Vec<(i32, i32)> = moments
                .iter()
                .map(|moment| {
                    let row = self.row(plan, reading(*moment));

                    (row, row)
                })
                .collect();

            scale::bridge(&mut runs);

            for (column, (top, bottom)) in runs.into_iter().enumerate() {
                let (top, bottom) = (top.max(plan.top()), bottom.min(plan.bottom()));

                if top <= bottom {
                    pen.vline(plan.left() + column as i32, top, bottom, color);
                }
            }
        }
    }

    /// The name and newest value of each reading, along the top, in the
    /// reading's colour; a name that does not fit is left off.
    fn legend(&self, pen: &mut Pen<'_, Renderer>, plan: &Plan, palette: &Palette) {
        let face = Face::BODY;
        let advance = i32::from(face.advance());
        let frame = plan.frame;

        let newest = self.moments(plan).last().copied().unwrap_or(self.now);
        let entries: Vec<(String, String)> = self
            .series
            .iter()
            .map(|(name, reading)| {
                (
                    (*name).to_owned(),
                    format!("{:.1}{}", reading(newest), self.unit),
                )
            })
            .collect();

        let row: Vec<_> = entries
            .iter()
            .map(|(name, value)| {
                (
                    i32::from(face.width(name) + face.width(value)) + advance,
                    End::Start,
                )
            })
            .collect();
        let placed = scale::row(&row, frame.x..=frame.x + frame.width - 1, 2 * advance);

        for (index, ((name, value), left)) in entries.into_iter().zip(placed).enumerate() {
            if let Some(left) = left {
                let after = left + i32::from(face.width(&name)) + advance;

                pen.text(
                    face,
                    name,
                    Point::new(left, 0),
                    Anchor::TOP_LEFT,
                    channel(palette, index),
                );
                pen.text(
                    face,
                    value,
                    Point::new(after, 0),
                    Anchor::TOP_LEFT,
                    palette.ink,
                );
            }
        }
    }
}

/// The multiples of `step` inside `range`, from the bottom.
fn divisions(range: &RangeInclusive<f32>, step: f32) -> Vec<f32> {
    let first = (range.start() / step).ceil() as i32;
    let last = (range.end() / step).floor() as i32;

    (first..=last).map(|k| k as f32 * step).collect()
}

/// Where the parts of a [`Trend`] go in its bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    frame: Rectangle<i32>,
    step: f32,
}

impl Plan {
    fn left(&self) -> i32 {
        self.frame.x + 1
    }

    fn top(&self) -> i32 {
        self.frame.y + 1
    }

    fn bottom(&self) -> i32 {
        self.frame.y + self.frame.height - 2
    }

    fn columns(&self) -> i32 {
        self.frame.width - 2
    }
}

/// What the frame of a [`Trend`] is drawn from.
type Chrome = (Palette, Plan, RangeInclusive<f32>, u32);

impl<Message> canvas::Program<Message, Theme> for Trend<'_> {
    type State = Memo<Chrome, Renderer>;

    fn draw(
        &self,
        chrome: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let palette = theme.palette();
        let size = px::floor(bounds.size());

        let Some(plan) = self.plan(size.width, size.height) else {
            return Vec::new();
        };

        let key = (*palette, plan, self.range.clone(), self.span.to_bits());

        let chrome = chrome.draw(renderer, bounds.size(), key, |frame| {
            self.chrome(&mut Pen::new(frame), &plan, size.width, palette);
        });

        let mut frame = Frame::new(renderer, bounds.size());

        {
            let mut pen = Pen::new(&mut frame);

            self.traces(&mut pen, &plan, palette);
            self.legend(&mut pen, &plan, palette);
        }

        vec![chrome, frame.into_geometry()]
    }
}

quadrille::canvas_widget!(Trend<'a>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_moment_is_the_last_column() {
        let chart = trend(100.0, 60.0, 22.0..=30.0, "V");
        let plan = chart.plan(300, 120).unwrap();

        assert_eq!(chart.column(&plan, 0.0), plan.left() + plan.columns() - 1);
        assert_eq!(chart.column(&plan, 60.0), plan.left());
    }

    #[test]
    fn a_moment_keeps_its_column_until_a_whole_column_has_passed() {
        let plan = trend(0.0, 60.0, 0.0..=1.0, "").plan(300, 120).unwrap();
        let pitch = 60.0 / (plan.columns() - 1) as f32;
        let at = |now: f32| trend(now, 60.0, 0.0..=1.0, "").moments(&plan);

        assert_eq!(at(100.25 * pitch), at(100.75 * pitch));
        assert_eq!(
            at(100.5 * pitch)[1..],
            at(101.5 * pitch)[..at(0.0).len() - 1]
        );
    }

    #[test]
    fn values_are_held_inside_the_screen() {
        let chart = trend(100.0, 60.0, 22.0..=30.0, "V");
        let plan = chart.plan(300, 120).unwrap();

        assert_eq!(chart.row(&plan, 30.0), plan.top());
        assert_eq!(chart.row(&plan, 40.0), plan.top());
        assert_eq!(chart.row(&plan, 22.0), plan.bottom());
        assert_eq!(
            divisions(&(22.0..=30.0), 2.0),
            vec![22.0, 24.0, 26.0, 28.0, 30.0]
        );
    }
}
