//! Instruments: canvases that draw a reading.
//!
//! Each instrument is a small description of what to show, and a widget that
//! draws it as a canvas, filling the space it is given unless sized. They draw with a [`Pen`](crate::draw::Pen), so every
//! mark is on the pixel grid.
/// Makes an instrument a widget: it lays out at its own size, and is drawn and
/// updated as the canvas [`Program`](iced_widget::canvas::Program) it is.
macro_rules! canvas_widget {
    ($instrument:ident $(<$lifetime:lifetime>)?) => {
        impl$(<$lifetime>)? iced_widget::core::widget::Meta for $instrument$(<$lifetime>)? {}

        impl<$($lifetime,)? Message, Renderer>
            iced_widget::core::Widget<Message, crate::Theme, Renderer> for $instrument$(<$lifetime>)?
        where
            Renderer: iced_widget::graphics::geometry::Renderer + 'static,
            Self: iced_widget::canvas::Program<Message, crate::Theme, Renderer>,
        {
            fn tag(&self) -> iced_widget::core::widget::tree::Tag {
                iced_widget::core::Widget::<Message, crate::Theme, Renderer>::tag(&self.canvas())
            }

            fn state(&self) -> iced_widget::core::widget::tree::State {
                iced_widget::core::Widget::<Message, crate::Theme, Renderer>::state(&self.canvas())
            }

            fn size(&self) -> iced_widget::core::Size<iced_widget::core::Length> {
                iced_widget::core::Size::new(self.width, self.height)
            }

            fn layout(
                &mut self,
                tree: &mut iced_widget::core::widget::Tree,
                _renderer: &Renderer,
                limits: &iced_widget::core::layout::Limits,
            ) {
                tree.size = iced_widget::core::layout::atomic(limits, self.width, self.height);
            }

            fn update(
                &mut self,
                tree: &mut iced_widget::core::widget::Tree,
                event: &iced_widget::core::Event,
                layout: iced_widget::core::Layout,
                cursor: iced_widget::core::mouse::Cursor,
                renderer: &Renderer,
                shell: &mut iced_widget::core::Shell<'_, Message>,
                viewport: &iced_widget::core::Rectangle,
            ) {
                iced_widget::core::Widget::<Message, crate::Theme, Renderer>::update(
                    &mut self.canvas(),
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
                tree: &iced_widget::core::widget::Tree,
                layout: iced_widget::core::Layout,
                cursor: iced_widget::core::mouse::Cursor,
                viewport: &iced_widget::core::Rectangle,
                renderer: &Renderer,
            ) -> iced_widget::core::mouse::Interaction {
                iced_widget::core::Widget::<Message, crate::Theme, Renderer>::mouse_interaction(
                    &self.canvas(),
                    tree,
                    layout,
                    cursor,
                    viewport,
                    renderer,
                )
            }

            fn draw(
                &self,
                tree: &iced_widget::core::widget::Tree,
                renderer: &mut Renderer,
                theme: &crate::Theme,
                style: &iced_widget::core::renderer::Style,
                layout: iced_widget::core::Layout,
                cursor: iced_widget::core::mouse::Cursor,
                viewport: &iced_widget::core::Rectangle,
            ) {
                iced_widget::core::Widget::<Message, crate::Theme, Renderer>::draw(
                    &self.canvas(),
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

        impl$(<$lifetime>)? $instrument$(<$lifetime>)? {
            /// A canvas drawing the instrument, at its size.
            fn canvas<Message, Renderer>(
                &self,
            ) -> iced_widget::Canvas<&Self, Message, crate::Theme, Renderer>
            where
                Renderer: iced_widget::graphics::geometry::Renderer,
                Self: iced_widget::canvas::Program<Message, crate::Theme, Renderer>,
            {
                iced_widget::canvas(self).width(self.width).height(self.height)
            }
        }
    };
}

mod dial;
mod plot;
mod scale;
mod spectrum;
mod tape;
mod waterfall;

pub use dial::{Dial, dial};
pub use plot::{Cursors, Plot, Trace, channel, plot, trace};
pub use scale::engineering;
pub use spectrum::{Spectrum, spectrum};
pub use tape::{Marker, Side, Tape, tape};
pub use waterfall::{History, Ramp, Waterfall, waterfall};

use std::cell::RefCell;

use iced_widget::canvas::Cache;
use iced_widget::graphics::geometry;

/// Clears `cache` unless `key` is what it was last drawn from.
fn keep<K: PartialEq, Renderer: geometry::Renderer>(
    cache: &Cache<Renderer>,
    drawn: &RefCell<Option<K>>,
    key: K,
) {
    let mut drawn = drawn.borrow_mut();

    if drawn.as_ref() != Some(&key) {
        cache.clear();
        *drawn = Some(key);
    }
}
