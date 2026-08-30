//! Compiles Imp graphs into FunDSP audio units.

use aura_imp::{get_node_type, Graph, NodeId, PortId, PortReference, PrimitiveValue, Registry};
use fundsp::audiounit::AudioUnit;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

use crate::{sine_hz, to_unit};

type TargetKey = (NodeId, PortId);

#[derive(Debug, Error, PartialEq)]
pub enum CompileError {
    #[error("graph has no output node")]
    MissingOutput,
    #[error("multiple output nodes are not supported")]
    MultipleOutputs,
    #[error("unknown node type `{node_type}` on node `{node}`")]
    UnknownNodeType { node: NodeId, node_type: String },
    #[error("unwired required input `{port}` on node `{node}`")]
    UnwiredInput { node: NodeId, port: PortId },
    #[error("cycle detected at node `{node}`")]
    CycleDetected { node: NodeId },
    #[error("expected numeric control value on node `{node}` port `{port}`")]
    ExpectedNumber { node: NodeId, port: PortId },
    #[error("node `{node}` port `{port}` is not a control output")]
    NotControlOutput { node: NodeId, port: PortId },
}

/// Compiles an Imp graph into a FunDSP audio unit.
pub fn compile_graph(graph: &Graph, registry: &Registry) -> Result<Box<dyn AudioUnit>, CompileError> {
    let output_node = find_output_node(graph)?;
    let edges_by_target = index_edges_by_target(graph);

    let mut visiting = BTreeSet::new();
    compile_audio_node(
        graph,
        registry,
        &edges_by_target,
        &mut visiting,
        &output_node,
    )
}

fn find_output_node(graph: &Graph) -> Result<NodeId, CompileError> {
    let outputs: Vec<_> = graph
        .nodes
        .values()
        .filter(|node| node.node_type == "output")
        .map(|node| node.id.clone())
        .collect();

    match outputs.as_slice() {
        [] => Err(CompileError::MissingOutput),
        [id] => Ok(id.clone()),
        _ => Err(CompileError::MultipleOutputs),
    }
}

fn index_edges_by_target(graph: &Graph) -> BTreeMap<TargetKey, PortReference> {
    let mut edges_by_target = BTreeMap::new();
    for edge in graph.edges.values() {
        let key = (edge.to.node.clone(), edge.to.port.clone());
        edges_by_target.insert(key, edge.from.clone());
    }
    edges_by_target
}

fn compile_audio_node(
    graph: &Graph,
    registry: &Registry,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
) -> Result<Box<dyn AudioUnit>, CompileError> {
    if !visiting.insert(node_id.to_string()) {
        return Err(CompileError::CycleDetected {
            node: node_id.into(),
        });
    }

    let node = graph
        .nodes
        .get(node_id)
        .expect("node id must exist in graph");

    let result = match node.node_type.as_str() {
        "output" => {
            let source = wired_source(edges_by_target, node_id, "value")?;
            compile_audio_node(graph, registry, edges_by_target, visiting, &source.node)
        }
        "sine_hz" => {
            let frequency = resolve_control(
                graph,
                registry,
                edges_by_target,
                visiting,
                node_id,
                "frequency",
            )?;
            Ok(to_unit(sine_hz(frequency)))
        }
        other => Err(CompileError::UnknownNodeType {
            node: node_id.into(),
            node_type: other.into(),
        }),
    };

    visiting.remove(node_id);
    result
}

fn wired_source(
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    node_id: &str,
    port_id: &str,
) -> Result<PortReference, CompileError> {
    edges_by_target
        .get(&(node_id.into(), port_id.into()))
        .cloned()
        .ok_or_else(|| CompileError::UnwiredInput {
            node: node_id.into(),
            port: port_id.into(),
        })
}

fn resolve_control(
    graph: &Graph,
    registry: &Registry,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    port_id: &str,
) -> Result<f32, CompileError> {
    if let Some(literal) = graph.nodes.get(node_id).and_then(|node| node.inputs.get(port_id)) {
        return primitive_to_f32(node_id, port_id, literal);
    }

    if let Some(source) = edges_by_target
        .get(&(node_id.into(), port_id.into()))
        .cloned()
    {
        return resolve_control_from_source(
            graph,
            registry,
            edges_by_target,
            visiting,
            &source,
        );
    }

    if let Some(node_type) = graph
        .nodes
        .get(node_id)
        .and_then(|node| get_node_type(registry, &node.node_type))
    {
        if let Some(port) = node_type.inputs.get(port_id) {
            if let Some(default) = &port.default_value {
                return primitive_to_f32(node_id, port_id, default);
            }
        }
    }

    Err(CompileError::UnwiredInput {
        node: node_id.into(),
        port: port_id.into(),
    })
}

fn resolve_control_from_source(
    graph: &Graph,
    registry: &Registry,
    _edges_by_target: &BTreeMap<TargetKey, PortReference>,
    _visiting: &mut BTreeSet<NodeId>,
    source: &PortReference,
) -> Result<f32, CompileError> {
    let source_node = graph
        .nodes
        .get(&source.node)
        .expect("edge source node must exist");

    match source_node.node_type.as_str() {
        "parameter" if source.port == "value" => {
            let value = source_node
                .inputs
                .get("value")
                .ok_or_else(|| CompileError::UnwiredInput {
                    node: source.node.clone(),
                    port: "value".into(),
                })?;
            primitive_to_f32(&source.node, "value", value)
        }
        "sine_hz" => Err(CompileError::NotControlOutput {
            node: source.node.clone(),
            port: source.port.clone(),
        }),
        "output" => Err(CompileError::NotControlOutput {
            node: source.node.clone(),
            port: source.port.clone(),
        }),
        other if get_node_type(registry, other).is_some() => Err(CompileError::NotControlOutput {
            node: source.node.clone(),
            port: source.port.clone(),
        }),
        other => Err(CompileError::UnknownNodeType {
            node: source.node.clone(),
            node_type: other.into(),
        }),
    }
}

fn primitive_to_f32(
    node_id: &str,
    port_id: &str,
    value: &PrimitiveValue,
) -> Result<f32, CompileError> {
    match value {
        PrimitiveValue::Number(n) => Ok(*n as f32),
        _ => Err(CompileError::ExpectedNumber {
            node: node_id.into(),
            port: port_id.into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_imp::dsp::{dsp_registry, sine_graph};
    use aura_render::{render_offline, RenderSpec};
    use aura_sample::SampleRate;

    #[test]
    fn compile_sine_graph_produces_oscillator() {
        let graph = sine_graph(440.0);
        let registry = dsp_registry().expect("registry");
        let mut unit = compile_graph(&graph, &registry).expect("compile should succeed");

        unit.set_sample_rate(44_100.0);
        unit.reset();

        let mut output = [0.0f32];
        unit.tick(&[], &mut output);
        assert!(output[0].abs() <= 1.0);
    }

    #[test]
    fn compile_sine_graph_renders_expected_frame_count() {
        let graph = sine_graph(440.0);
        let registry = dsp_registry().expect("registry");
        let mut unit = compile_graph(&graph, &registry).expect("compile should succeed");

        let spec = RenderSpec {
            sample_rate: SampleRate::RATE_44100,
            duration_secs: 0.1,
            channels: 1,
        };

        let wave = render_offline(spec, &mut *unit).expect("render should succeed");
        assert_eq!(wave.len(), 4_410);
    }
}
