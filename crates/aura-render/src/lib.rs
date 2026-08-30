//! Offline audio rendering for Aura.

use aura_sample::{duration_to_frame_count, SampleRate};
use thiserror::Error;

/// Pure function of time in seconds.
pub trait Sampler: Send + Sync {
    fn at(&self, time_secs: f64) -> f32;
}

/// Parameters for an offline render pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderSpec {
    pub sample_rate: SampleRate,
    pub duration_secs: f64,
    pub channels: u16,
}

impl Default for RenderSpec {
    fn default() -> Self {
        Self {
            sample_rate: SampleRate::default(),
            duration_secs: 10.0,
            channels: 1,
        }
    }
}

impl RenderSpec {
    pub fn expected_frame_count(&self) -> usize {
        duration_to_frame_count(self.sample_rate, self.duration_secs)
    }
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("rendered frame count {actual} does not match expected {expected}")]
    FrameCountMismatch { expected: usize, actual: usize },
}

/// Samples a pure Time → Sample function over the render interval.
pub fn sample_offline(spec: RenderSpec, sampler: &dyn Sampler) -> Result<Vec<f32>, RenderError> {
    let frame_count = spec.expected_frame_count();
    let rate = spec.sample_rate.get() as f64;
    let mut buffer = Vec::with_capacity(frame_count);

    for frame in 0..frame_count {
        let t = frame as f64 / rate;
        buffer.push(sampler.at(t));
    }

    if buffer.len() != frame_count {
        return Err(RenderError::FrameCountMismatch {
            expected: frame_count,
            actual: buffer.len(),
        });
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_dsp::sine;

    struct Sine440;

    impl Sampler for Sine440 {
        fn at(&self, t: f64) -> f32 {
            sine(440.0, t)
        }
    }

    #[test]
    fn sample_offline_produces_expected_frame_count() {
        let spec = RenderSpec {
            sample_rate: SampleRate::RATE_44100,
            duration_secs: 1.0,
            channels: 1,
        };

        let buffer = sample_offline(spec, &Sine440).expect("sample should succeed");
        assert_eq!(buffer.len(), 44_100);
    }
}
