// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::HashSet, path::Path};

use log::{debug, trace};
use petgraph::{graph::NodeIndex, stable_graph::StableGraph};
use ra_ap_hir::{self, Crate};
use ra_ap_ide::{AnalysisHost, RootDatabase};
use ra_ap_ide_db::FxHashMap;
use ra_ap_paths::AbsPathBuf;
use ra_ap_proc_macro_api::ProcMacroServer;
use ra_ap_project_model::{
    CargoConfig, CargoFeatures, InvocationLocation, InvocationStrategy, PackageData,
    ProjectManifest, ProjectWorkspace, RustcSource, TargetData, UnsetTestCrates,
};
use ra_ap_rust_analyzer::cli::load_cargo::{load_workspace, LoadCargoConfig};
use ra_ap_syntax::{ast, AstNode, SourceFile};
use ra_ap_vfs::Vfs;
use structopt::StructOpt;

use crate::{
    graph::{
        builder::{Builder as GraphBuilder, Options as GraphBuilderOptions},
        edge::Edge,
        node::Node,
        util::{self, idx_of_node_with_path},
    },
    options::{
        general::Options as GeneralOptions, graph::Options as GraphOptions,
        project::Options as ProjectOptions,
    },
    target::{select_package, select_target},
};

pub mod graph;
pub mod tree;

#[derive(StructOpt, Clone, PartialEq, Eq, Debug)]
pub enum Command {
    #[structopt(name = "tree", about = "Print crate as a tree.")]
    Tree(tree::Options),

    #[structopt(
        name = "graph",
        about = "Print crate as a graph.",
        after_help = r#"
        If you have xdot installed on your system, you can run this using:
        `cargo modules generate dependencies | xdot -`
        "#
    )]
    Graph(graph::Options),
}

impl Command {
    pub(crate) fn sanitize(&mut self) {
        if self.graph_options().with_tests && !self.project_options().cfg_test {
            debug!("Enabling `--cfg-test`, which is implied by `--with-tests`");
            self.project_options_mut().cfg_test = true;
        }
    }

    pub fn run(&self) -> Result<(), anyhow::Error> {
        let general_options = self.general_options();
        let project_options = self.project_options();
        let graph_options = self.graph_options();

        let project_path = project_options.manifest_path.as_path().canonicalize()?;
        let cargo_config = self.cargo_config(project_options);
        let load_config = self.load_config();

        let progress = |string| {
            trace!("Progress: {}", string);
        };

        let project_workspace =
            self.load_project_workspace(&project_path, &cargo_config, &progress)?;

        let (package, target) = self.select_target(&project_workspace, project_options)?;

        if general_options.verbose {
            eprintln!();
            eprintln!("crate");
            eprintln!("└── package: {}", package.name);
            eprintln!("    └── target: {}", target.name);
            eprintln!();
        }

        let (host, vfs, _proc_macro_client) = self.analyze_project_workspace(
            project_workspace,
            &cargo_config,
            &load_config,
            &progress,
        )?;
        let db = host.raw_database();

        let krate = self.find_crate(db, &vfs, &target)?;

        let (graph, start_node_idxs) = self.build_graph(db, &vfs, krate, graph_options)?;

        trace!("Generating ...");

        match self {
            #[allow(unused_variables)]
            Self::Tree(options) => {
                for start_node_idx in start_node_idxs {
                    let command = tree::Command::new(options.clone());
                    command.run(&graph, start_node_idx, krate, db)?;
                }
                Ok(())
            }
            #[allow(unused_variables)]
            Self::Graph(options) => {
                for start_node_idx in start_node_idxs {
                    let command = graph::Command::new(options.clone());
                    command.run(&graph, start_node_idx, krate, db)?;
                }
                Ok(())
            }
        }
    }

    fn cargo_config(&self, project_options: &ProjectOptions) -> CargoConfig {
        // List of features to activate (or deactivate).
        let features = if project_options.all_features {
            CargoFeatures::All
        } else {
            CargoFeatures::Selected {
                features: project_options.features.clone(),
                no_default_features: project_options.no_default_features,
            }
        };

        // Target triple
        let target = project_options.target.clone();

        // Whether to load sysroot crates (`std`, `core` & friends).
        let sysroot = if project_options.with_sysroot && self.with_sysroot() {
            Some(RustcSource::Discover)
        } else {
            None
        };

        // rustc private crate source
        let rustc_source = None;

        // crates to disable `#[cfg(test)]` on
        let unset_test_crates = match project_options.cfg_test {
            true => UnsetTestCrates::None,
            false => UnsetTestCrates::All,
        };

        // Setup RUSTC_WRAPPER to point to `rust-analyzer` binary itself.
        // (We use that to compile only proc macros and build scripts
        // during the initial `cargo check`.)
        let wrap_rustc_in_build_scripts = true;

        let run_build_script_command = None;

        // FIXME: support extra environment variables via CLI:
        let extra_env = FxHashMap::default();

        let invocation_strategy = InvocationStrategy::PerWorkspace;
        let invocation_location = InvocationLocation::Workspace;

        CargoConfig {
            features,
            target,
            sysroot,
            rustc_source,
            unset_test_crates,
            wrap_rustc_in_build_scripts,
            run_build_script_command,
            extra_env,
            invocation_strategy,
            invocation_location,
        }
    }

    fn load_config(&self) -> LoadCargoConfig {
        LoadCargoConfig {
            load_out_dirs_from_check: true,
            with_proc_macro: false,
            prefill_caches: false,
        }
    }

    fn load_project_workspace(
        &self,
        project_path: &Path,
        cargo_config: &CargoConfig,
        progress: &dyn Fn(String),
    ) -> anyhow::Result<ProjectWorkspace> {
        let root = AbsPathBuf::assert(std::env::current_dir()?.join(project_path));
        let root = ProjectManifest::discover_single(&root)?;

        ProjectWorkspace::load(root, cargo_config, &progress)
    }

    fn select_target(
        &self,
        project_workspace: &ProjectWorkspace,
        options: &ProjectOptions,
    ) -> anyhow::Result<(PackageData, TargetData)> {
        let cargo_workspace = match project_workspace {
            ProjectWorkspace::Cargo { cargo, .. } => Ok(cargo),
            ProjectWorkspace::Json { .. } => Err(anyhow::anyhow!("Unexpected JSON workspace")),
            ProjectWorkspace::DetachedFiles { .. } => {
                Err(anyhow::anyhow!("Unexpected detached files"))
            }
        }?;

        let package_idx = select_package(cargo_workspace, options)?;
        let package = cargo_workspace[package_idx].clone();
        debug!("Selected package: {:#?}", package.name);

        let target_idx = select_target(cargo_workspace, package_idx, options)?;
        let target = cargo_workspace[target_idx].clone();
        debug!("Selected target: {:#?}", target.name);

        Ok((package, target))
    }

    fn analyze_project_workspace(
        &self,
        mut workspace: ProjectWorkspace,
        cargo_config: &CargoConfig,
        load_config: &LoadCargoConfig,
        progress: &dyn Fn(String),
    ) -> anyhow::Result<(AnalysisHost, Vfs, Option<ProcMacroServer>)> {
        if load_config.load_out_dirs_from_check {
            let build_scripts = workspace.run_build_scripts(cargo_config, progress)?;
            workspace.set_build_scripts(build_scripts)
        }

        load_workspace(workspace, &cargo_config.extra_env, load_config)
    }

    fn find_crate(
        &self,
        db: &RootDatabase,
        vfs: &Vfs,
        target: &TargetData,
    ) -> anyhow::Result<Crate> {
        let crates = Crate::all(db);

        let target_root_path = target.root.as_path();

        let krate = crates.into_iter().find(|krate| {
            let vfs_path = vfs.file_path(krate.root_file(db));
            let crate_root_path = vfs_path.as_path().unwrap();

            crate_root_path == target_root_path
        });

        krate.ok_or_else(|| anyhow::anyhow!("Crate not found"))
    }

    fn build_graph(
        &self,
        db: &RootDatabase,
        vfs: &Vfs,
        krate: Crate,
        options: &GraphOptions,
    ) -> anyhow::Result<(StableGraph<Node, Edge>, HashSet<NodeIndex>)> {
        let graph_builder = {
            let builder_options = self.builder_options();
            GraphBuilder::new(builder_options, db, vfs, krate)
        };

        let crate_name = krate.display_name(db).unwrap().to_string();

        let focus_on = match &options.focus_on {
            Some(string) => string.to_owned(),
            None => crate_name.clone(),
        };

        let syntax = format!("use {};", focus_on);
        let use_tree: ast::UseTree = Self::parse_ast(&syntax);

        trace!("Constructing graph ...");

        let mut graph = graph_builder.build(krate)?;

        trace!("Searching for focus nodes in graph ...");

        let focus_node_idxs: Vec<NodeIndex> = graph
            .node_indices()
            .filter(|node_idx| {
                let node = &graph[*node_idx];
                let node_path: ast::Path = {
                    let syntax = node.path.join("::");
                    Self::parse_ast(&syntax)
                };
                Self::use_tree_matches_path(&use_tree, &node_path)
            })
            .collect();

        if focus_node_idxs.is_empty() {
            anyhow::bail!("No node found matching use tree '{:?}'", focus_on);
        }

        let crate_path = vec![crate_name];
        let crate_node_idx = idx_of_node_with_path(&graph, &crate_path[..], db).unwrap();
        let crate_node_idxs: HashSet<_> = HashSet::from_iter([crate_node_idx]);

        let max_depth = options.max_depth.unwrap_or(usize::MAX);
        util::shrink_graph(&mut graph, focus_node_idxs.iter(), max_depth);

        Ok((graph, crate_node_idxs))
    }

    // https://github.com/rust-lang/rust-analyzer/blob/36a70b7435c48837018c71576d7bb4e8f763f501/crates/syntax/src/ast/make.rs#L821
    fn parse_ast<N: AstNode>(text: &str) -> N {
        let parse = SourceFile::parse(text);
        let node = match parse.tree().syntax().descendants().find_map(N::cast) {
            Some(it) => it,
            None => {
                let node = std::any::type_name::<N>();
                panic!("Failed to make ast node `{node}` from text {text}")
            }
        };
        let node = node.clone_subtree();
        assert_eq!(node.syntax().text_range().start(), 0.into());
        node
    }

    fn use_tree_matches_path(use_tree: &ast::UseTree, path: &ast::Path) -> bool {
        let mut path_segments_iter = path.segments();

        if let Some(use_tree_path) = use_tree.path() {
            for use_tree_segment in use_tree_path.segments() {
                match path_segments_iter.next() {
                    Some(path_segment) => {
                        if use_tree_segment.syntax().text() == path_segment.syntax().text() {
                            continue;
                        } else {
                            return false;
                        }
                    }
                    None => {
                        return false;
                    }
                }
            }
        }

        let path_segments: Vec<_> = path_segments_iter.collect();

        if path_segments.is_empty() {
            return use_tree.is_simple_path() || Self::tree_contains_self(use_tree);
        }

        if use_tree.star_token().is_some() {
            return path_segments.len() == 1;
        }

        let path_suffix = ast::make::path_from_segments(path_segments, false);

        use_tree
            .use_tree_list()
            .into_iter()
            .flat_map(|list| list.use_trees())
            .any(|use_tree| Self::use_tree_matches_path(&use_tree, &path_suffix))
    }

    fn path_is_self(path: &ast::Path) -> bool {
        path.segment().and_then(|seg| seg.self_token()).is_some() && path.qualifier().is_none()
    }

    fn tree_is_self(tree: &ast::UseTree) -> bool {
        tree.path()
            .as_ref()
            .map(Self::path_is_self)
            .unwrap_or(false)
    }

    fn tree_contains_self(tree: &ast::UseTree) -> bool {
        tree.use_tree_list()
            .map(|tree_list| tree_list.use_trees().any(|tree| Self::tree_is_self(&tree)))
            .unwrap_or(false)
    }

    fn with_sysroot(&self) -> bool {
        match self {
            Self::Tree(_) => false,
            Self::Graph(options) => {
                // We only need to include sysroot if we include extern uses
                // and didn't explicitly request sysroot to be excluded:
                options.with_uses && options.with_externs
            }
        }
    }

    fn general_options(&self) -> &GeneralOptions {
        match self {
            Self::Tree(options) => &options.general,
            Self::Graph(options) => &options.general,
        }
    }

    #[allow(dead_code)]
    fn general_options_mut(&mut self) -> &mut GeneralOptions {
        match self {
            Self::Tree(options) => &mut options.general,
            Self::Graph(options) => &mut options.general,
        }
    }

    fn project_options(&self) -> &ProjectOptions {
        match self {
            Self::Tree(options) => &options.project,
            Self::Graph(options) => &options.project,
        }
    }

    #[allow(dead_code)]
    fn project_options_mut(&mut self) -> &mut ProjectOptions {
        match self {
            Self::Tree(options) => &mut options.project,
            Self::Graph(options) => &mut options.project,
        }
    }

    fn graph_options(&self) -> &GraphOptions {
        match self {
            Self::Tree(options) => &options.graph,
            Self::Graph(options) => &options.graph,
        }
    }

    #[allow(dead_code)]
    fn graph_options_mut(&mut self) -> &mut GraphOptions {
        match self {
            Self::Tree(options) => &mut options.graph,
            Self::Graph(options) => &mut options.graph,
        }
    }

    fn builder_options(&self) -> GraphBuilderOptions {
        match self {
            Self::Tree(options) => GraphBuilderOptions {
                focus_on: options.graph.focus_on.clone(),
                max_depth: options.graph.max_depth,
                with_types: options.graph.with_types,
                with_fns: options.graph.with_fns,
                with_tests: options.graph.with_tests,
                with_orphans: options.graph.with_orphans,
                with_uses: false,
                with_externs: false,
            },
            Self::Graph(options) => GraphBuilderOptions {
                focus_on: options.graph.focus_on.clone(),
                max_depth: options.graph.max_depth,
                with_types: options.graph.with_types,
                with_fns: options.graph.with_fns,
                with_tests: options.graph.with_tests,
                with_orphans: options.graph.with_orphans,
                with_uses: options.with_uses,
                with_externs: options.with_externs,
            },
        }
    }
}
