//! Aura command-line interface.

use aura_dsp::{sine_hz, to_unit};
use aura_io_wav::{write_wav32, WavError};
use aura_render::{render_offline, RenderError, RenderSpec};
use aura_sample::{SampleRate, SampleRateError};
use std::env;
use std::path::PathBuf;
use thiserror::Error;

const DEFAULT_FREQUENCY: f32 = 440.0;
const DEFAULT_DURATION: f64 = 10.0;
const DEFAULT_SAMPLE_RATE: u32 = 44_100;
const DEFAULT_OUTPUT: &str = "output/sine.wav";

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    SampleRate(#[from] SampleRateError),
    #[error("{0}")]
    Render(#[from] RenderError),
    #[error("{0}")]
    Wav(#[from] WavError),
    #[error("failed to create output directory: {0}")]
    CreateDir(#[from] std::io::Error),
    #[error("invalid --{flag} value `{value}`: {reason}")]
    InvalidArg { flag: &'static str, value: String, reason: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenerateSineConfig {
    pub frequency: f32,
    pub duration_secs: f64,
    pub sample_rate: SampleRate,
    pub output: PathBuf,
}

impl Default for GenerateSineConfig {
    fn default() -> Self {
        Self {
            frequency: DEFAULT_FREQUENCY,
            duration_secs: DEFAULT_DURATION,
            sample_rate: SampleRate::default(),
            output: PathBuf::from(DEFAULT_OUTPUT),
        }
    }
}

/// Generates a sine wave and writes it to a WAV file.
pub fn generate_sine(config: &GenerateSineConfig) -> Result<(), CliError> {
    let spec = RenderSpec {
        sample_rate: config.sample_rate,
        duration_secs: config.duration_secs,
        channels: 1,
    };

    let mut unit = to_unit(sine_hz(config.frequency));
    let wave = render_offline(spec, &mut *unit)?;

    if let Some(parent) = config.output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    write_wav32(&config.output, &wave)?;
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<GenerateSineConfig, CliError> {
    let mut config = GenerateSineConfig::default();
    let mut iter = args.into_iter().peekable();

    while let Some(arg) = iter.next() {
        let arg = arg.as_ref();
        match arg {
            "--frequency" | "-f" => {
                let value = iter.next().ok_or_else(|| CliError::InvalidArg {
                    flag: "frequency",
                    value: String::new(),
                    reason: "missing value".into(),
                })?;
                config.frequency = value.as_ref().parse().map_err(|_| CliError::InvalidArg {
                    flag: "frequency",
                    value: value.as_ref().into(),
                    reason: "expected a number".into(),
                })?;
            }
            "--duration" | "-d" => {
                let value = iter.next().ok_or_else(|| CliError::InvalidArg {
                    flag: "duration",
                    value: String::new(),
                    reason: "missing value".into(),
                })?;
                config.duration_secs = value.as_ref().parse().map_err(|_| CliError::InvalidArg {
                    flag: "duration",
                    value: value.as_ref().into(),
                    reason: "expected a number".into(),
                })?;
            }
            "--sample-rate" | "-r" => {
                let value = iter.next().ok_or_else(|| CliError::InvalidArg {
                    flag: "sample-rate",
                    value: String::new(),
                    reason: "missing value".into(),
                })?;
                let rate: u32 = value.as_ref().parse().map_err(|_| CliError::InvalidArg {
                    flag: "sample-rate",
                    value: value.as_ref().into(),
                    reason: "expected a positive integer".into(),
                })?;
                config.sample_rate = SampleRate::new(rate)?;
            }
            "--output" | "-o" => {
                let value = iter.next().ok_or_else(|| CliError::InvalidArg {
                    flag: "output",
                    value: String::new(),
                    reason: "missing value".into(),
                })?;
                config.output = PathBuf::from(value.as_ref());
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(CliError::InvalidArg {
                    flag: "argument",
                    value: other.into(),
                    reason: "unknown argument; use --help for usage".into(),
                });
            }
        }
    }

    Ok(config)
}

fn print_help() {
    eprintln!(
        "Usage: aura [OPTIONS]

Generate a sine wave and write it to a WAV file.

Options:
  -f, --frequency <HZ>      Sine frequency in Hz (default: {DEFAULT_FREQUENCY})
  -d, --duration <SECS>     Duration in seconds (default: {DEFAULT_DURATION})
  -r, --sample-rate <HZ>    Sample rate in Hz (default: {DEFAULT_SAMPLE_RATE})
  -o, --output <PATH>       Output WAV path (default: {DEFAULT_OUTPUT})
  -h, --help                Show this help message
"
    );
}

fn main() {
    let config = match parse_args(env::args().skip(1)) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("Run `aura --help` for usage.");
            std::process::exit(1);
        }
    };

    if let Err(error) = generate_sine(&config) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }

    eprintln!(
        "Wrote {:.1}s sine wave at {} Hz to {}",
        config.duration_secs,
        config.frequency,
        config.output.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_io_wav::verify_wav;
    use tempfile::TempDir;

    #[test]
    fn parse_args_defaults() {
        let config = parse_args(std::iter::empty::<&str>()).expect("defaults should parse");
        assert_eq!(config, GenerateSineConfig::default());
    }

    #[test]
    fn parse_args_custom_values() {
        let config = parse_args([
            "--frequency",
            "220",
            "--duration",
            "2.5",
            "--sample-rate",
            "48000",
            "--output",
            "custom.wav",
        ])
        .expect("custom args should parse");

        assert_eq!(config.frequency, 220.0);
        assert_eq!(config.duration_secs, 2.5);
        assert_eq!(config.sample_rate, SampleRate::RATE_48000);
        assert_eq!(config.output, PathBuf::from("custom.wav"));
    }

    #[test]
    fn generate_sine_writes_valid_wav() {
        let dir = TempDir::new().expect("temp dir");
        let output = dir.path().join("sine.wav");
        let config = GenerateSineConfig {
            frequency: 440.0,
            duration_secs: 0.5,
            sample_rate: SampleRate::RATE_44100,
            output: output.clone(),
        };

        generate_sine(&config).expect("generate should succeed");

        let metadata = verify_wav(&output).expect("verify should succeed");
        assert_eq!(metadata.frame_count, 22_050);
        assert!((metadata.sample_rate - 44_100.0).abs() < 1.0);
        assert_eq!(metadata.channels, 1);
    }
}
