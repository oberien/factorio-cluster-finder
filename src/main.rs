use std::collections::{HashMap, HashSet};

use crate::graphviz::{DotGraph, NodeIndex};
use petgraph::Direction;

mod graphviz;

fn main() {
    env_logger::init();
    let dot = std::fs::read_to_string("recipe.dot").unwrap();
    let graph= graphviz::parse(&dot);

    fn subgraph_neighbors_with_duplicates<'a>(subgraph: &'a HashSet<NodeIndex>, graph: &'a DotGraph) -> impl Iterator<Item = NodeIndex> + 'a {
        subgraph.iter()
            .copied()
            .flat_map(move |node_idx| graph.neighbors_undirected(node_idx))
            .filter(move |neighbor_idx| !subgraph.contains(neighbor_idx))
    }
    fn score(subgraph: &HashSet<NodeIndex>, graph: &DotGraph) -> (usize, usize) {
        // number of dependencies, i.e., number of components required as input
        let num_deps = subgraph.iter()
            .copied()
            .flat_map(|node_idx| graph.neighbors_directed(node_idx, Direction::Outgoing))
            .filter(|neighbor_idx| !subgraph.contains(neighbor_idx))
            .count();

        // Number of outputs needed by other components,
        // i.e. number of distinct output products required by other components.
        // However, we shouldn't count sole inputs as output components (e.g. don't pipe through iron-plates).
        let num_outputs = subgraph.iter()
            .copied()
            .filter(|node_idx|
                graph.neighbors_directed(*node_idx, Direction::Incoming)
                    .filter(|neighbor_idx| !subgraph.contains(neighbor_idx))
                    .next().is_some()
            ).filter(|node_ix|
                graph.neighbors_directed(*node_ix, Direction::Outgoing)
                    .filter(|neighbor_ix| subgraph.contains(neighbor_ix))
                    .next().is_some()
            ).count();
        (num_deps, num_outputs)
    }

    // greedy
    let mut node_set: HashSet<_> = graph.node_indices().collect();
    let mut current_cluster = HashSet::new();

    let search_for = &["sulfuric-acid"];

    for name in search_for {
        let item = graph.id_map()[*name];
        current_cluster.insert(item);
        node_set.remove(&item);
    }
    println!("starting with {} (score: {:?})", search_for.join(", "), score(&current_cluster, &graph));
    loop {
        let mut scores = Vec::new();
        for node_idx in subgraph_neighbors_with_duplicates(&current_cluster, &graph) {
            let mut cluster = current_cluster.clone();
            cluster.insert(node_idx);
            let (num_deps, num_outputs) = score(&cluster, &graph);
            scores.push((node_idx, num_deps, num_outputs));
        }

        let (current_deps, current_outputs) = score(&current_cluster, &graph);

        let mut added_something = false;

        for (node_idx, num_deps, num_outputs) in scores.iter().cloned() {
            if current_cluster.contains(&node_idx) {
                continue;
            }
            let score = num_deps + num_outputs;
            let current_score = current_deps + current_outputs;
            if score <= current_score || (num_deps == current_deps && num_outputs > current_outputs) {
                println!("    adding {} (score: {:?})", graph[node_idx].id, (num_deps, num_outputs));
                current_cluster.insert(node_idx);
                node_set.remove(&node_idx);
                added_something = true;
            }
        }
        println!("    ---------");

        if !added_something {
            scores.sort_by_key(|(_, num_deps, num_outputs)| num_deps + num_outputs);
            let lowest = scores[0];
            for (node_idx, num_deps, num_outputs) in scores {
                let score = num_deps + num_outputs;
                let lowest_score = lowest.1 + lowest.2;
                if score <= lowest_score {
                    println!("    lowest would have been {} (score: {:?})", graph[node_idx].id, score);
                }
            }
            let next = *node_set.iter().next().unwrap();
            node_set.remove(&next);
            current_cluster.clear();
            current_cluster.insert(next);
            println!("starting with {} (score: {:?})", graph[next].id, score(&current_cluster, &graph));
            break;
        }
    }
}
