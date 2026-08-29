//! WAV file I/O for Aura.

use fundsp::wave::Wave;
use std::path::Path;
use thiserror::Error;

/// Metadata extracted from a WAV file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WavMetadata {
    pub sample_rate: f64,
    pub frame_count: usize,
    pub channels: usize,
}

#[derive(Debug, Error)]
pub enum WavError {
    #[error("failed to write WAV file to {path}: {source}")]
    Write {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("failed to read WAV file from {path}: {source}")]
    Read {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Writes a wave as a 32-bit float WAV file.
pub fn write_wav32(path: &Path, wave: &Wave) -> Result<(), WavError> {
    let path_str = path.to_string_lossy().into_owned();
    wave.save_wav32(&path_str).map_err(|source| WavError::Write {
        path: path_str,
        source: Box::new(source),
    })
}

/// Loads a WAV file and returns its metadata.
pub fn verify_wav(path: &Path) -> Result<WavMetadata, WavError> {
    let path_str = path.to_string_lossy().into_owned();
    let wave = Wave::load(&path_str).map_err(|source| WavError::Read {
        path: path_str.clone(),
        source: Box::new(source),
    })?;

    Ok(WavMetadata {
        sample_rate: wave.sample_rate(),
        frame_count: wave.len(),
        channels: wave.channels(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_dsp::{sine_hz, to_unit};
    use aura_render::{render_offline, RenderSpec};
    use aura_sample::SampleRate;
    use tempfile::NamedTempFile;

    #[test]
    fn write_and_verify_round_trip() {
        let spec = RenderSpec {
            sample_rate: SampleRate::RATE_44100,
            duration_secs: 0.1,
            channels: 1,
        };
        let mut unit = to_unit(sine_hz(440.0));
        let wave = render_offline(spec, &mut *unit).expect("render should succeed");

        let file = NamedTempFile::new().expect("temp file");
        write_wav32(file.path(), &wave).expect("write should succeed");

        let metadata = verify_wav(file.path()).expect("verify should succeed");
        assert_eq!(metadata.frame_count, wave.len());
        assert!((metadata.sample_rate - 44_100.0).abs() < 1.0);
        assert_eq!(metadata.channels, 1);
    }
}
