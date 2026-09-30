use std::ops::RangeInclusive;

use iced_widget::canvas::{self, Frame, Geometry};
use iced_widget::core::{Element, Length, Point, Rectangle, mouse};
use iced_widget::graphics::geometry;

use crate::draw::{Anchor, Direction, Pen, rectangle};
use crate::{Face, Theme};

/// Which side of an instrument a mark sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The left side.
    Left,
    /// The right side.
    Right,
}

/// How a [`Tape`] marks its current value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// Corner brackets around the centre label.
    Brackets,
    /// A solid pointer on one side, aimed at the centre row.
    Pointer(Side),
}

/// A vertical value tape: labels every `step` scrolling past a fixed centre,
/// larger values above.
///
/// The tape moves in whole pixels: a value between two labels puts them the
/// matching fraction of the pitch from the centre, rounded.
pub struct Tape<'a> {
    value: f32,
    step: f32,
    face: Face,
    pitch: u16,
    marker: Marker,
    range: Option<RangeInclusive<f32>>,
    format: Box<dyn Fn(f32) -> String + 'a>,
    width: Length,
    height: Length,
}

/// Creates a [`Tape`] showing `value` with a label every `step`.
pub fn tape<'a>(value: f32, step: f32) -> Tape<'a> {
    Tape {
        value,
        step,
        face: Face::BODY,
        pitch: Face::BODY.line() + Face::BODY.line() / 2,
        marker: Marker::Brackets,
        range: None,
        format: Box::new(|value| format!("{value:.0}")),
        width: Length::Fill,
        height: Length::Fill,
    }
}

impl<'a> Tape<'a> {
    /// Sets the [`Face`] of the labels and a pitch to match it.
    pub fn face(mut self, face: Face) -> Self {
        self.face = face;
        self.pitch = face.line() + face.line() / 2;
        self
    }

    /// Sets the pixels between two labels.
    pub fn pitch(mut self, pitch: u16) -> Self {
        self.pitch = pitch.max(1);
        self
    }

    /// Sets how the current value is marked.
    pub fn marker(mut self, marker: Marker) -> Self {
        self.marker = marker;
        self
    }

    /// Only labels values in `range`.
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = Some(range);
        self
    }

    /// Sets how a label's value is written.
    pub fn format(mut self, format: impl Fn(f32) -> String + 'a) -> Self {
        self.format = Box::new(format);
        self
    }

    /// Sets the width of the [`Tape`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Tape`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for Tape<'_>
where
    Renderer: geometry::Renderer,
{
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let palette = theme.palette();
        let mut frame = Frame::new(renderer, bounds.size());

        {
            let (width, height) = (bounds.width as i32, bounds.height as i32);
            let centre = height.div_euclid(2);
            let middle = width.div_euclid(2);
            let pitch = i32::from(self.pitch);
            let position = self.value / self.step;
            let reach = height / pitch / 2 + 2;
            let nearest = position.round() as i32;

            let cap = i32::from(self.face.cap());
            let mut pen = Pen::new(&mut frame);

            for k in nearest - reach..=nearest + reach {
                let value = k as f32 * self.step;

                if self
                    .range
                    .as_ref()
                    .is_some_and(|range| !range.contains(&value))
                {
                    continue;
                }

                let offset = ((k as f32 - position) * pitch as f32).round() as i32;
                let y = centre - offset;

                // A label cut by the edge reads as a different number: only
                // whole labels are drawn.
                let top = y - i32::from(self.face.cap_top()) - cap.div_euclid(2);

                if top < 0 || top + i32::from(self.face.line()) > height {
                    continue;
                }
                let distance = (y - centre).abs() as f32 / (height as f32 / 2.0);

                let color = if distance < 0.45 {
                    palette.ink
                } else if distance < 0.75 {
                    palette.muted
                } else {
                    palette.faint
                };

                pen.text(
                    self.face,
                    (self.format)(value),
                    Point::new(middle, y),
                    Anchor::CENTRE,
                    color,
                );
            }

            let weight = if self.face.size() >= 20 { 2 } else { 1 };

            match self.marker {
                Marker::Brackets => {
                    let label = (self.format)(nearest as f32 * self.step);
                    let span = i32::from(self.face.width(&label)) + 2 * (4 + weight);
                    let tall = cap + 2 * (2 + weight);

                    pen.brackets(
                        rectangle(
                            middle - span.div_euclid(2),
                            centre - tall.div_euclid(2),
                            span,
                            tall,
                        ),
                        (cap / 2).max(3),
                        weight,
                        palette.accent,
                    );
                }
                Marker::Pointer(side) => {
                    let size = (cap / 2).max(2);

                    let (tip, towards) = match side {
                        Side::Right => (Point::new(width - 1 - size, centre), Direction::Left),
                        Side::Left => (Point::new(size, centre), Direction::Right),
                    };

                    pen.arrowhead(tip, towards, size, palette.accent);
                }
            }
        }

        vec![frame.into_geometry()]
    }
}

impl<'a, Message, Renderer> From<Tape<'a>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + 'a,
{
    fn from(tape: Tape<'a>) -> Self {
        let (width, height) = (tape.width, tape.height);

        iced_widget::canvas(tape).width(width).height(height).into()
    }
}
