use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use hir::Crate;
use log::trace;
use petgraph::graph::{EdgeIndex, NodeIndex};
use ra_ap_hir::{self as hir, ModuleSource};
use ra_ap_ide_db::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::{
    graph::{orphans::add_orphan_nodes_to, Edge, Graph, Node},
    options::graph::Options,
};

#[derive(Debug)]
pub struct Builder<'a> {
    options: Options,
    db: &'a RootDatabase,
    vfs: &'a Vfs,
    graph: Graph,
    nodes: HashMap<String, NodeIndex<usize>>,
}

impl<'a> Builder<'a> {
    pub fn new(options: Options, db: &'a RootDatabase, vfs: &'a Vfs) -> Self {
        let graph = Graph::default();
        let nodes = HashMap::default();
        Self {
            options,
            db,
            vfs,
            graph,
            nodes,
        }
    }

    pub fn build(mut self, krate: Crate) -> anyhow::Result<Graph> {
        let root_module = krate.root_module(self.db);
        let root_module_def = hir::ModuleDef::Module(root_module);

        trace!("Scanning project ...");

        self.add_module_def(root_module_def, true, krate)?;

        Ok(self.graph)
    }

    fn add_module_def(
        &mut self,
        module_def: hir::ModuleDef,
        recursive: bool,
        krate: Crate,
    ) -> anyhow::Result<Option<NodeIndex<usize>>> {
        let path = self.module_path(module_def);

        trace!("Scanning module {:?}", path);

        let module = if let hir::ModuleDef::Module(module) = module_def {
            Some(module)
        } else {
            None
        };

        if !self.options.with_types && module.is_none() {
            return Ok(None);
        }

        // Check if already we have any node registered for this path, obtaining its index:
        let node_idx = match self.nodes.get(&path) {
            Some(node_idx) => *node_idx,
            None => {
                let is_external = module_def
                    .module(self.db)
                    .map_or(false, |module| module.krate() != krate);
                self.add_module_node(module_def, &path, is_external)
            }
        };

        if !recursive {
            return Ok(Some(node_idx));
        }

        if let Some(module) = module {
            for (_name, scope_def) in module.scope(self.db, None) {
                let scope_module_def = if let hir::ScopeDef::ModuleDef(scope_module_def) = scope_def
                {
                    scope_module_def
                } else {
                    continue;
                };

                let is_module = match scope_module_def {
                    hir::ModuleDef::Module(_) => true,
                    _ => false,
                };

                if !self.options.with_types && !is_module {
                    continue;
                }

                let is_local = Some(module) == scope_module_def.module(self.db);

                if !self.options.with_uses && !is_local {
                    continue;
                }

                let scope_node_idx = self.add_module_def(scope_module_def, is_local, krate)?;

                if let Some(scope_node_idx) = scope_node_idx {
                    let edge = if is_local { Edge::HasA } else { Edge::UsesA };
                    self.add_edge(node_idx, scope_node_idx, edge);
                }
            }

            if self.options.with_orphans {
                add_orphan_nodes_to(&mut self.graph, node_idx);
            }
        }

        Ok(Some(node_idx))
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

    fn crate_name(&self, krate: hir::Crate) -> String {
        // Obtain the crate's declaration name:
        let display_name = &krate.display_name(self.db).unwrap();

        // Since a crate's name may contain `-` we canonicalize it by replacing with `_`:
        display_name.replace("-", "_")
    }

    fn module(&self, module_def: hir::ModuleDef) -> hir::Module {
        match module_def {
            hir::ModuleDef::Module(module) => module,
            _ => module_def.module(self.db).unwrap(),
        }
    }

    fn module_path(&self, module_def: hir::ModuleDef) -> String {
        let module = self.module(module_def);

        // Obtain the module's canonicalized name:
        let crate_name = self.crate_name(module.krate());

        let relative_canonical_path = module_def.canonical_path(self.db);

        match relative_canonical_path {
            Some(canonical_path) => format!("{}::{}", crate_name, canonical_path),
            None => crate_name.to_owned(),
        }
    }

    fn add_module_node(
        &mut self,
        module_def: hir::ModuleDef,
        module_path: &str,
        is_external: bool,
    ) -> NodeIndex<usize> {
        trace!("Adding module node: {:?}", module_path);

        let module_path = self.module_path(module_def);

        let node = self.make_node(module_def, &module_path, is_external);

        let node_idx = self.graph.add_node(node);
        self.nodes.insert(module_path, node_idx);

        node_idx
    }

    fn add_edge(
        &mut self,
        source_idx: NodeIndex<usize>,
        target_idx: NodeIndex<usize>,
        edge: Edge,
    ) -> EdgeIndex<usize> {
        trace!("Adding edge: {:?} -> {:?}", source_idx, target_idx);

        self.graph.add_edge(source_idx, target_idx, edge)
    }

    fn make_node(&self, module_def: hir::ModuleDef, module_path: &str, is_external: bool) -> Node {
        let path = module_path.to_owned();
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
            is_external,
        }
    }
}
