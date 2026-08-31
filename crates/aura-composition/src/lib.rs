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

    /// Quarter-note beats per bar (4/4 → 4.0, 6/8 → 3.0).
    pub fn quarter_beats_per_bar(&self) -> f64 {
        f64::from(self.beats_per_bar) * 4.0 / f64::from(self.beat_unit)
    }
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

/// Discrete semitone on the 12-TET chromatic scale. Not a frequency.
/// Encoded as semitones above C0; prefer named constants in user-facing code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Semitone(pub i16);

impl Semitone {
    pub const A4: Self = Self(69);
    pub const A3: Self = Self(57);

    /// Convert to frequency in Hz using A4 = 440 Hz equal temperament.
    pub fn to_hz(self) -> f64 {
        440.0 * 2.0_f64.powf(f64::from(self.0 - 69) / 12.0)
    }

    /// Semitone offset from this key pitch.
    pub fn transpose(self, offset: i16) -> Self {
        Self(self.0 + offset)
    }
}

/// Absolute semitones or interval offsets from root (for extensions, sus, omissions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordTone {
    Interval(i16),
    Semitone(Semitone),
}

/// A chord: root, voiced tones, and optional bass for slash chords.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    pub root: Semitone,
    pub tones: Vec<ChordTone>,
    pub bass: Option<Semitone>,
}

impl Chord {
    pub fn from_triad(root: Semitone, third: i16, fifth: i16) -> Self {
        Self::from_intervals(root, &[0, third, fifth])
    }

    pub fn from_intervals(root: Semitone, intervals: &[i16]) -> Self {
        Self {
            root,
            tones: intervals
                .iter()
                .copied()
                .map(ChordTone::Interval)
                .collect(),
            bass: None,
        }
    }

    /// Resolved absolute semitones for arpeggiation and voicing.
    pub fn semitones(&self) -> Vec<Semitone> {
        self.tones
            .iter()
            .map(|tone| match tone {
                ChordTone::Interval(offset) => self.root.transpose(*offset),
                ChordTone::Semitone(s) => *s,
            })
            .collect()
    }
}

/// A chord held for a span of beat time.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordRegion {
    pub start_beats: f64,
    pub duration_beats: f64,
    pub chord: Chord,
}

/// Timed sequence of chords on a beat grid.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordProgression {
    pub regions: Vec<ChordRegion>,
}

impl ChordProgression {
    /// Total span in quarter-note beats (end of last region).
    pub fn duration_beats(&self) -> f64 {
        self.regions
            .iter()
            .map(|region| region.start_beats + region.duration_beats)
            .fold(0.0_f64, f64::max)
    }

    /// Active chord at the given beat position (quarter-note beat grid).
    pub fn chord_at_beats(&self, beat: f64) -> &Chord {
        self.regions
            .iter()
            .find(|region| {
                beat >= region.start_beats - 1e-9
                    && beat < region.start_beats + region.duration_beats - 1e-9
            })
            .map(|region| &region.chord)
            .unwrap_or_else(|| {
                self.regions
                    .last()
                    .map(|region| &region.chord)
                    .expect("progression must have at least one region")
            })
    }

    /// Active chord at the given time in seconds at the given tempo.
    pub fn chord_at_secs(&self, time_secs: f64, tempo: Tempo) -> &Chord {
        let beat = time_secs / tempo.seconds_per_beat();
        self.chord_at_beats(beat)
    }
}

/// Per-frame tempo lookup for data signals.
pub trait TempoSignal: Send + Sync {
    fn tempo_at(&self, time_secs: f64) -> Tempo;
}

impl TempoSignal for Tempo {
    fn tempo_at(&self, _time_secs: f64) -> Tempo {
        *self
    }
}

/// Per-frame time signature lookup for data signals.
pub trait TimeSignatureSignal: Send + Sync {
    fn time_signature_at(&self, time_secs: f64) -> TimeSignature;
}

impl TimeSignatureSignal for TimeSignature {
    fn time_signature_at(&self, _time_secs: f64) -> TimeSignature {
        *self
    }
}

/// Per-frame chord lookup for data signals (requires tempo for sec→beat conversion).
pub trait ChordSignal: Send + Sync {
    fn chord_at(&self, time_secs: f64, tempo: Tempo) -> Chord;
}

impl ChordSignal for ChordProgression {
    fn chord_at(&self, time_secs: f64, tempo: Tempo) -> Chord {
        self.chord_at_secs(time_secs, tempo).clone()
    }
}

/// Chord progression with optional unbounded loop (modulus over cycle length).
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressionSignal {
    pub progression: ChordProgression,
    pub looping: bool,
}

impl ProgressionSignal {
    pub fn finite(progression: ChordProgression) -> Self {
        Self {
            progression,
            looping: false,
        }
    }

    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    pub fn chord_at_beats(&self, beat: f64) -> &Chord {
        let beat = if self.looping {
            wrap_beat(beat, self.progression.duration_beats())
        } else {
            beat
        };
        self.progression.chord_at_beats(beat)
    }
}

impl ChordSignal for ProgressionSignal {
    fn chord_at(&self, time_secs: f64, tempo: Tempo) -> Chord {
        let beat = time_secs / tempo.seconds_per_beat();
        self.chord_at_beats(beat).clone()
    }
}

/// Score with optional unbounded loop (modulus over cycle duration at sample time).
#[derive(Debug, Clone, PartialEq)]
pub struct ScoreSignal {
    pub score: Score,
    pub looping: bool,
}

impl ScoreSignal {
    pub fn finite(score: Score) -> Self {
        Self {
            score,
            looping: false,
        }
    }

    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}

/// Wraps beat time into `[0, cycle_beats)` for looped lookup.
pub fn wrap_beat(beat: f64, cycle_beats: f64) -> f64 {
    if cycle_beats <= 1e-9 {
        return beat;
    }
    let wrapped = beat % cycle_beats;
    if wrapped < 0.0 {
        wrapped + cycle_beats
    } else {
        wrapped
    }
}

/// Natural minor scale degrees relative to key root (semitone offsets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NaturalMinor;

impl NaturalMinor {
    /// i triad intervals from key root.
    pub const TRIAD_I: [i16; 3] = [0, 3, 7];
    /// VI (major) triad intervals from key root.
    pub const TRIAD_VI: [i16; 3] = [8, 12, 15];
    /// III (major) triad intervals from key root.
    pub const TRIAD_III: [i16; 3] = [3, 7, 10];
    /// VII (major) triad intervals from key root.
    pub const TRIAD_VII: [i16; 3] = [10, 14, 17];

    pub fn triad_i(key_root: Semitone) -> Chord {
        Chord::from_intervals(key_root, &Self::TRIAD_I)
    }

    pub fn triad_vi(key_root: Semitone) -> Chord {
        let root = key_root.transpose(8);
        Chord::from_triad(root, 4, 7)
    }

    pub fn triad_iii(key_root: Semitone) -> Chord {
        let root = key_root.transpose(3);
        Chord::from_triad(root, 4, 7)
    }

    pub fn triad_vii(key_root: Semitone) -> Chord {
        let root = key_root.transpose(10);
        Chord::from_triad(root, 4, 7)
    }
}

/// Per-note custom parameters keyed by name (heterogeneous by key, uniform f64 values).
pub type NoteParams = HashMap<String, f64>;

/// Well-known per-note parameter keys.
pub mod note_params {
    pub const CUTOFF_MULT: &str = "cutoff_mult";
}

/// A single note event in beat time.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteEvent {
    pub semitone: Semitone,
    pub start_beats: f64,
    pub duration_beats: f64,
    pub velocity: f32,
    pub params: NoteParams,
}

impl NoteEvent {
    pub fn new(semitone: Semitone, start_beats: f64, duration_beats: f64) -> Self {
        Self {
            semitone,
            start_beats,
            duration_beats,
            velocity: 1.0,
            params: NoteParams::new(),
        }
    }

    pub fn with_velocity(mut self, velocity: f32) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_param(mut self, key: &str, value: f64) -> Self {
        self.params.insert(key.to_string(), value);
        self
    }

    pub fn param_or(&self, key: &str, default: f64) -> f64 {
        self.params.get(key).copied().unwrap_or(default)
    }
}

/// A score: tempo, meter, and ordered note events.
#[derive(Debug, Clone, PartialEq)]
pub struct Score {
    pub time_signature: TimeSignature,
    pub tempo: Tempo,
    pub events: Vec<NoteEvent>,
    /// Musical loop period in beats. When unset, loops use content end (`duration_beats`).
    pub cycle_beats: Option<f64>,
}

impl Score {
    pub fn new(time_signature: TimeSignature, tempo: Tempo, events: Vec<NoteEvent>) -> Self {
        Self {
            time_signature,
            tempo,
            events,
            cycle_beats: None,
        }
    }

    pub fn with_cycle_beats(mut self, cycle_beats: f64) -> Self {
        self.cycle_beats = Some(cycle_beats);
        self
    }

    /// Sets loop cycle length in whole measures (bars) at the score time signature.
    pub fn with_cycle_measures(mut self, measures: f64) -> Self {
        self.cycle_beats = Some(measures * self.time_signature.quarter_beats_per_bar());
        self
    }

    /// Total duration in beats (end of last event).
    pub fn duration_beats(&self) -> f64 {
        self.events
            .iter()
            .map(|e| e.start_beats + e.duration_beats)
            .fold(0.0_f64, f64::max)
    }

    /// Beat length of one loop cycle for pattern repetition.
    pub fn loop_cycle_beats(&self) -> f64 {
        self.cycle_beats.unwrap_or_else(|| self.duration_beats())
    }

    /// Loop cycle length in seconds at the score tempo.
    pub fn loop_cycle_secs(&self) -> f64 {
        beats_to_seconds(self.loop_cycle_beats(), self.tempo)
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
    fn semitone_a4_is_440_hz() {
        assert!((Semitone::A4.to_hz() - 440.0).abs() < 1e-6);
    }

    #[test]
    fn quarter_beats_per_bar_four_four() {
        assert!((TimeSignature::FOUR_FOUR.quarter_beats_per_bar() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn chord_semitones_from_intervals() {
        let chord = Chord::from_intervals(Semitone::A3, &[0, 3, 7]);
        let semitones: Vec<i16> = chord.semitones().iter().map(|s| s.0).collect();
        assert_eq!(semitones, vec![57, 60, 64]);
    }

    #[test]
    fn progression_chord_at_beats() {
        let progression = ChordProgression {
            regions: vec![
                ChordRegion {
                    start_beats: 0.0,
                    duration_beats: 4.0,
                    chord: NaturalMinor::triad_i(Semitone::A3),
                },
                ChordRegion {
                    start_beats: 4.0,
                    duration_beats: 4.0,
                    chord: NaturalMinor::triad_vi(Semitone::A3),
                },
            ],
        };
        assert_eq!(progression.chord_at_beats(0.0).root.0, 57);
        assert_eq!(progression.chord_at_beats(4.0).root.0, 65);
    }

    #[test]
    fn note_event_param_or_returns_default_when_missing() {
        let event = NoteEvent::new(Semitone(36), 0.0, 0.5);
        assert!((event.param_or(note_params::CUTOFF_MULT, 1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn note_event_with_param_stores_value() {
        let event = NoteEvent::new(Semitone(36), 0.0, 0.5)
            .with_param(note_params::CUTOFF_MULT, 0.95);
        assert!((event.param_or(note_params::CUTOFF_MULT, 1.0) - 0.95).abs() < 1e-9);
    }

    #[test]
    fn note_event_with_velocity_stores_value() {
        let event = NoteEvent::new(Semitone(36), 0.0, 0.5).with_velocity(0.8);
        assert!((event.velocity - 0.8).abs() < 1e-6);
    }

    #[test]
    fn score_loop_cycle_defaults_to_content_duration() {
        let score = Score::new(
            TimeSignature::FOUR_FOUR,
            Tempo::default(),
            vec![NoteEvent::new(Semitone(60), 0.0, 1.0)],
        );
        assert!((score.loop_cycle_beats() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn score_duration_beats() {
        let score = Score::new(
            TimeSignature::FOUR_FOUR,
            Tempo::default(),
            vec![
                NoteEvent::new(Semitone(60), 0.0, 1.0),
                NoteEvent::new(Semitone(64), 1.0, 2.0),
            ],
        );
        assert!((score.duration_beats() - 3.0).abs() < 1e-9);
        assert!((score.duration_secs() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn progression_duration_beats() {
        let progression = ChordProgression {
            regions: vec![
                ChordRegion {
                    start_beats: 0.0,
                    duration_beats: 4.0,
                    chord: NaturalMinor::triad_i(Semitone::A3),
                },
                ChordRegion {
                    start_beats: 4.0,
                    duration_beats: 4.0,
                    chord: NaturalMinor::triad_vi(Semitone::A3),
                },
            ],
        };
        assert!((progression.duration_beats() - 8.0).abs() < 1e-9);
    }

    #[test]
    fn looped_progression_repeats_at_cycle_boundary() {
        let progression = ProgressionSignal::finite(ChordProgression {
            regions: vec![
                ChordRegion {
                    start_beats: 0.0,
                    duration_beats: 4.0,
                    chord: NaturalMinor::triad_i(Semitone::A3),
                },
                ChordRegion {
                    start_beats: 4.0,
                    duration_beats: 4.0,
                    chord: NaturalMinor::triad_vi(Semitone::A3),
                },
            ],
        })
        .with_looping(true);
        assert_eq!(progression.chord_at_beats(0.0).root.0, 57);
        assert_eq!(progression.chord_at_beats(16.0).root.0, 57);
        assert_eq!(progression.chord_at_beats(20.0).root.0, 65);
    }

    #[test]
    fn sort_events_by_start_orders_events() {
        let mut events = vec![
            NoteEvent::new(Semitone(64), 2.0, 1.0),
            NoteEvent::new(Semitone(60), 0.0, 1.0),
        ];
        sort_events_by_start(&mut events);
        assert!((events[0].start_beats - 0.0).abs() < 1e-9);
        assert!((events[1].start_beats - 2.0).abs() < 1e-9);
    }
}
