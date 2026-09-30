//! The font files the toolkit ships.
//!
//! All of them are licensed under the SIL Open Font License 1.1; the license
//! text is in `fonts/OFL.txt` beside them.

/// Departure Mono by Helena Zhang: 11 px em, a 7 × 14 cell.
pub const DEPARTURE_MONO: &[u8] = include_bytes!("../fonts/DepartureMono-Regular.otf");

/// Departure Mono Tight: Departure Mono's glyphs in a 6 × 12 cell.
///
/// Derived from Departure Mono by narrowing the advance to 6 px. Box-drawing
/// and block glyphs are cropped to the narrower cell so they still tile.
pub const DEPARTURE_MONO_TIGHT: &[u8] = include_bytes!("../fonts/DepartureMonoTight-Regular.ttf");

/// Every font above, for [`iced::Settings::fonts`].
pub const ALL: &[&[u8]] = &[DEPARTURE_MONO, DEPARTURE_MONO_TIGHT];
