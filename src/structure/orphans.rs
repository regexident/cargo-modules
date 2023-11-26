// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::HashSet,
    fs::{self, ReadDir},
    path::{Path, PathBuf},
};

use log::{debug, trace};
use ra_ap_hir as hir;

use crate::{
    item::{attr::ItemAttrs, Item},
    structure::tree::Node,
};

pub(crate) fn orphan_nodes_for(node: &Node) -> Vec<Node> {
    assert!(node.item.is_file());
    assert!(matches!(node.item.hir, Some(hir::ModuleDef::Module(_))));

    trace!("Searching for orphans of {:?}", node.item.path);

    let file_path: &Path = node.item.file_path.as_ref().expect("file-level  module");

    let dir_path_buf = match mod_dir(file_path) {
        Some(path_buf) => path_buf,
        None => {
            debug!(
                "Could not obtain module directory path for {:?}",
                node.item.path
            );
            return vec![];
        }
    };
    let dir_path = dir_path_buf.as_path();

    if !dir_path.exists() {
        debug!("Module directory for {:?} not found", node.item.path);
        return vec![];
    }

    let read_dir = match fs::read_dir(dir_path) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            debug!("Module directory for {:?} not readable", node.item.path);
            return vec![];
        }
    };

    let crate_name = node.item.crate_name.clone();
    let parent_path = node.item.path.clone();

    let existing_submodule_names: HashSet<String> = node
        .subnodes
        .iter()
        .map(|node| node.display_name())
        .collect();

    possible_orphans_in(read_dir)
        .filter_map(|possible_orphan| {
            if existing_submodule_names.contains(&possible_orphan.name) {
                return None;
            }

            let mut path = parent_path.clone();
            path.push(possible_orphan.name);

            Some(orphan_node(
                crate_name.clone(),
                path,
                possible_orphan.file_path,
            ))
        })
        .collect()
}

fn orphan_node(crate_name: Option<String>, path: Vec<String>, file_path: PathBuf) -> Node {
    let file_path = Some(file_path);
    let hir = None;
    let visibility = None;
    let attrs = {
        let cfgs = vec![];
        let test = None;
        ItemAttrs { cfgs, test }
    };
    let kind = None;

    let item = Item {
        crate_name,
        path,
        file_path,
        hir,
        visibility,
        attrs,
        kind,
    };

    Node::new(item, vec![])
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

pub struct PossibleOrphan {
    pub name: String,
    pub file_path: PathBuf,
}

fn possible_orphans_in(read_dir: ReadDir) -> impl Iterator<Item = PossibleOrphan> {
    fn is_possible_identifier(name: &str) -> bool {
        name.chars().all(|c| c.is_alphanumeric())
    }

    read_dir.into_iter().filter_map(|dir_entry| {
        let entry_path = match dir_entry {
            Ok(dir_entry) => dir_entry.path(),
            Err(_) => {
                // If we can't retrieve the entry, then skip over it:
                return None;
            }
        };
        let file_stem = match entry_path.file_stem().and_then(|os_str| os_str.to_str()) {
            Some(name) => name,
            None => {
                // A rust file should have a file-stem (aka file-name)
                // if the file doesn't, then skip over it:
                return None;
            }
        };
        let extension = entry_path.extension().and_then(|os_str| os_str.to_str());

        // Skip any files with names that could not be valid identifiers:
        if !is_possible_identifier(file_stem) {
            return None;
        }

        let ignored_names = ["lib", "main", "mod"];
        let is_ignored_name = ignored_names.contains(&file_stem);

        let file_path = if entry_path.is_dir() {
            // If it's a directory, then there might be a 'mod.rs' file within:
            entry_path.join("mod.rs")
        } else if extension == Some("rs") && !is_ignored_name {
            // If it's a '.rs' file then we already know the path:
            entry_path.clone()
        } else {
            // The directory can contain other arbitrary files
            // as such we simply skip over them:
            return None;
        };

        // Check if our file guesses actually exist on the file-system
        // if they don't, simply skip over them:
        if !file_path.exists() {
            return None;
        }

        let name = file_stem.to_owned();

        Some(PossibleOrphan { name, file_path })
    })
}
