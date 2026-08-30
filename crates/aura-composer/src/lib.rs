//! Procedural composition generators for Aura.

use aura_composition::{
    NaturalMinor, NoteEvent, Pitch, Score, Tempo, TimeSignature,
};

/// Configuration for a minor-key arpeggio generator.
#[derive(Debug, Clone, PartialEq)]
pub struct MinorArpeggioConfig {
    pub root: Pitch,
    pub bars: u32,
    pub tempo: Tempo,
    pub time_signature: TimeSignature,
    /// Subdivision of a whole note (8 = eighth notes).
    pub subdivision: u8,
}

impl Default for MinorArpeggioConfig {
    fn default() -> Self {
        Self {
            root: Pitch(57), // A3
            bars: 2,
            tempo: Tempo::default(),
            time_signature: TimeSignature::FOUR_FOUR,
            subdivision: 8,
        }
    }
}

/// Generates a 4/4 natural-minor i-chord arpeggio (root, minor third, fifth).
pub fn minor_arpeggio(config: MinorArpeggioConfig) -> Score {
    let note_duration_beats = 4.0 / f64::from(config.subdivision);
    let beats_per_bar = f64::from(config.time_signature.beats_per_bar);
    let total_beats = beats_per_bar * f64::from(config.bars);

    let mut events = Vec::new();
    let mut beat = 0.0;
    let mut degree = 0usize;

    while beat < total_beats - 1e-9 {
        let semitone_offset = NaturalMinor::TRIAD_I[degree % 3];
        let pitch = config.root.transpose(semitone_offset);
        events.push(NoteEvent::new(pitch, beat, note_duration_beats));
        beat += note_duration_beats;
        degree += 1;
    }

    Score::new(config.time_signature, config.tempo, events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_composition::sort_events_by_start;

    #[test]
    fn minor_arpeggio_emits_expected_event_count() {
        let score = minor_arpeggio(MinorArpeggioConfig::default());
        // 2 bars × 4 beats × 2 eighth-notes per beat = 16 events
        assert_eq!(score.events.len(), 16);
    }

    #[test]
    fn minor_arpeggio_start_times_are_monotonic() {
        let score = minor_arpeggio(MinorArpeggioConfig::default());
        for window in score.events.windows(2) {
            assert!(window[1].start_beats >= window[0].start_beats);
        }
    }

    #[test]
    fn minor_arpeggio_pitches_cycle_i_chord() {
        let config = MinorArpeggioConfig::default();
        let root = config.root;
        let score = minor_arpeggio(config);
        let expected: Vec<i16> = (0..score.events.len())
            .map(|i| {
                let offset = NaturalMinor::TRIAD_I[i % 3];
                root.transpose(offset).0
            })
            .collect();
        let actual: Vec<i16> = score.events.iter().map(|e| e.pitch.0).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn minor_arpeggio_duration_is_two_bars() {
        let score = minor_arpeggio(MinorArpeggioConfig::default());
        assert!((score.duration_beats() - 8.0).abs() < 1e-9);
    }

    #[test]
    fn events_can_be_sorted_without_reordering_equal_starts() {
        let mut score = minor_arpeggio(MinorArpeggioConfig::default());
        score.events.reverse();
        sort_events_by_start(&mut score.events);
        minor_arpeggio_start_times_are_monotonic();
    }
}
