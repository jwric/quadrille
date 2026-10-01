//! Canvases as widgets, and drawings kept until what they show changes.
//!
//! A visualization is usually a canvas [`Program`] behind a builder: a value
//! to show and how to show it. [`canvas_widget!`](crate::canvas_widget) makes
//! such a type a widget of its own size, and a [`Memo`] keeps its slow parts
//! (a frame, a scale, a raster) until what they are drawn from changes.
//!
//! [`Program`]: iced_widget::canvas::Program
use std::cell::RefCell;
use std::fmt;

use iced_widget::canvas::{Cache, Frame, Geometry};
use iced_widget::core::Size;
use iced_widget::graphics::geometry;

/// A drawing kept until what it is drawn from changes.
///
/// A memo is drawn for a key: whatever the drawing depends on, such as a
/// palette, a range and a layout. It draws again only when it is asked for
/// another key, or at another size, and otherwise hands back the geometry it
/// already has.
pub struct Memo<K, Renderer>
where
    Renderer: geometry::Renderer,
{
    cache: Cache<Renderer>,
    key: RefCell<Option<K>>,
}

impl<K, Renderer> Memo<K, Renderer>
where
    K: PartialEq,
    Renderer: geometry::Renderer,
{
    /// A memo with nothing drawn yet.
    pub fn new() -> Self {
        Self {
            cache: Cache::new(),
            key: RefCell::new(None),
        }
    }

    /// The drawing `draw` makes of `key` at `size`, drawn again only if the
    /// key or the size has changed since the last call.
    pub fn draw(
        &self,
        renderer: &Renderer,
        size: Size,
        key: K,
        draw: impl FnOnce(&mut Frame<Renderer>),
    ) -> Geometry<Renderer> {
        {
            let mut drawn = self.key.borrow_mut();

            if drawn.as_ref() != Some(&key) {
                self.cache.clear();
                *drawn = Some(key);
            }
        }

        self.cache.draw(renderer, size, draw)
    }

    /// Forgets the drawing, so the next call draws again.
    pub fn clear(&self) {
        self.cache.clear();
        *self.key.borrow_mut() = None;
    }
}

impl<K, Renderer> Default for Memo<K, Renderer>
where
    K: PartialEq,
    Renderer: geometry::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, Renderer> fmt::Debug for Memo<K, Renderer>
where
    K: fmt::Debug,
    Renderer: geometry::Renderer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memo")
            .field("key", &self.key.borrow())
            .finish_non_exhaustive()
    }
}

/// Makes a canvas program a widget of its own size.
///
/// The type must implement [`canvas::Program`] for the quadrille
/// [`Theme`](crate::Theme), and have `width` and `height` fields of type
/// [`Length`]: the widget lays out at that size, and is drawn and updated as
/// the program, over its own bounds. Name a lifetime the type takes after it:
/// `canvas_widget!(Chart<'a>)`.
///
/// ```no_run
/// use iced::Widget as _;
/// use iced::widget::canvas::{self, Frame, Geometry};
/// use iced::{Length, Rectangle, Renderer, mouse};
/// use quadrille::Theme;
/// use quadrille::draw::Pen;
///
/// /// A lamp that fills the space it is given.
/// struct Glow {
///     on: bool,
///     width: Length,
///     height: Length,
/// }
///
/// impl<Message> canvas::Program<Message, Theme> for Glow {
///     type State = ();
///
///     fn draw(
///         &self,
///         _state: &(),
///         renderer: &Renderer,
///         theme: &Theme,
///         bounds: Rectangle,
///         _cursor: mouse::Cursor,
///     ) -> Vec<Geometry> {
///         let palette = theme.palette();
///         let size = quadrille::px::floor(bounds.size());
///         let mut frame = Frame::new(renderer, bounds.size());
///         let color = if self.on { palette.live } else { palette.raised };
///
///         Pen::new(&mut frame).fill(
///             quadrille::draw::rectangle(0, 0, size.width, size.height),
///             color,
///         );
///
///         vec![frame.into_geometry()]
///     }
/// }
///
/// quadrille::canvas_widget!(Glow);
///
/// fn view<'a, Message: 'a>() -> quadrille::Element<'a, Message> {
///     iced::widget::row![Glow { on: true, width: Length::Fixed(5.0), height: Length::Fixed(5.0) }]
///         .boxed()
/// }
/// ```
///
/// [`canvas::Program`]: iced_widget::canvas::Program
/// [`Length`]: iced_widget::core::Length
#[macro_export]
macro_rules! canvas_widget {
    ($program:ident $(<$lifetime:lifetime>)?) => {
        impl$(<$lifetime>)? $crate::__private::iced_widget::core::widget::Meta
            for $program$(<$lifetime>)?
        {
        }

        impl<$($lifetime,)? Message, Renderer>
            $crate::__private::iced_widget::core::Widget<Message, $crate::Theme, Renderer>
            for $program$(<$lifetime>)?
        where
            Renderer: $crate::__private::iced_widget::graphics::geometry::Renderer + 'static,
            Self: $crate::__private::iced_widget::canvas::Program<Message, $crate::Theme, Renderer>,
        {
            fn tag(&self) -> $crate::__private::iced_widget::core::widget::tree::Tag {
                $crate::__private::iced_widget::core::Widget::<Message, $crate::Theme, Renderer>::tag(
                    &$crate::__private::iced_widget::canvas(self).width(self.width).height(self.height),
                )
            }

            fn state(&self) -> $crate::__private::iced_widget::core::widget::tree::State {
                $crate::__private::iced_widget::core::Widget::<Message, $crate::Theme, Renderer>::state(
                    &$crate::__private::iced_widget::canvas(self).width(self.width).height(self.height),
                )
            }

            fn size(
                &self,
            ) -> $crate::__private::iced_widget::core::Size<$crate::__private::iced_widget::core::Length>
            {
                $crate::__private::iced_widget::core::Size::new(self.width, self.height)
            }

            fn layout(
                &mut self,
                tree: &mut $crate::__private::iced_widget::core::widget::Tree,
                _renderer: &Renderer,
                limits: &$crate::__private::iced_widget::core::layout::Limits,
            ) {
                tree.size = $crate::__private::iced_widget::core::layout::atomic(
                    limits,
                    self.width,
                    self.height,
                );
            }

            fn update(
                &mut self,
                tree: &mut $crate::__private::iced_widget::core::widget::Tree,
                event: &$crate::__private::iced_widget::core::Event,
                layout: $crate::__private::iced_widget::core::Layout,
                cursor: $crate::__private::iced_widget::core::mouse::Cursor,
                renderer: &Renderer,
                shell: &mut $crate::__private::iced_widget::core::Shell<'_, Message>,
                viewport: &$crate::__private::iced_widget::core::Rectangle,
            ) {
                $crate::__private::iced_widget::core::Widget::<Message, $crate::Theme, Renderer>::update(
                    &mut $crate::__private::iced_widget::canvas(&*self)
                        .width(self.width)
                        .height(self.height),
                    tree,
                    event,
                    layout,
                    cursor,
                    renderer,
                    shell,
                    viewport,
                );
            }

            fn mouse_interaction(
                &self,
                tree: &$crate::__private::iced_widget::core::widget::Tree,
                layout: $crate::__private::iced_widget::core::Layout,
                cursor: $crate::__private::iced_widget::core::mouse::Cursor,
                viewport: &$crate::__private::iced_widget::core::Rectangle,
                renderer: &Renderer,
            ) -> $crate::__private::iced_widget::core::mouse::Interaction {
                $crate::__private::iced_widget::core::Widget::<Message, $crate::Theme, Renderer>::mouse_interaction(
                    &$crate::__private::iced_widget::canvas(self).width(self.width).height(self.height),
                    tree,
                    layout,
                    cursor,
                    viewport,
                    renderer,
                )
            }

            fn draw(
                &self,
                tree: &$crate::__private::iced_widget::core::widget::Tree,
                renderer: &mut Renderer,
                theme: &$crate::Theme,
                style: &$crate::__private::iced_widget::core::renderer::Style,
                layout: $crate::__private::iced_widget::core::Layout,
                cursor: $crate::__private::iced_widget::core::mouse::Cursor,
                viewport: &$crate::__private::iced_widget::core::Rectangle,
            ) {
                $crate::__private::iced_widget::core::Widget::<Message, $crate::Theme, Renderer>::draw(
                    &$crate::__private::iced_widget::canvas(self).width(self.width).height(self.height),
                    tree,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    viewport,
                );
            }
        }
    };
}
