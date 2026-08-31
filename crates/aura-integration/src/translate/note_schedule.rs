//! Sample-time note lookup from a score.

use aura_composition::{beats_to_seconds, wrap_beat, NoteParams, Score, ScoreSignal, Semitone, Tempo};
use aura_sample::SampleRate;

#[derive(Debug, Clone)]
pub struct ScheduledNote {
    pub semitone: Semitone,
    pub start_beats: f64,
    pub duration_beats: f64,
    pub velocity: f32,
    pub params: NoteParams,
}

impl ScheduledNote {
    pub fn param_or(&self, key: &str, default: f64) -> f64 {
        self.params.get(key).copied().unwrap_or(default)
    }
}

#[derive(Debug, Clone)]
pub struct NoteSchedule {
    notes: Vec<ScheduledNote>,
    tempo: Tempo,
    seconds_per_beat: f64,
    cycle_beats: f64,
    looping: bool,
}

impl NoteSchedule {
    pub fn from_score(score: &Score, sample_rate: SampleRate) -> Self {
        let _ = sample_rate;
        let notes = score
            .events
            .iter()
            .map(|event| ScheduledNote {
                semitone: event.semitone,
                start_beats: event.start_beats,
                duration_beats: event.duration_beats,
                velocity: event.velocity,
                params: event.params.clone(),
            })
            .collect();

        let tempo = score.tempo;
        Self {
            notes,
            tempo,
            seconds_per_beat: tempo.seconds_per_beat(),
            cycle_beats: score.loop_cycle_beats(),
            looping: false,
        }
    }

    pub fn from_score_signal(signal: &ScoreSignal, sample_rate: SampleRate) -> Self {
        let mut schedule = Self::from_score(&signal.score, sample_rate);
        schedule.looping = signal.looping;
        schedule
    }

    pub fn cycle_beats(&self) -> f64 {
        self.cycle_beats
    }

    pub fn cycle_secs(&self) -> f64 {
        beats_to_seconds(self.cycle_beats, self.tempo)
    }

    pub fn note_duration_secs(&self, note: &ScheduledNote) -> f64 {
        note.duration_beats * self.seconds_per_beat
    }

    pub fn active_at(&self, t: f64) -> Option<&ScheduledNote> {
        let beat = self.wrapped_beat(t);
        self.notes.iter().find(|note| {
            beat >= note.start_beats && beat < note.start_beats + note.duration_beats
        })
    }

    fn beat_at(&self, t: f64) -> f64 {
        t / self.seconds_per_beat
    }

    fn wrapped_beat(&self, t: f64) -> f64 {
        let beat = self.beat_at(t);
        if self.looping {
            wrap_beat(beat, self.cycle_beats)
        } else {
            beat
        }
    }

    fn cycle_index(&self, t: f64) -> f64 {
        if self.looping && self.cycle_beats > 1e-9 {
            (self.beat_at(t) / self.cycle_beats).floor()
        } else {
            0.0
        }
    }

    /// Elapsed seconds from the active note's onset (correct across loop cycles).
    pub fn note_local_secs(&self, t: f64) -> Option<f64> {
        let note = self.active_at(t)?;
        Some((self.wrapped_beat(t) - note.start_beats) * self.seconds_per_beat)
    }

    /// Global start time of the active note instance (for `time_elapsed` wiring).
    pub fn note_global_start_secs(&self, t: f64) -> Option<f64> {
        let note = self.active_at(t)?;
        if self.looping {
            let start_beats = self.cycle_index(t) * self.cycle_beats + note.start_beats;
            Some(beats_to_seconds(start_beats, self.tempo))
        } else {
            Some(beats_to_seconds(note.start_beats, self.tempo))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_composer::{
        arpeggio, bass_line, drum_grid, ArpeggioConfig, BassLineConfig, DrumGridConfig, DrumLane,
    };
    use aura_composition::{note_params, ScoreSignal};

    #[test]
    fn active_at_finds_note_window() {
        let score = arpeggio(ArpeggioConfig {
            subdivision: 16,
            ..Default::default()
        });
        let schedule = NoteSchedule::from_score(&score, SampleRate::RATE_44100);
        assert!(schedule.active_at(0.0).is_some());
        assert!(schedule.active_at(10.0).is_none());
    }

    #[test]
    fn active_at_wraps_when_looping() {
        let score = arpeggio(ArpeggioConfig::default());
        let signal = ScoreSignal::finite(score).with_looping(true);
        let schedule = NoteSchedule::from_score_signal(&signal, SampleRate::RATE_44100);
        let cycle = schedule.cycle_secs();
        assert!(schedule.active_at(cycle).is_some());
        assert!(schedule.active_at(cycle + 0.01).is_some());
    }

    #[test]
    fn note_local_secs_wraps_across_cycles() {
        let score = arpeggio(ArpeggioConfig::default());
        let signal = ScoreSignal::finite(score).with_looping(true);
        let schedule = NoteSchedule::from_score_signal(&signal, SampleRate::RATE_44100);
        let cycle = schedule.cycle_secs();
        let local_second_cycle = schedule.note_local_secs(cycle + 0.25);
        let local_first_cycle = schedule.note_local_secs(0.25);
        assert!(local_second_cycle.is_some());
        assert!((local_second_cycle.unwrap() - local_first_cycle.unwrap()).abs() < 1e-9);
    }

    #[test]
    fn note_global_start_advances_per_cycle() {
        let score = drum_grid(DrumGridConfig {
            lane: DrumLane::Kick,
            ..Default::default()
        });
        let signal = ScoreSignal::finite(score).with_looping(true);
        let schedule = NoteSchedule::from_score_signal(&signal, SampleRate::RATE_44100);
        let cycle = schedule.cycle_secs();
        let start_cycle_two = schedule
            .note_global_start_secs(cycle)
            .expect("kick on cycle boundary");
        assert!((start_cycle_two - cycle).abs() < 1e-9);
    }

    #[test]
    fn looped_demo_channels_stay_phase_locked_over_many_cycles() {
        use aura_composer::{
            arpeggio, bass_line, drum_grid, epic_minor_progression, ArpeggioConfig,
            BassLineConfig, DrumGridConfig, DrumLane,
        };
        use aura_composition::{ScoreSignal, Semitone, TimeSignature};

        let progression = epic_minor_progression(Semitone::A3, 1, TimeSignature::FOUR_FOUR);
        let arp_config = ArpeggioConfig {
            progression: progression.clone(),
            ..Default::default()
        };
        let bass_config = BassLineConfig {
            progression: progression.clone(),
            ..Default::default()
        };

        let arp = NoteSchedule::from_score_signal(
            &ScoreSignal::finite(arpeggio(arp_config)).with_looping(true),
            SampleRate::RATE_44100,
        );
        let kick = NoteSchedule::from_score_signal(
            &ScoreSignal::finite(drum_grid(DrumGridConfig {
                lane: DrumLane::Kick,
                ..Default::default()
            }))
            .with_looping(true),
            SampleRate::RATE_44100,
        );
        let bass = NoteSchedule::from_score_signal(
            &ScoreSignal::finite(bass_line(bass_config)).with_looping(true),
            SampleRate::RATE_44100,
        );

        assert!((arp.cycle_beats() - 16.0).abs() < 1e-9);
        assert!((kick.cycle_beats() - 4.0).abs() < 1e-9);
        assert!((bass.cycle_beats() - 16.0).abs() < 1e-9);

        let master_cycle = arp.cycle_secs();

        for cycle in 0..120_u32 {
            let t = f64::from(cycle) * master_cycle + 0.01;
            let phase = 0.01;

            let arp_ref = arp.active_at(phase).expect("arp at cycle start");
            let arp_late = arp.active_at(t).expect("arp late cycle");
            assert_eq!(
                arp_ref.semitone, arp_late.semitone,
                "arp pitch drift at t={t}"
            );

            let arp_local = arp.note_local_secs(t).expect("arp local");
            let arp_local_ref = arp.note_local_secs(phase).expect("arp local ref");
            assert!(
                (arp_local - arp_local_ref).abs() < 1e-6,
                "arp elapsed drift at t={t}: {arp_local} vs {arp_local_ref}"
            );

            kick.active_at(t)
                .expect("kick should fire on downbeat each master cycle");
            bass.active_at(t)
                .expect("bass should fire on downbeat each master cycle");

            let kick_local = kick.note_local_secs(t).expect("kick local");
            let kick_local_ref = kick.note_local_secs(phase).expect("kick local ref");
            assert!(
                (kick_local - kick_local_ref).abs() < 1e-6,
                "kick elapsed drift at t={t}"
            );

            let bass_local = bass.note_local_secs(t).expect("bass local");
            let bass_local_ref = bass.note_local_secs(phase).expect("bass local ref");
            assert!(
                (bass_local - bass_local_ref).abs() < 1e-6,
                "bass elapsed drift at t={t}"
            );
        }
    }

    #[test]
    fn scheduled_note_preserves_params() {
        let score = bass_line(BassLineConfig::default());
        let schedule = NoteSchedule::from_score(&score, SampleRate::RATE_44100);
        let note = schedule.active_at(0.0).expect("active note");
        assert!(note.param_or(note_params::CUTOFF_MULT, 1.0) > 0.0);
    }
}
