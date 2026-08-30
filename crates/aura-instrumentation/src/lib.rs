//! Instrument definitions and per-note sampling for Aura.

mod adsr;
mod sampler;
mod sine;

pub use adsr::LinearAdsr;
pub use sampler::{sample_schedule, Instrument, OfflineSampleSpec, SampleError};
pub use sine::SineInstrument;
