//! Musical composition node library.

use crate::signals::{signal, CONTROL, SCORE, TIME};
use imp_core_types::{NodeLibrary, NodeType, Port, PrimitiveValue};
use std::collections::BTreeMap;

fn port(id: &str, signal_type: imp_core_types::SignalType, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value,
    }
}

/// Returns the Aura music node library.
pub fn music_node_library() -> NodeLibrary {
    let control = signal(CONTROL);
    let score = signal(SCORE);
    let time = signal(TIME);

    let mut types = BTreeMap::new();
    types.insert(
        "minor_arpeggio".into(),
        NodeType {
            id: "minor_arpeggio".into(),
            inputs: BTreeMap::from([
                (
                    "root".into(),
                    port("root", control.clone(), Some(PrimitiveValue::Number(57.0))),
                ),
                (
                    "bars".into(),
                    port("bars", control.clone(), Some(PrimitiveValue::Number(2.0))),
                ),
                (
                    "tempo".into(),
                    port("tempo", control.clone(), Some(PrimitiveValue::Number(120.0))),
                ),
                (
                    "subdivision".into(),
                    port(
                        "subdivision",
                        control.clone(),
                        Some(PrimitiveValue::Number(8.0)),
                    ),
                ),
            ]),
            outputs: BTreeMap::from([(
                "score".into(),
                port("score", score.clone(), None),
            )]),
        },
    );
    types.insert(
        "note_at_time".into(),
        NodeType {
            id: "note_at_time".into(),
            inputs: BTreeMap::from([
                ("score".into(), port("score", score.clone(), None)),
                ("time".into(), port("time", time.clone(), None)),
            ]),
            outputs: BTreeMap::from([
                (
                    "pitch".into(),
                    port("pitch", control.clone(), None),
                ),
                (
                    "active".into(),
                    port("active", control.clone(), None),
                ),
                (
                    "note_start".into(),
                    port("note_start", time.clone(), None),
                ),
                (
                    "note_duration".into(),
                    port("note_duration", time.clone(), None),
                ),
            ]),
        },
    );
    types.insert(
        "pitch_to_hz".into(),
        NodeType {
            id: "pitch_to_hz".into(),
            inputs: BTreeMap::from([(
                "pitch".into(),
                port("pitch", control, None),
            )]),
            outputs: BTreeMap::from([(
                "frequency".into(),
                port("frequency", signal(CONTROL), None),
            )]),
        },
    );

    NodeLibrary {
        id: "aura.music".into(),
        types,
    }
}
