#![deny(missing_docs)]

//! # Graphs

/// This is the crate-level documentation.
/// This crate provides functionality for working with graphs.
pub mod graph;

/// Trait representing a node value in a graph.
pub trait NodeValue<NI: graph::Idx, NV> {
    /// Get the value of a node.
    fn node_value(&self, node: NI) -> &NV;
}
