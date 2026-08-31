//! Envelope node library.

use crate::signals::{signal, CONTROL, TIME};
use imp_core_types::{NodeLibrary, NodeType, Port, PrimitiveValue};
use std::collections::BTreeMap;

fn port(id: &str, signal_type: imp_core_types::SignalType, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value,
    }
}

/// Returns the Aura envelope node library.
pub fn envelope_node_library() -> NodeLibrary {
    let control = signal(CONTROL);
    let time = signal(TIME);

    let mut types = BTreeMap::new();
    types.insert(
        "linear_adsr".into(),
        NodeType {
            id: "linear_adsr".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([
                (
                    "attack".into(),
                    port("attack", control.clone(), Some(PrimitiveValue::Number(0.005))),
                ),
                (
                    "decay".into(),
                    port("decay", control.clone(), Some(PrimitiveValue::Number(0.05))),
                ),
                (
                    "sustain".into(),
                    port("sustain", control.clone(), Some(PrimitiveValue::Number(0.8))),
                ),
                (
                    "release".into(),
                    port("release", control.clone(), Some(PrimitiveValue::Number(0.05))),
                ),
                ("elapsed".into(), port("elapsed", time.clone(), None)),
                (
                    "note_duration".into(),
                    port("note_duration", control.clone(), None),
                ),
            ]),
            outputs: BTreeMap::from([(
                "value".into(),
                port("value", control, None),
            )]),
        },
    );
    types.insert(
        "note_envelope".into(),
        NodeType {
            id: "note_envelope".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([
                ("score".into(), port("score", signal(crate::signals::SCORE), None)),
                ("time".into(), port("time", time, None)),
            ]),
            outputs: BTreeMap::from([(
                "value".into(),
                port("value", signal(CONTROL), None),
            )]),
        },
    );

    NodeLibrary {
        id: "aura.envelope".into(),
        types,
    }
}
