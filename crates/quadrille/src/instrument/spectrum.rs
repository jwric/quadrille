use std::borrow::Cow;
use std::ops::RangeInclusive;

use iced_widget::canvas::{self, Frame, Geometry};
use iced_widget::core::{Length, Point, Rectangle, mouse};
use iced_widget::graphics::geometry;

use super::GUTTER;
use crate::canvas::Memo;
use crate::draw::{Anchor, Dash, Direction, Horizontal, Pen, Vertical, rectangle};
use crate::scale::{self, End};
use crate::theme::mix;
use crate::{Face, Palette, Theme, px};

/// The size of a marker's pointer, from its tip to its base.
const POINTER: i32 = 2;
/// The tone of the lowest band of the fill, from the void to the live
/// colour, and how much each band above it adds.
const FILL: (f32, f32) = (0.12, 0.06);

/// A spectrum: a level for each bin across a span of frequencies, drawn as
/// a trace with a stepped fill under it.
///
/// The fill is banded by the level divisions, each band a step closer to
/// the live colour than the one below. A [`hold`](Self::hold) draws each
/// bin's held peak as a dotted mark above the trace; [`markers`] point at a
/// frequency and read out its level above the plot. Frequency labels are
/// left off where they would collide; their ticks stay.
///
/// [`markers`]: Self::marker
pub struct Spectrum<'a> {
    levels: Cow<'a, [f32]>,
    hold: Option<Cow<'a, [f32]>>,
    range: RangeInclusive<f32>,
    step: f32,
    span: RangeInclusive<f32>,
    unit: &'static str,
    markers: Vec<f32>,
    fill: bool,
    gutter: u16,
    width: Length,
    height: Length,
}

/// Creates a [`Spectrum`] of `levels`, in dB from -100 to 0.
pub fn spectrum<'a>(levels: impl Into<Cow<'a, [f32]>>) -> Spectrum<'a> {
    let levels = levels.into();
    let bins = levels.len() as f32;

    Spectrum {
        levels,
        hold: None,
        range: -100.0..=0.0,
        step: 20.0,
        span: 0.0..=bins,
        unit: "Hz",
        markers: Vec::new(),
        fill: true,
        gutter: GUTTER,
        width: Length::Fill,
        height: Length::Fill,
    }
}

impl<'a> Spectrum<'a> {
    /// Sets the levels from the bottom of the plot to the top, and the dB
    /// between two level divisions.
    pub fn range(mut self, range: RangeInclusive<f32>, step: f32) -> Self {
        self.range = range;
        self.step = step.abs().max(f32::EPSILON);
        self
    }

    /// Sets the frequencies the bins span, from the low edge of the first
    /// to the high edge of the last, and their unit.
    pub fn span(mut self, span: RangeInclusive<f32>, unit: &'static str) -> Self {
        self.span = span;
        self.unit = unit;
        self
    }

    /// Marks each bin's held peak.
    pub fn hold(mut self, hold: impl Into<Cow<'a, [f32]>>) -> Self {
        self.hold = Some(hold.into());
        self
    }

    /// Adds a marker at `frequency`, numbered in the order added.
    pub fn marker(mut self, frequency: f32) -> Self {
        self.markers.push(frequency);
        self
    }

    /// Sets whether the area under the trace is filled.
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// Sets the width of the margin left of the plot, where the levels are
    /// labelled.
    pub fn gutter(mut self, gutter: u16) -> Self {
        self.gutter = gutter;
        self
    }

    /// Sets the width of the [`Spectrum`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Spectrum`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    fn layout(&self, width: i32, height: i32) -> Option<Plan> {
        let strip = i32::from(Face::BODY.line()) + px::TIGHT as i32;
        let gutter = i32::from(self.gutter).min(width);
        let header = if self.markers.is_empty() { 0 } else { strip };
        let footer = if height - header - strip >= 16 {
            strip
        } else {
            0
        };
        let frame = rectangle(gutter, header, width - gutter, height - header - footer);

        (frame.width >= 3 && frame.height >= 3).then_some(Plan {
            frame,
            header: header > 0,
            footer: footer > 0,
        })
    }
}

/// Where the parts of a [`Spectrum`] go in its bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Plan {
    frame: Rectangle<i32>,
    header: bool,
    footer: bool,
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

    fn columns(&self) -> usize {
        (self.frame.width - 2) as usize
    }
}

impl Spectrum<'_> {
    /// The row of `level`, held inside the plot.
    fn row(&self, plan: &Plan, level: f32) -> i32 {
        let (low, high) = (*self.range.start(), *self.range.end());
        let fraction = if high == low {
            0.0
        } else {
            ((level - low) / (high - low)).clamp(0.0, 1.0)
        };

        plan.bottom() - (fraction * (plan.bottom() - plan.top()) as f32).round() as i32
    }

    /// The column `frequency` falls in, if it is inside the span.
    fn column(&self, plan: &Plan, frequency: f32) -> Option<i32> {
        let (start, end) = (*self.span.start(), *self.span.end());
        let fraction = (frequency - start) / (end - start);

        (0.0..=1.0).contains(&fraction).then(|| {
            let column = (fraction * plan.columns() as f32) as i32;

            plan.left() + column.min(plan.columns() as i32 - 1)
        })
    }

    /// The level of the bin `frequency` falls in.
    fn level(&self, frequency: f32) -> f32 {
        let (start, end) = (*self.span.start(), *self.span.end());
        let count = self.levels.len();
        let bin = ((frequency - start) / (end - start) * count as f32) as usize;

        self.levels
            .get(bin.min(count.saturating_sub(1)))
            .copied()
            .unwrap_or(f32::NEG_INFINITY)
    }

    /// The highest level in each column.
    fn peaks(&self, levels: &[f32], columns: usize) -> Vec<f32> {
        (0..columns)
            .map(|column| {
                levels[scale::bins(column, columns, levels.len())]
                    .iter()
                    .fold(f32::NEG_INFINITY, |peak, &level| peak.max(level))
            })
            .collect()
    }

    /// The level divisions inside the range, from the bottom.
    fn divisions(&self) -> Vec<f32> {
        let (low, high) = (*self.range.start(), *self.range.end());
        let first = (low / self.step).ceil() as i32;
        let last = (high / self.step).floor() as i32;

        (first..=last).map(|k| k as f32 * self.step).collect()
    }

    fn chrome<Renderer: geometry::Renderer>(
        &self,
        pen: &mut Pen<'_, Renderer>,
        plan: &Plan,
        width: i32,
        palette: &Palette,
    ) {
        let face = Face::BODY;
        let frame = plan.frame;
        let (left, right) = (plan.left(), frame.x + frame.width - 2);
        let cap = i32::from(face.cap());

        pen.fill(frame, palette.void);

        // Level divisions: a dotted rule each, labelled in the gutter where
        // the label fits whole and clear of the one below.
        let mut clear = i32::MAX;

        for level in self.divisions() {
            let y = self.row(plan, level);

            if y > plan.top() && y < plan.bottom() {
                pen.dashed(
                    Point::new(left, y),
                    Point::new(right, y),
                    Dash::SPARSE,
                    palette.faint,
                );
            }

            let text = format!("{level:.0}");
            let top = y - cap.div_euclid(2);
            let fits = i32::from(face.width(&text)) + 4 <= frame.x;

            if fits && top >= 0 && top + cap < clear {
                pen.text(
                    face,
                    text,
                    Point::new(frame.x - 4, y),
                    Anchor::RIGHT,
                    palette.muted,
                );
                clear = top - px::TIGHT as i32;
            }
        }

        // Frequency divisions: a tick up from the bottom edge and a dotted
        // rule each, labelled below where the labels do not collide.
        let (start, end) = (*self.span.start(), *self.span.end());
        let pitch = 8 * i32::from(face.advance());
        let step = scale::step(end - start, (frame.width / pitch).max(1) as u32);
        let magnitude = start.abs().max(end.abs());

        let mut labels = Vec::new();
        let mut value = (start / step).ceil() * step;

        while value <= end {
            if let Some(x) = self.column(plan, value) {
                if x > left && x < right {
                    pen.dashed(
                        Point::new(x, plan.top()),
                        Point::new(x, plan.bottom() - 3),
                        Dash::SPARSE,
                        palette.faint,
                    );
                }

                pen.vline(x, plan.bottom() - 2, plan.bottom(), palette.line);
                labels.push((x, scale::tick(value, step, magnitude)));
            }

            value += step;
        }

        pen.outline(frame, palette.edge);

        if plan.footer {
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
    }

    fn trace<Renderer: geometry::Renderer>(
        &self,
        pen: &mut Pen<'_, Renderer>,
        plan: &Plan,
        palette: &Palette,
    ) {
        let columns = plan.columns();

        if self.levels.is_empty() || columns == 0 {
            return;
        }

        let peaks = self.peaks(&self.levels, columns);
        let rows: Vec<i32> = peaks.iter().map(|level| self.row(plan, *level)).collect();
        let x = |column: usize| plan.left() + column as i32;

        if self.fill {
            let divisions = self.divisions();
            let mut edges: Vec<i32> = divisions
                .iter()
                .map(|level| self.row(plan, *level))
                .filter(|y| *y < plan.bottom())
                .collect();

            edges.insert(0, plan.bottom() + 1);

            for (band, pair) in edges
                .iter()
                .copied()
                .chain(std::iter::once(plan.top()))
                .collect::<Vec<_>>()
                .windows(2)
                .enumerate()
            {
                let (floor, ceiling) = (pair[0] - 1, pair[1]);
                let tone = mix(palette.void, palette.live, FILL.0 + FILL.1 * band as f32);

                for (column, row) in rows.iter().enumerate() {
                    let top = (row + 1).max(ceiling);

                    if top <= floor {
                        pen.vline(x(column), top, floor, tone);
                    }
                }
            }
        }

        if let Some(hold) = self.hold.as_deref().filter(|hold| !hold.is_empty()) {
            for (column, level) in self.peaks(hold, columns).into_iter().enumerate() {
                let y = self.row(plan, level);

                if column % 2 == 0 && y < rows[column] - 2 {
                    pen.pixel(Point::new(x(column), y), palette.accent);
                }
            }
        }

        let mut runs: Vec<(i32, i32)> = rows.iter().map(|row| (*row, *row)).collect();

        scale::bridge(&mut runs);

        for (column, (top, bottom)) in runs.into_iter().enumerate() {
            pen.vline(x(column), top, bottom, palette.live);
        }
    }

    fn markers<Renderer: geometry::Renderer>(
        &self,
        pen: &mut Pen<'_, Renderer>,
        plan: &Plan,
        palette: &Palette,
    ) {
        let face = Face::BODY;
        let cap = i32::from(face.cap());
        let mut readouts = Vec::new();

        for (index, frequency) in self.markers.iter().enumerate() {
            let Some(x) = self.column(plan, *frequency) else {
                continue;
            };

            let column = (x - plan.left()) as usize;
            let level = self.level(*frequency);
            let trace = self.peaks(&self.levels, plan.columns())[column];
            let tip = (self.row(plan, trace) - 2).max(plan.top() + POINTER);

            pen.arrowhead(Point::new(x, tip), Direction::Down, POINTER, palette.accent);

            let number = (index + 1).to_string();
            let baseline = tip - POINTER - 1;

            let half = i32::from(face.width(&number)).div_euclid(2);
            let inside = x - half >= plan.frame.x && x + half < plan.frame.x + plan.frame.width;

            if baseline - cap >= plan.top() && inside {
                pen.text(
                    face,
                    number.clone(),
                    Point::new(x, baseline),
                    Anchor::new(Horizontal::Centre, Vertical::Baseline),
                    palette.accent,
                );
            }

            readouts.push((
                format!("M{number}"),
                format!(
                    "{} {level:.0} dB",
                    scale::engineering(*frequency, self.unit)
                ),
            ));
        }

        if !plan.header {
            return;
        }

        let frame = plan.frame;
        let advance = i32::from(face.advance());
        let row: Vec<_> = readouts
            .iter()
            .map(|(tag, value)| {
                (
                    i32::from(face.width(tag) + face.width(value)) + advance,
                    End::Start,
                )
            })
            .collect();
        let placed = scale::row(&row, frame.x..=frame.x + frame.width - 1, 2 * advance);

        for ((tag, value), left) in readouts.into_iter().zip(placed) {
            if let Some(left) = left {
                let after = left + i32::from(face.width(&tag)) + advance;

                pen.text(
                    face,
                    tag,
                    Point::new(left, 0),
                    Anchor::TOP_LEFT,
                    palette.accent,
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

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for Spectrum<'_>
where
    Renderer: geometry::Renderer + 'static,
{
    type State = Memo<Key, Renderer>;

    fn draw(
        &self,
        chrome: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let palette = theme.palette();
        let size = px::floor(bounds.size());

        let Some(plan) = self.layout(size.width, size.height) else {
            return Vec::new();
        };

        let key = Key {
            palette: *palette,
            range: self.range.clone(),
            step: self.step,
            span: self.span.clone(),
            plan,
        };

        let chrome = chrome.draw(renderer, bounds.size(), key, |frame| {
            self.chrome(&mut Pen::new(frame), &plan, size.width, palette);
        });

        let mut frame = Frame::new(renderer, bounds.size());

        {
            let mut pen = Pen::new(&mut frame);

            self.trace(&mut pen, &plan, palette);
            self.markers(&mut pen, &plan, palette);
        }

        vec![chrome, frame.into_geometry()]
    }
}

/// What the chrome of a [`Spectrum`] is drawn from.
#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    palette: Palette,
    range: RangeInclusive<f32>,
    step: f32,
    span: RangeInclusive<f32>,
    plan: Plan,
}

crate::canvas_widget!(Spectrum<'a>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frequency_lands_in_the_column_that_holds_it() {
        let spectrum = spectrum(vec![0.0; 100]).span(0.0..=1000.0, "Hz");
        let plan = spectrum.layout(28 + 102, 60).unwrap();

        assert_eq!(plan.columns(), 100);
        assert_eq!(spectrum.column(&plan, 0.0), Some(29));
        assert_eq!(spectrum.column(&plan, 505.0), Some(29 + 50));
        assert_eq!(spectrum.column(&plan, 1000.0), Some(29 + 99));
        assert_eq!(spectrum.column(&plan, 1001.0), None);
    }

    #[test]
    fn levels_are_held_inside_the_plot() {
        let spectrum = spectrum(vec![0.0; 10]).range(-100.0..=0.0, 20.0);
        let plan = spectrum.layout(60, 115).unwrap();

        assert_eq!(spectrum.row(&plan, 0.0), plan.top());
        assert_eq!(spectrum.row(&plan, 20.0), plan.top());
        assert_eq!(spectrum.row(&plan, -100.0), plan.bottom());
        assert_eq!(spectrum.row(&plan, -500.0), plan.bottom());
        assert_eq!(
            spectrum.divisions(),
            vec![-100.0, -80.0, -60.0, -40.0, -20.0, 0.0]
        );
    }
}
