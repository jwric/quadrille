use std::ops::RangeInclusive;

use iced_widget::core::{Color, Point};
use iced_widget::graphics::geometry;

use super::{Anchor, Horizontal, Lettering, Pen, Vertical};

impl<Renderer> Pen<'_, Renderer>
where
    Renderer: geometry::Renderer,
{
    /// A vertical dimension down column `x` across the rows of `span`: end
    /// ticks, and the line broken around `label`, which is set level across
    /// the middle.
    pub fn vertical_dimension(
        &mut self,
        x: i32,
        span: RangeInclusive<i32>,
        label: Lettering<'_>,
        line: Color,
    ) {
        const TICK: i32 = 2;
        const GAP: i32 = 2;

        let (top, bottom) = (*span.start().min(span.end()), *span.start().max(span.end()));
        let cap = i32::from(label.face.cap());
        let middle = top + (bottom - top).div_euclid(2);
        let label_top = middle - cap.div_euclid(2);

        self.hline(x - TICK, x + TICK, top, line);
        self.hline(x - TICK, x + TICK, bottom, line);

        if !label.text.is_empty() && label_top - GAP > top && label_top + cap + GAP <= bottom {
            self.vline(x, top + 1, label_top - GAP - 1, line);
            self.vline(x, label_top + cap + GAP, bottom - 1, line);
            self.text(
                label.face,
                label.text,
                Point::new(x, label_top),
                Anchor::new(Horizontal::Centre, Vertical::CapTop),
                label.color,
            );
        } else {
            self.vline(x, top + 1, bottom - 1, line);
        }
    }
}
