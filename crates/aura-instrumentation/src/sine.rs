//! Sine oscillator instrument with per-note ADSR.

use crate::adsr::LinearAdsr;
use crate::sampler::Instrument;
use aura_dsp::{sine_hz, to_unit};
use aura_sample::SampleRate;
use aura_scheduler::{ScheduleContext, ScheduledEvent};
use fundsp::audiounit::AudioUnit;

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
        let frequency = scheduled.event.pitch.to_hz() as f32;
        let frames = scheduled.duration_frames;
        let note_duration_secs = frames as f64 / rate;
        let velocity = scheduled.event.velocity;

        let mut unit: Box<dyn AudioUnit> = to_unit(sine_hz(frequency));
        unit.set_sample_rate(rate);
        unit.reset();

        let mut sample = [0.0f32];
        let mut buffer = Vec::with_capacity(frames);

        for frame in 0..frames {
            unit.tick(&[], &mut sample);
            let elapsed = frame as f64 / rate;
            let env = self.adsr.value_at(elapsed, note_duration_secs);
            buffer.push(sample[0] * env * velocity);
        }

        buffer
    }
}
