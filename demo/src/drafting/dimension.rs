use std::ops::RangeInclusive;

use iced::advanced::graphics::geometry;
use iced::{Color, Point};

use quadrille::draw::{Anchor, Horizontal, Lettering, Pen, Vertical};

/// See [`Drafting::dimension`](super::Drafting::dimension).
pub(super) fn dimension<Renderer: geometry::Renderer>(
    pen: &mut Pen<'_, Renderer>,
    span: RangeInclusive<i32>,
    y: i32,
    label: Lettering<'_>,
    line: Color,
) {
    const TICK: i32 = 2;
    const GAP: i32 = 2;

    let (left, right) = (*span.start().min(span.end()), *span.start().max(span.end()));
    let width = i32::from(label.face.width(label.text));
    let middle = left + (right - left).div_euclid(2);
    let label_left = middle - width.div_euclid(2);

    pen.vline(left, y - TICK, y + TICK, line);
    pen.vline(right, y - TICK, y + TICK, line);

    if width > 0 && label_left - GAP > left && label_left + width + GAP <= right {
        pen.hline(left + 1, label_left - GAP - 1, y, line);
        pen.hline(label_left + width + GAP, right - 1, y, line);
        pen.text(
            label.face,
            label.text,
            Point::new(label_left, y),
            Anchor::LEFT,
            label.color,
        );
    } else {
        pen.hline(left + 1, right - 1, y, line);
    }
}

/// See [`Drafting::vertical_dimension`](super::Drafting::vertical_dimension).
pub(super) fn vertical_dimension<Renderer: geometry::Renderer>(
    pen: &mut Pen<'_, Renderer>,
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

    pen.hline(x - TICK, x + TICK, top, line);
    pen.hline(x - TICK, x + TICK, bottom, line);

    if !label.text.is_empty() && label_top - GAP > top && label_top + cap + GAP <= bottom {
        pen.vline(x, top + 1, label_top - GAP - 1, line);
        pen.vline(x, label_top + cap + GAP, bottom - 1, line);
        pen.text(
            label.face,
            label.text,
            Point::new(x, label_top),
            Anchor::new(Horizontal::Centre, Vertical::CapTop),
            label.color,
        );
    } else {
        pen.vline(x, top + 1, bottom - 1, line);
    }
}
