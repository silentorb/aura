//! Musical notation types and utilities for Aura.

use std::collections::HashMap;
use thiserror::Error;

/// Beats per bar and beat unit (e.g. 4/4 → beats_per_bar=4, beat_unit=4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSignature {
    pub beats_per_bar: u8,
    pub beat_unit: u8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            beats_per_bar: 4,
            beat_unit: 4,
        }
    }
}

impl TimeSignature {
    pub const FOUR_FOUR: Self = Self {
        beats_per_bar: 4,
        beat_unit: 4,
    };
}

/// Tempo in beats per minute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tempo {
    pub bpm: f64,
}

impl Default for Tempo {
    fn default() -> Self {
        Self { bpm: 120.0 }
    }
}

impl Tempo {
    pub fn new(bpm: f64) -> Result<Self, CompositionError> {
        if bpm <= 0.0 || !bpm.is_finite() {
            return Err(CompositionError::InvalidTempo { bpm });
        }
        Ok(Self { bpm })
    }

    /// Seconds per quarter-note beat at this tempo.
    pub fn seconds_per_beat(&self) -> f64 {
        60.0 / self.bpm
    }
}

/// Pitch as a MIDI-style semitone number (0 = C0, 69 = A4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pitch(pub i16);

impl Pitch {
    pub const A4: Self = Self(69);

    pub fn from_semitone(semitone: i16) -> Self {
        Self(semitone)
    }

    /// Convert to frequency in Hz using A4 = 440 Hz equal temperament.
    pub fn to_hz(self) -> f64 {
        440.0 * 2.0_f64.powf(f64::from(self.0 - 69) / 12.0)
    }

    /// Semitone offset from this pitch.
    pub fn transpose(self, semitones: i16) -> Self {
        Self(self.0 + semitones)
    }
}

/// Natural minor scale degrees relative to root (semitone offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NaturalMinor;

impl NaturalMinor {
    /// i, ii°, III, iv, v, VI, VII as semitone offsets from root.
    pub const TRIAD_I: [i16; 3] = [0, 3, 7];
}

/// Per-note custom parameters (extensible stub for v1).
pub type NoteParams = HashMap<String, f64>;

/// A single note event in beat time.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteEvent {
    pub pitch: Pitch,
    pub start_beats: f64,
    pub duration_beats: f64,
    pub velocity: f32,
    pub params: NoteParams,
}

impl NoteEvent {
    pub fn new(pitch: Pitch, start_beats: f64, duration_beats: f64) -> Self {
        Self {
            pitch,
            start_beats,
            duration_beats,
            velocity: 1.0,
            params: NoteParams::new(),
        }
    }
}

/// A score: tempo, meter, and ordered note events.
#[derive(Debug, Clone, PartialEq)]
pub struct Score {
    pub time_signature: TimeSignature,
    pub tempo: Tempo,
    pub events: Vec<NoteEvent>,
}

impl Score {
    pub fn new(time_signature: TimeSignature, tempo: Tempo, events: Vec<NoteEvent>) -> Self {
        Self {
            time_signature,
            tempo,
            events,
        }
    }

    /// Total duration in beats (end of last event).
    pub fn duration_beats(&self) -> f64 {
        self.events
            .iter()
            .map(|e| e.start_beats + e.duration_beats)
            .fold(0.0_f64, f64::max)
    }

    /// Total duration in seconds at the score tempo.
    pub fn duration_secs(&self) -> f64 {
        beats_to_seconds(self.duration_beats(), self.tempo)
    }
}

/// Converts beat count to seconds at the given tempo.
pub fn beats_to_seconds(beats: f64, tempo: Tempo) -> f64 {
    beats * tempo.seconds_per_beat()
}

/// Sorts events by start time (stable).
pub fn sort_events_by_start(events: &mut [NoteEvent]) {
    events.sort_by(|a, b| {
        a.start_beats
            .partial_cmp(&b.start_beats)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[derive(Debug, Error, PartialEq)]
pub enum CompositionError {
    #[error("invalid tempo: {bpm} BPM")]
    InvalidTempo { bpm: f64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tempo_seconds_per_beat() {
        let tempo = Tempo::new(120.0).expect("valid tempo");
        assert!((tempo.seconds_per_beat() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn pitch_a4_is_440_hz() {
        assert!((Pitch::A4.to_hz() - 440.0).abs() < 1e-6);
    }

    #[test]
    fn score_duration_beats() {
        let score = Score::new(
            TimeSignature::FOUR_FOUR,
            Tempo::default(),
            vec![
                NoteEvent::new(Pitch(60), 0.0, 1.0),
                NoteEvent::new(Pitch(64), 1.0, 2.0),
            ],
        );
        assert!((score.duration_beats() - 3.0).abs() < 1e-9);
        assert!((score.duration_secs() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn sort_events_by_start_orders_events() {
        let mut events = vec![
            NoteEvent::new(Pitch(64), 2.0, 1.0),
            NoteEvent::new(Pitch(60), 0.0, 1.0),
        ];
        sort_events_by_start(&mut events);
        assert!((events[0].start_beats - 0.0).abs() < 1e-9);
        assert!((events[1].start_beats - 2.0).abs() < 1e-9);
    }
}
