#![allow(ellipsis_inclusive_range_patterns)]

use std::collections::{HashMap, HashSet};

use log::*;

use crate::graphviz::{Graph, GraphType, Node, Edge, DotGraph, DotGraphBuilder};

/// Immediate representation of the type of a global attribute
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum AttributeType {
    Graph,
    Node,
    Edge,
}

/// Immediate representation of a global graph attribute
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GlobalAttribute {
    _type: AttributeType,
    attributes: HashMap<String, String>,
}

impl GlobalAttribute {
    pub fn new(_type: AttributeType, attributes: HashMap<String, String>) -> GlobalAttribute {
        GlobalAttribute {
            _type,
            attributes,
        }
    }
}

/// A dot language `stmt`
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement {
    Node(Node),
    Edge(EdgeInternal),
    GlobalAttribute(GlobalAttribute),
}

/// Immediate representation of an Edge with attributes
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EdgeInternal {
    attributes: HashMap<String, String>,
    nodes: Vec<String>,
}

/// Immediate representation of a DotGraph
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GraphInternal {
    strict: bool,
    _type: GraphType,
    id: Option<String>,
    statements: Vec<Statement>,
}

include!(concat!(env!("OUT_DIR"), "/dot.rs"));

/// Parses a dot language graph without subgraphs and ports into a DotGraph
pub fn parse(s: &str) -> DotGraph {
    debug!("parsing str to DotGraph");
    let mut graph_internal: GraphInternal = graph(s).unwrap();
    let mut graph_attributes = HashMap::new();
    let mut node_attributes = HashMap::new();
    let mut edge_attributes = HashMap::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    debug!("Converting statements into values");
    for stmt in graph_internal.statements.drain(..) {
        match stmt {
            Statement::GlobalAttribute(mut attr) => match attr._type {
                AttributeType::Graph => graph_attributes.extend(attr.attributes.drain()),
                AttributeType::Node => node_attributes.extend(attr.attributes.drain()),
                AttributeType::Edge => edge_attributes.extend(attr.attributes.drain()),
            },
            Statement::Node(node) => nodes.push(node),
            Statement::Edge(edge) => edges.push(edge),
        }
    }

    let mut graph = Graph::new();
    let mut node_id_set = HashSet::new();
    debug!("Adding all node definitions to Graph");
    for node in nodes {
        if !node_id_set.contains(&node.id) {
            node_id_set.insert(node.id.clone());
            graph.add_node(node);
        }
    }
    // Graphviz doesn't require all nodes to be defined beforehand.
    // Instead, undefined nodes used in edges become nodes without attributes.
    debug!("Adding nodes from edge-definitions to graph");
    for edge in &edges {
        for node_id in &edge.nodes {
            if !node_id_set.contains(node_id) {
                node_id_set.insert(node_id.clone());
                graph.add_node(Node {
                    id: node_id.clone(),
                    attributes: Default::default(),
                });
            }
        }
    }

    let edge_fn = move |graph: &DotGraph| {
        edges.iter()
            .flat_map(|e| {
                let attributes = &e.attributes;
                e.nodes.iter()
                    .zip(e.nodes.iter().skip(1))
                    .map(move |(source, target)| (
                        Edge::new(attributes.clone()),
                        *graph.id_map().get(source).unwrap(),
                        *graph.id_map().get(target).unwrap(),
                    ))
            }).collect()
    };

    DotGraphBuilder::new(graph_internal._type)
        .strict(graph_internal.strict)
        .id(graph_internal.id)
        .graph_attributes(graph_attributes)
        .node_attributes(node_attributes)
        .edge_attributes(edge_attributes)
        .graph(graph)
        .edges_fn(edge_fn)
        .build()
}

#[test]
fn test_escaped() {
    assert_eq!(escaped(r#"\""#).unwrap(), r#"""#);
    assert_eq!(escaped(r"\\").unwrap(), r#"\"#);
}

#[test]
fn test_double_quoted_inner() {
    assert_eq!(doubleQuotedInner("foo\\\"bar\\\"baz qux").unwrap(), "foo\"bar\"baz qux");
}
#[test]
#[should_panic]
fn test_double_quoted_inner2() {
    println!("{:?}", doubleQuotedInner("\"").unwrap());
}

#[test]
fn test_double_quoted_string() {
    assert_eq!(doubleQuotedString("\"foo\\\" bar\\\" baz\"").unwrap(), "foo\" bar\" baz");
}

#[test]
fn test_id() {
    assert_eq!(id("foobar").unwrap(), "foobar");
    assert_eq!(id("\"foo bar\\\" baz\"").unwrap(), "foo bar\" baz");
    assert_eq!(id("1337").unwrap(), "1337");
    assert_eq!(id(".42").unwrap(), ".42");
    assert_eq!(id("322.69").unwrap(), "322.69");
}
