// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use log::trace;
use petgraph::graph::{EdgeIndex, NodeIndex};
use ra_ap_hir::{self as hir, Crate, ModuleSource};
use ra_ap_ide_db::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::graph::{
    edge::{Edge, EdgeKind},
    node::{
        attr::{NodeAttrs, NodeCfgAttr, NodeTestAttr},
        visibility::NodeVisibility,
        Node, NodeKind,
    },
    orphans::add_orphan_nodes_to,
    util, Graph,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Options {
    pub focus_on: Option<String>,
    pub max_depth: Option<usize>,
    pub with_types: bool,
    pub with_tests: bool,
    pub with_orphans: bool,
    pub with_uses: bool,
    pub with_externs: bool,
}

#[derive(Debug)]
pub struct Builder<'a> {
    options: Options,
    db: &'a RootDatabase,
    vfs: &'a Vfs,
    krate: hir::Crate,
    graph: Graph,
    nodes: HashMap<String, NodeIndex>,
    edges: HashMap<(NodeIndex, EdgeKind, NodeIndex), EdgeIndex>,
}

impl<'a> Builder<'a> {
    pub fn new(options: Options, db: &'a RootDatabase, vfs: &'a Vfs, krate: hir::Crate) -> Self {
        let graph = Graph::default();
        let nodes = HashMap::default();
        let edges = HashMap::default();

        Self {
            options,
            db,
            vfs,
            krate,
            graph,
            nodes,
            edges,
        }
    }

    pub fn build(mut self, krate: Crate) -> anyhow::Result<Graph> {
        trace!("Scanning project ...");

        self.process_crate(krate)?;

        Ok(self.graph)
    }

    fn process_crate(&mut self, krate: Crate) -> anyhow::Result<()> {
        let root_module = krate.root_module(self.db);
        let root_module_def = hir::ModuleDef::Module(root_module);

        self.process_owned_module_def(None, root_module_def)?;

        Ok(())
    }

    fn process_owned_module_def(
        &mut self,
        owner: Option<(hir::Module, NodeIndex)>,
        owned_module_def: hir::ModuleDef,
    ) -> anyhow::Result<Option<NodeIndex>> {
        if !self.options.with_types {
            // Check if target is a type (i.e. not a module).
            let is_module = matches!(owned_module_def, hir::ModuleDef::Module(_));

            // If it is a type we bail out:
            if !is_module {
                return Ok(None);
            }
        }

        let (_owner_module, owner_idx) = match owner {
            Some((module, idx)) => (Some(module), Some(idx)),
            None => (None, None),
        };

        if !self.options.with_tests {
            if let hir::ModuleDef::Function(owned_function) = owned_module_def {
                if util::is_test_function(owned_function, self.db) {
                    return Ok(None);
                }
            }
        }

        let owned_idx = match self.add_node(owned_module_def) {
            Some(owned_idx) => owned_idx,
            None => return Ok(None),
        };

        if let Some(owner_idx) = owner_idx {
            let edge = Edge {
                kind: EdgeKind::Owns,
            };
            self.add_edge(owner_idx, owned_idx, edge);
        }

        if let hir::ModuleDef::Module(owned_module) = owned_module_def {
            trace!(
                "Scanning module {:?}",
                util::path(owned_module_def, self.db)
            );

            let mut local_definitions: HashSet<hir::ModuleDef> = HashSet::new();

            for module_sub_def in owned_module.declarations(self.db) {
                local_definitions.insert(module_sub_def);

                self.process_owned_module_def(Some((owned_module, owned_idx)), module_sub_def)?;
            }

            if self.options.with_orphans && local_definitions.is_empty() {
                add_orphan_nodes_to(&mut self.graph, owned_idx);
            }

            for (_name, scope_def) in owned_module.scope(self.db, None) {
                if let hir::ScopeDef::ModuleDef(scope_module_def) = scope_def {
                    // Skip local child declarations:
                    if local_definitions.contains(&scope_module_def) {
                        continue;
                    }

                    self.process_used_module_def((owned_module, owned_idx), scope_module_def);
                }
            }
        }

        if self.options.with_orphans {
            add_orphan_nodes_to(&mut self.graph, owned_idx);
        }

        Ok(Some(owned_idx))
    }

    fn process_used_module_def(
        &mut self,
        user: (hir::Module, NodeIndex),
        used_module_def: hir::ModuleDef,
    ) -> Option<NodeIndex> {
        if !self.options.with_uses {
            return None;
        }

        let (_user_module, user_idx) = user;

        let mut resolved_module_def = Some(used_module_def);

        if !self.options.with_types {
            // Check if target is a type (i.e. not a module).
            let is_module = matches!(used_module_def, hir::ModuleDef::Module(_));

            // If it is a type we need to resolve to its parent module instead:
            if !is_module {
                let parent_module = used_module_def.module(self.db);
                resolved_module_def = parent_module.map(hir::ModuleDef::Module);
            }
        }

        let module_def_krate = util::krate(used_module_def, self.db);

        // Check if target is from an extern crate.
        // If it is we need to resolve to its parent module instead:
        if module_def_krate != Some(self.krate) {
            resolved_module_def = if self.options.with_externs {
                module_def_krate.map(|krate| hir::ModuleDef::Module(krate.root_module(self.db)))
            } else {
                None
            };
        }

        // Depending on the options we might not want to add the target as is,
        // but resolve it to its module or crate, e.g., so we do that here:
        let resolved_module_def = match resolved_module_def {
            Some(resolved_module_def) => resolved_module_def,
            None => return None,
        };

        self.add_node(resolved_module_def).map(|used_idx| {
            let edge = Edge {
                kind: EdgeKind::Uses,
            };
            self.add_edge(user_idx, used_idx, edge);
            used_idx
        })
    }

    fn add_node(&mut self, module_def: hir::ModuleDef) -> Option<NodeIndex> {
        let node_id = util::path(module_def, self.db);

        trace!("Adding module node: {:?}", node_id);

        // Check if we already added an equivalent node:
        match self.nodes.get(&node_id) {
            Some(node_idx) => {
                // If we did indeed already process it, then retrieve its index:
                Some(*node_idx)
            }
            None => {
                // Otherwise try to add a node:
                let node = match self.node_weight(module_def) {
                    Some(node) => node,
                    None => return None,
                };

                let node_idx = self.graph.add_node(node);
                self.nodes.insert(node_id, node_idx);

                Some(node_idx)
            }
        }
    }

    fn add_edge(&mut self, source_idx: NodeIndex, target_idx: NodeIndex, edge: Edge) -> EdgeIndex {
        let edge_id = (source_idx, edge.kind, target_idx);

        trace!(
            "Adding edge: {:?} --({:?})-> {:?}",
            edge_id.0,
            edge_id.1,
            edge_id.2
        );

        // Check if we already added an equivalent edge:
        match self.edges.get(&edge_id) {
            Some(edge_idx) => {
                // If we did indeed already process it, then retrieve its index:
                *edge_idx
            }
            None => {
                // Otherwise add an edge:
                let edge_idx = self.graph.add_edge(source_idx, target_idx, edge);
                self.edges.insert(edge_id, edge_idx);

                edge_idx
            }
        }
    }

    fn node_weight(&self, module_def: hir::ModuleDef) -> Option<Node> {
        let krate = {
            let krate = util::krate(module_def, self.db);
            krate.map(|krate| util::krate_name(krate, self.db))
        };

        let path: Vec<_> = util::path(module_def, self.db)
            .split("::")
            .map(|s| s.to_owned())
            .collect();

        let file_path = {
            match module_def {
                hir::ModuleDef::Module(module) => Some(module),
                _ => None,
            }
            .and_then(|module| {
                self.module_file(module.definition_source(self.db))
                    .map(Into::into)
            })
        };

        let kind = match NodeKind::from(module_def, self.db) {
            Some(kind) => kind,
            None => return None,
        };

        let visibility = Some(NodeVisibility::new(module_def, self.db));

        let attrs = {
            let cfgs: Vec<_> = self.cfg_attrs(module_def);
            let test = self.test_attr(module_def);
            NodeAttrs { cfgs, test }
        };

        Some(Node {
            krate,
            path,
            file_path,
            kind,
            visibility,
            attrs,
        })
    }

    fn cfg_attrs(&self, module_def: hir::ModuleDef) -> Vec<NodeCfgAttr> {
        util::cfgs(module_def, self.db)
            .into_iter()
            .filter_map(NodeCfgAttr::new)
            .collect()
    }

    fn test_attr(&self, module_def: hir::ModuleDef) -> Option<NodeTestAttr> {
        let function = match module_def {
            hir::ModuleDef::Function(function) => function,
            _ => return None,
        };

        if util::is_test_function(function, self.db) {
            Some(NodeTestAttr)
        } else {
            None
        }
    }

    fn module_file(&self, module_source: hir::InFile<hir::ModuleSource>) -> Option<PathBuf> {
        let is_file_module: bool = match &module_source.value {
            ModuleSource::SourceFile(_) => true,
            ModuleSource::Module(_) => false,
            ModuleSource::BlockExpr(_) => false,
        };

        if !is_file_module {
            return None;
        }

        let file_id = module_source.file_id.original_file(self.db);
        let vfs_path = self.vfs.file_path(file_id);
        let abs_path = vfs_path.as_path().expect("Could not convert to path");

        let path: &Path = abs_path.as_ref();

        let file_extension = path.extension().and_then(|ext| ext.to_str());

        if file_extension != Some("rs") {
            return None;
        }

        Some(path.to_owned())
    }
}
