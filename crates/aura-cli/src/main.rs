//! Aura command-line interface — graph-driven audio rendering.

use aura_integration::{
    load_graph, parse_duration_spec, render_graph_to_pcm, write_pcm_to_wav, DurationSpec,
    IntegrationError, SampleSpec,
};
use aura_sample::{SampleRate, SampleRateError};
use std::env;
use std::path::PathBuf;
use thiserror::Error;

const DEFAULT_DURATION: &str = "10s";
const DEFAULT_TEMPO: f64 = 120.0;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    SampleRate(#[from] SampleRateError),
    #[error("{0}")]
    Duration(#[from] aura_integration::DurationParseError),
    #[error("{0}")]
    Integration(#[from] IntegrationError),
    #[error("missing required --graph path")]
    MissingGraph,
    #[error("missing required --output path")]
    MissingOutput,
    #[error("invalid --{flag} value `{value}`: {reason}")]
    InvalidArg {
        flag: &'static str,
        value: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderGraphConfig {
    pub graph: PathBuf,
    pub output: PathBuf,
    pub duration: DurationSpec,
    pub sample_rate: SampleRate,
}

fn parse_args(args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<RenderGraphConfig, CliError> {
    let mut graph = None;
    let mut output = None;
    let mut duration = DEFAULT_DURATION.to_string();
    let mut tempo = DEFAULT_TEMPO;
    let mut sample_rate = SampleRate::default();

    let mut iter = args.into_iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--graph" | "-g" => {
                graph = Some(next_value(&mut iter, "graph")?);
            }
            "--output" | "-o" => {
                output = Some(next_value(&mut iter, "output")?);
            }
            "--duration" | "-d" => {
                duration = next_value(&mut iter, "duration")?;
            }
            "--tempo" => {
                let value = next_value(&mut iter, "tempo")?;
                tempo = value.parse().map_err(|_| CliError::InvalidArg {
                    flag: "tempo",
                    value,
                    reason: "expected a positive number".into(),
                })?;
            }
            "--sample-rate" | "-r" => {
                let value = next_value(&mut iter, "sample-rate")?;
                let hz: u32 = value.parse().map_err(|_| CliError::InvalidArg {
                    flag: "sample-rate",
                    value: value.clone(),
                    reason: "expected an integer sample rate".into(),
                })?;
                sample_rate = SampleRate::new(hz)?;
            }
            other => {
                return Err(CliError::InvalidArg {
                    flag: "argument",
                    value: other.to_string(),
                    reason: "unknown flag".into(),
                });
            }
        }
    }

    let graph = graph.ok_or(CliError::MissingGraph)?;
    let output = output.ok_or(CliError::MissingOutput)?;
    let duration = parse_duration_spec(&duration, tempo)?;

    Ok(RenderGraphConfig {
        graph: PathBuf::from(graph),
        output: PathBuf::from(output),
        duration,
        sample_rate,
    })
}

fn next_value(
    iter: &mut std::iter::Peekable<impl Iterator<Item = impl AsRef<str>>>,
    flag: &'static str,
) -> Result<String, CliError> {
    iter.next()
        .map(|value| value.as_ref().to_string())
        .ok_or_else(|| CliError::InvalidArg {
            flag,
            value: String::new(),
            reason: "missing value".into(),
        })
}

fn print_help() {
    eprintln!(
        "aura — render an Imp graph to WAV\n\n\
         Usage:\n  \
         aura --graph PATH --output PATH [OPTIONS]\n\n\
         Options:\n  \
         -g, --graph PATH         Imp graph JSON file\n  \
         -o, --output PATH        Output WAV path\n  \
         -d, --duration SPEC      Duration: seconds (10, 10s) or measures (2m)\n  \
         --tempo BPM              Tempo for measure durations (default: 120)\n  \
         -r, --sample-rate HZ     Sample rate (default: 44100)\n  \
         -h, --help               Show this help"
    );
}

pub fn render_graph(config: &RenderGraphConfig) -> Result<(), CliError> {
    let graph = load_graph(&config.graph)?;
    let spec = SampleSpec {
        sample_rate: config.sample_rate,
        duration: config.duration,
    };
    let pcm = render_graph_to_pcm(&graph, spec)?;
    write_pcm_to_wav(&config.output, &pcm, config.sample_rate)?;
    Ok(())
}

fn main() {
    let config = match parse_args(env::args().skip(1)) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("Run aura --help for usage.");
            std::process::exit(1);
        }
    };

    if let Err(error) = render_graph(&config) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }

    eprintln!(
        "Wrote {} from {}",
        config.output.display(),
        config.graph.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_requires_graph_and_output() {
        let err = parse_args(["--graph", "demos/sine.json"]).unwrap_err();
        assert!(matches!(err, CliError::MissingOutput));
    }

    #[test]
    fn parse_args_reads_duration_and_sample_rate() {
        let config = parse_args([
            "--graph",
            "demos/sine.json",
            "--output",
            "output/sine.wav",
            "--duration",
            "1s",
            "--sample-rate",
            "44100",
        ])
        .expect("parse");

        assert_eq!(config.duration, DurationSpec::Seconds(1.0));
        assert_eq!(config.sample_rate, SampleRate::RATE_44100);
    }
}
