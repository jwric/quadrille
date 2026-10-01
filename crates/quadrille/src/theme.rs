//! The theme: a palette named for roles, and the styles derived from it.
//!
//! A [`Palette`] has a handful of surfaces, three inks, two kinds of line and
//! a few signal colours. Every style in the toolkit reads one of these roles,
//! so swapping the palette re-dresses the whole interface: nothing outside a
//! palette names a hex.
mod catalog;

use std::borrow::Cow;

use iced_widget::core::Color;
use iced_widget::core::theme::{self, Base, Mode};

/// A quadrille theme: a named [`Palette`].
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    name: Cow<'static, str>,
    palette: Palette,
}

impl Theme {
    /// Grey on charcoal with an amber accent: a terminal at night.
    pub const TERMINAL: Self = Self::new_static("Terminal", Palette::TERMINAL);

    /// Graphite on off-white with a pumpkin accent: a printout by day.
    pub const PAPER: Self = Self::new_static("Paper", Palette::PAPER);

    /// A green phosphor tube.
    pub const PHOSPHOR: Self = Self::new_static("Phosphor", Palette::PHOSPHOR);

    /// An amber phosphor tube.
    pub const AMBER: Self = Self::new_static("Amber", Palette::AMBER);

    /// A reflective monochrome LCD.
    pub const LCD: Self = Self::new_static("LCD", Palette::LCD);

    /// Every built-in theme.
    pub const ALL: &'static [Self] = &[
        Self::TERMINAL,
        Self::PAPER,
        Self::PHOSPHOR,
        Self::AMBER,
        Self::LCD,
    ];

    /// Creates a [`Theme`] from a name and a [`Palette`].
    pub fn new(name: impl Into<Cow<'static, str>>, palette: Palette) -> Self {
        Self {
            name: name.into(),
            palette,
        }
    }

    const fn new_static(name: &'static str, palette: Palette) -> Self {
        Self {
            name: Cow::Borrowed(name),
            palette,
        }
    }

    /// The [`Palette`] of the theme.
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// The name of the theme.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::TERMINAL
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl Base for Theme {
    fn default(preference: Mode) -> Self {
        match preference {
            Mode::Light => Self::PAPER,
            Mode::Dark | Mode::None => Self::TERMINAL,
        }
    }

    fn mode(&self) -> Mode {
        if self.palette.is_dark() {
            Mode::Dark
        } else {
            Mode::Light
        }
    }

    fn base(&self) -> theme::Style {
        theme::Style {
            background_color: self.palette.void,
            text_color: self.palette.ink,
        }
    }

    fn seed(&self) -> Option<theme::palette::Seed> {
        Some(theme::palette::Seed {
            background: self.palette.ground,
            text: self.palette.ink,
            primary: self.palette.accent,
            success: self.palette.live,
            warning: self.palette.caution,
            danger: self.palette.alarm,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// The colours of a [`Theme`], by role.
///
/// Surfaces run from [`void`](Self::void) to [`hover`](Self::hover); a
/// control changes state by stepping along them rather than by taking a new
/// colour. Signal colours carry meaning and are spent sparingly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Behind everything: the window, and the glass of a display.
    pub void: Color,
    /// A panel's face, which text and controls sit on.
    pub ground: Color,
    /// A control's face: keys, unlit lamps, the unfilled part of a meter.
    pub raised: Color,
    /// A control under the cursor.
    pub hover: Color,
    /// The one colour of borders and dividers, always a hairline.
    pub edge: Color,

    /// Text and marks.
    pub ink: Color,
    /// Labels: present, not competing.
    pub muted: Color,
    /// Engraving: disabled legends, minor ticks, the parts one skims past.
    pub faint: Color,
    /// Schematic line work: recessive against the ground, never competing
    /// with what is drawn over it.
    pub line: Color,

    /// State: what is selected, focused, engaged or powered.
    pub accent: Color,
    /// Text and marks drawn on the accent, like an inverse label.
    pub on_accent: Color,
    /// Passive emphasis: a marked row, a highlight behind text.
    pub highlight: Color,
    /// Live data: a trace, a lit segment.
    pub live: Color,
    /// Worth a look: degraded, filling, falling back.
    pub caution: Color,
    /// Wrong: faults, overloads, the red end of a meter.
    pub alarm: Color,
}

impl Palette {
    /// The palette of [`Theme::TERMINAL`].
    pub const TERMINAL: Self = Self {
        void: rgb(0x1a, 0x1a, 0x1a),
        ground: rgb(0x22, 0x22, 0x22),
        raised: rgb(0x2e, 0x2e, 0x2e),
        hover: rgb(0x3a, 0x3a, 0x3a),
        edge: rgb(0x48, 0x48, 0x44),
        ink: rgb(0xc0, 0xc0, 0xc0),
        muted: rgb(0x8e, 0x8e, 0x8e),
        faint: rgb(0x5a, 0x5a, 0x5a),
        line: rgb(0x6c, 0x6c, 0x58),
        accent: rgb(0xff, 0xa1, 0x33),
        on_accent: rgb(0x1a, 0x1a, 0x1a),
        highlight: rgb(0xbc, 0xca, 0xbb),
        live: rgb(0x9f, 0xc7, 0x9a),
        caution: rgb(0xe4, 0xc6, 0x4c),
        alarm: rgb(0xe8, 0x55, 0x3e),
    };

    /// The palette of [`Theme::PAPER`].
    pub const PAPER: Self = Self {
        void: rgb(0xe2, 0xe2, 0xe2),
        ground: rgb(0xee, 0xee, 0xee),
        raised: rgb(0xe0, 0xe0, 0xdc),
        hover: rgb(0xd2, 0xd2, 0xcc),
        edge: rgb(0xb0, 0xb0, 0xa6),
        ink: rgb(0x44, 0x44, 0x44),
        muted: rgb(0x66, 0x66, 0x66),
        faint: rgb(0x9a, 0x9a, 0x94),
        line: rgb(0x8a, 0x8a, 0x6f),
        accent: rgb(0xe4, 0x7b, 0x1a),
        on_accent: rgb(0x22, 0x22, 0x22),
        highlight: rgb(0xbc, 0xca, 0xbb),
        live: rgb(0x3f, 0x7a, 0x4a),
        caution: rgb(0xa8, 0x86, 0x1a),
        alarm: rgb(0xc0, 0x39, 0x2b),
    };

    /// The palette of [`Theme::PHOSPHOR`].
    pub const PHOSPHOR: Self = Self {
        void: rgb(0x04, 0x0a, 0x06),
        ground: rgb(0x08, 0x12, 0x0b),
        raised: rgb(0x10, 0x22, 0x16),
        hover: rgb(0x18, 0x31, 0x20),
        edge: rgb(0x1f, 0x42, 0x2b),
        ink: rgb(0x7d, 0xf0, 0xa0),
        muted: rgb(0x4c, 0xa8, 0x6c),
        faint: rgb(0x2a, 0x5e, 0x3c),
        line: rgb(0x2f, 0x6e, 0x46),
        accent: rgb(0xd2, 0xff, 0xdf),
        on_accent: rgb(0x04, 0x0a, 0x06),
        highlight: rgb(0x1c, 0x4a, 0x2e),
        live: rgb(0x7d, 0xf0, 0xa0),
        caution: rgb(0xd8, 0xf0, 0x6a),
        alarm: rgb(0xff, 0x6a, 0x5a),
    };

    /// The palette of [`Theme::AMBER`].
    pub const AMBER: Self = Self {
        void: rgb(0x0d, 0x09, 0x04),
        ground: rgb(0x15, 0x0f, 0x08),
        raised: rgb(0x22, 0x19, 0x0e),
        hover: rgb(0x30, 0x23, 0x13),
        edge: rgb(0x46, 0x33, 0x1a),
        ink: rgb(0xff, 0xb3, 0x47),
        muted: rgb(0xb8, 0x7d, 0x2e),
        faint: rgb(0x62, 0x44, 0x1f),
        line: rgb(0x7e, 0x58, 0x27),
        accent: rgb(0xff, 0xdc, 0xa4),
        on_accent: rgb(0x0d, 0x09, 0x04),
        highlight: rgb(0x40, 0x2d, 0x14),
        live: rgb(0xff, 0xb3, 0x47),
        caution: rgb(0xff, 0xd4, 0x70),
        alarm: rgb(0xff, 0x5c, 0x3c),
    };

    /// The palette of [`Theme::LCD`].
    pub const LCD: Self = Self {
        void: rgb(0x9a, 0xa4, 0x8b),
        ground: rgb(0xb0, 0xba, 0x9f),
        raised: rgb(0xa4, 0xae, 0x93),
        hover: rgb(0x98, 0xa2, 0x87),
        edge: rgb(0x6c, 0x76, 0x5e),
        ink: rgb(0x22, 0x2a, 0x1c),
        muted: rgb(0x46, 0x51, 0x3c),
        faint: rgb(0x7a, 0x85, 0x6a),
        line: rgb(0x5c, 0x67, 0x50),
        accent: rgb(0x22, 0x2a, 0x1c),
        on_accent: rgb(0xb0, 0xba, 0x9f),
        highlight: rgb(0x94, 0x9f, 0x82),
        live: rgb(0x22, 0x2a, 0x1c),
        caution: rgb(0x5a, 0x4c, 0x12),
        alarm: rgb(0x70, 0x1e, 0x12),
    };

    /// Whether the ground is darker than the ink.
    pub fn is_dark(&self) -> bool {
        luminance(self.ground) < luminance(self.ink)
    }

    /// The colour to draw text in on `fill`: the ink or the void, whichever
    /// stands further from it.
    pub fn on(&self, fill: Color) -> Color {
        let fill = luminance(fill);

        if (luminance(self.ink) - fill).abs() >= (luminance(self.void) - fill).abs() {
            self.ink
        } else {
            self.void
        }
    }
}

/// A [`Color`] from 8-bit channels, at compile time.
pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

/// `color` with its channels scaled by `factor`: a dimmer shade of the same
/// hue, for putting a layer under another without alpha.
pub fn dim(color: Color, factor: f32) -> Color {
    Color {
        r: quantize(color.r * factor),
        g: quantize(color.g * factor),
        b: quantize(color.b * factor),
        a: color.a,
    }
}

/// The colour a fraction `t` of the way from `from` to `to`.
///
/// Channels are rounded to 8 bits, so a mix names a colour the display can
/// show and two mixes of the same colours are the same colour.
pub fn mix(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);

    Color {
        r: quantize(from.r + (to.r - from.r) * t),
        g: quantize(from.g + (to.g - from.g) * t),
        b: quantize(from.b + (to.b - from.b) * t),
        a: from.a + (to.a - from.a) * t,
    }
}

fn quantize(channel: f32) -> f32 {
    (channel.clamp(0.0, 1.0) * 255.0).round() / 255.0
}

fn luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_knows_its_mode() {
        for theme in Theme::ALL {
            let expected = if theme == &Theme::PAPER || theme == &Theme::LCD {
                Mode::Light
            } else {
                Mode::Dark
            };

            assert_eq!(theme.mode(), expected, "{theme}");
        }
    }

    #[test]
    fn mixes_land_on_eight_bit_colours() {
        let mixed = mix(rgb(0, 0, 0), rgb(255, 255, 255), 0.5);

        assert_eq!(mixed.r * 255.0, (mixed.r * 255.0).round());
        assert_eq!(mix(rgb(10, 20, 30), rgb(40, 50, 60), 0.0), rgb(10, 20, 30));
        assert_eq!(mix(rgb(10, 20, 30), rgb(40, 50, 60), 1.0), rgb(40, 50, 60));
    }
}
