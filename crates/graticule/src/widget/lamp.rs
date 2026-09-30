//! An indicator lamp.
use iced_widget::core::layout::{self, Layout};
use iced_widget::core::renderer;
use iced_widget::core::widget::Tree;
use iced_widget::core::{Color, Element, Length, Rectangle, Size};
use iced_widget::core::{Widget, mouse};

use crate::px;

/// An indicator lamp: a square of colour when lit, dark glass in a hairline
/// when not.
pub struct Lamp<Theme = crate::Theme>
where
    Theme: Catalog,
{
    on: bool,
    size: f32,
    class: Theme::Class<'static>,
}

/// Creates a [`Lamp`] that is lit when `on`.
pub fn lamp<Theme: Catalog>(on: bool) -> Lamp<Theme> {
    Lamp::new(on)
}

impl<Theme> Lamp<Theme>
where
    Theme: Catalog,
{
    /// The default side of a lamp: six pixels, which centres on the capitals
    /// of [`Face::BODY`](crate::Face::BODY) in a line of it.
    pub const SIZE: u16 = 6;

    /// Creates a [`Lamp`] that is lit when `on`.
    pub fn new(on: bool) -> Self {
        Self {
            on,
            size: f32::from(Self::SIZE),
            class: Theme::default(),
        }
    }

    /// Sets the side of the [`Lamp`], in pixels.
    pub fn size(mut self, size: u16) -> Self {
        self.size = f32::from(size);
        self
    }

    /// Sets the style of the [`Lamp`].
    pub fn style(mut self, style: fn(&Theme, bool) -> Style) -> Self
    where
        Theme::Class<'static>: From<StyleFn<Theme>>,
    {
        self.class = (style as StyleFn<Theme>).into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Lamp<Theme>
where
    Theme: Catalog,
    Renderer: iced_widget::core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.size), Length::Fixed(self.size))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.size, self.size)
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
        let style = theme.style(&self.class, self.on);

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle::new(
                    iced_widget::core::Point::new(bounds.x as f32, bounds.y as f32),
                    Size::new(bounds.width as f32, bounds.height as f32),
                ),
                border: crate::style::hairline(style.ring),
                snap: true,
                ..renderer::Quad::default()
            },
            style.glass,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Lamp<Theme>> for Element<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: iced_widget::core::Renderer,
{
    fn from(lamp: Lamp<Theme>) -> Self {
        Element::new(lamp)
    }
}

/// The appearance of a [`Lamp`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The glass.
    pub glass: Color,
    /// The hairline around the glass.
    pub ring: Color,
}

/// The theme catalog of a [`Lamp`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class, lit or not.
    fn style(&self, class: &Self::Class<'_>, on: bool) -> Style;
}

/// A styling function for a [`Lamp`].
pub type StyleFn<Theme> = fn(&Theme, bool) -> Style;

impl Catalog for crate::Theme {
    type Class<'a> = StyleFn<Self>;

    fn default<'a>() -> Self::Class<'a> {
        accent
    }

    fn style(&self, class: &Self::Class<'_>, on: bool) -> Style {
        class(self, on)
    }
}

fn tone(theme: &crate::Theme, on: bool, lit: Color) -> Style {
    let palette = theme.palette();

    if on {
        Style {
            glass: lit,
            ring: lit,
        }
    } else {
        Style {
            glass: palette.raised,
            ring: palette.edge,
        }
    }
}

/// Lit in the accent: something is engaged.
pub fn accent(theme: &crate::Theme, on: bool) -> Style {
    tone(theme, on, theme.palette().accent)
}

/// Lit in the live colour: something is running.
pub fn live(theme: &crate::Theme, on: bool) -> Style {
    tone(theme, on, theme.palette().live)
}

/// Lit in the caution colour.
pub fn caution(theme: &crate::Theme, on: bool) -> Style {
    tone(theme, on, theme.palette().caution)
}

/// Lit in the alarm colour.
pub fn alarm(theme: &crate::Theme, on: bool) -> Style {
    tone(theme, on, theme.palette().alarm)
}
