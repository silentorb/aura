//! Sample primitives and audio timing types for Aura.

pub use dasp_sample::Sample;
pub use dasp_signal::Signal;

use thiserror::Error;

/// Supported PCM sample rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleRate(u32);

impl SampleRate {
    pub const RATE_44100: Self = Self(44_100);
    pub const RATE_48000: Self = Self(48_000);

    /// Creates a sample rate, validating against common PCM rates.
    pub fn new(rate: u32) -> Result<Self, SampleRateError> {
        if is_common_rate(rate) {
            Ok(Self(rate))
        } else {
            Err(SampleRateError::Unsupported(rate))
        }
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl Default for SampleRate {
    fn default() -> Self {
        Self::RATE_44100
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SampleRateError {
    #[error("unsupported sample rate: {0} Hz")]
    Unsupported(u32),
}

/// Converts a duration in seconds to a frame count at the given sample rate.
pub fn duration_to_frame_count(rate: SampleRate, duration_secs: f64) -> usize {
    (duration_secs * rate.get() as f64).round() as usize
}

fn is_common_rate(rate: u32) -> bool {
    matches!(
        rate,
        8_000 | 11_025 | 16_000 | 22_050 | 32_000 | 44_100 | 48_000 | 88_200 | 96_000 | 176_400
            | 192_000 | 384_000
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_to_frame_count_ten_seconds_at_44100() {
        assert_eq!(
            duration_to_frame_count(SampleRate::RATE_44100, 10.0),
            441_000
        );
    }

    #[test]
    fn sample_rate_accepts_common_rates() {
        assert!(SampleRate::new(44_100).is_ok());
        assert!(SampleRate::new(48_000).is_ok());
    }

    #[test]
    fn sample_rate_rejects_uncommon_rates() {
        assert_eq!(
            SampleRate::new(44_099),
            Err(SampleRateError::Unsupported(44_099))
        );
    }
}
