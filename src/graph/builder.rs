use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use hir::Crate;
use log::trace;
use petgraph::graph::{EdgeIndex, NodeIndex};
use ra_ap_hir::{self as hir, ModuleSource};
use ra_ap_ide_db::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::graph::{orphans::add_orphan_nodes_to, util, Edge, EdgeKind, Graph, Node};

#[derive(Clone, PartialEq, Debug)]
pub struct Options {
    pub focus_on: Option<String>,
    pub max_depth: Option<usize>,
    pub with_types: bool,
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

        let owned_idx = self.add_node(owned_module_def);

        if let Some(owner_idx) = owner_idx {
            self.add_edge(owner_idx, owned_idx, Edge::Owns);
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

            for (_name, scope_def) in owned_module.scope(self.db, None) {
                if let hir::ScopeDef::ModuleDef(scope_module_def) = scope_def {
                    // Skip local child declarations:
                    if local_definitions.contains(&scope_module_def) {
                        continue;
                    }

                    self.process_used_module_def((owned_module, owned_idx), scope_module_def)?;
                }
            }

            if self.options.with_orphans {
                add_orphan_nodes_to(&mut self.graph, owned_idx);
            }
        }

        Ok(Some(owned_idx))
    }

    fn process_used_module_def(
        &mut self,
        user: (hir::Module, NodeIndex),
        used_module_def: hir::ModuleDef,
    ) -> anyhow::Result<Option<NodeIndex>> {
        if !self.options.with_uses {
            return Ok(None);
        }

        let (_user_module, user_idx) = user;

        let mut resolved_module_def = Some(used_module_def);

        if !self.options.with_types {
            // Check if target is a type (i.e. not a module).
            let is_module = matches!(used_module_def, hir::ModuleDef::Module(_));

            // If it is a type we need to resolve to its parent module instead:
            if !is_module {
                let parent_module = used_module_def.module(self.db);
                resolved_module_def = parent_module.map(|m| hir::ModuleDef::Module(m));
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
            None => return Ok(None),
        };

        let used_idx = self.add_node(resolved_module_def);

        self.add_edge(user_idx, used_idx, Edge::Uses);

        Ok(Some(used_idx))
    }

    fn add_node(&mut self, module_def: hir::ModuleDef) -> NodeIndex {
        let node_id = util::path(module_def, self.db);

        trace!("Adding module node: {:?}", node_id);

        // Check if we already added an equivalent node:
        match self.nodes.get(&node_id) {
            Some(node_idx) => {
                // If we did indeed already process it, then retrieve its index:
                *node_idx
            }
            None => {
                // Otherwise add a node:
                let node_idx = self.graph.add_node(self.node_weight(module_def));
                self.nodes.insert(node_id, node_idx);

                node_idx
            }
        }
    }

    fn add_edge(&mut self, source_idx: NodeIndex, target_idx: NodeIndex, edge: Edge) -> EdgeIndex {
        let edge_id = (source_idx, edge.kind(), target_idx);

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

    fn node_weight(&self, module_def: hir::ModuleDef) -> Node {
        let path = util::path(module_def, self.db);
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

        let hir = Some(module_def);

        Node {
            path,
            file_path,
            hir,
        }
    }

    fn module_file(&self, module_source: hir::InFile<hir::ModuleSource>) -> Option<PathBuf> {
        let is_file_module: bool = match &module_source.value {
            ModuleSource::SourceFile(_) => true,
            ModuleSource::Module(_) => false,
        };

        if !is_file_module {
            return None;
        }

        let file_id = module_source.file_id.original_file(self.db);
        let vfs_path = self.vfs.file_path(file_id);
        let abs_path = vfs_path.as_path().expect("Could not convert to path");

        let path: &Path = &*abs_path;

        let file_extension = path.extension().and_then(|ext| ext.to_str());

        if file_extension != Some("rs") {
            return None;
        }

        Some(path.to_owned())
    }
}
