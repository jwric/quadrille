//! The bench: a scope on two test points, the power buses over the last
//! minute, and the temperatures of the hull.
//!
//! CH1 is a 1 kHz tone with harmonics, a little noise and a glitch every
//! other cycle; CH2 a sine swept between 6 and 14 kHz. The scope triggers on
//! CH1. Every reading on the page is a function of the time, so a frame can
//! be drawn again by asking for the same second.
use std::f64::consts::TAU;

use iced::widget::{column, container, row, space};
use iced::{Alignment, Length};
use quadrille::instrument::{self, Cursors};
use quadrille::scale::engineering;
use quadrille::widget::{self, group, knob, label};
use quadrille::{Element, Theme, px, style};

use crate::thermal::{self, Field};
use crate::{Telemetry, trend};
use iced::Widget as _;

/// The seconds of the power buses the chart shows.
const MINUTE: f32 = 60.0;

/// Samples across the scope's screen.
const SAMPLES: usize = 1000;
/// Divisions across the screen.
const COLUMNS: u16 = 10;
/// The division the trigger sits on.
const PRETRIGGER: f64 = 2.0;
/// Volts a division, on both channels.
const VOLTS: f32 = 0.5;

/// The state of the page.
#[derive(Debug, Clone)]
pub struct Scope {
    timebase: Timebase,
    trigger: i32,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            timebase: Timebase::Micros200,
            trigger: 3,
        }
    }
}

/// A message of the page.
#[derive(Debug, Clone)]
pub enum Message {
    /// A timebase was chosen.
    Timebase(Timebase),
    /// The trigger knob turned, in tenths of a volt.
    Trigger(i32),
}

/// The time a division across the scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timebase {
    Micros100,
    Micros200,
    Micros500,
    Millis1,
}

impl Timebase {
    const ALL: [(Self, &'static str); 4] = [
        (Self::Micros100, "100µ"),
        (Self::Micros200, "200µ"),
        (Self::Micros500, "500µ"),
        (Self::Millis1, "1m"),
    ];

    fn seconds(self) -> f64 {
        match self {
            Self::Micros100 => 100e-6,
            Self::Micros200 => 200e-6,
            Self::Micros500 => 500e-6,
            Self::Millis1 => 1e-3,
        }
    }
}

impl Scope {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Timebase(timebase) => self.timebase = timebase,
            Message::Trigger(trigger) => self.trigger = trigger,
        }
    }

    pub fn view(&self, telemetry: &Telemetry) -> Element<'_, Message> {
        let now = f64::from(telemetry.elapsed);
        let level = self.trigger as f32 / 10.0;

        let left = column![
            self.screen(now, level),
            space::vertical(),
            row![
                group(
                    "TIME/DIV",
                    widget::selector(Timebase::ALL, Some(self.timebase), Message::Timebase),
                ),
                group(
                    "TRIGGER",
                    row![
                        knob(-20..=20, self.trigger, 1, Message::Trigger).diameter(19),
                        column![
                            reading("LEVEL", engineering(level, "V")),
                            reading("SLOPE", "CH1 ↗".to_owned()),
                        ]
                        .spacing(px::TIGHT),
                    ]
                    .spacing(px::WIDE)
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill),
            ]
            .spacing(px::WIDE),
        ]
        .width(264.0);

        let field = Field::of(telemetry);

        let right = column![
            trend::trend(telemetry.elapsed, MINUTE, 22.0..=30.0, "V")
                .series("BUS A", |t| Telemetry::at(t).bus[0])
                .series("BUS B", |t| Telemetry::at(t).bus[1])
                .series("BUS C", |t| Telemetry::at(t).bus[2])
                .gutter(thermal::GUTTER)
                .height(128.0),
            thermal::thermal(field),
        ]
        .spacing(px::GAP)
        .width(Length::Fill);

        let one = measure(now, ch1);
        let two = measure(now, ch2);
        let survey = field.survey();

        let readouts = row![
            channel_group("CH1", 0, &one),
            channel_group("CH2", 1, &two),
            group(
                "THERMAL",
                row![
                    column![
                        reading("MAX", celsius(survey.hottest)),
                        reading("MIN", celsius(survey.coldest)),
                        reading("MEAN", celsius(survey.mean)),
                    ]
                    .spacing(px::TIGHT)
                    .width(Length::Fill),
                    column![
                        reading("SUN", format!("{:03.0}°", field.sun)),
                        reading("BAY", celsius(field.bay())),
                        reading("AFT", celsius(field.aft())),
                    ]
                    .spacing(px::TIGHT)
                    .width(Length::Fill),
                ]
                .spacing(px::FAR),
            )
            .width(Length::FillPortion(2)),
        ]
        .spacing(px::FAR);

        container(
            column![
                row![left, right].spacing(px::FAR).height(Length::Fill),
                readouts,
            ]
            .spacing(px::WIDE),
        )
        .padding(px::WIDE)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::container::ground)
        .boxed()
    }

    /// The scope's screen: both channels from a little before the trigger,
    /// and cursors across one cycle of CH1.
    fn screen<'a>(&self, now: f64, level: f32) -> Element<'a, Message> {
        let per_division = self.timebase.seconds();
        let window = per_division * f64::from(COLUMNS);
        let trigger = triggered(now, f64::from(level)).unwrap_or(now);
        let start = trigger - PRETRIGGER * per_division;

        let sample = |signal: fn(f64) -> f64| -> Vec<f32> {
            (0..SAMPLES)
                .map(|i| signal(start + window * i as f64 / (SAMPLES - 1) as f64) as f32)
                .collect()
        };

        let cycle = PRETRIGGER + 1e-3 / per_division;

        let mut screen = instrument::plot()
            .push(
                instrument::trace(sample(ch1))
                    .scale(VOLTS, "V")
                    .offset(1.5)
                    .label("CH1"),
            )
            .push(
                instrument::trace(sample(ch2))
                    .scale(VOLTS, "V")
                    .offset(-2.5)
                    .label("CH2"),
            )
            .horizontal(per_division as f32, "s")
            .trigger(PRETRIGGER as f32, level)
            .height(229.0);

        if cycle <= f64::from(COLUMNS) {
            screen = screen.cursors(Cursors::Vertical(PRETRIGGER as f32, cycle as f32));
        }

        screen.boxed()
    }
}

/// A temperature, signed, in whole degrees.
fn celsius(degrees: f32) -> String {
    format!("{degrees:+.0}°C")
}

/// A label and a value on one line, the value set flush right.
fn reading<'a>(name: &'a str, value: String) -> Element<'a, Message> {
    row![
        label(name).style(style::text::muted),
        space::horizontal(),
        label(value),
    ]
    .boxed()
}

/// A channel's measurements, under its name in the channel's colour.
fn channel_group<'a>(name: &'a str, index: usize, measured: &Measured) -> Element<'a, Message> {
    let color = move |theme: &Theme| widget::group::Style {
        name: instrument::channel(theme.palette(), index),
        ..widget::group::default(theme)
    };

    group(
        name,
        column![
            reading("FREQ", engineering(measured.frequency as f32, "Hz")),
            reading("VPP", engineering(measured.peak_to_peak as f32, "V")),
            reading("RMS", engineering(measured.rms as f32, "V")),
        ]
        .spacing(px::TIGHT),
    )
    .style(color)
    .width(Length::FillPortion(1))
    .boxed()
}

/// CH1: the tone, a trace of noise, and a glitch 15 µs wide in the trough
/// of every other cycle.
fn ch1(t: f64) -> f64 {
    let cycle = 1_000.0 * t;
    let glitch = cycle.floor() as i64 % 2 == 1 && (0.80..0.815).contains(&cycle.fract());

    clean(t) + 0.004 * noise(t, 1) + if glitch { 0.6 } else { 0.0 }
}

/// A 1 kHz tone with harmonics that breathes a little: what the trigger
/// follows.
fn clean(t: f64) -> f64 {
    let phase = TAU * 1_000.0 * t;
    let swell = 1.0 + 0.06 * (TAU * t / 11.0).sin();

    0.9 * swell
        * (phase.sin()
            + 0.2 * (3.0 * phase).sin()
            + 0.08 * (5.0 * phase + 0.4).sin()
            + 0.03 * (2.0 * phase + 1.0).sin())
}

/// CH2: a sine swept between 6 and 14 kHz every seven seconds, and a trace
/// of noise.
fn ch2(t: f64) -> f64 {
    let sweep = 10_000.0 * t - 4_000.0 * 7.0 / TAU * (TAU * t / 7.0).cos();

    0.6 * (TAU * sweep).sin() + 0.004 * noise(t, 2)
}

/// Noise from -1 to 1, fixed for each microsecond and channel.
fn noise(t: f64, channel: u64) -> f64 {
    let mut x = ((t * 1e6).floor() as i64 as u64) ^ channel.wrapping_mul(0x9e37_79b9_7f4a_7c15);

    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 31;

    (x >> 11) as f64 / (1u64 << 52) as f64 - 1.0
}

/// The time CH1 next rises through `level`, within two cycles of `now`.
fn triggered(now: f64, level: f64) -> Option<f64> {
    const STEP: f64 = 1e-6;

    let mut before = clean(now);

    for i in 1..2_000 {
        let t = now + i as f64 * STEP;
        let after = clean(t);

        if before < level && after >= level {
            return Some(t);
        }

        before = after;
    }

    None
}

/// What the scope measures on a channel over the last 5 ms.
struct Measured {
    frequency: f64,
    peak_to_peak: f64,
    rms: f64,
}

fn measure(now: f64, signal: fn(f64) -> f64) -> Measured {
    const STEP: f64 = 2e-6;
    const COUNT: usize = 2_500;

    let samples: Vec<f64> = (0..COUNT)
        .map(|i| signal(now - (COUNT - i) as f64 * STEP))
        .collect();

    let (low, high) = samples
        .iter()
        .fold((f64::MAX, f64::MIN), |(low, high), &v| {
            (low.min(v), high.max(v))
        });
    let rms = (samples.iter().map(|v| v * v).sum::<f64>() / COUNT as f64).sqrt();

    // Rising crossings of zero, with a little hysteresis against the noise.
    let mut armed = false;
    let mut crossings = Vec::new();

    for (i, pair) in samples.windows(2).enumerate() {
        if pair[0] < -0.1 {
            armed = true;
        }

        if armed && pair[0] < 0.0 && pair[1] >= 0.0 {
            let fraction = -pair[0] / (pair[1] - pair[0]);

            crossings.push((i as f64 + fraction) * STEP);
            armed = false;
        }
    }

    let frequency = match (crossings.first(), crossings.last()) {
        (Some(first), Some(last)) if crossings.len() > 1 => {
            (crossings.len() - 1) as f64 / (last - first)
        }
        _ => 0.0,
    };

    Measured {
        frequency,
        peak_to_peak: high - low,
        rms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scope_measures_ch1() {
        let measured = measure(42.0, ch1);

        assert!(
            (measured.frequency - 1_000.0).abs() < 5.0,
            "{}",
            measured.frequency
        );
    }
}
