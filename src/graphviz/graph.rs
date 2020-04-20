use std::io::{Write, Result};
use std::collections::HashMap;
use std::cell::{Ref, RefCell};
use std::ops::{Deref, DerefMut};

use petgraph::graph::{self, DiGraph, DefaultIx};
use petgraph::visit::EdgeRef;

/// Type alias for the graph representation of petgraph's graph used in this module.
pub type Graph = DiGraph<Node, Edge>;
pub type GraphIndex = DefaultIx;
pub type NodeIndex = graph::NodeIndex<GraphIndex>;
pub type EdgeIndex = graph::EdgeIndex<GraphIndex>;

/// Defines the type of a graph.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum GraphType {
    /// An undirected graph where an edge between A and B implies the same edge to exist between
    /// B and A.
    Graph,
    /// A directed graph where an edge between A and B does *not* imply an edge between B and A.
    Digraph,
}

/// A node inside the graph.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Node {
    /// Id / name of the node.
    pub id: String,
    /// Attributes of this node as defined by the
    /// [dot language specification](http://www.graphviz.org/doc/info/lang.html).
    pub attributes: HashMap<String, String>,
}

impl Node {
    /// Creates a new node with given id and attributes.
    pub fn new(id: String, attributes: HashMap<String, String>) -> Node {
        Node {
            id,
            attributes,
        }
    }
}

/// An edge between two nodes inside the graph.
///
/// The metadata information of which nodes are connected by this edge is held by the wrapped
/// petgraph's graph.
/// Only additional information allowed by the dot language is part of this struct.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Edge {
    /// Attributes of this edge as defined by the
    /// [dot languge specification](http://www.graphviz.org/doc/info/lang.html).
    pub attributes: HashMap<String, String>,
}

impl Edge {
    /// Creates a new edge with given attributes.
    pub fn new(attributes: HashMap<String, String>) -> Edge {
        Edge {
            attributes: attributes,
        }
    }
}

/// Wrapper around [`petgraph::DiGraph`] including [dot language](http://www.graphviz.org/doc/info/lang.html)
/// specific fields and attributes.
///
/// `DotGraph` derefs (mutably) into [`petgraph::DiGraph`].
/// Thus, you can use all its functions and directly access the wrapped internal graph.
///
/// [`petgraph::DiGraph`]: https://docs.rs/petgraph/0.4.9/petgraph/graph/type.DiGraph.html
#[derive(Debug, Clone)]
pub struct DotGraph {
    /// Specifies if this graph is strict.
    pub strict: bool,
    /// Defines the type of this graph.
    pub _type: GraphType,
    /// Id / Name
    pub id: Option<String>,
    /// Global `graph` attributes
    pub graph_attributes: HashMap<String, String>,
    /// Global `node` attributes
    pub node_attributes: HashMap<String, String>,
    /// Global `edge` attributes
    pub edge_attributes: HashMap<String, String>,
    /// Internal wrapped petgraph graph
    graph: Graph,
    /// Map from labels to the node; lazily generated
    label_map: RefCell<Option<HashMap<String, NodeIndex>>>,
    /// Map from ids to the node; lazily generated
    id_map: RefCell<Option<HashMap<String, NodeIndex>>>,
}

impl DotGraph {
    /// Creates a new graph given all attributes. Prefer using
    /// [`DotGraphBuilder`](struct.DotGraphBuilder.html) instead.
    pub fn new(strict: bool, _type: GraphType, id: Option<String>, graph_attributes: HashMap<String, String>,
               node_attributes: HashMap<String, String>, edge_attributes: HashMap<String, String>,
               graph: Graph) -> DotGraph {
        DotGraph {
            strict: strict,
            _type: _type,
            id: id,
            graph_attributes: graph_attributes,
            node_attributes: node_attributes,
            edge_attributes: edge_attributes,
            graph: graph,
            label_map: RefCell::new(None),
            id_map: RefCell::new(None),
        }
    }

    /// Lazily returns a map from the label graphviz node property to the according NodeIndex.
    ///
    /// If `deref_mut` is used, this map will be regenerated lazily.
    pub fn label_map(&self) -> Ref<HashMap<String, NodeIndex>> {
        let label_map = self.label_map.borrow();
        if label_map.is_some() {
            return Ref::map(label_map, |opt| opt.as_ref().unwrap());
        }
        drop(label_map);
        let map = self.graph.node_indices()
            .filter_map(|ix| self.graph[ix].attributes.get("label").map(|l| (l.clone(), ix)))
            .collect();
        *self.label_map.borrow_mut() = Some(map);
        Ref::map(self.label_map.borrow(), |opt| opt.as_ref().unwrap())
    }

    /// Lazily returns a map from graphviz node ids to the according NodeIndex.
    ///
    /// If `deref_mut` is used, this map will be regenerated lazily.
    pub fn id_map(&self) -> Ref<HashMap<String, NodeIndex>> {
        let node_map = self.id_map.borrow();
        if node_map.is_some() {
            return Ref::map(node_map, |opt| opt.as_ref().unwrap());
        }
        drop(node_map);
        let map = self.graph.node_indices()
            .map(|ix| (self.graph[ix].id.clone(), ix))
            .collect();
        *self.id_map.borrow_mut() = Some(map);
        Ref::map(self.id_map.borrow(), |opt| opt.as_ref().unwrap())
    }

    /// Writes this graph in a dot compatible format to given writer.
    ///
    /// This method can be used to save a `DotGraph` to a file.
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write graph to.
    ///
    /// # Example
    /// ```ignore
    /// let graph = DotGraphBuilder::new().build();
    /// let mut file = File::create("foo.dot");
    /// graph.write(&mut file).unwrap();
    /// ```
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        if self.strict {
            write!(writer, "strict ")?;
        }
        match self._type {
            GraphType::Graph => write!(writer, "graph ")?,
            GraphType::Digraph => write!(writer, "digraph ")?,
        }
        if let Some(ref id) = self.id {
            write!(writer, "{:?} ", id)?;
        }
        writeln!(writer, "{{")?;

        if !self.graph_attributes.is_empty() {
            writeln!(writer, "  graph [")?;
            for (ref key, ref value) in self.graph_attributes.iter() {
                writeln!(writer, "    {} = {:?}", key, value)?;
            }
            writeln!(writer, "  ]")?;
        }
        if !self.node_attributes.is_empty() {
            writeln!(writer, "  node [")?;
            for (ref key, ref value) in self.node_attributes.iter() {
                writeln!(writer, "    {} = {:?}", key, value)?;
            }
            writeln!(writer, "  ]")?;
        }
        if !self.edge_attributes.is_empty() {
            writeln!(writer, "  edge [")?;
            for (ref key, ref value) in self.edge_attributes.iter() {
                writeln!(writer, "    {} = {:?}", key, value)?;
            }
            writeln!(writer, "  ]")?;
        }

        for ix in self.graph.node_indices() {
            let node = &self.graph[ix];
            writeln!(writer, "  {:?} [", node.id)?;
            for (ref key, ref value) in node.attributes.iter() {
                writeln!(writer, "    {} = \"{}\"", key, value)?;
            }
            writeln!(writer, "  ]")?;
        }

        for edgeref in self.graph.edge_references() {
            let edge = &self.graph[edgeref.id()];
            let source = &self.graph[edgeref.source()];
            let target = &self.graph[edgeref.target()];
            let edgeop = match self._type {
                GraphType::Digraph => "->",
                GraphType::Graph => "--",
            };
            write!(writer, "  {:?} {} {:?}", source.id, edgeop, target.id)?;
            writeln!(writer, "[")?;
            for (ref key, ref value) in edge.attributes.iter() {
                writeln!(writer, "    {} = {:?}", key, value)?;
            }
            writeln!(writer, "  ]")?;
        }

        writeln!(writer, "}}")?;
        Ok(())
    }
}

impl Deref for DotGraph {
    type Target = Graph;

    fn deref(&self) -> &Graph {
        &self.graph
    }
}

impl DerefMut for DotGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.id_map.borrow_mut().take();
        self.label_map.borrow_mut().take();
        &mut self.graph
    }
}
