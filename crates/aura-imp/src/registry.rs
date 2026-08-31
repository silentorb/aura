//! Aura Imp registry and JSON helpers.

use crate::{dsp, envelope, music, time};
use imp_core_types::Graph;
use imp_registry::{create_registry, load_library, DuplicateNodeTypeError, Registry};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Returns a registry with core Imp nodes and all Aura libraries loaded.
pub fn aura_registry() -> Result<Registry, DuplicateNodeTypeError> {
    let registry = create_registry();
    let registry = load_library(registry, imp_core_types::core_node_library())?;
    let registry = load_library(registry, time::time_node_library())?;
    let registry = load_library(registry, dsp::dsp_node_library())?;
    let registry = load_library(registry, envelope::envelope_node_library())?;
    load_library(registry, music::music_node_library())
}

#[derive(Debug, Error)]
pub enum GraphJsonError {
    #[error("failed to read graph JSON from {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse graph JSON from {path}: {source}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    #[error("failed to serialize graph JSON: {0}")]
    Serialize(serde_json::Error),
}

/// Loads an Imp graph from a JSON file.
pub fn graph_from_json_path(path: &Path) -> Result<Graph, GraphJsonError> {
    let path_str = path.to_string_lossy().into_owned();
    let contents = fs::read_to_string(path).map_err(|source| GraphJsonError::Read {
        path: path_str.clone(),
        source,
    })?;
    graph_from_json_str(&contents).map_err(|source| GraphJsonError::Parse {
        path: path_str,
        source,
    })
}

/// Parses an Imp graph from a JSON string.
pub fn graph_from_json_str(json: &str) -> Result<Graph, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serializes an Imp graph to a JSON string.
pub fn graph_to_json_string(graph: &Graph) -> Result<String, GraphJsonError> {
    serde_json::to_string_pretty(graph).map_err(GraphJsonError::Serialize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use imp_registry::get_node_type;

    #[test]
    fn aura_registry_contains_expected_nodes() {
        let registry = aura_registry().expect("registry");
        assert!(get_node_type(&registry, "time").is_some());
        assert!(get_node_type(&registry, "sine").is_some());
        assert!(get_node_type(&registry, "arpeggio").is_some());
        assert!(get_node_type(&registry, "epic_minor_progression").is_some());
        assert!(get_node_type(&registry, "constant_tempo").is_some());
        assert!(get_node_type(&registry, "constant_time_signature").is_some());
    }
}
