//! Aura DSP node library and graph helpers.

use imp_core_types::{
    core_node_library, Edge, Graph, Node, NodeLibrary, NodeType, Port, PortReference,
    PrimitiveValue, SignalType,
};
use imp_registry::{create_registry, load_library, Registry};
use std::collections::BTreeMap;

/// Signal type for a mono audio stream.
pub const AUDIO_MONO: &str = "audio_mono";

/// Signal type for a scalar control value.
pub const CONTROL: &str = "control";

fn signal(id: &str) -> SignalType {
    SignalType { id: id.into() }
}

fn port(id: &str, signal_type: SignalType, default_value: Option<PrimitiveValue>) -> Port {
    Port {
        id: id.into(),
        signal_type,
        default_value,
    }
}

/// Returns the Aura DSP node type library.
pub fn dsp_node_library() -> NodeLibrary {
    let control = signal(CONTROL);
    let audio_mono = signal(AUDIO_MONO);

    let mut types = BTreeMap::new();
    types.insert(
        "sine_hz".into(),
        NodeType {
            id: "sine_hz".into(),
            inputs: BTreeMap::from([(
                "frequency".into(),
                port(
                    "frequency",
                    control.clone(),
                    Some(PrimitiveValue::Number(440.0)),
                ),
            )]),
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

/// Returns a registry with core Imp nodes and Aura DSP nodes loaded.
pub fn dsp_registry() -> Result<Registry, imp_registry::DuplicateNodeTypeError> {
    let registry = create_registry();
    let registry = load_library(registry, core_node_library())?;
    load_library(registry, dsp_node_library())
}

/// Builds a minimal sine oscillator graph: `parameter` → `sine_hz` → `output`.
pub fn sine_graph(frequency: f64) -> Graph {
    let nodes = BTreeMap::from([
        (
            "freq".into(),
            Node {
                id: "freq".into(),
                node_type: "parameter".into(),
                inputs: BTreeMap::from([(
                    "value".into(),
                    PrimitiveValue::Number(frequency),
                )]),
            },
        ),
        (
            "osc".into(),
            Node {
                id: "osc".into(),
                node_type: "sine_hz".into(),
                inputs: BTreeMap::new(),
            },
        ),
        (
            "out".into(),
            Node {
                id: "out".into(),
                node_type: "output".into(),
                inputs: BTreeMap::new(),
            },
        ),
    ]);

    let edges = BTreeMap::from([
        (
            "freq_to_osc".into(),
            Edge {
                from: PortReference {
                    node: "freq".into(),
                    port: "value".into(),
                },
                to: PortReference {
                    node: "osc".into(),
                    port: "frequency".into(),
                },
            },
        ),
        (
            "osc_to_out".into(),
            Edge {
                from: PortReference {
                    node: "osc".into(),
                    port: "audio".into(),
                },
                to: PortReference {
                    node: "out".into(),
                    port: "value".into(),
                },
            },
        ),
    ]);

    Graph { nodes, edges }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imp_registry::get_node_type;

    #[test]
    fn dsp_registry_contains_sine_hz() {
        let registry = dsp_registry().expect("registry should load");
        let node_type = get_node_type(&registry, "sine_hz").expect("sine_hz should exist");
        assert!(node_type.inputs.contains_key("frequency"));
        assert!(node_type.outputs.contains_key("audio"));
    }

    #[test]
    fn sine_graph_has_expected_nodes_and_edges() {
        let graph = sine_graph(220.0);
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(
            graph.nodes["freq"].inputs["value"],
            PrimitiveValue::Number(220.0)
        );
    }
}
