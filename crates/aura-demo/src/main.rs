//! Surface integration reference program for Aura.

use aura_composer::{minor_arpeggio, MinorArpeggioConfig};
use aura_dsp::{sine_hz, to_unit};
use aura_instrumentation::{sample_schedule, OfflineSampleSpec, SineInstrument};
use aura_io_wav::{write_wav32, WavError};
use aura_render::{render_offline, RenderError, RenderSpec};
use aura_sample::{SampleRate, SampleRateError};
use aura_scheduler::schedule_offline;
use fundsp::wave::Wave;
use std::path::{Path, PathBuf};
use thiserror::Error;

const DEFAULT_FREQUENCY: f32 = 440.0;
const DEFAULT_SINE_DURATION: f64 = 10.0;
const SINE_OUTPUT: &str = "output/sine.wav";
const ARPEGGIO_OUTPUT: &str = "output/arpeggio.wav";

#[derive(Debug, Error)]
pub enum DemoError {
    #[error("{0}")]
    SampleRate(#[from] SampleRateError),
    #[error("{0}")]
    Render(#[from] RenderError),
    #[error("{0}")]
    Wav(#[from] WavError),
    #[error("{0}")]
    Schedule(#[from] aura_scheduler::ScheduleError),
    #[error("{0}")]
    Sample(#[from] aura_instrumentation::SampleError),
    #[error("failed to create output directory: {0}")]
    CreateDir(#[from] std::io::Error),
}

/// Generates a reference sine wave WAV file.
pub fn generate_sine(output: &Path, sample_rate: SampleRate) -> Result<(), DemoError> {
    let spec = RenderSpec {
        sample_rate,
        duration_secs: DEFAULT_SINE_DURATION,
        channels: 1,
    };

    let mut unit = to_unit(sine_hz(DEFAULT_FREQUENCY));
    let wave = render_offline(spec, &mut *unit)?;
    ensure_parent(output)?;
    write_wav32(output, &wave)?;
    Ok(())
}

/// Generates a reference minor arpeggio WAV file.
pub fn generate_arpeggio(output: &Path, sample_rate: SampleRate) -> Result<(), DemoError> {
    let score = minor_arpeggio(MinorArpeggioConfig::default());
    let schedule = schedule_offline(&score, sample_rate)?;
    let buffer = sample_schedule(
        &schedule,
        &SineInstrument::default(),
        OfflineSampleSpec { seed: 0 },
    )?;

    let wave = mono_samples_to_wave(sample_rate.get() as f64, &buffer);
    ensure_parent(output)?;
    write_wav32(output, &wave)?;
    Ok(())
}

fn mono_samples_to_wave(sample_rate: f64, samples: &[f32]) -> Wave {
    let mut wave = Wave::new(1, sample_rate);
    wave.resize(samples.len());
    for (index, &sample) in samples.iter().enumerate() {
        wave.set(0, index, sample);
    }
    wave
}

fn ensure_parent(path: &Path) -> Result<(), DemoError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn main() {
    let sample_rate = SampleRate::default();

    let sine_output = PathBuf::from(SINE_OUTPUT);
    if let Err(error) = generate_sine(&sine_output, sample_rate) {
        eprintln!("error generating sine: {error}");
        std::process::exit(1);
    }
    eprintln!(
        "Wrote {:.1}s sine wave at {} Hz to {}",
        DEFAULT_SINE_DURATION,
        DEFAULT_FREQUENCY,
        sine_output.display()
    );

    let arpeggio_output = PathBuf::from(ARPEGGIO_OUTPUT);
    if let Err(error) = generate_arpeggio(&arpeggio_output, sample_rate) {
        eprintln!("error generating arpeggio: {error}");
        std::process::exit(1);
    }
    eprintln!("Wrote arpeggio to {}", arpeggio_output.display());
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_io_wav::verify_wav;
    use tempfile::TempDir;

    #[test]
    fn generate_sine_writes_valid_wav() {
        let dir = TempDir::new().expect("temp dir");
        let output = dir.path().join("sine.wav");
        generate_sine(&output, SampleRate::RATE_44100).expect("generate sine");

        let metadata = verify_wav(&output).expect("verify");
        assert_eq!(metadata.frame_count, 441_000);
        assert_eq!(metadata.channels, 1);
    }

    #[test]
    fn generate_arpeggio_writes_valid_wav() {
        let dir = TempDir::new().expect("temp dir");
        let output = dir.path().join("arpeggio.wav");
        let sample_rate = SampleRate::RATE_44100;

        generate_arpeggio(&output, sample_rate).expect("generate arpeggio");

        let score = minor_arpeggio(MinorArpeggioConfig::default());
        let schedule = schedule_offline(&score, sample_rate).expect("schedule");
        let metadata = verify_wav(&output).expect("verify");
        assert_eq!(metadata.frame_count, schedule.total_frames);
        assert_eq!(metadata.channels, 1);
    }
}
