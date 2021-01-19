use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use log::{debug, trace};
use petgraph::{graph::NodeIndex, visit::EdgeRef};

use crate::{
    graph::{Edge, Graph, Node},
    orphans::PossibleOrphansIterator,
};

pub(crate) fn add_orphan_nodes_to(graph: &mut Graph, module_idx: NodeIndex<usize>) {
    let module_node = graph[module_idx].clone();

    trace!("Searching for orphans of {:?}", module_node.path);

    let file_path_buf = match &module_node.file_path {
        Some(file_path) => file_path.clone(),
        None => {
            debug!("Module {:?} is not at file-level", module_node.path);
            return;
        }
    };
    let file_path = file_path_buf.as_path();

    let dir_path_buf = match mod_dir(file_path) {
        Some(path_buf) => path_buf,
        None => {
            debug!(
                "Could not obtain module directory path for {:?}",
                module_node.path
            );
            return;
        }
    };
    let dir_path = dir_path_buf.as_path();

    let existing_modules = sub_module_nodes(graph, module_idx);

    let existing_module_names: HashSet<String> = existing_modules
        .into_iter()
        .map(|module| module.name())
        .collect();

    if !dir_path.exists() {
        debug!("Module directory for {:?} not found", module_node.path);
        return;
    }

    let read_dir = match fs::read_dir(dir_path) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            debug!("Module directory for {:?} not readable", module_node.path);
            return;
        }
    };

    for possible_orphan in PossibleOrphansIterator::new(read_dir) {
        let crate::orphans::PossibleOrphan { name, path } = possible_orphan;

        if existing_module_names.contains(&name) {
            continue;
        }

        add_orphan_node(graph, module_idx, path.as_path(), &name);
    }
}

fn add_orphan_node(
    graph: &mut Graph,
    module_idx: NodeIndex<usize>,
    orphan_file_path: &Path,
    orphan_name: &str,
) {
    let module_node = &graph[module_idx];

    let orphan_node = {
        let path = format!("{}::{}", module_node.path, orphan_name);
        let file_path = Some(orphan_file_path.to_owned());
        let hir = None;
        let is_external = false;
        Node {
            path,
            file_path,
            hir,
            is_external,
        }
    };

    let orphan_idx = graph.add_node(orphan_node);

    let edge = Edge::HasA;
    graph.add_edge(module_idx, orphan_idx, edge);
}

fn sub_module_nodes(graph: &mut Graph, module_node_idx: NodeIndex<usize>) -> Vec<Node> {
    graph
        .edges_directed(module_node_idx, petgraph::Direction::Outgoing)
        .filter_map(|edge_ref| {
            let child = &graph[edge_ref.target()];
            Some(child.clone())
        })
        .collect()
}

fn mod_dir(file_path: &Path) -> Option<PathBuf> {
    let file_stem = file_path.file_stem().and_then(|os_str| os_str.to_str());
    let extension = file_path.extension().and_then(|os_str| os_str.to_str());

    match (file_stem, extension) {
        (Some("lib"), Some("rs")) | (Some("main"), Some("rs")) | (Some("mod"), Some("rs")) => {
            file_path.parent().map(|p| p.to_path_buf()).into()
        }
        (Some(file_stem), Some("rs")) => file_path.parent().map(|p| p.join(file_stem)).into(),
        _ => None,
    }
}
