//! Procedural composition generators for Aura.

use aura_composition::{
    ChordProgression, ChordRegion, NaturalMinor, NoteEvent, Score, Semitone, Tempo, TimeSignature,
};

/// Configuration for a progression-driven arpeggio generator.
#[derive(Debug, Clone, PartialEq)]
pub struct ArpeggioConfig {
    pub progression: ChordProgression,
    pub tempo: Tempo,
    pub time_signature: TimeSignature,
    pub bars: u32,
    /// Subdivision of a whole note (8 = eighth notes).
    pub subdivision: u8,
}

impl Default for ArpeggioConfig {
    fn default() -> Self {
        Self {
            progression: epic_minor_progression(Semitone::A3, 1, TimeSignature::FOUR_FOUR),
            tempo: Tempo::default(),
            time_signature: TimeSignature::FOUR_FOUR,
            bars: 4,
            subdivision: 4,
        }
    }
}

/// Epic i → VI → III → VII minor-key progression, one chord per bar.
pub fn epic_minor_progression(
    key_root: Semitone,
    bars_per_chord: u32,
    time_signature: TimeSignature,
) -> ChordProgression {
    let beats_per_chord = bars_per_chord as f64 * time_signature.quarter_beats_per_bar();
    let chords = [
        NaturalMinor::triad_i(key_root),
        NaturalMinor::triad_vi(key_root),
        NaturalMinor::triad_iii(key_root),
        NaturalMinor::triad_vii(key_root),
    ];

    let regions = chords
        .iter()
        .enumerate()
        .map(|(i, chord)| ChordRegion {
            start_beats: i as f64 * beats_per_chord,
            duration_beats: beats_per_chord,
            chord: chord.clone(),
        })
        .collect();

    ChordProgression { regions }
}

/// Generates an arpeggio driven by a chord progression (chord at each note start).
///
/// The arpeggio degree resets at each bar boundary so the pattern aligns with the
/// progression meter (e.g. four quarter-note picks per bar in 4/4: root–third–fifth–root).
pub fn arpeggio(config: ArpeggioConfig) -> Score {
    let note_duration_beats = 4.0 / f64::from(config.subdivision);
    let beats_per_bar = config.time_signature.quarter_beats_per_bar();
    let total_beats = beats_per_bar * f64::from(config.bars);

    let mut events = Vec::new();
    let mut beat = 0.0;

    while beat < total_beats - 1e-9 {
        let beat_in_bar = beat % beats_per_bar;
        let degree = (beat_in_bar / note_duration_beats).round() as usize;

        let chord = config.progression.chord_at_beats(beat);
        let semitones = chord.semitones();
        let semitone = semitones[degree % semitones.len()];
        events.push(NoteEvent::new(semitone, beat, note_duration_beats));
        beat += note_duration_beats;
    }

    Score::new(config.time_signature, config.tempo, events)
}

/// Drum lane for alternating kick/snare on a quarter-note grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumLane {
    Kick = 0,
    Snare = 1,
}

/// Configuration for a kick/snare grid pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct DrumGridConfig {
    pub tempo: Tempo,
    pub time_signature: TimeSignature,
    pub bars: u32,
    pub lane: DrumLane,
    /// Duration of each hit in quarter-note beats.
    pub hit_duration_beats: f64,
}

impl Default for DrumGridConfig {
    fn default() -> Self {
        Self {
            tempo: Tempo::default(),
            time_signature: TimeSignature::FOUR_FOUR,
            bars: 4,
            lane: DrumLane::Kick,
            hit_duration_beats: 0.25,
        }
    }
}

/// Generates kick or snare hits on alternating quarter-note beats.
///
/// Kick on beats 0, 2, 4, …; snare on beats 1, 3, 5, … (kick–snare–kick–snare per bar).
pub fn drum_grid(config: DrumGridConfig) -> Score {
    let beats_per_bar = config.time_signature.quarter_beats_per_bar();
    let total_beats = beats_per_bar * f64::from(config.bars);
    let mut events = Vec::new();
    let mut beat = 0.0;
    let mut beat_index = 0u32;

    while beat < total_beats - 1e-9 {
        let is_kick = beat_index.is_multiple_of(2);
        let emit = match config.lane {
            DrumLane::Kick => is_kick,
            DrumLane::Snare => !is_kick,
        };

        if emit {
            events.push(NoteEvent::new(
                Semitone(0),
                beat,
                config.hit_duration_beats,
            ));
        }

        beat += 1.0;
        beat_index += 1;
    }

    Score::new(config.time_signature, config.tempo, events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_composition::sort_events_by_start;

    #[test]
    fn arpeggio_emits_expected_event_count() {
        let score = arpeggio(ArpeggioConfig::default());
        // 4 bars × 4 quarter-note picks per bar = 16 events
        assert_eq!(score.events.len(), 16);
    }

    #[test]
    fn arpeggio_start_times_are_monotonic() {
        let score = arpeggio(ArpeggioConfig::default());
        for window in score.events.windows(2) {
            assert!(window[1].start_beats >= window[0].start_beats);
        }
    }

    #[test]
    fn arpeggio_semitones_change_per_chord_region() {
        let score = arpeggio(ArpeggioConfig::default());
        // First bar (i chord): root–third–fifth–root on quarter beats
        let i_tones: Vec<i16> = score
            .events
            .iter()
            .filter(|e| e.start_beats < 4.0 - 1e-9)
            .map(|e| e.semitone.0)
            .collect();
        assert_eq!(i_tones, vec![57, 60, 64, 57]);

        // Second bar (VI chord): F–A–C–F
        let vi_tones: Vec<i16> = score
            .events
            .iter()
            .filter(|e| e.start_beats >= 4.0 - 1e-9 && e.start_beats < 8.0 - 1e-9)
            .map(|e| e.semitone.0)
            .collect();
        assert_eq!(vi_tones, vec![65, 69, 72, 65]);
    }

    #[test]
    fn arpeggio_starts_each_bar_on_chord_root() {
        let score = arpeggio(ArpeggioConfig::default());
        let progression = epic_minor_progression(Semitone::A3, 1, TimeSignature::FOUR_FOUR);
        let beats_per_bar = 4.0;
        for bar in 0..4 {
            let bar_start = bar as f64 * beats_per_bar;
            let first = score
                .events
                .iter()
                .find(|e| (e.start_beats - bar_start).abs() < 1e-9)
                .expect("note on bar downbeat");
            let chord = progression.chord_at_beats(bar_start);
            assert_eq!(first.semitone.0, chord.root.0);
        }
    }

    #[test]
    fn arpeggio_duration_is_four_bars() {
        let score = arpeggio(ArpeggioConfig::default());
        assert!((score.duration_beats() - 16.0).abs() < 1e-9);
    }

    #[test]
    fn epic_progression_has_four_regions() {
        let progression = epic_minor_progression(Semitone::A3, 1, TimeSignature::FOUR_FOUR);
        assert_eq!(progression.regions.len(), 4);
        assert!((progression.regions[0].duration_beats - 4.0).abs() < 1e-9);
    }

    #[test]
    fn events_can_be_sorted_without_reordering_equal_starts() {
        let mut score = arpeggio(ArpeggioConfig::default());
        score.events.reverse();
        sort_events_by_start(&mut score.events);
        arpeggio_start_times_are_monotonic();
    }

    #[test]
    fn drum_grid_kick_emits_on_even_beats() {
        let score = drum_grid(DrumGridConfig {
            bars: 2,
            lane: DrumLane::Kick,
            ..Default::default()
        });
        assert_eq!(score.events.len(), 4);
        let starts: Vec<f64> = score.events.iter().map(|e| e.start_beats).collect();
        assert_eq!(starts, vec![0.0, 2.0, 4.0, 6.0]);
    }

    #[test]
    fn drum_grid_snare_emits_on_odd_beats() {
        let score = drum_grid(DrumGridConfig {
            bars: 2,
            lane: DrumLane::Snare,
            ..Default::default()
        });
        assert_eq!(score.events.len(), 4);
        let starts: Vec<f64> = score.events.iter().map(|e| e.start_beats).collect();
        assert_eq!(starts, vec![1.0, 3.0, 5.0, 7.0]);
    }

    #[test]
    fn drum_grid_eight_bars_has_sixteen_kicks() {
        let score = drum_grid(DrumGridConfig {
            bars: 8,
            lane: DrumLane::Kick,
            ..Default::default()
        });
        assert_eq!(score.events.len(), 16);
        let last = score.events.last().expect("last kick");
        assert!((last.start_beats - 30.0).abs() < 1e-9);
    }

    #[test]
    fn drum_grid_start_times_are_monotonic() {
        let score = drum_grid(DrumGridConfig {
            bars: 4,
            lane: DrumLane::Snare,
            ..Default::default()
        });
        for window in score.events.windows(2) {
            assert!(window[1].start_beats > window[0].start_beats);
        }
    }
}
