use std::collections::HashMap;

use log::*;

use crate::graphviz::{
    Node,
    Edge,
    NodeIndex,
    DotGraph,
    GraphType,
    Graph
};

/// Builder to easily create a [`DotGraph`].
///
/// [`DotGraph`]: struct.DotGraph.html
pub struct DotGraphBuilder {
    strict: Option<bool>,
    _type: GraphType,
    id: Option<Option<String>>,
    graph_attributes: Option<HashMap<String, String>>,
    node_attributes: Option<HashMap<String, String>>,
    edge_attributes: Option<HashMap<String, String>>,
    nodes: Option<Vec<Node>>,
    edges: Option<Vec<(Edge, NodeIndex, NodeIndex)>>,
    // we can't use a generic type, because we can't get a named default type
    edges_fn: Option<Box<dyn FnOnce(&DotGraph) -> Vec<(Edge, NodeIndex, NodeIndex)>>>,
    graph: Option<Graph>,
}

impl DotGraphBuilder {
    /// Creates a new Builder and initializes it with default values.
    pub fn new(_type: GraphType) -> DotGraphBuilder {
        DotGraphBuilder {
            strict: None,
            _type: _type,
            id: None,
            graph_attributes: None,
            node_attributes: None,
            edge_attributes: None,
            nodes: None,
            edges: None,
            edges_fn: None,
            graph: None,
        }
    }
    /// Sets or unsets this graph's `strict` attribute as defined by the
    /// [dot language specification](http://www.graphviz.org/doc/info/lang.html).
    pub fn strict(mut self, strict: bool) -> DotGraphBuilder {
        self.strict = Some(strict);
        self
    }
    /// Sets the id / name of the graph.
    pub fn id(mut self, id: Option<String>) -> DotGraphBuilder {
        self.id = Some(id);
        self
    }
    /// Sets this graph's global `graph` attributes.
    pub fn graph_attributes(mut self, attrs: HashMap<String, String>) -> DotGraphBuilder {
        self.graph_attributes = Some(attrs);
        self
    }
    /// Sets this graph's global `node` attributes.
    pub fn node_attributes(mut self, attrs: HashMap<String, String>) -> DotGraphBuilder {
        self.node_attributes = Some(attrs);
        self
    }
    /// Sets this graph's global `edge` attributes.
    pub fn edge_attributes(mut self, attrs: HashMap<String, String>) -> DotGraphBuilder {
        self.edge_attributes = Some(attrs);
        self
    }
    /// Sets the internal graph with all nodes and edges.
    pub fn graph(mut self, graph: Graph) -> DotGraphBuilder {
        self.graph = Some(graph);
        self
    }
    /// Sets the list of nodes, which will be added to the given or default graph.
    pub fn nodes(mut self, nodes: Vec<Node>) -> DotGraphBuilder {
        self.nodes = Some(nodes);
        self
    }
    /// Sets the list of edges, which will be added to the given or default graph.
    pub fn edges(mut self, edges: Vec<(Edge, NodeIndex, NodeIndex)>) -> DotGraphBuilder {
        self.edges = Some(edges);
        self
    }
    /// Sets an edge-function which will be called after the graph is fully built and given the graph
    /// returns a list of edges which will be added to the graph.
    ///
    /// This function will be applied last, after every other property has been assigned to the graph.
    /// Therefore, the passed graph will be as finished as possible given all other set attributes.
    pub fn edges_fn(mut self, edges_fn: impl FnOnce(&DotGraph) -> Vec<(Edge, NodeIndex, NodeIndex)> + 'static) -> DotGraphBuilder {
        self.edges_fn = Some(Box::new(edges_fn));
        self
    }

    /// Builds and returns the graph.
    pub fn build(self) -> DotGraph {
        debug!("Building graph from DotGraphBuilder");
        let mut graph = self.graph.unwrap_or_default();
        if let Some(nodes) = self.nodes {
            for node in nodes {
                graph.add_node(node);
            }
        }

        if let Some(edges) = self.edges {
            for (edge, source, target) in edges {
                graph.add_edge(source, target, edge);
            }
        }

        let mut dot_graph = DotGraph::new(
            self.strict.unwrap_or(false),
            self._type,
            self.id.unwrap_or(None),
            self.graph_attributes.unwrap_or(HashMap::new()),
            self.node_attributes.unwrap_or(HashMap::new()),
            self.edge_attributes.unwrap_or(HashMap::new()),
            graph,
        );

        debug!("applying edge function");
        if let Some(edges_fn) = self.edges_fn {
            let edges = edges_fn(&dot_graph);
            for (edge, source, target) in edges {
                dot_graph.add_edge(source, target, edge);
            }
        }
        dot_graph
    }
}
