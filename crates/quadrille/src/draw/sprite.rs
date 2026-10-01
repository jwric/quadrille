use iced_widget::core::Rectangle;

/// A 1-bit bitmap written as art, one string per row, `#` for a lit pixel.
///
/// ```
/// use quadrille::draw::Sprite;
///
/// const DIAMOND: Sprite = Sprite::new(&[
///     "..#..",
///     ".#.#.",
///     "#...#",
///     ".#.#.",
///     "..#..",
/// ]);
///
/// assert_eq!((DIAMOND.width(), DIAMOND.height()), (5, 5));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sprite {
    rows: &'static [&'static str],
}

impl Sprite {
    /// Creates a [`Sprite`] from its rows.
    ///
    /// # Panics
    ///
    /// Panics if the rows are not all the same length, which in a `const` is
    /// a compile error.
    pub const fn new(rows: &'static [&'static str]) -> Self {
        let mut i = 1;

        while i < rows.len() {
            assert!(
                rows[i].len() == rows[0].len(),
                "every row of a sprite is the same width"
            );
            i += 1;
        }

        Self { rows }
    }

    /// The width of the sprite.
    pub const fn width(&self) -> i32 {
        if self.rows.is_empty() {
            0
        } else {
            self.rows[0].len() as i32
        }
    }

    /// The height of the sprite.
    pub const fn height(&self) -> i32 {
        self.rows.len() as i32
    }

    /// Whether the pixel at `(x, y)` is lit.
    pub fn lit(&self, x: i32, y: i32) -> bool {
        usize::try_from(y)
            .ok()
            .and_then(|y| self.rows.get(y))
            .zip(usize::try_from(x).ok())
            .and_then(|(row, x)| row.as_bytes().get(x))
            == Some(&b'#')
    }

    /// The lit pixels as horizontal strips, relative to the top-left corner.
    pub fn strips(&self) -> impl Iterator<Item = Rectangle<i32>> + '_ {
        self.rows.iter().enumerate().flat_map(|(y, row)| {
            let bytes = row.as_bytes();
            let mut x = 0;

            std::iter::from_fn(move || {
                while x < bytes.len() && bytes[x] != b'#' {
                    x += 1;
                }

                if x == bytes.len() {
                    return None;
                }

                let start = x;

                while x < bytes.len() && bytes[x] == b'#' {
                    x += 1;
                }

                Some(Rectangle {
                    x: start as i32,
                    y: y as i32,
                    width: (x - start) as i32,
                    height: 1,
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARROW: Sprite = Sprite::new(&["#..", "##.", "###", "##.", "#.."]);

    #[test]
    fn strips_cover_exactly_the_lit_pixels() {
        let strips: Vec<_> = ARROW.strips().collect();

        assert_eq!(strips.len(), 5);
        assert_eq!(strips[2].width, 3);

        let lit: i32 = strips.iter().map(|strip| strip.width).sum();

        assert_eq!(lit, 9);
        assert!(ARROW.lit(2, 2));
        assert!(!ARROW.lit(2, 1));
        assert!(!ARROW.lit(-1, 0));
    }
}
