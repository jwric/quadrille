//! A group of widgets under a named rule.
use iced_widget::core::layout::{self, Layout};
use iced_widget::core::renderer;
use iced_widget::core::widget::{Meta, Operation, Tree, tree};
use iced_widget::core::{Color, Event, Length, Point, Rectangle, Size, Vector};
use iced_widget::core::{Shell, Widget, mouse, overlay};

use crate::{Face, px};

/// A group of widgets under a rule broken by the group's name, with the ends
/// of the rule dropping a few pixels: `┌── NAME ──┐`.
///
/// There are no sides and no bottom. The rule says where the group starts;
/// the space around it says where it ends.
pub struct Group<'a, W, Theme>
where
    Theme: Catalog,
{
    name: String,
    face: Face,
    content: W,
    width: Length,
    height: Length,
    spacing: u16,
    drop: u16,
    class: Theme::Class<'a>,
}

impl<'a, W, Theme> Group<'a, W, Theme>
where
    Theme: Catalog,
{
    /// Creates a [`Group`] of `content` named `name`.
    ///
    /// Like a container, a group fills the space its content fills.
    pub fn new(name: impl Into<String>, content: W) -> Self {
        Self {
            name: name.into(),
            face: Face::BODY,
            content,
            width: Length::Fit,
            height: Length::Fit,
            spacing: px::GAP as u16,
            drop: 3,
            class: Theme::default(),
        }
    }

    /// Sets the [`Face`] of the name.
    pub fn face(mut self, face: Face) -> Self {
        self.face = face;
        self
    }

    /// Sets the width of the [`Group`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Group`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the space between the name's line and the content.
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets how far the ends of the rule drop.
    pub fn drop(mut self, drop: u16) -> Self {
        self.drop = drop;
        self
    }

    /// Sets the style of the [`Group`].
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    fn header(&self) -> f32 {
        f32::from(self.face.line() + self.spacing)
    }

    fn min_width(&self) -> f32 {
        f32::from(self.face.width(&self.name) + 4 * self.face.advance())
    }
}

impl<W, Theme> Meta for Group<'_, W, Theme> where Theme: Catalog {}

impl<W, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Group<'_, W, Theme>
where
    Theme: Catalog,
    Renderer: iced_widget::core::text::Renderer,
    W: Widget<Message, Theme, Renderer>,
{
    fn tag(&self) -> tree::Tag {
        self.content.tag()
    }

    fn state(&self) -> tree::State {
        self.content.state()
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));

        let size = self.content.size();
        self.width = self.width.stack(size.width);
        self.height = self.height.stack(size.height);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        let header = self.header();
        let limits = limits.width(self.width).height(self.height);

        let content = &mut tree.children[0];

        self.content
            .layout(content, renderer, &limits.shrink(Size::new(0.0, header)));
        content.translation = Vector::new(0.0, header);

        let intrinsic = Size::new(
            content.size.width.max(self.min_width()),
            content.size.height + header,
        );

        tree.size = limits.resolve(self.width, self.height, intrinsic);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), viewport);
        operation.traverse(&mut |operation| {
            let (layout, tree) = layout.iter_mut(&mut tree.children).next().unwrap();

            self.content
                .operate(tree, layout, viewport, renderer, operation);
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let (layout, tree) = layout.iter_mut(&mut tree.children).next().unwrap();

        self.content
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let (layout, tree) = layout.iter(&tree.children).next().unwrap();

        self.content
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = px::interior(layout.bounds());
        let colors = theme.style(&self.class);

        let face = self.face;
        let y = bounds.y + i32::from(face.cap_top()) + i32::from(face.cap()).div_euclid(2);
        let (left, right) = (bounds.x, bounds.x + bounds.width - 1);
        let width = i32::from(face.width(&self.name));
        let gap = i32::from(face.advance());
        let label_left = left + (bounds.width - width).div_euclid(2);
        let drop = i32::from(self.drop);

        let hline = |renderer: &mut Renderer, from: i32, to: i32| {
            fill(renderer, from, y, to - from + 1, 1, colors.rule);
        };

        if width > 0 && label_left - gap > left && label_left + width + gap <= right {
            hline(renderer, left, label_left - gap);
            hline(renderer, label_left + width + gap - 1, right);

            renderer.fill_text(
                face.line_text(self.name.clone()),
                Point::new(label_left as f32, bounds.y as f32),
                colors.name,
                layout.bounds(),
            );
        } else {
            hline(renderer, left, right);
        }

        fill(renderer, left, y + 1, 1, drop, colors.rule);
        fill(renderer, right, y + 1, 1, drop, colors.rule);

        let (layout, tree) = layout.iter(&tree.children).next().unwrap();

        self.content
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        let (layout, tree) = layout.iter_mut(&mut tree.children).next().unwrap();

        self.content
            .overlay(tree, layout, renderer, viewport, translation, window)
    }
}

fn fill<Renderer: iced_widget::core::Renderer>(
    renderer: &mut Renderer,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: Color,
) {
    if width <= 0 || height <= 0 {
        return;
    }

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle::new(
                Point::new(x as f32, y as f32),
                Size::new(width as f32, height as f32),
            ),
            snap: true,
            ..renderer::Quad::default()
        },
        color,
    );
}

/// The appearance of a [`Group`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The colour of the rule.
    pub rule: Color,
    /// The colour of the name.
    pub name: Color,
}

/// The theme catalog of a [`Group`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Group`].
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

/// The rule in the schematic line colour, the name muted.
pub fn default(theme: &crate::Theme) -> Style {
    let palette = theme.palette();

    Style {
        rule: palette.line,
        name: palette.muted,
    }
}

/// The rule and the name in the accent, for the group that has the focus.
pub fn accent(theme: &crate::Theme) -> Style {
    let palette = theme.palette();

    Style {
        rule: palette.accent,
        name: palette.accent,
    }
}
