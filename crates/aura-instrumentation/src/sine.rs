//! Sine oscillator instrument with per-note ADSR.

use crate::adsr::LinearAdsr;
use crate::sampler::Instrument;
use aura_dsp::sine;
use aura_sample::SampleRate;
use aura_scheduler::{ScheduleContext, ScheduledEvent};

/// Sine wave instrument with a linear ADSR envelope per note.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SineInstrument {
    pub adsr: LinearAdsr,
}

impl Instrument for SineInstrument {
    fn render_note(
        &self,
        scheduled: &ScheduledEvent,
        _ctx: &ScheduleContext<'_>,
        sample_rate: SampleRate,
    ) -> Vec<f32> {
        let rate = sample_rate.get() as f64;
        let frequency = scheduled.event.semitone.to_hz() as f32;
        let frames = scheduled.duration_frames;
        let note_duration_secs = frames as f64 / rate;
        let velocity = scheduled.event.velocity;
        let start_secs = scheduled.start_frame as f64 / rate;

        let mut buffer = Vec::with_capacity(frames);

        for frame in 0..frames {
            let global_t = start_secs + frame as f64 / rate;
            let local_t = global_t - start_secs;
            let env = self.adsr.value_at(local_t, note_duration_secs);
            buffer.push(sine(frequency, local_t) * env * velocity);
        }

        buffer
    }
}
