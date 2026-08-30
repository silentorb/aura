//! FunDSP graph builders for Aura synthesis.

mod compile;

use fundsp::audiounit::AudioUnit;
use fundsp::prelude32::{An, AudioNode};

pub use compile::{compile_graph, CompileError};

/// Returns a mono sine oscillator at `frequency` Hz.
pub fn sine_hz(frequency: f32) -> An<impl AudioNode> {
    fundsp::prelude32::sine_hz(frequency)
}

/// Converts a static audio node into a dynamic audio unit for rendering.
pub fn to_unit(node: An<impl AudioNode + 'static>) -> Box<dyn AudioUnit> {
    Box::new(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_hz_oscillates_within_unit_range() {
        let mut unit = to_unit(sine_hz(440.0));
        unit.set_sample_rate(44_100.0);
        unit.reset();

        let mut output = [0.0f32];
        unit.tick(&[], &mut output);
        assert!(output[0].abs() <= 1.0);

        let period = (44_100.0_f64 / 440.0).round() as usize;
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for _ in 0..period {
            unit.tick(&[], &mut output);
            min = min.min(output[0]);
            max = max.max(output[0]);
        }

        assert!(max > 0.9, "expected positive peak near 1.0, got {max}");
        assert!(min < -0.9, "expected negative peak near -1.0, got {min}");
    }
}
