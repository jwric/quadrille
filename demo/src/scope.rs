//! Signal instruments: a scope, a spectrum and its waterfall.
//!
//! Two channels on a bench: CH1 a 1 kHz tone with harmonics, a little noise
//! and a glitch every other cycle; CH2 a carrier swept around 10 kHz with a
//! beacon keyed on and off beside it. The scope triggers on CH1, and the
//! analyser takes both at once. Every sample is a function of the time, so a
//! frame can be drawn again by asking for the same second; only the
//! waterfall's history is carried from tick to tick, and it starts out as if
//! the analyser had been running.
use std::cell::OnceCell;
use std::f64::consts::TAU;

use iced::widget::{column, container, row, space};
use iced::{Alignment, Length};
use quadrille::instrument::{self, Cursors, History};
use quadrille::scale::engineering;
use quadrille::widget::{self, group, knob, label};
use quadrille::{Element, Theme, px, style};

use crate::Telemetry;
use iced::Widget as _;

/// The analyser's sample rate: 1024 points make bins 40 Hz wide.
const RATE: f64 = 40_960.0;
/// Points in a transform.
const POINTS: usize = 1024;
/// Bins from 0 Hz to half the sample rate.
const BINS: usize = POINTS / 2;
/// The width of a bin, in Hz.
const WIDTH: f32 = (RATE / POINTS as f64) as f32;
/// Rows of history the waterfall keeps.
const DEPTH: usize = 160;
/// The seconds between two ticks of the clock, and between two rows.
const TICK: f64 = 0.066;
/// The analyser's floor, in dB.
const FLOOR: f32 = -100.0;
/// The analyser's ceiling, in dB.
const CEILING: f32 = 0.0;
/// The dB of a division on the analyser.
const DIVISION: f32 = 20.0;
/// The waterfall's floor, in dB: the analyser's noise stays dark under it.
const APERTURE: f32 = -76.0;
/// How far a held peak falls each row, in dB.
const FALL: f32 = 0.6;

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
    history: OnceCell<History>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            timebase: Timebase::Micros200,
            trigger: 3,
            history: OnceCell::new(),
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

    /// Adds the analyser's row for this tick to the waterfall, once the page
    /// has been shown.
    pub fn tick(&mut self, telemetry: &Telemetry) {
        if let Some(history) = self.history.get_mut() {
            history.push(&analyse(f64::from(telemetry.elapsed)));
        }
    }

    pub fn view(&self, telemetry: &Telemetry) -> Element<'_, Message> {
        let now = f64::from(telemetry.elapsed);
        let history = self.history.get_or_init(|| seeded(now));
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

        let latest = history.row(0).unwrap_or(&[]);
        let peak = strongest(latest, 0.0..f32::MAX);
        let carrier = strongest(latest, 5_500.0..f32::MAX);

        let right = column![
            instrument::spectrum(latest)
                .range(FLOOR..=CEILING, DIVISION)
                .span(-WIDTH / 2.0..=(BINS as f32 - 0.5) * WIDTH, "Hz")
                .hold(history.hold(FALL))
                .marker(peak.0)
                .marker(carrier.0)
                .height(128.0),
            instrument::waterfall(history)
                .range(APERTURE..=CEILING)
                .rate((1.0 / TICK) as f32),
        ]
        .spacing(px::GAP)
        .width(Length::Fill);

        let one = measure(now, ch1);
        let two = measure(now, ch2);
        let floor = median(latest);

        let readouts = row![
            channel_group("CH1", 0, &one),
            channel_group("CH2", 1, &two),
            group(
                "ANALYSER",
                row![
                    column![
                        reading("M1", engineering(peak.0, "Hz")),
                        reading("M2", engineering(carrier.0, "Hz")),
                        reading("FLOOR", format!("{floor:.0} dB")),
                    ]
                    .spacing(px::TIGHT)
                    .width(Length::Fill),
                    column![
                        reading("LVL", format!("{:.1} dB", peak.1)),
                        reading("LVL", format!("{:.1} dB", carrier.1)),
                        reading("SNR", format!("{:.0} dB", peak.1 - floor)),
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

/// CH1 as the probe sees it: the tone, and a glitch 15 µs wide in the
/// trough of every other cycle.
fn ch1(t: f64) -> f64 {
    let cycle = 1_000.0 * t;
    let glitch = cycle.floor() as i64 % 2 == 1 && (0.80..0.815).contains(&cycle.fract());

    tone(t) + if glitch { 0.6 } else { 0.0 }
}

/// CH1 as the analyser sees it, through a filter that a glitch does not
/// pass: a 1 kHz tone with harmonics that breathes a little, a trace of
/// noise, and a weak carrier wandering around 12 kHz far under it.
fn tone(t: f64) -> f64 {
    let wander = 12_000.0 * t + 400.0 * 5.0 / TAU * (1.0 - (TAU * t / 5.0).cos());

    clean(t) + 0.004 * (TAU * wander).sin() + 0.004 * noise(t, 1)
}

/// The 1 kHz tone alone, which the trigger follows.
fn clean(t: f64) -> f64 {
    let phase = TAU * 1_000.0 * t;
    let swell = 1.0 + 0.06 * (TAU * t / 11.0).sin();

    0.9 * swell
        * (phase.sin()
            + 0.2 * (3.0 * phase).sin()
            + 0.08 * (5.0 * phase + 0.4).sin()
            + 0.03 * (2.0 * phase + 1.0).sin())
}

/// CH2: a carrier swept between 6 and 14 kHz every seven seconds, and a
/// beacon at 17 kHz keyed on and off every 1.2 seconds.
fn ch2(t: f64) -> f64 {
    let sweep = 10_000.0 * t - 4_000.0 * 7.0 / TAU * (TAU * t / 7.0).cos();
    let keyed = if (t / 1.2).floor() as i64 % 2 == 0 {
        0.05
    } else {
        0.0
    };

    0.6 * (TAU * sweep).sin() + keyed * (TAU * 17_000.0 * t).sin() + 0.004 * noise(t, 2)
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

/// The analyser's row at `t`: both channels, Hann windowed, in dB of a volt.
fn analyse(t: f64) -> Vec<f32> {
    let mut re: Vec<f64> = (0..POINTS)
        .map(|n| {
            let time = t - (POINTS - n) as f64 / RATE;
            let window = 0.5 - 0.5 * (TAU * n as f64 / POINTS as f64).cos();

            (tone(time) + ch2(time)) * window
        })
        .collect();
    let mut im = vec![0.0; POINTS];

    fft(&mut re, &mut im);

    // A Hann window passes half a sine's amplitude.
    let scale = 2.0 / (POINTS as f64 * 0.5);

    (0..BINS)
        .map(|k| {
            let magnitude = (re[k] * re[k] + im[k] * im[k]).sqrt() * scale;

            (20.0 * magnitude.max(1e-9).log10()) as f32
        })
        .collect()
}

/// The history for the moment `now`, as if the analyser had been running.
fn seeded(now: f64) -> History {
    let mut history = History::new(BINS, DEPTH);

    for age in (0..DEPTH).rev() {
        history.push(&analyse(now - age as f64 * TICK));
    }

    history
}

/// An in-place radix-2 transform of a power-of-two length.
fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    let mut j = 0;

    for i in 1..n {
        let mut bit = n >> 1;

        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }

        j |= bit;

        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    let mut length = 2;

    while length <= n {
        let angle = -TAU / length as f64;

        for start in (0..n).step_by(length) {
            for k in 0..length / 2 {
                let (sin, cos) = (angle * k as f64).sin_cos();
                let (a, b) = (start + k, start + k + length / 2);
                let (x, y) = (re[b] * cos - im[b] * sin, re[b] * sin + im[b] * cos);

                re[b] = re[a] - x;
                im[b] = im[a] - y;
                re[a] += x;
                im[a] += y;
            }
        }

        length <<= 1;
    }
}

/// The frequency and level of the strongest bin whose frequency is in
/// `band`.
fn strongest(levels: &[f32], band: std::ops::Range<f32>) -> (f32, f32) {
    levels
        .iter()
        .enumerate()
        .map(|(bin, level)| (bin as f32 * WIDTH, *level))
        .filter(|(frequency, _)| band.contains(frequency))
        .fold((0.0, f32::NEG_INFINITY), |best, bin| {
            if bin.1 > best.1 { bin } else { best }
        })
}

/// The median level of a row: the analyser's noise floor.
fn median(levels: &[f32]) -> f32 {
    let mut sorted = levels.to_vec();

    sorted.sort_by(f32::total_cmp);
    sorted.get(sorted.len() / 2).copied().unwrap_or(FLOOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tone_lands_in_its_bin_at_its_level() {
        let tone = 1.0;
        let mut re: Vec<f64> = (0..POINTS)
            .map(|n| {
                let window = 0.5 - 0.5 * (TAU * n as f64 / POINTS as f64).cos();

                tone * (TAU * 25.0 * n as f64 / POINTS as f64).sin() * window
            })
            .collect();
        let mut im = vec![0.0; POINTS];

        fft(&mut re, &mut im);

        let magnitude = (re[25] * re[25] + im[25] * im[25]).sqrt() * 2.0 / (POINTS as f64 * 0.5);

        assert!((magnitude - tone).abs() < 1e-6, "{magnitude}");
    }

    #[test]
    fn the_scope_measures_ch1() {
        let measured = measure(42.0, ch1);

        assert!(
            (measured.frequency - 1_000.0).abs() < 5.0,
            "{}",
            measured.frequency
        );
    }

    #[test]
    fn the_same_second_draws_the_same_waterfall() {
        let (a, b) = (seeded(42.0), seeded(42.0));

        assert!(a.rows().zip(b.rows()).all(|(a, b)| a == b));
    }
}
