//! Offline audio rendering for Aura.

use aura_sample::{duration_to_frame_count, SampleRate};
use fundsp::audiounit::AudioUnit;
use fundsp::wave::Wave;
use thiserror::Error;

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

/// Renders an audio unit offline into a FunDSP `Wave`.
pub fn render_offline(spec: RenderSpec, unit: &mut dyn AudioUnit) -> Result<Wave, RenderError> {
    let wave = Wave::render(
        spec.sample_rate.get() as f64,
        spec.duration_secs,
        unit,
    );

    let expected = spec.expected_frame_count();
    let actual = wave.len();
    if actual != expected {
        return Err(RenderError::FrameCountMismatch { expected, actual });
    }

    Ok(wave)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_dsp::{sine_hz, to_unit};

    #[test]
    fn render_offline_produces_expected_frame_count() {
        let spec = RenderSpec {
            sample_rate: SampleRate::RATE_44100,
            duration_secs: 1.0,
            channels: 1,
        };
        let mut unit = to_unit(sine_hz(440.0));

        let wave = render_offline(spec, &mut *unit).expect("render should succeed");

        assert_eq!(wave.len(), 44_100);
    }
}
