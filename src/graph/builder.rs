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

use crate::graph::{orphans::add_orphan_nodes_to, Edge, Graph, Node};

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
    graph: Graph,
    nodes: HashMap<String, NodeIndex>,
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
    ) -> anyhow::Result<Option<NodeIndex>> {
        let is_module = matches!(module_def, hir::ModuleDef::Module(_));

        if !self.options.with_types && !is_module {
            return Ok(None);
        }

        let module_def_krate = {
            match module_def {
                hir::ModuleDef::Module(module) => Some(module),
                module_def @ _ => module_def.module(self.db),
            }
            .map(|module| module.krate())
        };

        let is_external = module_def_krate != Some(krate);

        let module_def = if is_external {
            if !self.options.with_externs {
                return Ok(None);
            }

            if let Some(module_def_krate) = module_def_krate {
                hir::ModuleDef::Module(module_def_krate.root_module(self.db))
            } else {
                module_def
            }
        } else {
            module_def
        };

        let path = self.module_path(module_def);

        // Check if we already have any node registered for this path,
        // otherwise add a node:

        let node_idx = match self.nodes.get(&path) {
            Some(node_idx) => *node_idx,
            None => self.add_module_node(module_def, module_def_krate, &path),
        };

        if !recursive {
            return Ok(Some(node_idx));
        }

        if let hir::ModuleDef::Module(module) = module_def {
            trace!("Scanning module {:?}", path);

            for (_name, scope_def) in module.scope(self.db, None) {
                let scope_module_def = if let hir::ScopeDef::ModuleDef(scope_module_def) = scope_def
                {
                    scope_module_def
                } else {
                    continue;
                };

                let is_module = matches!(scope_module_def, hir::ModuleDef::Module(_));

                if !self.options.with_types && !is_module {
                    continue;
                }

                let is_local = Some(module) == scope_module_def.module(self.db);

                if !self.options.with_uses && !is_local {
                    continue;
                }

                let scope_node_idx = self.add_module_def(scope_module_def, is_local, krate)?;

                if let Some(scope_node_idx) = scope_node_idx {
                    if self.graph.contains_edge(node_idx, scope_node_idx) {
                        // Avoid adding redundant edges:
                        continue;
                    }

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
            None => crate_name,
        }
    }

    fn add_module_node(
        &mut self,
        module_def: hir::ModuleDef,
        krate: Option<hir::Crate>,
    ) -> NodeIndex {
        let module_path = self.module_path(module_def);

        trace!("Adding module node: {:?}", module_path);

        let node = self.make_node(module_def, krate, &module_path);

        let node_idx = self.graph.add_node(node);
        self.nodes.insert(module_path, node_idx);

        node_idx
    }

    fn add_edge(&mut self, source_idx: NodeIndex, target_idx: NodeIndex, edge: Edge) -> EdgeIndex {
        trace!("Adding edge: {:?} -> {:?}", source_idx, target_idx);

        self.graph.add_edge(source_idx, target_idx, edge)
    }

    fn make_node(&self, hir: hir::ModuleDef, krate: Option<hir::Crate>, module_path: &str) -> Node {
        let path = module_path.to_owned();
        let file_path = {
            match hir {
                hir::ModuleDef::Module(module) => Some(module),
                _ => None,
            }
            .and_then(|module| {
                self.module_file(module.definition_source(self.db))
                    .map(Into::into)
            })
        };
        let hir = Some(hir);

        Node {
            path,
            file_path,
            hir,
            krate,
        }
    }
}
