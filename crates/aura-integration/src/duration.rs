//! Duration specification for graph rendering.

use aura_composition::{Tempo, TimeSignature};
use aura_sample::SampleRate;

/// How long to sample a graph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DurationSpec {
    Seconds(f64),
    Measures {
        measures: f64,
        tempo: Tempo,
        time_signature: TimeSignature,
    },
}

impl DurationSpec {
    pub fn to_seconds(self) -> Result<f64, DurationParseError> {
        match self {
            Self::Seconds(secs) => {
                if secs <= 0.0 || !secs.is_finite() {
                    return Err(DurationParseError::InvalidSeconds(secs));
                }
                Ok(secs)
            }
            Self::Measures {
                measures,
                tempo,
                time_signature,
            } => {
                if measures <= 0.0 || !measures.is_finite() {
                    return Err(DurationParseError::InvalidMeasures(measures));
                }
                Ok(
                    measures
                        * tempo.seconds_per_beat()
                        * time_signature.quarter_beats_per_bar(),
                )
            }
        }
    }

    /// Measures at 4/4 (backward-compatible helper).
    pub fn measures_4_4(measures: f64, tempo: Tempo) -> Self {
        Self::Measures {
            measures,
            tempo,
            time_signature: TimeSignature::FOUR_FOUR,
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DurationParseError {
    #[error("invalid duration seconds: {0}")]
    InvalidSeconds(f64),
    #[error("invalid duration measures: {0}")]
    InvalidMeasures(f64),
}

/// Parameters for sampling a translated graph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleSpec {
    pub sample_rate: SampleRate,
    pub duration: DurationSpec,
}

impl SampleSpec {
    pub fn duration_secs(&self) -> Result<f64, DurationParseError> {
        self.duration.to_seconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_convert_using_tempo() {
        let spec = DurationSpec::measures_4_4(2.0, Tempo::default());
        assert!((spec.to_seconds().expect("seconds") - 4.0).abs() < 1e-9);
    }

    #[test]
    fn measures_use_quarter_beats_per_bar() {
        let spec = DurationSpec::Measures {
            measures: 2.0,
            tempo: Tempo::default(),
            time_signature: TimeSignature::FOUR_FOUR,
        };
        assert!((spec.to_seconds().expect("seconds") - 4.0).abs() < 1e-9);
    }
}
