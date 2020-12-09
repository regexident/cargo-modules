use std::{collections::HashMap, path::Path};

use petgraph::graph::NodeIndex;
use ra_ap_hir as hir;
use ra_ap_ide_db::RootDatabase;
use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, TargetKind};
use ra_ap_vfs::{AbsPathBuf, Vfs};

use crate::{
    graph::{Edge, EdgeKind, Graph, Node},
    visitor::Visitor,
};

#[derive(Default, Debug)]
struct Modules {
    map: HashMap<hir::Module, NodeIndex<usize>>,
}

#[derive(Debug)]
pub struct GraphBuilder<'a> {
    db: &'a RootDatabase,
    vfs: Vfs,
    graph: Graph,
    modules: Modules,
}

impl<'a> GraphBuilder<'a> {
    pub fn new(db: &'a RootDatabase, vfs: Vfs) -> Self {
        let graph = Graph::default();
        let modules = Modules::default();
        Self {
            db,
            vfs,
            graph,
            modules,
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
                panic!();
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
            // let krate_name: &str = &krate.declaration_name(self.db).unwrap();
            // println!("Crate: {:?}", krate_name);

            let vfs_path = self.vfs.file_path(krate.root_file(self.db));

            let krate_root_path = vfs_path.as_path().unwrap();

            // println!("{:?} vs. {:?}", krate_root_path, target_root_path);

            if krate_root_path != target_root_path {
                continue;
            }

            let root_module = krate.root_module(self.db);
            let root_module_def = hir::ModuleDef::Module(root_module);

            self.walk(&root_module_def, self.db);

            // println!();
        }

        Ok(self.graph)
    }
}

impl<'a> Visitor for GraphBuilder<'a> {
    fn visit_module(&mut self, module: &hir::Module) {
        let krate = module.krate();
        let krate_name: &str = &krate.declaration_name(self.db).unwrap();
        let canonical_krate_name = krate_name.replace("-", "_");

        let module = *module;

        if let Some(_module_idx) = self.modules.map.get(&module) {
            // let module_name = module.name(self.db);
            // println!("Skipping module {:?}", module_name);
            return;
        }

        let module_def = hir::ModuleDef::Module(module);
        let relative_canonical_path = module_def.canonical_path(self.db);
        let absolute_canonical_path = match relative_canonical_path {
            Some(canonical_path) => format!("{}::{}", canonical_krate_name, canonical_path),
            None => canonical_krate_name.clone(),
        };

        let canonical_name = module
            .name(self.db)
            .map(|it| it.to_string())
            .unwrap_or(canonical_krate_name);

        // println!("Module: {:?}", absolute_canonical_path);

        let parent_module = module.parent(self.db);
        let parent_module_idx = parent_module.and_then(|m| self.modules.map.get(&m).cloned());

        let module_is_root = parent_module.is_none();
        let visibility = module_def.definition_visibility(self.db);

        let node = Node {
            visibility,
            name: canonical_name,
            path: absolute_canonical_path,
            is_root: module_is_root,
            def: module_def,
        };

        let module_idx = self.graph.add_node(node);
        self.modules.map.insert(module, module_idx);

        if let Some(parent_module_idx) = parent_module_idx {
            let kind = EdgeKind::HasA;
            let edge = Edge { kind };
            self.graph.add_edge(parent_module_idx, module_idx, edge);
        }
    }
}
