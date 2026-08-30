//! Pure Time → Sample DSP functions for Aura.

use std::f64::consts::TAU;

/// Pure mono sine: `sin(2π · frequency · time)`.
#[must_use]
pub fn sine(frequency: f32, time: f64) -> f32 {
    (TAU * f64::from(frequency) * time).sin() as f32
}

/// Multiplies two sample/control values at time `t`.
#[must_use]
pub fn multiply(a: f32, b: f32) -> f32 {
    a * b
}

/// Converts pitch in Hz to frequency for oscillators (identity helper).
#[must_use]
pub fn hz(hz: f64) -> f32 {
    hz as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_is_pure_in_time() {
        let a = sine(440.0, 0.25);
        let b = sine(440.0, 0.25);
        assert_eq!(a, b);
    }

    #[test]
    fn sine_crosses_zero_at_start() {
        assert!(sine(440.0, 0.0).abs() < 1e-6);
    }

    #[test]
    fn sine_peak_near_quarter_period() {
        let period = 1.0 / 440.0;
        let peak = sine(440.0, period / 4.0);
        assert!(peak > 0.99);
    }
}
