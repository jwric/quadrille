//! A pixel icon.
use iced_widget::core::layout::{self, Layout};
use iced_widget::core::renderer;
use iced_widget::core::widget::Tree;
use iced_widget::core::{Color, Element, Length, Point, Rectangle, Size, Widget, mouse};

use crate::draw::Sprite;
use crate::px;

/// A [`Sprite`] as a widget, drawn in the text colour it inherits.
///
/// Inheriting the colour means an icon inside a button follows the button's
/// state the way its legend does.
pub struct Icon<Theme> {
    sprite: Sprite,
    color: Option<fn(&Theme) -> Color>,
    height: Option<f32>,
}

impl<Theme> Icon<Theme> {
    /// Creates an [`Icon`] of `sprite`.
    pub fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
            color: None,
            height: None,
        }
    }

    /// Draws the icon in a colour of the theme instead of the inherited one.
    pub fn color(mut self, color: fn(&Theme) -> Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Centres the icon in a box `height` pixels tall, such as a line of text,
    /// so it sits level with the text beside it.
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(f32::from(height));
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Icon<Theme>
where
    Renderer: iced_widget::core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(self.sprite.width() as f32),
            Length::Fixed(self.height.unwrap_or(self.sprite.height() as f32)),
        )
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(
            limits,
            self.sprite.width() as f32,
            self.height.unwrap_or(self.sprite.height() as f32),
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = px::interior(layout.bounds());
        let color = self.color.map_or(style.text_color, |color| color(theme));
        let top = bounds.y + px::centre(bounds.height, self.sprite.height());

        for strip in self.sprite.strips() {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new((bounds.x + strip.x) as f32, (top + strip.y) as f32),
                        Size::new(strip.width as f32, strip.height as f32),
                    ),
                    snap: true,
                    ..renderer::Quad::default()
                },
                color,
            );
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Icon<Theme>> for Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: iced_widget::core::Renderer,
{
    fn from(icon: Icon<Theme>) -> Self {
        Element::new(icon)
    }
}
