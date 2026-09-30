use std::ops::RangeInclusive;

use iced_widget::canvas::{self, Frame, Geometry};
use iced_widget::core::{Element, Length, Point, Rectangle, mouse};
use iced_widget::graphics::geometry;

use crate::Theme;
use crate::draw::{Pen, shape};

/// A semicircular dial: a heavy arc over its hub, ticks inside it, and a
/// needle pointing at the value.
pub struct Dial {
    value: f32,
    range: RangeInclusive<f32>,
    ticks: u16,
    redline: Option<f32>,
    width: Length,
    height: Length,
}

/// Creates a [`Dial`] showing `value` in `range`.
pub fn dial(range: RangeInclusive<f32>, value: f32) -> Dial {
    Dial {
        value,
        range,
        ticks: 4,
        redline: None,
        width: Length::Fill,
        height: Length::Fill,
    }
}

impl Dial {
    /// Sets the number of intervals between ticks across the arc.
    pub fn ticks(mut self, ticks: u16) -> Self {
        self.ticks = ticks;
        self
    }

    /// Marks the arc past `value` in the alarm colour.
    pub fn redline(mut self, value: f32) -> Self {
        self.redline = Some(value);
        self
    }

    /// Sets the width of the [`Dial`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Dial`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    fn bearing(&self, value: f32) -> f32 {
        let (start, end) = (*self.range.start(), *self.range.end());
        let fraction = if end == start {
            0.0
        } else {
            ((value - start) / (end - start)).clamp(0.0, 1.0)
        };

        270.0 + 180.0 * fraction
    }
}

impl<Message, Renderer> canvas::Program<Message, Theme, Renderer> for Dial
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
            let radius = ((width - 1) / 2).min(height - 2).max(4);
            let centre = Point::new(width.div_euclid(2), radius + 1);
            let mut pen = Pen::new(&mut frame);

            let red = self.redline.map(|value| self.bearing(value));

            pen.thick_arc(
                centre,
                radius,
                2,
                270.0,
                red.unwrap_or(90.0),
                palette.accent,
            );

            if let Some(red) = red {
                pen.thick_arc(centre, radius, 2, red, 90.0, palette.alarm);
            }

            for i in 0..=self.ticks {
                let bearing = 270.0 + 180.0 * f32::from(i) / f32::from(self.ticks.max(1));
                let outer = shape::polar(centre, bearing, (radius - 4) as f32);
                let inner = shape::polar(centre, bearing, (radius - 7) as f32);

                pen.line(inner, outer, palette.muted);
            }

            let tip = shape::polar(centre, self.bearing(self.value), (radius - 5) as f32);

            pen.line(centre, tip, palette.ink);
            pen.disc(centre, 1, palette.ink);
        }

        vec![frame.into_geometry()]
    }
}

impl<'a, Message, Renderer> From<Dial> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: geometry::Renderer + 'a,
{
    fn from(dial: Dial) -> Self {
        let (width, height) = (dial.width, dial.height);

        iced_widget::canvas(dial).width(width).height(height).into()
    }
}
