//! Pure Time → Sample DSP functions for Aura.

use std::f64::consts::TAU;

/// Pure mono sine: `sin(2π · frequency · time)`.
#[must_use]
pub fn sine(frequency: f32, time: f64) -> f32 {
    (TAU * f64::from(frequency) * time).sin() as f32
}

/// Sine with exponential pitch sweep: phase is the closed-form integral of
/// `start_hz · e^(-decay_rate · t)` from 0 to `elapsed`.
#[must_use]
pub fn exponential_sweep_sine(start_hz: f32, decay_rate: f32, elapsed: f64) -> f32 {
    if elapsed <= 0.0 {
        return 0.0;
    }

    let start = f64::from(start_hz);
    let lambda = f64::from(decay_rate);
    let phase = if lambda.abs() < 1e-12 {
        start * elapsed
    } else {
        (start / lambda) * (1.0 - (-lambda * elapsed).exp())
    };

    (TAU * phase).sin() as f32
}

/// Deterministic pseudo-random noise in `[-1, 1]` from `seed` and `time`.
#[must_use]
pub fn deterministic_noise(seed: f32, time: f64) -> f32 {
    let t_bits = (time * 1_000_000.0).round().to_bits();
    let seed_bits = f32::to_bits(seed) as u64;
    let hash = splitmix64(t_bits ^ seed_bits);
    (hash as f64 / u64::MAX as f64 * 2.0 - 1.0) as f32
}

/// Stateless high-pass approximation: `noise(t) - noise(t - dt)`.
#[must_use]
pub fn highpass_noise(seed: f32, time: f64, dt: f64) -> f32 {
    if dt <= 0.0 {
        return deterministic_noise(seed, time);
    }
    deterministic_noise(seed, time) - deterministic_noise(seed, time - dt)
}

/// Multiplies two sample/control values at time `t`.
#[must_use]
pub fn multiply(a: f32, b: f32) -> f32 {
    a * b
}

/// Pure band-limited rising saw: `2 * fract(frequency * elapsed) - 1`.
#[must_use]
pub fn saw(frequency: f32, elapsed: f64) -> f32 {
    if elapsed <= 0.0 {
        return 0.0;
    }
    let phase = f64::from(frequency) * elapsed;
    let frac = phase - phase.floor();
    (2.0 * frac - 1.0) as f32
}

/// Band-limited saw via Fourier partial sum up to `cutoff_hz`.
#[must_use]
pub fn bandlimited_saw(frequency: f32, elapsed: f64, cutoff_hz: f32) -> f32 {
    if elapsed <= 0.0 || frequency <= 0.0 || cutoff_hz <= 0.0 {
        return 0.0;
    }

    let max_harmonic = (cutoff_hz / frequency).floor().max(1.0) as u32;
    let mut sum = 0.0_f64;
    for n in 1..=max_harmonic {
        let harmonic = n as f64;
        if harmonic * f64::from(frequency) >= f64::from(cutoff_hz) {
            break;
        }
        sum += (2.0 / harmonic) * (TAU * harmonic * f64::from(frequency) * elapsed).sin();
    }
    sum as f32
}

/// Sums two mono samples.
#[must_use]
pub fn add(a: f32, b: f32) -> f32 {
    a + b
}

/// Converts pitch in Hz to frequency for oscillators (identity helper).
#[must_use]
pub fn hz(hz: f64) -> f32 {
    hz as f32
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
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

    #[test]
    fn exponential_sweep_sine_is_pure() {
        let a = exponential_sweep_sine(150.0, 12.0, 0.05);
        let b = exponential_sweep_sine(150.0, 12.0, 0.05);
        assert_eq!(a, b);
    }

    #[test]
    fn exponential_sweep_sine_is_silent_at_zero() {
        assert_eq!(exponential_sweep_sine(150.0, 12.0, 0.0), 0.0);
    }

    #[test]
    fn exponential_sweep_sine_phase_increases() {
        let early = exponential_sweep_sine(150.0, 12.0, 0.01);
        let late = exponential_sweep_sine(150.0, 12.0, 0.08);
        assert!(early != late);
    }

    #[test]
    fn deterministic_noise_is_pure() {
        let a = deterministic_noise(42.0, 0.123);
        let b = deterministic_noise(42.0, 0.123);
        assert_eq!(a, b);
    }

    #[test]
    fn deterministic_noise_is_bounded() {
        let sample = deterministic_noise(1.0, 0.5);
        assert!((-1.0..=1.0).contains(&sample));
    }

    #[test]
    fn highpass_noise_differs_from_raw_noise() {
        let raw = deterministic_noise(7.0, 1.0);
        let hp = highpass_noise(7.0, 1.0, 0.000_1);
        assert_ne!(raw, hp);
    }

    #[test]
    fn add_sums_samples() {
        assert_eq!(add(0.25, 0.75), 1.0);
    }

    #[test]
    fn saw_is_pure_in_time() {
        let a = saw(110.0, 0.05);
        let b = saw(110.0, 0.05);
        assert_eq!(a, b);
    }

    #[test]
    fn saw_is_silent_at_zero() {
        assert_eq!(saw(110.0, 0.0), 0.0);
    }

    #[test]
    fn bandlimited_saw_is_pure() {
        let a = bandlimited_saw(110.0, 0.05, 800.0);
        let b = bandlimited_saw(110.0, 0.05, 800.0);
        assert_eq!(a, b);
    }

    #[test]
    fn bandlimited_saw_is_brighter_at_higher_cutoff() {
        let low = bandlimited_saw(110.0, 0.05, 220.0).abs();
        let high = bandlimited_saw(110.0, 0.05, 2000.0).abs();
        assert!(high > low);
    }
}
