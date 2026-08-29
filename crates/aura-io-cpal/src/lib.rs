//! Real-time audio playback for Aura via CPAL.
//!
//! Playback is deferred until the dev environment supports audio output.

use cpal::SampleFormat;
use dasp_sample::Sample;

/// Real-time playback via CPAL. Deferred until dev environment supports audio output.
pub struct PlaybackEngine;

impl PlaybackEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlaybackEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts an `f32` sample to the target CPAL sample type.
pub fn convert_sample<T>(sample: f32) -> T
where
    T: Sample + dasp_sample::FromSample<f32>,
{
    T::from_sample(sample)
}

/// Returns whether the given CPAL sample format uses floating-point encoding.
pub fn is_float_format(format: SampleFormat) -> bool {
    format.is_float()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_converts_to_i16() {
        let sample: i16 = convert_sample(0.5);
        assert!(sample > 0);
    }

    #[test]
    fn f32_converts_to_f32() {
        let sample: f32 = convert_sample(0.5);
        assert!((sample - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn float_format_detection() {
        assert!(is_float_format(SampleFormat::F32));
        assert!(!is_float_format(SampleFormat::I16));
    }
}
