//! WAV file I/O for Aura.

use aura_sample::SampleRate;
use std::fs::File;
use std::io::{Read, Write};
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
    #[error("unsupported WAV format in {path}")]
    UnsupportedFormat { path: String },
}

/// Writes mono 32-bit float PCM as a WAV file.
pub fn write_wav32(path: &Path, samples: &[f32], sample_rate: SampleRate) -> Result<(), WavError> {
    let path_str = path.to_string_lossy().into_owned();
    write_wav32_inner(&path_str, samples, sample_rate).map_err(|source| WavError::Write {
        path: path_str,
        source,
    })
}

fn write_wav32_inner(path: &str, samples: &[f32], sample_rate: SampleRate) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = File::create(path)?;
    let channels: u16 = 1;
    let bits_per_sample: u16 = 32;
    let byte_rate = sample_rate.get() * u32::from(channels) * u32::from(bits_per_sample) / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_bytes = (samples.len() * 4) as u32;
    let fmt_chunk_size = 16u32;
    let riff_chunk_size = 4 + (8 + fmt_chunk_size) + (8 + data_bytes);

    file.write_all(b"RIFF")?;
    file.write_all(&riff_chunk_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;
    file.write_all(b"fmt ")?;
    file.write_all(&fmt_chunk_size.to_le_bytes())?;
    file.write_all(&3u16.to_le_bytes())?; // IEEE float
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.get().to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;
    for &sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}

/// Loads a WAV file and returns its metadata.
pub fn verify_wav(path: &Path) -> Result<WavMetadata, WavError> {
    let path_str = path.to_string_lossy().into_owned();
    read_wav_metadata(&path_str).map_err(|source| match source {
        WavError::UnsupportedFormat { .. } => source,
        other => WavError::Read {
            path: path_str,
            source: Box::new(other),
        },
    })
}

fn read_wav_metadata(path: &str) -> Result<WavMetadata, WavError> {
    let mut file = File::open(path).map_err(|source| WavError::Read {
        path: path.to_string(),
        source: Box::new(source),
    })?;

    let mut riff = [0u8; 4];
    file.read_exact(&mut riff).map_err(|source| WavError::Read {
        path: path.to_string(),
        source: Box::new(source),
    })?;
    if &riff != b"RIFF" {
        return Err(WavError::UnsupportedFormat {
            path: path.to_string(),
        });
    }

    let mut _size = [0u8; 4];
    file.read_exact(&mut _size).ok();

    let mut wave = [0u8; 4];
    file.read_exact(&mut wave).map_err(|source| WavError::Read {
        path: path.to_string(),
        source: Box::new(source),
    })?;
    if &wave != b"WAVE" {
        return Err(WavError::UnsupportedFormat {
            path: path.to_string(),
        });
    }

    let mut sample_rate = 0u32;
    let mut channels = 0u16;
    let mut frame_count = 0usize;

    loop {
        let mut chunk_id = [0u8; 4];
        if file.read_exact(&mut chunk_id).is_err() {
            break;
        }
        let mut chunk_size_bytes = [0u8; 4];
        file.read_exact(&mut chunk_size_bytes).map_err(|source| WavError::Read {
            path: path.to_string(),
            source: Box::new(source),
        })?;
        let chunk_size = u32::from_le_bytes(chunk_size_bytes) as usize;

        if &chunk_id == b"fmt " {
            let mut fmt = vec![0u8; chunk_size];
            file.read_exact(&mut fmt).map_err(|source| WavError::Read {
                path: path.to_string(),
                source: Box::new(source),
            })?;
            if fmt.len() >= 16 {
                channels = u16::from_le_bytes([fmt[2], fmt[3]]);
                sample_rate = u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]);
            }
        } else if &chunk_id == b"data" {
            if chunk_size.is_multiple_of(4) {
                frame_count = chunk_size / 4 / usize::from(channels.max(1));
            }
            break;
        } else {
            let mut skip = vec![0u8; chunk_size];
            file.read_exact(&mut skip).ok();
        }
    }

    Ok(WavMetadata {
        sample_rate: f64::from(sample_rate),
        frame_count,
        channels: usize::from(channels),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_dsp::sine;
    use aura_render::{sample_offline, RenderSpec, Sampler};
    use aura_sample::SampleRate;
    use tempfile::NamedTempFile;

    struct Sine440;

    impl Sampler for Sine440 {
        fn at(&self, t: f64) -> f32 {
            sine(440.0, t)
        }
    }

    #[test]
    fn write_and_verify_round_trip() {
        let spec = RenderSpec {
            sample_rate: SampleRate::RATE_44100,
            duration_secs: 0.1,
            channels: 1,
        };
        let samples = sample_offline(spec, &Sine440).expect("sample should succeed");

        let file = NamedTempFile::new().expect("temp file");
        write_wav32(file.path(), &samples, spec.sample_rate).expect("write should succeed");

        let metadata = verify_wav(file.path()).expect("verify should succeed");
        assert_eq!(metadata.frame_count, samples.len());
        assert!((metadata.sample_rate - 44_100.0).abs() < 1.0);
        assert_eq!(metadata.channels, 1);
    }
}
