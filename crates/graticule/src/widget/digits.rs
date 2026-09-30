//! A number whose digits are dials.
use std::ops::RangeInclusive;

use iced_widget::core::layout::{self, Layout};
use iced_widget::core::renderer;
use iced_widget::core::text::Renderer as TextRenderer;
use iced_widget::core::widget::tree::{self, Tree};
use iced_widget::core::{
    Clipboard, Color, Element, Event, Font, Length, Point, Rectangle, Shell, Size, Widget, mouse,
};

use crate::{Face, px};

/// A whole number shown to a fixed number of digits, each of which the wheel
/// steps by its own place value: the wheel over the hundreds steps by a
/// hundred.
///
/// Leading zeros are drawn faint and digits can be grouped with a separator,
/// as on the dial of a receiver: `014.074.000`.
pub struct Digits<'a, Message, Theme = crate::Theme>
where
    Theme: Catalog,
{
    value: i64,
    places: u8,
    group: Option<(u8, char)>,
    face: Face,
    range: RangeInclusive<i64>,
    on_change: Option<Box<dyn Fn(i64) -> Message + 'a>>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme> Digits<'a, Message, Theme>
where
    Theme: Catalog,
{
    /// Creates [`Digits`] showing `value` to `places` digits.
    pub fn new(value: i64, places: u8) -> Self {
        let places = places.clamp(1, 18);
        let largest = 10_i64.pow(u32::from(places)) - 1;

        Self {
            value,
            places,
            group: None,
            face: Face::DISPLAY,
            range: 0..=largest,
            on_change: None,
            class: Theme::default(),
        }
    }

    /// Separates every `size` digits, counted from the right, with
    /// `separator`.
    pub fn group(mut self, size: u8, separator: char) -> Self {
        self.group = (size > 0).then_some((size, separator));
        self
    }

    /// Sets the [`Face`] of the digits.
    pub fn face(mut self, face: Face) -> Self {
        self.face = face;
        self
    }

    /// Limits the values the wheel can reach.
    pub fn range(mut self, range: RangeInclusive<i64>) -> Self {
        self.range = range;
        self
    }

    /// Makes the digits turnable: `on_change` receives each new value.
    pub fn on_change(mut self, on_change: impl Fn(i64) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(on_change));
        self
    }

    /// Sets the style of the [`Digits`].
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// The characters shown, each with the place value it steps, if it is a
    /// digit.
    fn cells(&self) -> Vec<(char, Option<u8>)> {
        let digits = format!(
            "{:0width$}",
            self.value.unsigned_abs(),
            width = usize::from(self.places)
        );
        let count = digits.len();
        let mut cells = Vec::with_capacity(count * 2);

        for (i, digit) in digits.chars().enumerate() {
            let place = (count - 1 - i) as u8;

            if let Some((size, separator)) = self.group
                && i > 0
                && (count - i) % usize::from(size) == 0
            {
                cells.push((separator, None));
            }

            cells.push((digit, Some(place)));
        }

        cells
    }

    fn cell_at(&self, x: f32) -> Option<usize> {
        let cell = (x / f32::from(self.face.advance())).floor();

        (cell >= 0.0).then_some(cell as usize)
    }
}

/// The state of [`Digits`].
#[derive(Debug, Default)]
struct State {
    hovered: Option<usize>,
    travel: f32,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Digits<'_, Message, Theme>
where
    Theme: Catalog,
    Renderer: TextRenderer<Font = Font>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = self.cells().len() as f32 * f32::from(self.face.advance());

        layout::atomic(limits, width, f32::from(self.face.line()))
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
            Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) => {
                let hovered = cursor
                    .position_in(bounds)
                    .and_then(|position| self.cell_at(position.x))
                    .filter(|&cell| {
                        self.cells()
                            .get(cell)
                            .is_some_and(|(_, place)| place.is_some())
                    });

                if hovered != state.hovered {
                    state.hovered = hovered;
                    state.travel = 0.0;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let Some(on_change) = &self.on_change else {
                    return;
                };

                let Some(place) = state
                    .hovered
                    .and_then(|cell| self.cells().get(cell).and_then(|(_, place)| *place))
                else {
                    return;
                };

                if cursor.position_in(bounds).is_none() {
                    return;
                }

                let notches = notches(&mut state.travel, *delta);

                if notches != 0 {
                    let step = 10_i64.saturating_pow(u32::from(place));
                    let value = self
                        .value
                        .saturating_add(step.saturating_mul(i64::from(notches)))
                        .clamp(*self.range.start(), *self.range.end());

                    if value != self.value {
                        shell.publish(on_change(value));
                    }
                }

                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if self.on_change.is_some() && state.hovered.is_some() {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let style = theme.style(&self.class);
        let bounds = px::interior(layout.bounds());
        let advance = i32::from(self.face.advance());

        let mut leading = true;

        for (i, (character, place)) in self.cells().into_iter().enumerate() {
            let x = bounds.x + i as i32 * advance;

            if place.is_some() && character != '0' || place == Some(0) {
                leading = false;
            }

            let hovered = self.on_change.is_some() && state.hovered == Some(i);

            let color = if hovered {
                style.hovered
            } else if place.is_none() {
                style.separator
            } else if leading {
                style.zero
            } else {
                style.digit
            };

            renderer.fill_text(
                self.face.line_text(character.to_string()),
                Point::new(x as f32, bounds.y as f32),
                color,
                *viewport,
            );

            if hovered {
                let y = bounds.y + i32::from(self.face.baseline()) + 1;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(x as f32, y as f32),
                            Size::new(advance as f32, f32::from(self.face.scale())),
                        ),
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    style.hovered,
                );
            }
        }
    }
}

/// The whole notches in one wheel event, carrying what is left over.
///
/// A line is a notch, as X11, Wayland and Firefox report the wheel; a hundred
/// pixels is one, as Chrome does. No event counts for more than one, so a
/// touchpad's stream of small deltas adds up instead of racing.
fn notches(travel: &mut f32, delta: mouse::ScrollDelta) -> i32 {
    let step = match delta {
        mouse::ScrollDelta::Lines { y, .. } => y,
        mouse::ScrollDelta::Pixels { y, .. } => y / 100.0,
    };

    *travel += step.clamp(-1.0, 1.0);

    let whole = travel.trunc();
    *travel -= whole;

    whole as i32
}

impl<'a, Message, Theme, Renderer> From<Digits<'a, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: TextRenderer<Font = Font> + 'a,
{
    fn from(digits: Digits<'a, Message, Theme>) -> Self {
        Element::new(digits)
    }
}

/// The appearance of [`Digits`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// A significant digit.
    pub digit: Color,
    /// A leading zero.
    pub zero: Color,
    /// A group separator.
    pub separator: Color,
    /// The digit under the cursor, and its underline.
    pub hovered: Color,
}

/// The theme catalog of [`Digits`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for [`Digits`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for crate::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// Ink digits, faint zeros and separators, the accent under the hand.
pub fn default(theme: &crate::Theme) -> Style {
    let palette = theme.palette();

    Style {
        digit: palette.ink,
        zero: palette.faint,
        separator: palette.muted,
        hovered: palette.accent,
    }
}

/// Digits lit in the accent, like the tuned frequency of a receiver.
pub fn lamp(theme: &crate::Theme) -> Style {
    let palette = theme.palette();

    Style {
        digit: palette.accent,
        zero: crate::theme::dim(palette.accent, 0.45),
        separator: palette.muted,
        hovered: palette.ink,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digits(value: i64) -> Digits<'static, (), crate::Theme> {
        Digits::new(value, 9).group(3, '.')
    }

    #[test]
    fn cells_group_from_the_right() {
        let cells: String = digits(14_074_000).cells().iter().map(|(c, _)| c).collect();

        assert_eq!(cells, "014.074.000");
    }

    #[test]
    fn every_digit_knows_its_place() {
        let places: Vec<_> = digits(5)
            .cells()
            .iter()
            .filter_map(|(_, place)| *place)
            .collect();

        assert_eq!(places, (0..9).rev().collect::<Vec<u8>>());
    }

    #[test]
    fn a_notch_is_a_notch_on_every_platform() {
        let mut travel = 0.0;
        let lines = |y| mouse::ScrollDelta::Lines { x: 0.0, y };
        let pixels = |y| mouse::ScrollDelta::Pixels { x: 0.0, y };

        assert_eq!(notches(&mut travel, lines(3.0)), 1);
        assert_eq!(notches(&mut travel, pixels(-100.0)), -1);
        assert_eq!(
            (0..10)
                .map(|_| notches(&mut travel, pixels(10.0)))
                .sum::<i32>(),
            1
        );
    }
}
