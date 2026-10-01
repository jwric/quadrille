//! Pixel fonts at sizes where one font pixel is one virtual pixel.
//!
//! A pixel font is only crisp at its native em or a whole multiple of it, and
//! only when its baseline lands on a whole pixel. A [`Face`] pins both: it is a
//! font, a scale and a line height whose baseline is known to be whole, so text
//! set in it can be aligned by arithmetic instead of by eye.
//!
//! cosmic-text centres a font's ascent-plus-descent box in the line and puts
//! the baseline at `(line - (ascent + descent)) / 2 + ascent`. That is a whole
//! pixel only when `line` and `ascent + descent` have the same parity, which
//! [`Face::new`] checks at compile time.
use std::borrow::Cow;

use iced_widget::core::{Font, Padding};
use iced_widget::text::{self, IntoFragment, LineHeight, Shaping, Wrapping};

/// The pixel metrics of a font at its native size.
///
/// Every value is in font pixels, which are virtual pixels when the font is
/// set at its native [`em`](Self::em).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metrics {
    /// The native size: the font size at which a font pixel is one pixel.
    pub em: u16,
    /// The font's ascent, as cosmic-text reads it.
    pub ascent: u16,
    /// The font's descent below the baseline, as cosmic-text reads it.
    pub descent: u16,
    /// The advance of every glyph; the font is monospaced.
    pub advance: u16,
    /// The height of a capital letter.
    pub cap: u16,
    /// The height of a lowercase `x`.
    pub x_height: u16,
    /// How far a descender reaches below the baseline.
    pub depth: u16,
    /// The empty columns left of a typical glyph in its cell.
    pub left: u16,
    /// The empty columns right of a typical glyph in its cell.
    pub right: u16,
}

impl Metrics {
    /// Departure Mono at 11 px: a 5 × 8 capital in a 7 px cell.
    pub const DEPARTURE_MONO: Self = Self {
        em: 11,
        ascent: 11,
        descent: 3,
        advance: 7,
        cap: 8,
        x_height: 6,
        depth: 2,
        left: 1,
        right: 1,
    };

    /// Departure Mono Tight: Departure Mono's glyphs in a 6 px cell.
    pub const DEPARTURE_MONO_TIGHT: Self = Self {
        advance: 6,
        right: 0,
        ..Self::DEPARTURE_MONO
    };
}

/// A pixel font at a whole scale, with a line height whose baseline is whole.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    /// The font.
    pub font: Font,
    metrics: Metrics,
    scale: u16,
    line: u16,
}

impl Face {
    /// The body face: Departure Mono Tight in a 6 × 12 cell.
    ///
    /// Two pixels over the capitals and two under the baseline, which is room
    /// for accents and descenders and nothing more.
    pub const BODY: Self = Self::new(
        Font::new("Departure Mono Tight"),
        Metrics::DEPARTURE_MONO_TIGHT,
        1,
        12,
    );

    /// The prose face: Departure Mono in its own 7 × 14 cell, for running text.
    pub const PROSE: Self = Self::new(Font::new("Departure Mono"), Metrics::DEPARTURE_MONO, 1, 14);

    /// Departure Mono at twice its size in a 14 × 24 cell, for readouts.
    pub const DISPLAY: Self =
        Self::new(Font::new("Departure Mono"), Metrics::DEPARTURE_MONO, 2, 24);

    /// Departure Mono at three times its size in a 21 × 36 cell.
    pub const HERO: Self = Self::new(Font::new("Departure Mono"), Metrics::DEPARTURE_MONO, 3, 36);

    /// Creates a [`Face`] for a font with the given [`Metrics`], set at
    /// `scale` times its native size in lines `line` pixels tall.
    ///
    /// # Panics
    ///
    /// Panics if `scale` is zero, or if the baseline of such a line would fall
    /// between two pixels. In a `const` this is a compile error.
    pub const fn new(font: Font, metrics: Metrics, scale: u16, line: u16) -> Self {
        assert!(scale > 0, "a face is at least its native size");

        let body = (metrics.ascent + metrics.descent) * scale;

        assert!(
            (line as i32 - body as i32) % 2 == 0,
            "the baseline of this line height falls between two pixels",
        );

        Self {
            font,
            metrics,
            scale,
            line,
        }
    }

    /// The same font at `scale` times this face's scale, with lines `scale`
    /// times as tall.
    pub const fn scaled(self, scale: u16) -> Self {
        Self::new(
            self.font,
            self.metrics,
            self.scale * scale,
            self.line * scale,
        )
    }

    /// The same face with lines `line` pixels tall.
    pub const fn with_line(self, line: u16) -> Self {
        Self::new(self.font, self.metrics, self.scale, line)
    }

    /// How many pixels one font pixel covers.
    pub const fn scale(self) -> u16 {
        self.scale
    }

    /// The same face with lines as tall as the font's ascent and descent, the
    /// height its box-drawing and block glyphs are drawn to: a column of them
    /// in consecutive lines joins up.
    pub const fn with_line_box(self) -> Self {
        let body = (self.metrics.ascent + self.metrics.descent) * self.scale;

        Self::new(self.font, self.metrics, self.scale, body)
    }

    /// The font size to set: the native em times the scale.
    pub const fn size(self) -> u16 {
        self.metrics.em * self.scale
    }

    /// The height of a line.
    pub const fn line(self) -> u16 {
        self.line
    }

    /// The width of one character cell.
    pub const fn advance(self) -> u16 {
        self.metrics.advance * self.scale
    }

    /// The distance from the top of a line to its baseline.
    pub const fn baseline(self) -> u16 {
        let body = (self.metrics.ascent + self.metrics.descent) * self.scale;
        let above = (self.line as i32 - body as i32) / 2;

        (above + (self.metrics.ascent * self.scale) as i32) as u16
    }

    /// The height of a capital letter.
    pub const fn cap(self) -> u16 {
        self.metrics.cap * self.scale
    }

    /// The height of a lowercase `x`.
    pub const fn x_height(self) -> u16 {
        self.metrics.x_height * self.scale
    }

    /// How far a descender reaches below the baseline.
    pub const fn depth(self) -> u16 {
        self.metrics.depth * self.scale
    }

    /// The distance from the top of a line to the top of its capitals.
    pub const fn cap_top(self) -> u16 {
        self.baseline() - self.cap()
    }

    /// Padding that leaves `air` pixels between a line of this face's ink and
    /// each side of its box, and `above` and `below` pixels over and under the
    /// line box.
    ///
    /// A glyph sits off-centre in its cell when its side bearings differ;
    /// padding the sides by the difference centres the ink, not the cells.
    pub fn padding(self, air: u16, above: u16, below: u16) -> Padding {
        let left = air.saturating_sub(self.metrics.left * self.scale);
        let right = air.saturating_sub(self.metrics.right * self.scale);

        Padding {
            top: f32::from(above),
            right: f32::from(right),
            bottom: f32::from(below),
            left: f32::from(left),
        }
    }

    /// A single line of `content` in this face, for a widget that draws its
    /// own text with [`fill_text`](iced::advanced::text::Renderer::fill_text).
    pub fn line_text(self, content: impl Into<String>) -> iced_widget::core::Text {
        let content = content.into();

        iced_widget::core::Text {
            bounds: iced_widget::core::Size::new(
                f32::from(self.width(&content)),
                f32::from(self.line),
            ),
            content,
            size: f32::from(self.size()).into(),
            line_height: LineHeight::Absolute(f32::from(self.line).into()),
            font: self.font,
            align_x: text::Alignment::Left,
            align_y: iced_widget::core::alignment::Vertical::Top,
            shaping: Shaping::Basic,
            wrapping: Wrapping::None,
            ellipsis: text::Ellipsis::None,
            // A pixel face is set at its native size on a whole line, so there
            // is nothing for metrics hinting to round.
            hint_factor: None,
        }
    }

    /// The width of `text` set in this face.
    pub fn width(self, text: &str) -> u16 {
        let cells = text.chars().count().min(u16::MAX as usize) as u16;

        cells.saturating_mul(self.advance())
    }

    /// How many whole cells fit in `width` pixels.
    pub fn columns(self, width: f32) -> usize {
        (width.max(0.0) / f32::from(self.advance())) as usize
    }

    /// `text` cut to fit in `width` pixels, ending in an ellipsis when cut.
    ///
    /// Text too long for even the ellipsis is dropped altogether: a label cut
    /// to one letter reads as a different label.
    pub fn fit(self, text: &str, width: f32) -> Cow<'_, str> {
        let columns = self.columns(width);
        let length = text.chars().count();

        if length <= columns {
            Cow::Borrowed(text)
        } else if columns < 2 {
            Cow::Borrowed("")
        } else {
            let kept: String = text.chars().take(columns - 1).collect();

            Cow::Owned(kept + "…")
        }
    }

    /// A text widget set in this face: native size, whole line height, no
    /// wrapping.
    pub fn text<'a, Theme>(self, fragment: impl IntoFragment<'a>) -> text::Text<'a, Theme>
    where
        Theme: text::Catalog,
    {
        text::Text::new(fragment)
            .font(self.font)
            .size(f32::from(self.size()))
            .line_height(LineHeight::Absolute(f32::from(self.line).into()))
            .wrapping(Wrapping::None)
            .shaping(Shaping::Basic)
    }
}

/// Spaces the characters of `text` a cell apart: `DEPARTURE` becomes
/// `D E P A R T U R E`.
///
/// Wide tracking done with whole cells keeps text on the character grid, where
/// letter spacing would move every glyph off it.
pub fn spaced(text: &str) -> String {
    let mut spaced = String::with_capacity(text.len() * 2);

    for (i, character) in text.chars().enumerate() {
        if i > 0 {
            spaced.push(' ');
        }

        spaced.push(character);
    }

    spaced
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_body_baseline_sits_two_pixels_above_the_line_bottom() {
        assert_eq!(Face::BODY.baseline(), 10);
        assert_eq!(Face::BODY.cap_top(), 2);
        assert_eq!(
            Face::BODY.line() - Face::BODY.baseline(),
            Face::BODY.depth()
        );
    }

    #[test]
    fn a_scaled_face_scales_every_metric() {
        let display = Face::PROSE.scaled(2).with_line(24);

        assert_eq!(display, Face::DISPLAY);
        assert_eq!(display.size(), 22);
        assert_eq!(display.advance(), 14);
        assert_eq!(display.cap(), 16);
        assert_eq!(display.baseline(), 20);
    }

    #[test]
    fn widths_count_characters_not_bytes() {
        assert_eq!(Face::BODY.width("°C"), 12);
        assert_eq!(Face::BODY.width(""), 0);
    }

    #[test]
    fn fitting_cuts_with_an_ellipsis_or_drops_the_text() {
        assert_eq!(Face::BODY.fit("AUTOMATIC", 60.0), "AUTOMATIC");
        assert_eq!(Face::BODY.fit("AUTOMATIC", 30.0), "AUTO…");
        assert_eq!(Face::BODY.fit("AUTOMATIC", 6.0), "");
    }

    #[test]
    fn spacing_puts_a_cell_between_characters() {
        assert_eq!(spaced("AUTO"), "A U T O");
        assert_eq!(spaced(""), "");
    }
}
