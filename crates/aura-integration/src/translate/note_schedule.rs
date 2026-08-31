//! Sample-time note lookup from a score.

use aura_composition::{Score, ScoreSignal, Semitone};
use aura_sample::SampleRate;
use aura_scheduler::schedule_offline;

#[derive(Debug, Clone)]
pub struct ScheduledNote {
    pub semitone: Semitone,
    pub start_secs: f64,
    pub duration_secs: f64,
}

#[derive(Debug, Clone)]
pub struct NoteSchedule {
    notes: Vec<ScheduledNote>,
    looping: bool,
    duration_secs: f64,
}

impl NoteSchedule {
    pub fn from_score(score: &Score, sample_rate: SampleRate) -> Self {
        let schedule = schedule_offline(score, sample_rate).expect("valid score schedule");
        let rate = sample_rate.get() as f64;
        let notes = schedule
            .events
            .iter()
            .map(|event| ScheduledNote {
                semitone: event.event.semitone,
                start_secs: event.start_frame as f64 / rate,
                duration_secs: event.duration_frames as f64 / rate,
            })
            .collect();

        Self {
            notes,
            looping: false,
            duration_secs: score.duration_secs(),
        }
    }

    pub fn from_score_signal(signal: &ScoreSignal, sample_rate: SampleRate) -> Self {
        let mut schedule = Self::from_score(&signal.score, sample_rate);
        schedule.looping = signal.looping;
        schedule
    }

    pub fn active_at(&self, t: f64) -> Option<&ScheduledNote> {
        let t = if self.looping && self.duration_secs > 1e-9 {
            t % self.duration_secs
        } else {
            t
        };
        self.notes.iter().find(|note| {
            t >= note.start_secs && t < note.start_secs + note.duration_secs
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_composer::{arpeggio, ArpeggioConfig};

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
        let cycle = schedule.duration_secs;
        assert!(schedule.active_at(cycle).is_some());
        assert!(schedule.active_at(cycle + 0.01).is_some());
    }
}
