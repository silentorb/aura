//! Offline musical event scheduling for Aura.

use aura_composition::{beats_to_seconds, sort_events_by_start, NoteEvent, Score};
use aura_sample::{duration_to_frame_count, SampleRate};
use thiserror::Error;

/// A note event placed on the sample timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvent {
    pub event: NoteEvent,
    pub start_frame: usize,
    pub duration_frames: usize,
}

/// Full offline schedule in sample frames.
#[derive(Debug, Clone, PartialEq)]
pub struct Schedule {
    pub sample_rate: SampleRate,
    pub events: Vec<ScheduledEvent>,
    pub total_frames: usize,
}

impl Schedule {
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

/// Context for sampling a note with access to future scheduled events.
#[derive(Debug, Clone, Copy)]
pub struct ScheduleContext<'a> {
    schedule: &'a Schedule,
    index: usize,
}

impl<'a> ScheduleContext<'a> {
    pub fn new(schedule: &'a Schedule, index: usize) -> Self {
        Self { schedule, index }
    }

    pub fn current(&self) -> &'a ScheduledEvent {
        &self.schedule.events[self.index]
    }

    /// Scheduled events after the current index (future from the instrument's view).
    pub fn future_events(&self) -> &'a [ScheduledEvent] {
        let next = self.index + 1;
        if next >= self.schedule.events.len() {
            &[]
        } else {
            &self.schedule.events[next..]
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

/// Builds an offline sample-frame schedule from a score.
pub fn schedule_offline(score: &Score, sample_rate: SampleRate) -> Result<Schedule, ScheduleError> {
    let mut events = score.events.clone();
    sort_events_by_start(&mut events);

    let mut scheduled = Vec::with_capacity(events.len());
    for event in events {
        if event.duration_beats <= 0.0 {
            return Err(ScheduleError::InvalidDuration {
                start_beats: event.start_beats,
                duration_beats: event.duration_beats,
            });
        }

        let start_secs = beats_to_seconds(event.start_beats, score.tempo);
        let duration_secs = beats_to_seconds(event.duration_beats, score.tempo);

        let start_frame = duration_to_frame_count(sample_rate, start_secs);
        let duration_frames = duration_to_frame_count(sample_rate, duration_secs).max(1);

        scheduled.push(ScheduledEvent {
            event,
            start_frame,
            duration_frames,
        });
    }

    let total_frames = scheduled
        .iter()
        .map(|e| e.start_frame + e.duration_frames)
        .max()
        .unwrap_or(0);

    Ok(Schedule {
        sample_rate,
        events: scheduled,
        total_frames,
    })
}

#[derive(Debug, Error, PartialEq)]
pub enum ScheduleError {
    #[error("invalid note duration at beat {start_beats}: {duration_beats}")]
    InvalidDuration {
        start_beats: f64,
        duration_beats: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_composition::{NoteEvent, Semitone, Tempo, TimeSignature};

    fn sample_score() -> Score {
        Score::new(
            TimeSignature::FOUR_FOUR,
            Tempo::new(120.0).expect("tempo"),
            vec![
                NoteEvent::new(Semitone(60), 0.0, 0.5),
                NoteEvent::new(Semitone(64), 0.5, 0.5),
            ],
        )
    }

    #[test]
    fn schedule_offline_frame_offsets_at_44100() {
        let score = sample_score();
        let schedule = schedule_offline(&score, SampleRate::RATE_44100).expect("schedule");

        assert_eq!(schedule.events[0].start_frame, 0);
        assert_eq!(schedule.events[0].duration_frames, 11_025);
        assert_eq!(schedule.events[1].start_frame, 11_025);
        assert_eq!(schedule.total_frames, 22_050);
    }

    #[test]
    fn schedule_offline_sorts_unordered_input() {
        let score = Score::new(
            TimeSignature::FOUR_FOUR,
            Tempo::default(),
            vec![
                NoteEvent::new(Semitone(64), 1.0, 0.5),
                NoteEvent::new(Semitone(60), 0.0, 0.5),
            ],
        );
        let schedule = schedule_offline(&score, SampleRate::RATE_44100).expect("schedule");
        assert!(schedule.events[0].start_frame <= schedule.events[1].start_frame);
    }

    #[test]
    fn schedule_context_future_events() {
        let score = sample_score();
        let schedule = schedule_offline(&score, SampleRate::RATE_44100).expect("schedule");
        let ctx = ScheduleContext::new(&schedule, 0);
        assert_eq!(ctx.future_events().len(), 1);
        let ctx_last = ScheduleContext::new(&schedule, 1);
        assert!(ctx_last.future_events().is_empty());
    }

    #[test]
    fn empty_score_has_zero_frames() {
        let score = Score::new(TimeSignature::FOUR_FOUR, Tempo::default(), vec![]);
        let schedule = schedule_offline(&score, SampleRate::RATE_44100).expect("schedule");
        assert_eq!(schedule.total_frames, 0);
        assert!(schedule.events.is_empty());
    }
}
