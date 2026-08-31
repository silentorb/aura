//! Musical composition node library.

use crate::signals::{
    signal, CHORD_PROGRESSION, CONTROL, SCORE, TEMPO, TIME, TIME_SIGNATURE,
};
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
    let tempo = signal(TEMPO);
    let time_signature = signal(TIME_SIGNATURE);

    let mut types = BTreeMap::new();
    types.insert(
        "constant_tempo".into(),
        NodeType {
            id: "constant_tempo".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([(
                "bpm".into(),
                port("bpm", control.clone(), Some(PrimitiveValue::Number(120.0))),
            )]),
            outputs: BTreeMap::from([(
                "tempo".into(),
                port("tempo", tempo.clone(), None),
            )]),
        },
    );
    types.insert(
        "constant_time_signature".into(),
        NodeType {
            id: "constant_time_signature".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([
                (
                    "beats_per_bar".into(),
                    port(
                        "beats_per_bar",
                        control.clone(),
                        Some(PrimitiveValue::Number(4.0)),
                    ),
                ),
                (
                    "beat_unit".into(),
                    port(
                        "beat_unit",
                        control.clone(),
                        Some(PrimitiveValue::Number(4.0)),
                    ),
                ),
            ]),
            outputs: BTreeMap::from([(
                "time_signature".into(),
                port("time_signature", time_signature.clone(), None),
            )]),
        },
    );
    types.insert(
        "epic_minor_progression".into(),
        NodeType {
            id: "epic_minor_progression".into(),
            type_params: Vec::new(),
            implementation: None,
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
                    "time_signature".into(),
                    port("time_signature", time_signature.clone(), None),
                ),
            ]),
            outputs: BTreeMap::from([(
                "progression".into(),
                port("progression", progression.clone(), None),
            )]),
        },
    );
    types.insert(
        "drum_grid".into(),
        NodeType {
            id: "drum_grid".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([
                (
                    "tempo".into(),
                    port("tempo", tempo.clone(), None),
                ),
                (
                    "time_signature".into(),
                    port("time_signature", time_signature.clone(), None),
                ),
                (
                    "bars".into(),
                    port("bars", control.clone(), Some(PrimitiveValue::Number(4.0))),
                ),
                (
                    "lane".into(),
                    port("lane", control.clone(), Some(PrimitiveValue::Number(0.0))),
                ),
            ]),
            outputs: BTreeMap::from([(
                "score".into(),
                port("score", score.clone(), None),
            )]),
        },
    );
    types.insert(
        "arpeggio".into(),
        NodeType {
            id: "arpeggio".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([
                (
                    "progression".into(),
                    port("progression", progression.clone(), None),
                ),
                (
                    "tempo".into(),
                    port("tempo", tempo.clone(), None),
                ),
                (
                    "time_signature".into(),
                    port("time_signature", time_signature.clone(), None),
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
            type_params: Vec::new(),
            implementation: None,
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
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([
                (
                    "progression".into(),
                    port("progression", progression.clone(), None),
                ),
                (
                    "tempo".into(),
                    port("tempo", tempo.clone(), None),
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
        "tempo_at_time".into(),
        NodeType {
            id: "tempo_at_time".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([
                ("tempo".into(), port("tempo", tempo.clone(), None)),
                ("time".into(), port("time", time.clone(), None)),
            ]),
            outputs: BTreeMap::from([(
                "bpm".into(),
                port("bpm", control.clone(), None),
            )]),
        },
    );
    types.insert(
        "time_signature_at_time".into(),
        NodeType {
            id: "time_signature_at_time".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([
                (
                    "time_signature".into(),
                    port("time_signature", time_signature.clone(), None),
                ),
                ("time".into(), port("time", time.clone(), None)),
            ]),
            outputs: BTreeMap::from([
                (
                    "beats_per_bar".into(),
                    port("beats_per_bar", control.clone(), None),
                ),
                (
                    "beat_unit".into(),
                    port("beat_unit", control.clone(), None),
                ),
            ]),
        },
    );
    types.insert(
        "semitone_to_hz".into(),
        NodeType {
            id: "semitone_to_hz".into(),
            type_params: Vec::new(),
            implementation: None,
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
