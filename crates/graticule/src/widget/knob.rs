//! A rotary control.
use std::ops::RangeInclusive;

use iced_widget::core::layout::{self, Layout};
use iced_widget::core::renderer;
use iced_widget::core::widget::{Tree, tree};
use iced_widget::core::{Clipboard, Shell, Widget, mouse, touch, window};
use iced_widget::core::{Color, Element, Event, Length, Point, Rectangle, Size};

use crate::draw::shape;
use crate::px;

/// The vertical drag that turns a knob through its whole range.
const TRAVEL: f32 = 96.0;

/// A rotary control: a pixel disc with an index line that sweeps 270°, from
/// seven o'clock through twelve to five, like the throw of a potentiometer.
///
/// Drag vertically to turn it, the whole range over 96 pixels of travel, or
/// scroll to turn it a step a notch. Values snap to steps counted from the
/// start of the range.
pub struct Knob<'a, Message, Theme = crate::Theme>
where
    Theme: Catalog,
{
    range: RangeInclusive<i32>,
    value: i32,
    step: i32,
    on_change: Box<dyn Fn(i32) -> Message + 'a>,
    diameter: u16,
    class: Theme::Class<'static>,
    status: Option<Status>,
}

impl<'a, Message, Theme> Knob<'a, Message, Theme>
where
    Theme: Catalog,
{
    /// The default diameter: odd, so the disc has a centre pixel.
    pub const DIAMETER: u16 = 15;

    /// Creates a [`Knob`] turning `value` through `range` in steps of
    /// `step`.
    pub fn new(
        range: RangeInclusive<i32>,
        value: i32,
        step: i32,
        on_change: impl Fn(i32) -> Message + 'a,
    ) -> Self {
        Self {
            value: value.clamp(*range.start(), *range.end()),
            range,
            step: step.max(1),
            on_change: Box::new(on_change),
            diameter: Self::DIAMETER,
            class: Theme::default(),
            status: None,
        }
    }

    /// Sets the diameter of the [`Knob`], rounded up to an odd number of
    /// pixels.
    pub fn diameter(mut self, diameter: u16) -> Self {
        self.diameter = diameter.max(7) | 1;
        self
    }

    /// Sets the style of the [`Knob`].
    pub fn style(mut self, style: fn(&Theme, Status) -> Style) -> Self
    where
        Theme::Class<'static>: From<StyleFn<Theme>>,
    {
        self.class = (style as StyleFn<Theme>).into();
        self
    }

    fn span(&self) -> i32 {
        *self.range.end() - *self.range.start()
    }

    /// `raw` on the nearest step, inside the range.
    fn snapped(&self, raw: i32) -> i32 {
        let start = *self.range.start();
        let steps = ((raw - start) as f32 / self.step as f32).round() as i32;

        (start + steps * self.step).clamp(start, *self.range.end())
    }

    /// The bearing of the index line: degrees clockwise from straight up.
    fn bearing(&self) -> f32 {
        let travelled = (self.value - *self.range.start()) as f32 / self.span().max(1) as f32;

        225.0 + 270.0 * travelled
    }

    fn turn(&mut self, to: i32, shell: &mut Shell<'_, Message>) {
        let value = self.snapped(to);

        // Events can arrive in a batch before the view is rebuilt, so the
        // knob keeps the value it published: the next event turns on from
        // there.
        if value != self.value {
            self.value = value;
            shell.publish((self.on_change)(value));
        }
    }
}

#[derive(Debug, Default)]
struct State {
    /// The cursor's row and the value when the drag began. A drag is
    /// measured from there rather than accumulated, so rounding cannot
    /// drift over a long one.
    drag: Option<(f32, i32)>,
    wheel: Notches,
}

/// Wheel travel carried across events, handed out as whole notches.
///
/// Platforms disagree about what a notch is: X11 and Wayland send one line,
/// Firefox three, Chrome about a hundred physical pixels, and a touchpad a
/// stream of small deltas. So one event never counts for more than a notch,
/// and small ones add up until together they are one.
#[derive(Debug, Clone, Copy, Default)]
struct Notches(f32);

impl Notches {
    const LINES: f32 = 1.0;
    // Pixel deltas are physical pixels, not virtual ones.
    const PIXELS: f32 = 100.0;

    /// Adds one wheel event and returns the whole notches it completes.
    fn add(&mut self, delta: mouse::ScrollDelta) -> i32 {
        let notches = match delta {
            mouse::ScrollDelta::Lines { y, .. } => y / Self::LINES,
            mouse::ScrollDelta::Pixels { y, .. } => y / Self::PIXELS,
        };

        self.0 += notches.clamp(-1.0, 1.0);

        let whole = self.0.trunc();

        self.0 -= whole;

        whole as i32
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Knob<'_, Message, Theme>
where
    Theme: Catalog,
    Renderer: iced_widget::core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        let side = f32::from(self.diameter);

        Size::new(Length::Fixed(side), Length::Fixed(side))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let side = f32::from(self.diameter);

        layout::atomic(limits, side, side)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(position) = cursor.position_over(bounds) {
                    state.drag = Some((position.y, self.value));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
                state.drag = None;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some((row, value)) = state.drag
                    && let Some(position) = cursor.land().position()
                {
                    let turned = (row - position.y) * self.span() as f32 / TRAVEL;

                    self.turn(value + turned.round() as i32, shell);
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let notches = state.wheel.add(*delta);

                if notches != 0 {
                    self.turn(self.value + notches * self.step, shell);
                }

                shell.capture_event();
            }
            Event::Window(window::Event::Unfocused) => {
                state.drag = None;
            }
            _ => {}
        }

        let status = if state.drag.is_some() {
            Status::Dragged
        } else if cursor.is_over(bounds) {
            Status::Hovered
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            self.status = Some(status);
        } else if self.status.is_some_and(|drawn| drawn != status) {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = px::interior(layout.bounds());
        let style = theme.style(&self.class, self.status.unwrap_or(Status::Active));

        let radius = (i32::from(self.diameter) - 1) / 2;
        let centre = Point::new(bounds.x + radius, bounds.y + radius);

        let mut fill = |x: i32, y: i32, width: i32, color: Color| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        iced_widget::core::Point::new(x as f32, y as f32),
                        Size::new(width as f32, 1.0),
                    ),
                    snap: true,
                    ..renderer::Quad::default()
                },
                color,
            );
        };

        for (disc, color) in [(radius, style.ring), (radius - 1, style.face)] {
            for (y, from, to) in shape::disc(centre, disc) {
                fill(from, y, to - from + 1, color);
            }
        }

        let bearing = self.bearing();
        let inner = shape::polar(centre, bearing, 1.0);
        let outer = shape::polar(centre, bearing, (radius - 2) as f32);

        for run in shape::runs(&shape::line(inner, outer), false) {
            for y in run.y..run.y + run.height {
                fill(run.x, y, run.width, style.index);
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if state.drag.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Knob<'a, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: iced_widget::core::Renderer,
{
    fn from(knob: Knob<'a, Message, Theme>) -> Self {
        Element::new(knob)
    }
}

/// The state of a [`Knob`] under the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// At rest.
    Active,
    /// Under the cursor.
    Hovered,
    /// Being turned.
    Dragged,
}

/// The appearance of a [`Knob`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The rim of the disc.
    pub ring: Color,
    /// The face of the disc.
    pub face: Color,
    /// The index line.
    pub index: Color,
}

/// The theme catalog of a [`Knob`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class in a [`Status`].
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Knob`].
pub type StyleFn<Theme> = fn(&Theme, Status) -> Style;

impl Catalog for crate::Theme {
    type Class<'a> = StyleFn<Self>;

    fn default<'a>() -> Self::Class<'a> {
        default
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// A raised face in an edge rim with an accent index: the face lifts a step
/// under the cursor, and the rim takes the accent while turning.
pub fn default(theme: &crate::Theme, status: Status) -> Style {
    let palette = theme.palette();

    match status {
        Status::Active => Style {
            ring: palette.edge,
            face: palette.raised,
            index: palette.accent,
        },
        Status::Hovered => Style {
            ring: palette.edge,
            face: palette.hover,
            index: palette.accent,
        },
        Status::Dragged => Style {
            ring: palette.accent,
            face: palette.hover,
            index: palette.accent,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn knob(range: RangeInclusive<i32>, value: i32, step: i32) -> Knob<'static, i32> {
        Knob::new(range, value, step, |value| value)
    }

    #[test]
    fn values_snap_to_steps_counted_from_the_start() {
        let knob = knob(400..=800, 500, 100);

        assert_eq!(knob.snapped(449), 400);
        assert_eq!(knob.snapped(450), 500);
        assert_eq!(knob.snapped(1_000), 800);
        assert_eq!(knob.snapped(0), 400);
    }

    #[test]
    fn the_index_sweeps_from_seven_to_five_o_clock() {
        assert_eq!(knob(0..=100, 0, 5).bearing(), 225.0);
        assert_eq!(knob(0..=100, 50, 5).bearing(), 360.0);
        assert_eq!(knob(0..=100, 100, 5).bearing(), 495.0);
        assert_eq!(knob(0..=100, 250, 5).value, 100);
    }

    #[test]
    fn a_notch_is_one_step_on_every_platform() {
        let lines = |y| mouse::ScrollDelta::Lines { x: 0.0, y };
        let pixels = |y| mouse::ScrollDelta::Pixels { x: 0.0, y };

        let mut wheel = Notches::default();

        assert_eq!(wheel.add(lines(1.0)), 1);
        assert_eq!(wheel.add(lines(3.0)), 1);
        assert_eq!(wheel.add(pixels(100.0)), 1);
        assert_eq!(wheel.add(pixels(-100.0)), -1);

        let mut touchpad = Notches::default();
        let notches: i32 = (0..10).map(|_| touchpad.add(pixels(10.0))).sum();

        assert_eq!(notches, 1);
    }
}
