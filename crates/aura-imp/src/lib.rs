//! Imp graph integration for Aura.
//!
//! Provides Aura-specific Imp node libraries and helpers. FunDSP lowering lives in `aura-dsp`.

pub mod dsp;

pub use imp_core_types::{
    core_node_library, Edge, EdgeId, Graph, InputValues, Node, NodeId, NodeLibrary, NodeType,
    NodeTypeId, Port, PortId, PortReference, Ports, PrimitiveValue, SignalType, SignalTypeId,
};
pub use imp_registry::{
    create_registry, get_node_type, list_libraries, list_node_types, load_library, DuplicateNodeTypeError,
    Registry,
};
