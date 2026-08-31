//! Offline schedule sampling and mix-down.

use aura_scheduler::{Schedule, ScheduleContext};
use thiserror::Error;

/// Parameters for offline sampling (seed reserved for future randomized instruments).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OfflineSampleSpec {
    pub seed: u64,
}

/// Renders a scheduled timeline through an instrument and returns a mono mix buffer.
pub fn sample_schedule(
    schedule: &Schedule,
    instrument: &dyn Instrument,
    _spec: OfflineSampleSpec,
) -> Result<Vec<f32>, SampleError> {
    let mut mix = vec![0.0f32; schedule.total_frames];
    let sample_rate = schedule.sample_rate();

    for (index, scheduled) in schedule.events.iter().enumerate() {
        let ctx = ScheduleContext::new(schedule, index);
        let note_buffer = instrument.render_note(scheduled, &ctx, sample_rate);

        if note_buffer.len() != scheduled.duration_frames {
            return Err(SampleError::FrameCountMismatch {
                expected: scheduled.duration_frames,
                actual: note_buffer.len(),
            });
        }

        let end = scheduled
            .start_frame
            .saturating_add(note_buffer.len())
            .min(mix.len());

        for (mix_frame, &sample) in mix[scheduled.start_frame..end]
            .iter_mut()
            .zip(note_buffer.iter())
        {
            *mix_frame += sample;
        }
    }

    Ok(mix)
}

/// Trait for instruments that render one scheduled note at a time.
pub trait Instrument {
    fn render_note(
        &self,
        scheduled: &aura_scheduler::ScheduledEvent,
        ctx: &ScheduleContext<'_>,
        sample_rate: aura_sample::SampleRate,
    ) -> Vec<f32>;
}

#[derive(Debug, Error, PartialEq)]
pub enum SampleError {
    #[error("note render length {actual} does not match scheduled duration {expected}")]
    FrameCountMismatch { expected: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sine::SineInstrument;
    use aura_composition::{NoteEvent, Score, Semitone, Tempo, TimeSignature};
    use aura_scheduler::schedule_offline;
    use aura_sample::SampleRate;

    fn arpeggio_score() -> Score {
        aura_composer::arpeggio(aura_composer::ArpeggioConfig {
            bars: 1,
            ..Default::default()
        })
    }

    #[test]
    fn sample_schedule_is_deterministic() {
        let score = arpeggio_score();
        let schedule = schedule_offline(&score, SampleRate::RATE_44100).expect("schedule");
        let instrument = SineInstrument::default();
        let spec = OfflineSampleSpec { seed: 0 };

        let a = sample_schedule(&schedule, &instrument, spec).expect("sample");
        let b = sample_schedule(&schedule, &instrument, spec).expect("sample");
        assert_eq!(a, b);
    }

    #[test]
    fn sample_schedule_has_audible_energy_during_notes() {
        let score = arpeggio_score();
        let schedule = schedule_offline(&score, SampleRate::RATE_44100).expect("schedule");
        let buffer =
            sample_schedule(&schedule, &SineInstrument::default(), OfflineSampleSpec { seed: 0 })
                .expect("sample");

        let rms = (buffer.iter().map(|s| s * s).sum::<f32>() / buffer.len() as f32).sqrt();
        assert!(rms > 0.01, "expected audible output, rms={rms}");
    }

    #[test]
    fn sample_schedule_leaves_silence_before_first_note() {
        let score = Score::new(
            TimeSignature::FOUR_FOUR,
            Tempo::default(),
            vec![NoteEvent::new(Semitone(60), 1.0, 0.5)],
        );
        let schedule = schedule_offline(&score, SampleRate::RATE_44100).expect("schedule");
        let buffer = sample_schedule(
            &schedule,
            &SineInstrument::default(),
            OfflineSampleSpec { seed: 0 },
        )
        .expect("sample");

        let silent_prefix = &buffer[..schedule.events[0].start_frame];
        assert!(silent_prefix.iter().all(|&s| s == 0.0));
    }
}
