// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use log::{debug, trace};
use petgraph::{graph::NodeIndex, visit::EdgeRef};

use crate::{
    graph::{
        edge::{Edge, EdgeKind},
        node::Node,
        Graph,
    },
    item::{attr::ItemAttrs, Item},
    orphans::PossibleOrphansIterator,
};

pub(crate) fn add_orphan_nodes_to(graph: &mut Graph, module_idx: NodeIndex) {
    let module_node = graph[module_idx].clone();

    trace!("Searching for orphans of {:?}", module_node.item.path);

    let file_path_buf = match &module_node.item.file_path {
        Some(file_path) => file_path.clone(),
        None => {
            debug!("Module {:?} is not at file-level", module_node.item.path);
            return;
        }
    };
    let file_path = file_path_buf.as_path();

    let dir_path_buf = match mod_dir(file_path) {
        Some(path_buf) => path_buf,
        None => {
            debug!(
                "Could not obtain module directory path for {:?}",
                module_node.item.path
            );
            return;
        }
    };
    let dir_path = dir_path_buf.as_path();

    let existing_modules = sub_module_nodes(graph, module_idx);

    let existing_module_names: HashSet<String> = existing_modules
        .into_iter()
        .map(|module| module.display_name())
        .collect();

    if !dir_path.exists() {
        debug!("Module directory for {:?} not found", module_node.item.path);
        return;
    }

    let read_dir = match fs::read_dir(dir_path) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            debug!(
                "Module directory for {:?} not readable",
                module_node.item.path
            );
            return;
        }
    };

    let mut possible_orphans: Vec<_> = PossibleOrphansIterator::new(read_dir).collect();

    // Directory traversal can be platform-dependent, so in order to make the output
    // uniform we give up some performance and sort the list of possible orphans:
    possible_orphans.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    for possible_orphan in possible_orphans {
        let crate::orphans::PossibleOrphan { name, path } = possible_orphan;

        if existing_module_names.contains(&name) {
            continue;
        }

        add_orphan_node(graph, module_idx, path.as_path(), &name);
    }
}

fn add_orphan_node(
    graph: &mut Graph,
    module_idx: NodeIndex,
    orphan_file_path: &Path,
    orphan_name: &str,
) {
    let module_node = &graph[module_idx];

    let orphan_node = {
        let krate = module_node.item.krate.clone();
        let path = {
            let mut path = module_node.item.path.clone();
            path.push(orphan_name.to_owned());
            path
        };
        let file_path = Some(orphan_file_path.to_owned());
        let hir = None;
        let visibility = None;
        let attrs = {
            let cfgs = vec![];
            let test = None;
            ItemAttrs { cfgs, test }
        };

        let item = Item {
            krate,
            path,
            file_path,
            hir,
            visibility,
            attrs,
        };

        Node::new(item)
    };

    let orphan_idx = graph.add_node(orphan_node);

    let edge = Edge {
        kind: EdgeKind::Owns,
    };
    graph.add_edge(module_idx, orphan_idx, edge);
}

fn sub_module_nodes(graph: &mut Graph, module_node_idx: NodeIndex) -> Vec<Node> {
    graph
        .edges_directed(module_node_idx, petgraph::Direction::Outgoing)
        .map(|edge_ref| {
            let child = &graph[edge_ref.target()];
            child.clone()
        })
        .collect()
}

fn mod_dir(file_path: &Path) -> Option<PathBuf> {
    let file_stem = file_path.file_stem().and_then(|os_str| os_str.to_str());
    let extension = file_path.extension().and_then(|os_str| os_str.to_str());

    match (file_stem, extension) {
        (Some("lib"), Some("rs")) | (Some("main"), Some("rs")) | (Some("mod"), Some("rs")) => {
            file_path.parent().map(|p| p.to_path_buf())
        }
        (Some(file_stem), Some("rs")) => file_path.parent().map(|p| p.join(file_stem)),
        _ => None,
    }
}
