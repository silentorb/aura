//! Imp graph → Sampler translation.

mod note_schedule;

use note_schedule::NoteSchedule;
use aura_composer::{arpeggio, drum_grid, epic_minor_progression, ArpeggioConfig, DrumGridConfig, DrumLane};
use aura_composition::{ChordProgression, ChordSignal, Score, Semitone, Tempo, TimeSignature};
use aura_dsp::{add, deterministic_noise, exponential_sweep_sine, highpass_noise, multiply, sine};
use aura_imp::{Graph, Node, NodeId, PortReference, PrimitiveValue, Registry};
use aura_instrumentation::LinearAdsr;
use aura_render::Sampler;
use aura_sample::SampleRate;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

type TargetKey = (NodeId, String);

#[derive(Debug, Error, PartialEq)]
pub enum TranslateError {
    #[error("graph has no output node")]
    MissingOutput,
    #[error("multiple output nodes are not supported")]
    MultipleOutputs,
    #[error("unknown node type `{node_type}` on node `{node}`")]
    UnknownNodeType { node: NodeId, node_type: String },
    #[error("unwired required input `{port}` on node `{node}`")]
    UnwiredInput { node: NodeId, port: String },
    #[error("cycle detected at node `{node}`")]
    CycleDetected { node: NodeId },
    #[error("expected numeric literal on node `{node}` port `{port}`")]
    ExpectedNumber { node: NodeId, port: String },
    #[error("node `{node}` output `{port}` is not available")]
    MissingOutputPort { node: NodeId, port: String },
    #[error("score input on node `{node}` must connect to a score-producing node")]
    InvalidScoreSource { node: NodeId },
    #[error("progression input on node `{node}` must connect to a chord progression node")]
    InvalidProgressionSource { node: NodeId },
    #[error("tempo input on node `{node}` must connect to a tempo-producing node")]
    InvalidTempoSource { node: NodeId },
    #[error("time signature input on node `{node}` must connect to a time signature node")]
    InvalidTimeSignatureSource { node: NodeId },
}

enum NodeValue {
    Signal(Box<dyn Sampler>),
    Score(Score),
    ChordProgression(ChordProgression),
    Tempo(Tempo),
    TimeSignature(TimeSignature),
}

/// Translates an Imp graph into a pure Time → Sample function.
pub fn translate_graph(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
) -> Result<Box<dyn Sampler>, TranslateError> {
    let output_node = find_output_node(graph)?;
    let edges_by_target = index_edges_by_target(graph);
    let mut visiting = BTreeSet::new();

    match translate_node(
        graph,
        registry,
        sample_rate,
        &edges_by_target,
        &mut visiting,
        &output_node,
        "value",
    )? {
        NodeValue::Signal(sampler) => Ok(sampler),
        NodeValue::Score(_)
        | NodeValue::ChordProgression(_)
        | NodeValue::Tempo(_)
        | NodeValue::TimeSignature(_) => {
            Err(TranslateError::MissingOutputPort {
                node: output_node,
                port: "value".into(),
            })
        }
    }
}

fn find_output_node(graph: &Graph) -> Result<NodeId, TranslateError> {
    let outputs: Vec<_> = graph
        .nodes
        .values()
        .filter(|node| node.node_type == "output")
        .map(|node| node.id.clone())
        .collect();

    match outputs.as_slice() {
        [] => Err(TranslateError::MissingOutput),
        [id] => Ok(id.clone()),
        _ => Err(TranslateError::MultipleOutputs),
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

fn translate_node(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    output_port: &str,
) -> Result<NodeValue, TranslateError> {
    if !visiting.insert(node_id.to_string()) {
        return Err(TranslateError::CycleDetected {
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
            translate_node(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                &source.node,
                &source.port,
            )
        }
        "parameter" => {
            let value = node
                .inputs
                .get("value")
                .ok_or_else(|| TranslateError::UnwiredInput {
                    node: node_id.into(),
                    port: "value".into(),
                })?;
            let number = primitive_to_f64(node_id, "value", value)?;
            Ok(NodeValue::Signal(Box::new(ConstantSampler(number as f32))))
        }
        "time" => Ok(NodeValue::Signal(Box::new(TimeSampler))),
        "time_elapsed" => {
            let start = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "start",
            )?;
            let _time = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "time",
            )?;
            Ok(NodeValue::Signal(Box::new(TimeElapsedSampler { start })))
        }
        "sine" => {
            let frequency = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "frequency",
            )?;
            let time = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "time",
            )?;
            Ok(NodeValue::Signal(Box::new(SineSampler { frequency, time })))
        }
        "multiply" => {
            let a = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "a",
            )?;
            let b = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "b",
            )?;
            Ok(NodeValue::Signal(Box::new(MultiplySampler { a, b })))
        }
        "add" => {
            let a = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "a",
            )?;
            let b = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "b",
            )?;
            Ok(NodeValue::Signal(Box::new(AddSampler { a, b })))
        }
        "exponential_sweep_sine" => {
            let start_hz = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "start_hz",
            )?;
            let decay_rate = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "decay_rate",
            )?;
            let elapsed = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "elapsed",
            )?;
            Ok(NodeValue::Signal(Box::new(ExponentialSweepSineSampler {
                start_hz,
                decay_rate,
                elapsed,
            })))
        }
        "deterministic_noise" => {
            let seed = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "seed",
            )?;
            let time = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "time",
            )?;
            Ok(NodeValue::Signal(Box::new(DeterministicNoiseSampler { seed, time })))
        }
        "highpass_noise" => {
            let seed = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "seed",
            )?;
            let time = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "time",
            )?;
            let dt = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "dt",
            )?;
            Ok(NodeValue::Signal(Box::new(HighpassNoiseSampler { seed, time, dt })))
        }
        "linear_adsr" => {
            let attack = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "attack",
            )?;
            let decay = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "decay",
            )?;
            let sustain = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "sustain",
            )?;
            let release = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "release",
            )?;
            let elapsed = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "elapsed",
            )?;
            let note_duration = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "note_duration",
            )?;
            Ok(NodeValue::Signal(Box::new(LinearAdsrSampler {
                attack,
                decay,
                sustain,
                release,
                elapsed,
                note_duration,
            })))
        }
        "semitone_to_hz" => {
            let semitone = signal_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "semitone",
            )?;
            Ok(NodeValue::Signal(Box::new(SemitoneToHzSampler { semitone })))
        }
        "epic_minor_progression" => Ok(NodeValue::ChordProgression(
            build_epic_minor_progression(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node,
            )?,
        )),
        "constant_tempo" => Ok(NodeValue::Tempo(build_constant_tempo(node)?)),
        "constant_time_signature" => Ok(NodeValue::TimeSignature(
            build_constant_time_signature(node)?,
        )),
        "tempo_at_time" => translate_tempo_at_time_port(
            graph,
            registry,
            sample_rate,
            edges_by_target,
            visiting,
            node_id,
            output_port,
        ),
        "time_signature_at_time" => translate_time_signature_at_time_port(
            graph,
            registry,
            sample_rate,
            edges_by_target,
            visiting,
            node_id,
            output_port,
        ),
        "arpeggio" => Ok(NodeValue::Score(build_arpeggio_score(
            graph,
            registry,
            sample_rate,
            edges_by_target,
            visiting,
            node,
        )?)),
        "drum_grid" => Ok(NodeValue::Score(build_drum_grid_score(
            graph,
            registry,
            sample_rate,
            edges_by_target,
            visiting,
            node,
        )?)),
        "note_at_time" => translate_note_at_time_port(
            graph,
            registry,
            sample_rate,
            edges_by_target,
            visiting,
            node_id,
            output_port,
        ),
        "chord_at_time" => translate_chord_at_time_port(
            graph,
            registry,
            sample_rate,
            edges_by_target,
            visiting,
            node_id,
            output_port,
        ),
        "note_envelope" => {
            let score = resolve_score_input(
                graph,
                registry,
                sample_rate,
                edges_by_target,
                visiting,
                node_id,
                "score",
            )?;
            let schedule = NoteSchedule::from_score(&score, sample_rate);
            Ok(NodeValue::Signal(Box::new(NoteEnvelopeSampler {
                schedule,
                adsr: LinearAdsr::default(),
            })))
        }
        other => {
            let _ = registry;
            Err(TranslateError::UnknownNodeType {
                node: node_id.into(),
                node_type: other.into(),
            })
        }
    };

    visiting.remove(node_id);
    result
}

fn translate_note_at_time_port(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    output_port: &str,
) -> Result<NodeValue, TranslateError> {
    let score = resolve_score_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        node_id,
        "score",
    )?;
    let schedule = NoteSchedule::from_score(&score, sample_rate);

    let sampler: Box<dyn Sampler> = match output_port {
        "semitone" => Box::new(NoteSemitoneSampler { schedule }),
        "active" => Box::new(NoteActiveSampler { schedule }),
        "note_start" => Box::new(NoteStartSampler { schedule }),
        "note_duration" => Box::new(NoteDurationSampler { schedule }),
        _ => {
            return Err(TranslateError::MissingOutputPort {
                node: node_id.into(),
                port: output_port.into(),
            })
        }
    };

    Ok(NodeValue::Signal(sampler))
}

fn translate_chord_at_time_port(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    output_port: &str,
) -> Result<NodeValue, TranslateError> {
    let progression = resolve_progression_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        node_id,
        "progression",
    )?;
    let tempo = resolve_tempo_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        node_id,
        "tempo",
    )?;
    let _time = signal_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        node_id,
        "time",
    )?;

    let sampler: Box<dyn Sampler> = match output_port {
        "root" => Box::new(ChordRootSampler { progression, tempo }),
        "active" => Box::new(ChordActiveSampler { progression, tempo }),
        _ => {
            return Err(TranslateError::MissingOutputPort {
                node: node_id.into(),
                port: output_port.into(),
            })
        }
    };

    Ok(NodeValue::Signal(sampler))
}

fn translate_tempo_at_time_port(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    output_port: &str,
) -> Result<NodeValue, TranslateError> {
    let tempo = resolve_tempo_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        node_id,
        "tempo",
    )?;
    let _time = signal_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        node_id,
        "time",
    )?;

    let sampler: Box<dyn Sampler> = match output_port {
        "bpm" => Box::new(TempoBpmSampler { tempo }),
        _ => {
            return Err(TranslateError::MissingOutputPort {
                node: node_id.into(),
                port: output_port.into(),
            })
        }
    };

    Ok(NodeValue::Signal(sampler))
}

fn translate_time_signature_at_time_port(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    output_port: &str,
) -> Result<NodeValue, TranslateError> {
    let time_signature = resolve_time_signature_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        node_id,
        "time_signature",
    )?;
    let _time = signal_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        node_id,
        "time",
    )?;

    let sampler: Box<dyn Sampler> = match output_port {
        "beats_per_bar" => Box::new(BeatsPerBarSampler { time_signature }),
        "beat_unit" => Box::new(BeatUnitSampler { time_signature }),
        _ => {
            return Err(TranslateError::MissingOutputPort {
                node: node_id.into(),
                port: output_port.into(),
            })
        }
    };

    Ok(NodeValue::Signal(sampler))
}

fn resolve_score_input(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    port: &str,
) -> Result<Score, TranslateError> {
    let source = wired_source(edges_by_target, node_id, port)?;
    match translate_node(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &source.node,
        &source.port,
    )? {
        NodeValue::Score(score) => Ok(score),
        _ => Err(TranslateError::InvalidScoreSource {
            node: source.node,
        }),
    }
}

fn resolve_progression_input(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    port: &str,
) -> Result<ChordProgression, TranslateError> {
    let source = wired_source(edges_by_target, node_id, port)?;
    match translate_node(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &source.node,
        &source.port,
    )? {
        NodeValue::ChordProgression(progression) => Ok(progression),
        _ => Err(TranslateError::InvalidProgressionSource {
            node: source.node,
        }),
    }
}

fn resolve_tempo_input(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    port: &str,
) -> Result<Tempo, TranslateError> {
    let Some(source) = edges_by_target.get(&(node_id.into(), port.into())) else {
        return Ok(Tempo::default());
    };
    match translate_node(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &source.node,
        &source.port,
    )? {
        NodeValue::Tempo(tempo) => Ok(tempo),
        _ => Err(TranslateError::InvalidTempoSource {
            node: source.node.clone(),
        }),
    }
}

fn resolve_time_signature_input(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    port: &str,
) -> Result<TimeSignature, TranslateError> {
    let Some(source) = edges_by_target.get(&(node_id.into(), port.into())) else {
        return Ok(TimeSignature::default());
    };
    match translate_node(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &source.node,
        &source.port,
    )? {
        NodeValue::TimeSignature(time_signature) => Ok(time_signature),
        _ => Err(TranslateError::InvalidTimeSignatureSource {
            node: source.node.clone(),
        }),
    }
}

fn signal_input(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node_id: &str,
    port: &str,
) -> Result<Box<dyn Sampler>, TranslateError> {
    if let Some(literal) = graph.nodes.get(node_id).and_then(|node| node.inputs.get(port)) {
        let number = primitive_to_f64(node_id, port, literal)?;
        return Ok(Box::new(ConstantSampler(number as f32)));
    }

    let source = wired_source(edges_by_target, node_id, port)?;
    match translate_node(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &source.node,
        &source.port,
    )? {
        NodeValue::Signal(sampler) => Ok(sampler),
        NodeValue::Score(_) => Err(TranslateError::InvalidScoreSource {
            node: source.node,
        }),
        NodeValue::ChordProgression(_) => Err(TranslateError::InvalidProgressionSource {
            node: source.node,
        }),
        NodeValue::Tempo(_) => Err(TranslateError::InvalidTempoSource {
            node: source.node,
        }),
        NodeValue::TimeSignature(_) => Err(TranslateError::InvalidTimeSignatureSource {
            node: source.node,
        }),
    }
}

fn build_constant_tempo(node: &Node) -> Result<Tempo, TranslateError> {
    Tempo::new(read_number_input(node, "bpm", 120.0)?).map_err(|_| TranslateError::ExpectedNumber {
        node: node.id.clone(),
        port: "bpm".into(),
    })
}

fn build_constant_time_signature(node: &Node) -> Result<TimeSignature, TranslateError> {
    let beats_per_bar = read_number_input(node, "beats_per_bar", 4.0)? as u8;
    let beat_unit = read_number_input(node, "beat_unit", 4.0)? as u8;
    Ok(TimeSignature {
        beats_per_bar,
        beat_unit,
    })
}

fn build_epic_minor_progression(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node: &Node,
) -> Result<ChordProgression, TranslateError> {
    let root = read_number_input(node, "root", Semitone::A3.0 as f64)? as i16;
    let bars_per_chord = read_number_input(node, "bars_per_chord", 1.0)? as u32;
    let time_signature = resolve_time_signature_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &node.id,
        "time_signature",
    )?;

    Ok(epic_minor_progression(
        Semitone(root),
        bars_per_chord,
        time_signature,
    ))
}

fn build_arpeggio_score(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node: &Node,
) -> Result<Score, TranslateError> {
    let progression = resolve_progression_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &node.id,
        "progression",
    )?;
    let tempo = resolve_tempo_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &node.id,
        "tempo",
    )?;
    let time_signature = resolve_time_signature_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &node.id,
        "time_signature",
    )?;
    let bars = read_number_input(node, "bars", 4.0)? as u32;
    let subdivision = read_number_input(node, "subdivision", 4.0)? as u8;

    Ok(arpeggio(ArpeggioConfig {
        progression,
        tempo,
        time_signature,
        bars,
        subdivision,
    }))
}

fn build_drum_grid_score(
    graph: &Graph,
    registry: &Registry,
    sample_rate: SampleRate,
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    visiting: &mut BTreeSet<NodeId>,
    node: &Node,
) -> Result<Score, TranslateError> {
    let tempo = resolve_tempo_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &node.id,
        "tempo",
    )?;
    let time_signature = resolve_time_signature_input(
        graph,
        registry,
        sample_rate,
        edges_by_target,
        visiting,
        &node.id,
        "time_signature",
    )?;
    let bars = read_number_input(node, "bars", 4.0)? as u32;
    let lane_value = read_number_input(node, "lane", 0.0)? as i32;
    let lane = match lane_value {
        0 => DrumLane::Kick,
        1 => DrumLane::Snare,
        _ => DrumLane::Kick,
    };

    Ok(drum_grid(DrumGridConfig {
        tempo,
        time_signature,
        bars,
        lane,
        hit_duration_beats: 0.25,
    }))
}

fn read_number_input(node: &Node, port: &str, default: f64) -> Result<f64, TranslateError> {
    match node.inputs.get(port) {
        Some(value) => primitive_to_f64(&node.id, port, value),
        None => Ok(default),
    }
}

fn wired_source(
    edges_by_target: &BTreeMap<TargetKey, PortReference>,
    node_id: &str,
    port_id: &str,
) -> Result<PortReference, TranslateError> {
    edges_by_target
        .get(&(node_id.into(), port_id.into()))
        .cloned()
        .ok_or_else(|| TranslateError::UnwiredInput {
            node: node_id.into(),
            port: port_id.into(),
        })
}

fn primitive_to_f64(
    node_id: &str,
    port_id: &str,
    value: &PrimitiveValue,
) -> Result<f64, TranslateError> {
    match value {
        PrimitiveValue::Number(n) => Ok(*n),
        _ => Err(TranslateError::ExpectedNumber {
            node: node_id.into(),
            port: port_id.into(),
        }),
    }
}

struct ConstantSampler(f32);

impl Sampler for ConstantSampler {
    fn at(&self, _t: f64) -> f32 {
        self.0
    }
}

struct TimeSampler;

impl Sampler for TimeSampler {
    fn at(&self, t: f64) -> f32 {
        t as f32
    }
}

struct TimeElapsedSampler {
    start: Box<dyn Sampler>,
}

impl Sampler for TimeElapsedSampler {
    fn at(&self, t: f64) -> f32 {
        let start = self.start.at(t) as f64;
        (t - start).max(0.0) as f32
    }
}

struct SineSampler {
    frequency: Box<dyn Sampler>,
    time: Box<dyn Sampler>,
}

impl Sampler for SineSampler {
    fn at(&self, t: f64) -> f32 {
        let frequency = self.frequency.at(t);
        let phase_time = self.time.at(t) as f64;
        sine(frequency, phase_time)
    }
}

struct MultiplySampler {
    a: Box<dyn Sampler>,
    b: Box<dyn Sampler>,
}

impl Sampler for MultiplySampler {
    fn at(&self, t: f64) -> f32 {
        multiply(self.a.at(t), self.b.at(t))
    }
}

struct AddSampler {
    a: Box<dyn Sampler>,
    b: Box<dyn Sampler>,
}

impl Sampler for AddSampler {
    fn at(&self, t: f64) -> f32 {
        add(self.a.at(t), self.b.at(t))
    }
}

struct ExponentialSweepSineSampler {
    start_hz: Box<dyn Sampler>,
    decay_rate: Box<dyn Sampler>,
    elapsed: Box<dyn Sampler>,
}

impl Sampler for ExponentialSweepSineSampler {
    fn at(&self, t: f64) -> f32 {
        exponential_sweep_sine(
            self.start_hz.at(t),
            self.decay_rate.at(t),
            self.elapsed.at(t) as f64,
        )
    }
}

struct DeterministicNoiseSampler {
    seed: Box<dyn Sampler>,
    time: Box<dyn Sampler>,
}

impl Sampler for DeterministicNoiseSampler {
    fn at(&self, t: f64) -> f32 {
        deterministic_noise(self.seed.at(t), self.time.at(t) as f64)
    }
}

struct HighpassNoiseSampler {
    seed: Box<dyn Sampler>,
    time: Box<dyn Sampler>,
    dt: Box<dyn Sampler>,
}

impl Sampler for HighpassNoiseSampler {
    fn at(&self, t: f64) -> f32 {
        highpass_noise(
            self.seed.at(t),
            self.time.at(t) as f64,
            self.dt.at(t) as f64,
        )
    }
}

struct LinearAdsrSampler {
    attack: Box<dyn Sampler>,
    decay: Box<dyn Sampler>,
    sustain: Box<dyn Sampler>,
    release: Box<dyn Sampler>,
    elapsed: Box<dyn Sampler>,
    note_duration: Box<dyn Sampler>,
}

impl Sampler for LinearAdsrSampler {
    fn at(&self, t: f64) -> f32 {
        let adsr = LinearAdsr {
            attack: self.attack.at(t) as f64,
            decay: self.decay.at(t) as f64,
            sustain: self.sustain.at(t),
            release: self.release.at(t) as f64,
        };
        adsr.value_at(
            self.elapsed.at(t) as f64,
            self.note_duration.at(t) as f64,
        )
    }
}

struct SemitoneToHzSampler {
    semitone: Box<dyn Sampler>,
}

impl Sampler for SemitoneToHzSampler {
    fn at(&self, t: f64) -> f32 {
        let value = self.semitone.at(t).round() as i16;
        Semitone(value).to_hz() as f32
    }
}

struct NoteSemitoneSampler {
    schedule: NoteSchedule,
}

impl Sampler for NoteSemitoneSampler {
    fn at(&self, t: f64) -> f32 {
        self.schedule
            .active_at(t)
            .map(|note| note.semitone.0 as f32)
            .unwrap_or(0.0)
    }
}

struct NoteActiveSampler {
    schedule: NoteSchedule,
}

impl Sampler for NoteActiveSampler {
    fn at(&self, t: f64) -> f32 {
        if self.schedule.active_at(t).is_some() {
            1.0
        } else {
            0.0
        }
    }
}

struct NoteStartSampler {
    schedule: NoteSchedule,
}

impl Sampler for NoteStartSampler {
    fn at(&self, t: f64) -> f32 {
        self.schedule
            .active_at(t)
            .map(|note| note.start_secs as f32)
            .unwrap_or(0.0)
    }
}

struct NoteDurationSampler {
    schedule: NoteSchedule,
}

impl Sampler for NoteDurationSampler {
    fn at(&self, t: f64) -> f32 {
        self.schedule
            .active_at(t)
            .map(|note| note.duration_secs as f32)
            .unwrap_or(0.0)
    }
}

struct NoteEnvelopeSampler {
    schedule: NoteSchedule,
    adsr: LinearAdsr,
}

impl Sampler for NoteEnvelopeSampler {
    fn at(&self, t: f64) -> f32 {
        match self.schedule.active_at(t) {
            None => 0.0,
            Some(note) => {
                let local = t - note.start_secs;
                self.adsr.value_at(local, note.duration_secs)
            }
        }
    }
}

struct ChordRootSampler {
    progression: ChordProgression,
    tempo: Tempo,
}

impl Sampler for ChordRootSampler {
    fn at(&self, t: f64) -> f32 {
        self.progression.chord_at(t, self.tempo).root.0 as f32
    }
}

struct ChordActiveSampler {
    progression: ChordProgression,
    tempo: Tempo,
}

impl Sampler for ChordActiveSampler {
    fn at(&self, t: f64) -> f32 {
        let beat = t / self.tempo.seconds_per_beat();
        if self
            .progression
            .regions
            .iter()
            .any(|region| {
                beat >= region.start_beats - 1e-9
                    && beat < region.start_beats + region.duration_beats - 1e-9
            })
        {
            1.0
        } else {
            0.0
        }
    }
}

struct TempoBpmSampler {
    tempo: Tempo,
}

impl Sampler for TempoBpmSampler {
    fn at(&self, _t: f64) -> f32 {
        self.tempo.bpm as f32
    }
}

struct BeatsPerBarSampler {
    time_signature: TimeSignature,
}

impl Sampler for BeatsPerBarSampler {
    fn at(&self, _t: f64) -> f32 {
        f32::from(self.time_signature.beats_per_bar)
    }
}

struct BeatUnitSampler {
    time_signature: TimeSignature,
}

impl Sampler for BeatUnitSampler {
    fn at(&self, _t: f64) -> f32 {
        f32::from(self.time_signature.beat_unit)
    }
}

pub fn infer_score_duration_secs(graph: &Graph, sample_rate: SampleRate) -> Option<f64> {
    let score_node = graph
        .nodes
        .values()
        .find(|node| node.node_type == "arpeggio")?;
    let edges_by_target = index_edges_by_target(graph);
    let mut visiting = BTreeSet::new();
    let registry = aura_imp::aura_registry().ok()?;
    let score = build_arpeggio_score(
        graph,
        &registry,
        sample_rate,
        &edges_by_target,
        &mut visiting,
        score_node,
    )
    .ok()?;
    let schedule = aura_scheduler::schedule_offline(&score, sample_rate).ok()?;
    Some(schedule.total_frames as f64 / sample_rate.get() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_imp::aura_registry;

    #[test]
    fn translate_sine_graph_is_pure() {
        let json = include_str!("../../../../demos/sine.json");
        let graph = aura_imp::graph_from_json_str(json).expect("graph");
        let registry = aura_registry().expect("registry");
        let sampler = translate_graph(&graph, &registry, SampleRate::RATE_44100).expect("translate");
        assert_eq!(sampler.at(0.25), sampler.at(0.25));
    }
}
