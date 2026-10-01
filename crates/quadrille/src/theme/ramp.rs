use iced_widget::core::Color;

use super::{Palette, mix};

/// Flat steps of colour from the void out through a palette's signal
/// colours, each standing further from the void than the last: the colours
/// of a heat map, a density or any level drawn as a tone.
///
/// The steps are built from the void, the line colour, the live colour, the
/// accent and the ink, in that order, keeping only the stops that stand
/// clearly further from the void than the stop before: a palette whose
/// accent is no brighter than its live colour leaves the accent out, and a
/// level that reads louder is always drawn louder. The steps between stops
/// are spaced evenly in lightness. A level takes the step it falls in, with
/// no blending between steps.
#[derive(Debug, Clone, PartialEq)]
pub struct Ramp {
    steps: Vec<Color>,
}

impl Ramp {
    /// The number of steps in [`Ramp::of`].
    pub const STEPS: usize = 12;

    /// The least a stop must stand further from the void than the one
    /// before to be kept, in CIE lightness.
    const GAIN: f32 = 4.0;

    /// The ramp of `palette`, in [`Ramp::STEPS`] steps.
    pub fn of(palette: &Palette) -> Self {
        Self::new(palette, Self::STEPS)
    }

    /// The ramp of `palette` in `steps` steps.
    pub fn new(palette: &Palette, steps: usize) -> Self {
        let steps = steps.max(2);
        let base = lightness(palette.void);
        let contrast = |color: Color| (lightness(color) - base).abs();

        let mut stops = vec![(palette.void, 0.0)];

        for color in [palette.line, palette.live, palette.accent, palette.ink] {
            let distance = contrast(color);

            if distance >= stops[stops.len() - 1].1 + Self::GAIN {
                stops.push((color, distance));
            }
        }

        let far = stops[stops.len() - 1].1;

        let steps = (0..steps)
            .map(|i| {
                let target = far * i as f32 / (steps - 1) as f32;
                let segment = stops
                    .windows(2)
                    .position(|pair| target <= pair[1].1)
                    .unwrap_or(stops.len().saturating_sub(2));

                match stops.get(segment..segment + 2) {
                    Some([(from, near), (to, next)]) => {
                        mix(*from, *to, (target - near) / (next - near))
                    }
                    _ => palette.void,
                }
            })
            .collect();

        Self { steps }
    }

    /// The step `level` falls in, for a level from 0 to 1.
    pub fn step(&self, level: f32) -> usize {
        let count = self.steps.len();

        if level.is_nan() {
            return 0;
        }

        ((level.clamp(0.0, 1.0) * count as f32) as usize).min(count - 1)
    }

    /// The colour of `level`, from 0 to 1.
    pub fn color(&self, level: f32) -> Color {
        self.steps[self.step(level)]
    }

    /// The colours of the steps, from the void out.
    pub fn steps(&self) -> &[Color] {
        &self.steps
    }
}

/// The CIE lightness of an sRGB colour, from 0 to 100.
fn lightness(color: Color) -> f32 {
    let linear = |channel: f32| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };

    let luminance = 0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b);

    if luminance > 216.0 / 24389.0 {
        116.0 * luminance.cbrt() - 16.0
    } else {
        luminance * 24389.0 / 27.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Theme;

    #[test]
    fn every_ramp_steps_away_from_the_void() {
        for theme in Theme::ALL {
            let palette = theme.palette();
            let ramp = Ramp::of(palette);
            let base = lightness(palette.void);
            let contrast: Vec<f32> = ramp
                .steps()
                .iter()
                .map(|c| (lightness(*c) - base).abs())
                .collect();

            assert_eq!(ramp.steps()[0], palette.void, "{theme}");

            for pair in contrast.windows(2) {
                assert!(pair[1] > pair[0], "{theme}: {contrast:?}");
            }
        }
    }

    #[test]
    fn a_ramp_leaves_out_a_stop_that_would_step_back() {
        let palette = Palette::TERMINAL;
        let ramp = Ramp::of(&palette);

        assert!(!ramp.steps().contains(&palette.accent));
        assert_eq!(ramp.steps().last(), Some(&palette.live));
    }

    #[test]
    fn levels_take_whole_steps() {
        let ramp = Ramp::new(&Palette::PHOSPHOR, 4);

        assert_eq!(ramp.step(0.0), 0);
        assert_eq!(ramp.step(0.24), 0);
        assert_eq!(ramp.step(0.25), 1);
        assert_eq!(ramp.step(0.99), 3);
        assert_eq!(ramp.step(1.0), 3);
        assert_eq!(ramp.step(-3.0), 0);
        assert_eq!(ramp.step(f32::NAN), 0);
        assert_eq!(ramp.color(0.6), ramp.steps()[2]);
    }
}
