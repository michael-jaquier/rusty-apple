#![deny(missing_docs)]

//! # Graphs

pub mod ball_and_stick;
/// This is the crate-level documentation.
/// This crate provides functionality for working with graphs.
pub mod graph;

pub trait NodeValue<NI: graph::Idx, NV> {
    fn node_value(&self, node: NI) -> &NV;
}
