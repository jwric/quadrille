use iced_widget::canvas::Image;
use iced_widget::core::image::{FilterMethod, Handle};
use iced_widget::core::{Color, Point, Rectangle, Size};
use iced_widget::graphics::geometry;

use super::Pen;

/// A picture set pixel by pixel, drawn one of its pixels to one pixel.
///
/// A raster is for what is cheaper to compute than to draw as shapes: a heat
/// map, a field of noise, a picture decoded from elsewhere. [`Pen::raster`]
/// draws it at a whole pixel without filtering, so each of its pixels is a
/// flat square of its colour on every renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Raster {
    /// A raster `width` × `height` pixels, every one of them `color`.
    pub fn new(width: u32, height: u32, color: Color) -> Self {
        let pixel = rgba(color);

        Self {
            width,
            height,
            pixels: pixel.repeat(width as usize * height as usize),
        }
    }

    /// A raster `width` × `height` pixels, each the colour `pixel` gives for
    /// its column and row.
    pub fn from_fn(width: u32, height: u32, mut pixel: impl FnMut(u32, u32) -> Color) -> Self {
        let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);

        for y in 0..height {
            for x in 0..width {
                pixels.extend(rgba(pixel(x, y)));
            }
        }

        Self {
            width,
            height,
            pixels,
        }
    }

    /// A raster `width` × `height` pixels in a few colours: each pixel the
    /// one of `colors` that `index` picks for its column and row.
    ///
    /// Each colour is turned into bytes once rather than once a pixel, which
    /// is the cheaper way to set a picture drawn from a palette's steps.
    ///
    /// # Panics
    ///
    /// Panics if `index` picks a colour `colors` does not have.
    pub fn indexed(
        width: u32,
        height: u32,
        colors: &[Color],
        mut index: impl FnMut(u32, u32) -> usize,
    ) -> Self {
        let colors: Vec<[u8; 4]> = colors.iter().map(|color| rgba(*color)).collect();
        let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);

        for y in 0..height {
            for x in 0..width {
                pixels.extend_from_slice(&colors[index(x, y)]);
            }
        }

        Self {
            width,
            height,
            pixels,
        }
    }

    /// Its width.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Its height.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Sets the pixel at column `x` and row `y` to `color`. A pixel outside
    /// the raster is left alone.
    pub fn set(&mut self, x: u32, y: u32, color: Color) {
        if let Some(index) = self.index(x, y) {
            self.pixels[index..index + 4].copy_from_slice(&rgba(color));
        }
    }

    /// The colour of the pixel at column `x` and row `y`.
    pub fn get(&self, x: u32, y: u32) -> Option<Color> {
        let index = self.index(x, y)?;
        let [r, g, b, a] = self.pixels[index..index + 4] else {
            return None;
        };

        Some(Color::from_rgba8(r, g, b, f32::from(a) / 255.0))
    }

    /// An image of the raster, for drawing it with iced's own widgets.
    ///
    /// Every handle is a new image to the renderer, which uploads it again:
    /// make one when the raster changes, not every frame.
    pub fn handle(&self) -> Handle {
        Handle::from_rgba(self.width, self.height, self.pixels.clone())
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        (x < self.width && y < self.height)
            .then(|| (y as usize * self.width as usize + x as usize) * 4)
    }
}

impl<Renderer> Pen<'_, Renderer>
where
    Renderer: geometry::Renderer,
{
    /// Draws `raster` with its top-left pixel at `at`, one of its pixels to
    /// one pixel, unfiltered.
    ///
    /// The raster is handed to the renderer as a new image each time it is
    /// drawn, so draw it into a frame that is kept, such as a
    /// [`Memo`](crate::canvas::Memo)'s, rather than into one drawn every
    /// frame.
    pub fn raster(&mut self, raster: &Raster, at: Point<i32>) {
        if raster.width == 0 || raster.height == 0 {
            return;
        }

        self.frame().draw_image(
            Rectangle::new(
                Point::new(at.x as f32, at.y as f32),
                Size::new(raster.width as f32, raster.height as f32),
            ),
            Image::new(raster.handle()).filter_method(FilterMethod::Nearest),
        );
    }
}

/// The 8-bit channels of `color`.
fn rgba(color: Color) -> [u8; 4] {
    let byte = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0).round() as u8;

    [byte(color.r), byte(color.g), byte(color.b), byte(color.a)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::rgb;

    #[test]
    fn a_raster_is_set_by_column_and_row() {
        let (black, white) = (rgb(0, 0, 0), rgb(255, 255, 255));
        let mut raster = Raster::new(3, 2, black);

        raster.set(2, 1, white);
        raster.set(3, 0, white);

        assert_eq!(raster.get(2, 1), Some(white));
        assert_eq!(raster.get(1, 1), Some(black));
        assert_eq!(raster.get(3, 0), None);
    }

    #[test]
    fn an_indexed_raster_is_the_raster_of_its_colours() {
        let colors = [rgb(10, 20, 30), rgb(200, 100, 0)];
        let pick = |x: u32, y: u32| ((x + y) % 2) as usize;

        assert_eq!(
            Raster::indexed(5, 3, &colors, pick),
            Raster::from_fn(5, 3, |x, y| colors[pick(x, y)]),
        );
    }

    #[test]
    fn a_raster_from_a_function_runs_row_by_row() {
        let raster = Raster::from_fn(4, 2, |x, y| rgb(x as u8, y as u8, 0));

        assert_eq!(raster.get(3, 0), Some(rgb(3, 0, 0)));
        assert_eq!(raster.get(1, 1), Some(rgb(1, 1, 0)));
        assert_eq!(raster.pixels.len(), 4 * 2 * 4);
    }
}
