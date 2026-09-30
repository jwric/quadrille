//! A spacecraft that does not exist, flying a pass that is a function of time.
//!
//! Every reading is computed from the seconds since launch, so a frame can be
//! reproduced by asking for the same second again.

/// Everything the console shows at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Telemetry {
    /// Seconds since launch.
    pub elapsed: f32,
    /// Roll, in degrees, positive to the right.
    pub roll: f32,
    /// Pitch, in degrees, positive nose up.
    pub pitch: f32,
    /// Heading, in degrees clockwise from north.
    pub yaw: f32,
    /// Altitude, in kilometres.
    pub altitude: f32,
    /// Acceleration, in metres per second squared.
    pub acceleration: f32,
    /// Oxidiser remaining, as a fraction.
    pub oxidiser: [f32; 3],
    /// Fuel remaining, as a fraction.
    pub fuel: [f32; 2],
    /// Bus voltages.
    pub bus: [f32; 3],
    /// Main engine thrust, as a fraction of rated.
    pub thrust: f32,
    /// Cabin and hull temperatures, in °C.
    pub temperature: (f32, f32),
    /// Downlink signal to noise, in dB.
    pub snr: f32,
}

impl Telemetry {
    /// The readings `elapsed` seconds after launch.
    pub fn at(elapsed: f32) -> Self {
        let t = elapsed;
        let wave = |period: f32, phase: f32| (std::f32::consts::TAU * t / period + phase).sin();
        let drain = |start: f32, rate: f32| (start - rate * t).rem_euclid(1.0).max(0.04);

        Self {
            elapsed,
            roll: 14.0 * wave(23.0, 0.0) + 3.0 * wave(7.0, 1.0),
            pitch: 6.0 * wave(31.0, 0.5) + 2.0,
            yaw: (134.0 + 0.8 * t).rem_euclid(360.0),
            altitude: 412.3 + 2.4 * wave(90.0, 0.0),
            acceleration: 10.0 + 6.0 * wave(17.0, 0.3) + 0.4 * wave(1.3, 0.0),
            oxidiser: [drain(0.74, 0.002), drain(0.74, 0.002), drain(0.71, 0.0021)],
            fuel: [drain(0.93, 0.0012), drain(0.91, 0.0013)],
            bus: [
                28.1 + 0.3 * wave(5.0, 0.0),
                27.9 + 0.2 * wave(6.0, 1.0),
                24.2 + 1.6 * wave(40.0, 2.0),
            ],
            thrust: 0.62 + 0.18 * wave(13.0, 0.0),
            temperature: (
                20.4 + 0.3 * wave(60.0, 0.0),
                -121.0 + 8.0 * wave(120.0, 1.0),
            ),
            snr: 18.0 + 6.0 * wave(9.0, 0.0) + 2.0 * wave(2.3, 1.0),
        }
    }

    /// Mission elapsed time as `HH:MM:SS.cc`.
    pub fn clock(&self) -> String {
        let hundredths = (self.elapsed.max(0.0) * 100.0).round() as u64;
        let seconds = hundredths / 100;

        format!(
            "{:02}:{:02}:{:02}.{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60,
            hundredths % 100,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_second_reads_the_same() {
        assert_eq!(Telemetry::at(42.5), Telemetry::at(42.5));
    }

    #[test]
    fn the_clock_reads_hours_minutes_seconds_and_hundredths() {
        assert_eq!(Telemetry::at(3723.45).clock(), "01:02:03.45");
    }
}
