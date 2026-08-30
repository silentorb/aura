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
        "multiply".into(),
        NodeType {
            id: "multiply".into(),
            inputs: BTreeMap::from([
                ("a".into(), port("a", audio_mono.clone(), None)),
                ("b".into(), port("b", audio_mono.clone(), None)),
            ]),
            outputs: BTreeMap::from([(
                "audio".into(),
                port("audio", audio_mono, None),
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
    fn dsp_library_contains_sine_and_multiply() {
        let library = dsp_node_library();
        assert!(library.types.contains_key("sine"));
        assert!(library.types.contains_key("multiply"));
    }
}
