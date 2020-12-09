use std::{collections::HashMap, ffi, fs, path::Path};

use log::{debug, trace};
use petgraph::graph::NodeIndex;
use ra_ap_hir::{self as hir, ModuleSource};
use ra_ap_ide_db::RootDatabase;
use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, TargetKind};
use ra_ap_vfs::{AbsPathBuf, Vfs};

use crate::{
    graph::{Edge, EdgeKind, Graph, ModuleNode, Node, NodeKind},
    visitor::hir::Visitor as HirVisitor,
};

#[derive(Default, Debug)]
pub struct Options {
    orphans: bool,
}

impl Options {
    pub fn new(orphans: bool) -> Self {
        Self { orphans }
    }
}

#[derive(Debug)]
pub struct GraphBuilder<'a> {
    db: &'a RootDatabase,
    vfs: Vfs,
    graph: Graph,
    modules: HashMap<String, NodeIndex<usize>>,
    options: Options,
}

impl<'a> GraphBuilder<'a> {
    pub fn new(db: &'a RootDatabase, vfs: Vfs, options: Options) -> Self {
        let graph = Graph::default();
        let modules = HashMap::default();
        Self {
            db,
            vfs,
            graph,
            modules,
            options,
        }
    }

    pub fn build(mut self, project_dir: &Path) -> anyhow::Result<Graph> {
        let root_path = AbsPathBuf::assert(std::env::current_dir()?.join(project_dir));
        let root = ProjectManifest::discover_single(&root_path)?;
        let load_out_dirs_from_check = true;
        let project_workspace = ProjectWorkspace::load(
            root,
            &CargoConfig {
                load_out_dirs_from_check,
                ..Default::default()
            },
            false,
        )?;

        let cargo_workspace = match project_workspace {
            ProjectWorkspace::Cargo { cargo, .. } => cargo,
            ProjectWorkspace::Json { .. } => {
                unreachable!();
            }
        };

        let member_packages: Vec<_> = cargo_workspace
            .packages()
            .filter_map(|idx| {
                let package = cargo_workspace[idx].clone();
                if package.is_member {
                    Some(package)
                } else {
                    None
                }
            })
            .collect();

        let target = member_packages
            .into_iter()
            .find_map(|package| {
                for idx in package.targets.into_iter() {
                    let target = &cargo_workspace[idx];
                    if target.kind == TargetKind::Lib {
                        return Some(target.clone());
                    }
                }
                None
            })
            .unwrap();

        let target_root_path = target.root.as_path();

        for krate in hir::Crate::all(self.db) {
            let crate_name: &str = &krate.declaration_name(self.db).unwrap();

            trace!("Crate: {:?}", crate_name);

            let vfs_path = self.vfs.file_path(krate.root_file(self.db));
            let crate_root_path = vfs_path.as_path().unwrap();

            if crate_root_path != target_root_path {
                continue;
            }

            let root_module = krate.root_module(self.db);
            let root_module_def = hir::ModuleDef::Module(root_module);

            self.walk(&root_module_def, self.db);
        }

        Ok(self.graph)
    }

    fn file_name(path: &Path) -> Option<String> {
        path.file_stem()
            .and_then(ffi::OsStr::to_str)
            .map(ToString::to_string)
    }

    fn is_unnamed_mod(path: &Path) -> Option<bool> {
        let is_file = path.is_file();
        let file_name = path.file_stem().and_then(|s| s.to_str());
        let file_extension = path.extension().and_then(|s| s.to_str());

        if !is_file {
            return Some(path.join("mod.rs").exists());
        }

        if file_extension != Some("rs") {
            return None;
        }

        match file_name {
            Some("mod") | Some("lib") | Some("main") => Some(true),
            Some(_) => Some(false),
            None => unreachable!(),
        }
    }

    fn mod_dir(&self, module_source: hir::InFile<hir::ModuleSource>) -> Option<AbsPathBuf> {
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

        let name_and_extension = vfs_path.name_and_extension().unwrap();

        match name_and_extension {
            ("lib", Some("rs")) | ("main", Some("rs")) | ("mod", Some("rs")) => {
                abs_path.parent().map(|p| p.to_path_buf())
            }
            (file_name, Some("rs")) => abs_path.parent().map(|p| p.join(file_name)),
            _ => unreachable!(),
        }
    }

    fn crate_name(&self, krate: hir::Crate) -> String {
        // Obtain the crate's declaration name:

        let declaration_name = &krate.declaration_name(self.db).unwrap();

        // Since a crate's name may contain `-` we canonicalize it by replacing with `_`:

        declaration_name.replace("-", "_")
    }

    fn absolute_module_path(&self, crate_name: &str, module_def: hir::ModuleDef) -> String {
        let relative_canonical_path = module_def.canonical_path(self.db);

        match relative_canonical_path {
            Some(canonical_path) => format!("{}::{}", crate_name, canonical_path),
            None => crate_name.to_owned(),
        }
    }

    fn orphan_names(&mut self, module: hir::Module) -> anyhow::Result<Vec<String>> {
        // If the module is the root module of a file we need to scan its
        // corresponding module directory for potentially orphaned source files:

        match &module.definition_source(self.db).value {
            ModuleSource::SourceFile(_) => {}
            ModuleSource::Module(_) => return Ok(vec![]),
        };

        let module_source = module.definition_source(self.db);

        let module_dir_path_buf = match self.mod_dir(module_source) {
            Some(path_buf) => path_buf,
            None => return Ok(vec![]),
        };

        debug!("Scanning module dir for orphans: {:?}", module_dir_path_buf);

        let module_dir_path = module_dir_path_buf.as_path();

        if !module_dir_path.exists() {
            return Ok(vec![]);
        }

        let mut orphan_names = vec![];

        let dir_entries = fs::read_dir(module_dir_path)?.filter_map(Result::ok);

        for dir_entry in dir_entries {
            let path_buf = dir_entry.path();
            let path = path_buf.as_path();

            if Self::is_unnamed_mod(path) != Some(false) {
                continue;
            }

            if let Some(orphan_name) = Self::file_name(path) {
                orphan_names.push(orphan_name);
            }
        }

        return Ok(orphan_names);
    }

    fn add_orphan_nodes(
        &mut self,
        module: hir::Module,
        module_path: &str,
        module_idx: NodeIndex<usize>,
    ) {
        // If the module is the root module of a file we need to scan its
        // corresponding module directory for potentially orphaned source files:

        let orphan_names = match self.orphan_names(module) {
            Ok(orphan_names) => orphan_names,
            Err(_) => return,
        };

        // Add the found orphans to the graph with an edge from the current module's node,
        // potentially replacing their nodes them with their actual modules later on:

        for orphan_name in orphan_names {
            self.add_orphan_node(&module_path, module_idx, &orphan_name);
        }
    }

    fn add_orphan_node(
        &mut self,
        module_path: &str,
        module_idx: NodeIndex<usize>,
        orphan_name: &str,
    ) {
        let orphan_path = format!("{}::{}", module_path, orphan_name);

        if self.modules.contains_key(&orphan_path) {
            return;
        }

        debug!("Orphan module: {:?}", orphan_path);

        // Collect all necessary information about the
        // orphan and construct a node with it:

        let orphan_node = Node {
            name: orphan_name.to_owned(),
            path: orphan_path.clone(),
            kind: NodeKind::Orphan,
        };

        debug!("Adding orphan node: {:?}", orphan_path);

        // Add a node for the orphan:

        let orphan_idx = self.graph.add_node(orphan_node);
        self.modules.insert(orphan_path, orphan_idx);

        // Connect the orphan node to the current module's node:

        let kind = EdgeKind::HasA;
        let orphan_edge = Edge { kind };
        self.graph.add_edge(module_idx, orphan_idx, orphan_edge);
    }

    fn add_module_node(
        &mut self,
        module_path: &str,
        node: Node,
        parent_module_idx: Option<NodeIndex<usize>>,
    ) -> NodeIndex<usize> {
        debug!("Adding node: {:?}", module_path);

        let module_idx = self.graph.add_node(node);
        self.modules.insert(module_path.to_owned(), module_idx);

        // If it's a sub-module, then add an edge to its parent:

        if let Some(parent_module_idx) = parent_module_idx {
            let kind = EdgeKind::HasA;
            let edge = Edge { kind };
            let _edge_idx = self.graph.add_edge(parent_module_idx, module_idx, edge);
        }

        module_idx
    }

    fn make_node(
        &self,
        module: hir::Module,
        module_name: &str,
        module_path: &str,
        parent_module: Option<hir::Module>,
    ) -> Node {
        let module_def = hir::ModuleDef::Module(module);

        let module_is_root = parent_module.is_none();
        let visibility = module_def.definition_visibility(self.db);

        let kind = NodeKind::Module(ModuleNode {
            visibility,
            def: module_def,
            is_root: module_is_root,
        });

        Node {
            name: module_name.to_owned(),
            path: module_path.to_owned(),
            kind,
        }
    }

    fn module_idx(&self, module: hir::Module, crate_name: &str) -> Option<NodeIndex<usize>> {
        let module_def = hir::ModuleDef::Module(module);
        let module_path = self.absolute_module_path(crate_name, module_def);
        self.modules.get(&module_path).cloned()
    }
}

impl<'a> HirVisitor for GraphBuilder<'a> {
    fn visit_module(&mut self, module: &hir::Module) {
        let module = *module;
        let module_def = hir::ModuleDef::Module(module);

        // Obtain the module's crate and its canonicalized name:

        let krate = module.krate();
        let crate_name = self.crate_name(krate);

        // Obtain the module's name and canonicalized absolute path:

        let module_name = module
            .name(self.db)
            .map(|it| it.to_string())
            .unwrap_or_else(|| crate_name.clone());
        let module_path = self.absolute_module_path(&crate_name, module_def);

        debug!("Visit module: {:?}", module_path);

        // Check if already we have any node registered for this path, obtaining its index:

        if let Some(module_idx) = self.modules.get(&module_path).cloned() {
            // A module should only ever be visited once, so if we find a module node,
            // then it has to be an orphan node that was added while visiting the parent module,
            // otherwise we skip it:

            if !self.graph[module_idx].is_orphan() {
                return;
            }

            // Remove the existing orphan node as it will be
            // replaced with the actual module's node:

            self.graph.remove_node(module_idx);
            self.modules.remove(&module_path);
        }

        // In order to add the module to our graph we need to know
        //  the index of the node corresponding to its parent module:

        let parent_module = module.parent(self.db);
        let parent_module_idx = parent_module.and_then(|m| self.module_idx(m, &crate_name));

        // Collect all necessary information about the module and construct a node with it:

        let node = self.make_node(module, &module_name, &module_path, parent_module);

        // Add the node to the graph and register its node index with its path:

        let module_idx = self.add_module_node(&module_path, node, parent_module_idx);

        if self.options.orphans {
            // If the module is the root module of a file we need to scan its
            // corresponding module directory for potentially orphaned source files:

            self.add_orphan_nodes(module, &module_path, module_idx);
        }
    }
}
