use log::{debug, trace};
use petgraph::{graph::NodeIndex, stable_graph::StableGraph};
use ra_ap_hir::{self, Crate};
use ra_ap_ide::{AnalysisHost, RootDatabase};
use ra_ap_paths::AbsPathBuf;
use ra_ap_project_model::{
    CargoConfig, PackageData, ProjectManifest, ProjectWorkspace, TargetData,
};
use ra_ap_rust_analyzer::cli::{load_workspace, LoadCargoConfig};
use ra_ap_vfs::Vfs;
use structopt::StructOpt;

use crate::{
    graph::{
        builder::{Builder as GraphBuilder, Options as GraphBuilderOptions},
        edge::Edge,
        node::Node,
        util,
    },
    options::{
        general::Options as GeneralOptions, graph::Options as GraphOptions,
        project::Options as ProjectOptions,
    },
    target::{select_package, select_target},
};

pub mod graph;
pub mod tree;

#[derive(StructOpt, Clone, PartialEq, Debug)]
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
    pub fn run(&self) -> Result<(), anyhow::Error> {
        let general_options = self.general_options();
        let project_options = self.project_options();
        let graph_options = self.graph_options();

        let progress = |string| {
            trace!("Progress: {}", string);
        };

        let project_workspace = self.load_project_workspace(&project_options, &progress)?;

        let (package, target) = self.select_target(&project_workspace, &project_options)?;

        if general_options.verbose {
            eprintln!();
            eprintln!("crate");
            eprintln!("└── package: {}", package.name);
            eprintln!("    └── target: {}", target.name);
            eprintln!();
        }

        let (host, vfs) = self.analyze_project_workspace(project_workspace, &progress)?;
        let db = host.raw_database();

        let krate = self.find_crate(db, &vfs, &target)?;

        let (graph, start_node_idx) = self.build_graph(db, &vfs, krate, graph_options)?;

        trace!("Generating ...");

        match &self {
            #[allow(unused_variables)]
            Self::Tree(options) => {
                let command = tree::Command::new(options.clone());
                command.run(&graph, start_node_idx, krate, db)
            }
            #[allow(unused_variables)]
            Self::Graph(options) => {
                let command = graph::Command::new(options.clone());
                command.run(&graph, start_node_idx, krate, db)
            }
        }
    }

    fn load_project_workspace(
        &self,
        options: &ProjectOptions,
        progress: &dyn Fn(String),
    ) -> anyhow::Result<ProjectWorkspace> {
        let project_path = options.manifest_path.as_path().canonicalize()?;

        let cargo_config = CargoConfig {
            // Do not activate the `default` feature.
            no_default_features: options.no_default_features,

            // Activate all available features
            all_features: options.all_features,

            // List of features to activate.
            // This will be ignored if `cargo_all_features` is true.
            features: options.features.clone(),

            // rustc target
            target: options.target.clone(),

            // Don't load sysroot crates (`std`, `core` & friends). Might be useful
            // when debugging isolated issues.
            no_sysroot: true,

            // rustc private crate source
            rustc_source: None,
        };

        let root = AbsPathBuf::assert(std::env::current_dir()?.join(project_path));
        let root = ProjectManifest::discover_single(&root)?;

        ProjectWorkspace::load(root, &cargo_config, &progress)
    }

    fn select_target(
        &self,
        project_workspace: &ProjectWorkspace,
        options: &ProjectOptions,
    ) -> anyhow::Result<(PackageData, TargetData)> {
        let cargo_workspace = match project_workspace {
            ProjectWorkspace::Cargo { cargo, .. } => Ok(cargo),
            ProjectWorkspace::Json { .. } => Err(anyhow::anyhow!("Unexpected JSON workspace")),
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
        project_workspace: ProjectWorkspace,
        progress: &dyn Fn(String),
    ) -> anyhow::Result<(AnalysisHost, Vfs)> {
        let load_cargo_config = LoadCargoConfig {
            load_out_dirs_from_check: false,
            with_proc_macro: true,
        };

        load_workspace(project_workspace, &load_cargo_config, &progress)
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
    ) -> anyhow::Result<(StableGraph<Node, Edge>, NodeIndex)> {
        let graph_builder = {
            let builder_options = self.builder_options();
            GraphBuilder::new(builder_options, db, &vfs, krate)
        };

        let focus_path: Vec<_> = {
            let path_string = options
                .focus_on
                .clone()
                .unwrap_or_else(|| krate.display_name(db).unwrap().to_string());
            path_string.split("::").map(|c| c.to_owned()).collect()
        };

        trace!("Constructing graph ...");

        let mut graph = graph_builder.build(krate)?;

        trace!("Searching for start node in graph ...");

        let start_node_idx = util::idx_of_node_with_path(&graph, &focus_path[..], db)?;

        trace!("Shrinking graph to desired depth ...");

        let max_depth = options.max_depth.unwrap_or(usize::MAX);
        util::shrink_graph(&mut graph, start_node_idx, max_depth);

        Ok((graph, start_node_idx))
    }

    fn general_options(&self) -> &GeneralOptions {
        match &self {
            Self::Tree(options) => &options.general,
            Self::Graph(options) => &options.general,
        }
    }

    fn project_options(&self) -> &ProjectOptions {
        match &self {
            Self::Tree(options) => &options.project,
            Self::Graph(options) => &options.project,
        }
    }

    fn graph_options(&self) -> &GraphOptions {
        match &self {
            Self::Tree(options) => &options.graph,
            Self::Graph(options) => &options.graph,
        }
    }

    fn builder_options(&self) -> GraphBuilderOptions {
        match &self {
            Self::Tree(options) => GraphBuilderOptions {
                focus_on: options.graph.focus_on.clone(),
                max_depth: options.graph.max_depth,
                with_types: options.graph.with_types,
                with_tests: options.graph.with_tests,
                with_orphans: options.graph.with_orphans,
                with_uses: false,
                with_externs: false,
            },
            Self::Graph(options) => GraphBuilderOptions {
                focus_on: options.graph.focus_on.clone(),
                max_depth: options.graph.max_depth,
                with_types: options.graph.with_types,
                with_tests: options.graph.with_tests,
                with_orphans: options.graph.with_orphans,
                with_uses: options.with_uses,
                with_externs: options.with_externs,
            },
        }
    }
}
