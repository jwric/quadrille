//! A bar gauge.
use std::ops::RangeInclusive;

use iced_widget::core::layout::{self, Layout};
use iced_widget::core::renderer;
use iced_widget::core::widget::Tree;
use iced_widget::core::{Color, Length, Point, Rectangle, Size};
use iced_widget::core::{Widget, mouse};

use crate::{Face, px};

/// A bar gauge: a hairline track filled from the left up to a value.
///
/// The fill ends on a whole pixel, rounded down, so a gauge never shows more
/// than its value. With [`segments`](Self::segments) it is a row of lit
/// segments instead, and a [`redline`](Self::redline) turns everything past
/// it to the alarm colour.
pub struct Bar<Theme = crate::Theme>
where
    Theme: Catalog,
{
    range: RangeInclusive<f32>,
    value: f32,
    redline: Option<f32>,
    segment: Option<u16>,
    width: Length,
    height: f32,
    class: Theme::Class<'static>,
}

impl<Theme> Bar<Theme>
where
    Theme: Catalog,
{
    /// Creates a [`Bar`] showing `value` in `range`.
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        Self {
            range,
            value,
            redline: None,
            segment: None,
            width: Length::Fill,
            height: f32::from(Face::BODY.cap()),
            class: Theme::default(),
        }
    }

    /// Sets the width of the [`Bar`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Bar`], in pixels.
    pub fn height(mut self, height: u16) -> Self {
        self.height = f32::from(height);
        self
    }

    /// Splits the fill into segments `width` pixels wide with a pixel between.
    pub fn segments(mut self, width: u16) -> Self {
        self.segment = Some(width.max(1));
        self
    }

    /// Marks everything past `value` as over the limit.
    pub fn redline(mut self, value: f32) -> Self {
        self.redline = Some(value);
        self
    }

    /// Sets the style of the [`Bar`].
    pub fn style(mut self, style: fn(&Theme) -> Style) -> Self
    where
        Theme::Class<'static>: From<StyleFn<Theme>>,
    {
        self.class = (style as StyleFn<Theme>).into();
        self
    }

    fn fraction(&self, value: f32) -> f32 {
        let (start, end) = (*self.range.start(), *self.range.end());

        if end == start {
            return 0.0;
        }

        ((value - start) / (end - start)).clamp(0.0, 1.0)
    }
}

impl<Theme> iced_widget::core::widget::Meta for Bar<Theme> where Theme: Catalog {}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Bar<Theme>
where
    Theme: Catalog,
    Renderer: iced_widget::core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(self.height))
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) {
        tree.size = layout::atomic(limits, self.width, self.height);
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = px::interior(layout.bounds());
        let style = theme.style(&self.class);

        if bounds.width < 3 || bounds.height < 3 {
            return;
        }

        let inner = bounds.width - 2;
        let lit = (self.fraction(self.value) * inner as f32).floor() as i32;
        let red = self
            .redline
            .map(|line| (self.fraction(line) * inner as f32).floor() as i32)
            .unwrap_or(inner);

        let mut fill = |x: i32, width: i32, color: Color| {
            if width > 0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(x as f32, bounds.y as f32),
                            Size::new(width as f32, bounds.height as f32),
                        ),
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    color,
                );
            }
        };

        match self.segment {
            None => {
                fill(bounds.x, bounds.width, style.track);
                fill(bounds.x + 1, lit.min(red), style.fill);
                fill(bounds.x + 1 + red, lit - red, style.over);
            }
            Some(segment) => {
                let step = i32::from(segment) + 1;
                let mut x = 0;

                while x + i32::from(segment) <= bounds.width {
                    let color = if x + i32::from(segment) > lit + 1 {
                        style.unlit
                    } else if x >= red {
                        style.over
                    } else {
                        style.fill
                    };

                    fill(bounds.x + x, i32::from(segment), color);
                    x += step;
                }
            }
        }

        if self.segment.is_none() {
            let border = Rectangle::new(
                Point::new(bounds.x as f32, bounds.y as f32),
                Size::new(bounds.width as f32, bounds.height as f32),
            );

            renderer.fill_quad(
                renderer::Quad {
                    bounds: border,
                    border: crate::style::hairline(style.edge),
                    snap: true,
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        }
    }
}

/// The appearance of a [`Bar`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Inside the track, behind the fill.
    pub track: Color,
    /// The hairline around the track.
    pub edge: Color,
    /// The fill up to the value.
    pub fill: Color,
    /// The fill past the redline.
    pub over: Color,
    /// An unlit segment.
    pub unlit: Color,
}

/// The theme catalog of a [`Bar`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Bar`].
pub type StyleFn<Theme> = fn(&Theme) -> Style;

impl Catalog for crate::Theme {
    type Class<'a> = StyleFn<Self>;

    fn default<'a>() -> Self::Class<'a> {
        default
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The accent in a hairline track.
pub fn default(theme: &crate::Theme) -> Style {
    let palette = theme.palette();

    Style {
        track: palette.void,
        edge: palette.accent,
        fill: palette.accent,
        over: palette.alarm,
        unlit: palette.raised,
    }
}

/// Live data: the live colour in an edge-coloured track.
pub fn live(theme: &crate::Theme) -> Style {
    let palette = theme.palette();

    Style {
        track: palette.void,
        edge: palette.edge,
        fill: palette.live,
        over: palette.alarm,
        unlit: palette.raised,
    }
}

/// Ink on the ground, for a gauge that is information rather than state.
pub fn ink(theme: &crate::Theme) -> Style {
    let palette = theme.palette();

    Style {
        track: palette.ground,
        edge: palette.muted,
        fill: palette.ink,
        over: palette.alarm,
        unlit: palette.raised,
    }
}
