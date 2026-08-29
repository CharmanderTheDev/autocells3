mod math;

use std::{fs, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use petgraph::Graph;
use petgraph::graph::{Edge, NodeIndex};
use petgraph_graphml::GraphMl;
use rust_igraph::{layout_reingold_tilford, layout_reingold_tilford_circular, RtMode};
use serde::Serialize;
use rand::{random, RngExt};
use crate::math::*;

const SCALE_FACTOR: f64 = 200.0;

#[derive(Clone, Debug, Serialize)]
pub struct NodeData {

    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct EdgeData {

    pub weight: f64,
}


fn delete_folder_contents(dir_path: &str) -> io::Result<()> {
    // Read the contents of the directory
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Delete subdirectories and their contents recursively
            fs::remove_dir_all(&path)?;
        } else {
            // Delete individual files or symlinks
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

fn tree_to_petgraph(tree: rust_igraph::Graph) -> petgraph::Graph<NodeData, EdgeData>
{
    let layout = layout_reingold_tilford_circular(&tree, Some(tree.vcount() - 1), RtMode::All).unwrap()
        .iter().map(|[x, y]| NodeData {x: x * SCALE_FACTOR, y: y * SCALE_FACTOR}).collect::<Vec<NodeData>>();

    let mut petgraph: Graph<NodeData, EdgeData> = petgraph::Graph::new();

    let node_ids: Vec<NodeIndex> = (0..tree.vcount()).map(|i| {

        petgraph.add_node(layout[i as usize].clone())
    }).collect();

    tree.edges().for_each(|(i, o)| {

        petgraph.add_edge(node_ids[i as usize], node_ids[o as usize], EdgeData { weight: 1.0 });
    });

    petgraph
}

/// places nodes at slightly randomized positions to allow force-based layouts to work correctly
fn graph_to_petgraph(graph: rust_igraph::Graph) -> petgraph::Graph<NodeData, EdgeData> {

    let mut petgraph: Graph<NodeData, EdgeData> = petgraph::Graph::new();

    let mut rng = rand::rng();

    let node_ids: Vec<NodeIndex> = (0..graph.vcount()).map(|i| {

        petgraph.add_node(NodeData { x: rng.random(), y: rng.random() })
    }).collect();

    graph.edges().for_each(|(i, o)| {

        petgraph.add_edge(node_ids[i as usize], node_ids[o as usize], EdgeData { weight: 1.0 });
    });

    petgraph
}

fn export_petgraph(petgraph: petgraph::Graph<NodeData, EdgeData>, name: String) {

    GraphMl::new(&petgraph)
        .export_node_weights(Box::new(| NodeData { x, y } | {
            vec![
                ("x".into(), format!("{x}").into()),
                ("y".into(), format!("{y}").into()),
            ]
        }))
        .export_edge_weights(Box::new(| EdgeData { weight } | {

            vec![
                ("Weight".into(), format!("{weight}").into())
            ]
        }))
        .to_writer(
            BufWriter::new(File::create(format!("out/{name}.graphml")).unwrap())
        ).unwrap();
}

fn render_action_graphs
<
const STATE_COUNT: usize,
const ADJACENT_COUNT: usize
>
(
    autocells: AutoCells<usize, STATE_COUNT, ADJACENT_COUNT>,
    leaf_culling: bool,
) {

    let mut loop_size_counts: HashMap<usize, usize> = HashMap::new();

    to_trees(&autocells.action_graph(), leaf_culling).into_iter().for_each(|(cycle_size, tree)| {

        let petgraph = tree_to_petgraph(tree);

        let loop_size_count = loop_size_counts.entry(cycle_size).or_insert(0);

        export_petgraph(petgraph, format!("{cycle_size}_{loop_size_count}"));

        *loop_size_count += 1;
    })
}

fn export_shape_graph
<
const STATE_COUNT: usize,
const ADJACENT_COUNT: usize,
>
(
    autocells: AutoCells<usize, STATE_COUNT, ADJACENT_COUNT>,
) {

    export_petgraph(graph_to_petgraph(autocells.shape_graph()), "shape_graph".to_string());
}

fn main() {

    delete_folder_contents("./out").unwrap();

    /*render_action_graphs::<2,3>(

        trivial_autocells(true),
        false
    );*/

    export_shape_graph::<4,4>(
        trivial_autocells(true)
    )
}