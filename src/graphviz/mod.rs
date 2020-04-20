//! Aggregation module of all graph related code.
//!
//! This module contains the struct holding a graph and methods to convert it into a grap-compatible
//! format.

mod graph;
mod builder;
mod dot;

pub use dot::parse;
pub use self::graph::{
    Graph,
    GraphIndex,
    NodeIndex,
    EdgeIndex,
    GraphType,
    Node,
    Edge,
    DotGraph,
};
pub use self::builder::DotGraphBuilder;
