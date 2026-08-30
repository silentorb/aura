//! Sample-time note lookup from a score.

use aura_composition::{Pitch, Score};
use aura_sample::SampleRate;
use aura_scheduler::schedule_offline;

#[derive(Debug, Clone)]
pub struct ScheduledNote {
    pub pitch: Pitch,
    pub start_secs: f64,
    pub duration_secs: f64,
}

#[derive(Debug, Clone)]
pub struct NoteSchedule {
    notes: Vec<ScheduledNote>,
}

impl NoteSchedule {
    pub fn from_score(score: &Score, sample_rate: SampleRate) -> Self {
        let schedule = schedule_offline(score, sample_rate).expect("valid score schedule");
        let rate = sample_rate.get() as f64;
        let notes = schedule
            .events
            .iter()
            .map(|event| ScheduledNote {
                pitch: event.event.pitch,
                start_secs: event.start_frame as f64 / rate,
                duration_secs: event.duration_frames as f64 / rate,
            })
            .collect();

        Self { notes }
    }

    pub fn active_at(&self, t: f64) -> Option<&ScheduledNote> {
        self.notes.iter().find(|note| {
            t >= note.start_secs && t < note.start_secs + note.duration_secs
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_composer::{minor_arpeggio, MinorArpeggioConfig};

    #[test]
    fn active_at_finds_note_window() {
        let score = minor_arpeggio(MinorArpeggioConfig::default());
        let schedule = NoteSchedule::from_score(&score, SampleRate::RATE_44100);
        assert!(schedule.active_at(0.0).is_some());
        assert!(schedule.active_at(10.0).is_none());
    }
}
