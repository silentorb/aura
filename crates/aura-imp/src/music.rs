//! Musical composition node library.

use crate::signals::{signal, CHORD_PROGRESSION, CONTROL, SCORE, TIME};
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
    let progression = signal(CHORD_PROGRESSION);

    let mut types = BTreeMap::new();
    types.insert(
        "epic_minor_progression".into(),
        NodeType {
            id: "epic_minor_progression".into(),
            inputs: BTreeMap::from([
                (
                    "root".into(),
                    port("root", control.clone(), Some(PrimitiveValue::Number(57.0))),
                ),
                (
                    "bars_per_chord".into(),
                    port(
                        "bars_per_chord",
                        control.clone(),
                        Some(PrimitiveValue::Number(1.0)),
                    ),
                ),
                (
                    "tempo".into(),
                    port("tempo", control.clone(), Some(PrimitiveValue::Number(120.0))),
                ),
            ]),
            outputs: BTreeMap::from([(
                "progression".into(),
                port("progression", progression.clone(), None),
            )]),
        },
    );
    types.insert(
        "arpeggio".into(),
        NodeType {
            id: "arpeggio".into(),
            inputs: BTreeMap::from([
                (
                    "progression".into(),
                    port("progression", progression.clone(), None),
                ),
                (
                    "bars".into(),
                    port("bars", control.clone(), Some(PrimitiveValue::Number(4.0))),
                ),
                (
                    "subdivision".into(),
                    port(
                        "subdivision",
                        control.clone(),
                        Some(PrimitiveValue::Number(4.0)),
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
                    "semitone".into(),
                    port("semitone", control.clone(), None),
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
        "chord_at_time".into(),
        NodeType {
            id: "chord_at_time".into(),
            inputs: BTreeMap::from([
                (
                    "progression".into(),
                    port("progression", progression.clone(), None),
                ),
                ("time".into(), port("time", time.clone(), None)),
            ]),
            outputs: BTreeMap::from([
                (
                    "root".into(),
                    port("root", control.clone(), None),
                ),
                (
                    "active".into(),
                    port("active", control.clone(), None),
                ),
            ]),
        },
    );
    types.insert(
        "semitone_to_hz".into(),
        NodeType {
            id: "semitone_to_hz".into(),
            inputs: BTreeMap::from([(
                "semitone".into(),
                port("semitone", control, None),
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
