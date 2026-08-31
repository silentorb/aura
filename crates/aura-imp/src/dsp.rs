//! Aura DSP node library.

use crate::signals::{signal, AUDIO_MONO, CONTROL, TIME};
use imp_core_types::{NodeLibrary, NodeType, Port, PrimitiveValue};
use std::collections::BTreeMap;

fn port(id: &str, signal_type: imp_core_types::SignalType, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value,
    }
}

/// Returns the Aura DSP node type library.
pub fn dsp_node_library() -> NodeLibrary {
    let control = signal(CONTROL);
    let time = signal(TIME);
    let audio_mono = signal(AUDIO_MONO);

    let mut types = BTreeMap::new();
    types.insert(
        "sine".into(),
        NodeType {
            id: "sine".into(),
            inputs: BTreeMap::from([
                (
                    "frequency".into(),
                    port(
                        "frequency",
                        control.clone(),
                        Some(PrimitiveValue::Number(440.0)),
                    ),
                ),
                ("time".into(), port("time", time.clone(), None)),
            ]),
            outputs: BTreeMap::from([(
                "audio".into(),
                port("audio", audio_mono.clone(), None),
            )]),
        },
    );
    types.insert(
        "exponential_sweep_sine".into(),
        NodeType {
            id: "exponential_sweep_sine".into(),
            inputs: BTreeMap::from([
                (
                    "start_hz".into(),
                    port(
                        "start_hz",
                        control.clone(),
                        Some(PrimitiveValue::Number(150.0)),
                    ),
                ),
                (
                    "decay_rate".into(),
                    port(
                        "decay_rate",
                        control.clone(),
                        Some(PrimitiveValue::Number(12.0)),
                    ),
                ),
                ("elapsed".into(), port("elapsed", time.clone(), None)),
            ]),
            outputs: BTreeMap::from([(
                "audio".into(),
                port("audio", audio_mono.clone(), None),
            )]),
        },
    );
    types.insert(
        "deterministic_noise".into(),
        NodeType {
            id: "deterministic_noise".into(),
            inputs: BTreeMap::from([
                (
                    "seed".into(),
                    port("seed", control.clone(), Some(PrimitiveValue::Number(0.0))),
                ),
                ("time".into(), port("time", time.clone(), None)),
            ]),
            outputs: BTreeMap::from([(
                "audio".into(),
                port("audio", audio_mono.clone(), None),
            )]),
        },
    );
    types.insert(
        "highpass_noise".into(),
        NodeType {
            id: "highpass_noise".into(),
            inputs: BTreeMap::from([
                (
                    "seed".into(),
                    port("seed", control.clone(), Some(PrimitiveValue::Number(0.0))),
                ),
                ("time".into(), port("time", time.clone(), None)),
                (
                    "dt".into(),
                    port("dt", control.clone(), Some(PrimitiveValue::Number(0.000_1))),
                ),
            ]),
            outputs: BTreeMap::from([(
                "audio".into(),
                port("audio", audio_mono.clone(), None),
            )]),
        },
    );
    types.insert(
        "multiply".into(),
        NodeType {
            id: "multiply".into(),
            inputs: BTreeMap::from([
                ("a".into(), port("a", audio_mono.clone(), None)),
                ("b".into(), port("b", audio_mono.clone(), None)),
            ]),
            outputs: BTreeMap::from([(
                "audio".into(),
                port("audio", audio_mono.clone(), None),
            )]),
        },
    );
    types.insert(
        "add".into(),
        NodeType {
            id: "add".into(),
            inputs: BTreeMap::from([
                ("a".into(), port("a", audio_mono.clone(), None)),
                ("b".into(), port("b", audio_mono, None)),
            ]),
            outputs: BTreeMap::from([(
                "audio".into(),
                port("audio", signal(AUDIO_MONO), None),
            )]),
        },
    );

    NodeLibrary {
        id: "aura.dsp".into(),
        types,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsp_library_contains_expected_nodes() {
        let library = dsp_node_library();
        assert!(library.types.contains_key("sine"));
        assert!(library.types.contains_key("multiply"));
        assert!(library.types.contains_key("add"));
        assert!(library.types.contains_key("exponential_sweep_sine"));
        assert!(library.types.contains_key("deterministic_noise"));
        assert!(library.types.contains_key("highpass_noise"));
    }
}
