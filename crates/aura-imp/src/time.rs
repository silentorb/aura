//! Time source node library.

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

/// Returns the Aura time node library.
pub fn time_node_library() -> NodeLibrary {
    let time = signal(TIME);
    let control = signal(CONTROL);

    let mut types = BTreeMap::new();
    types.insert(
        "time".into(),
        NodeType {
            id: "time".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([(
                "time".into(),
                port("time", time.clone(), None),
            )]),
        },
    );
    types.insert(
        "time_elapsed".into(),
        NodeType {
            id: "time_elapsed".into(),
            type_params: Vec::new(),
            implementation: None,
            inputs: BTreeMap::from([
                ("time".into(), port("time", time.clone(), None)),
                ("start".into(), port("start", control, None)),
            ]),
            outputs: BTreeMap::from([(
                "elapsed".into(),
                port("elapsed", time, None),
            )]),
        },
    );

    NodeLibrary {
        id: "aura.time".into(),
        types,
    }
}
