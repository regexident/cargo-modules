// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::Path;

use clap::Parser;
use log::{debug, trace};

use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::{self, Crate};
use ra_ap_ide::{AnalysisHost, RootDatabase};
use ra_ap_ide_db::FxHashMap;
use ra_ap_load_cargo::{load_workspace, LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_paths::AbsPathBuf;
use ra_ap_proc_macro_api::ProcMacroServer;
use ra_ap_project_model::{
    CargoConfig, CargoFeatures, CfgOverrides, InvocationLocation, InvocationStrategy, PackageData,
    ProjectManifest, ProjectWorkspace, RustLibSource, TargetData,
};
use ra_ap_vfs::Vfs;

use crate::{
    graph::{
        builder::Options as GraphBuilderOptions, command::Command as GenerateGraphCommand,
        filter::Options as GraphFilterOptions, options::Options as GraphOptions,
        printer::Options as GraphPrinterOptions,
    },
    options::{
        general::Options as GeneralOptions, project::Options as ProjectOptions,
        selection::Options as SelectionOptions,
    },
    target::{select_package, select_target},
    tree::{
        builder::Options as TreeBuilderOptions, command::Command as GenerateTreeCommand,
        filter::Options as TreeFilterOptions, options::Options as TreeOptions,
        printer::Options as TreePrinterOptions,
    },
};

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
pub enum Command {
    #[command(name = "tree", about = "Print crate as a tree.")]
    Tree(TreeOptions),

    #[command(
        name = "graph",
        about = "Print crate as a graph.",
        after_help = r#"
        If you have xdot installed on your system, you can run this using:
        `cargo modules generate dependencies | xdot -`
        "#
    )]
    Graph(GraphOptions),
}

impl Command {
    pub(crate) fn sanitize(&mut self) {
        if self.selection_options().tests && !self.project_options().cfg_test {
            debug!("Enabling `--cfg-test`, which is implied by `--tests`");
            self.project_options_mut().cfg_test = true;
        }
    }

    pub fn run(&self) -> Result<(), anyhow::Error> {
        let general_options = self.general_options();
        let project_options = self.project_options();

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

        match self {
            #[allow(unused_variables)]
            Self::Tree(options) => {
                let builder_options = TreeBuilderOptions {
                    orphans: options.orphans,
                };
                let filter_options = TreeFilterOptions {
                    focus_on: options.selection.focus_on.clone(),
                    max_depth: options.selection.max_depth,
                    acyclic: false,
                    modules: true,
                    types: options.selection.types,
                    traits: options.selection.traits,
                    fns: options.selection.fns,
                    tests: options.selection.tests,
                    uses: false,
                    externs: false,
                };
                let printer_options = TreePrinterOptions {
                    sort_by: options.sort_by,
                    sort_reversed: options.sort_reversed,
                };

                let command =
                    GenerateTreeCommand::new(builder_options, filter_options, printer_options);
                command.run(krate, db, &vfs)?;
                Ok(())
            }
            #[allow(unused_variables)]
            Self::Graph(options) => {
                let builder_options = GraphBuilderOptions {};
                let filter_options = GraphFilterOptions {
                    focus_on: options.selection.focus_on.clone(),
                    max_depth: options.selection.max_depth,
                    acyclic: options.acyclic,
                    types: options.selection.types,
                    traits: options.selection.traits,
                    fns: options.selection.fns,
                    tests: options.selection.tests,
                    modules: options.modules,
                    uses: options.uses,
                    externs: options.externs,
                };
                let printer_options = GraphPrinterOptions {
                    layout: options.layout,
                    full_paths: options.externs,
                };
                let command =
                    GenerateGraphCommand::new(builder_options, filter_options, printer_options);
                command.run(krate, db, &vfs)?;
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
        let sysroot = if project_options.sysroot && self.sysroot() {
            Some(RustLibSource::Discover)
        } else {
            None
        };

        // rustc private crate source
        let rustc_source = None;

        // crates to disable `#[cfg(test)]` on
        let cfg_overrides = match project_options.cfg_test {
            true => CfgOverrides {
                global: CfgDiff::new(vec![CfgAtom::Flag("test".into())], Vec::new()).unwrap(),
                selective: Default::default(),
            },
            false => CfgOverrides {
                global: CfgDiff::new(Vec::new(), vec![CfgAtom::Flag("test".into())]).unwrap(),
                selective: Default::default(),
            },
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

        let sysroot_src = None;

        let extra_args = vec![];

        let target_dir = None;

        CargoConfig {
            features,
            target,
            sysroot,
            rustc_source,
            cfg_overrides,
            wrap_rustc_in_build_scripts,
            run_build_script_command,
            extra_env,
            invocation_strategy,
            invocation_location,
            sysroot_src,
            extra_args,
            target_dir,
        }
    }

    fn load_config(&self) -> LoadCargoConfig {
        let load_out_dirs_from_check = true;
        let prefill_caches = false;
        let with_proc_macro_server = ProcMacroServerChoice::Sysroot;

        LoadCargoConfig {
            load_out_dirs_from_check,
            prefill_caches,
            with_proc_macro_server,
        }
    }

    fn load_project_workspace(
        &self,
        project_path: &Path,
        cargo_config: &CargoConfig,
        progress: &dyn Fn(String),
    ) -> anyhow::Result<ProjectWorkspace> {
        let root = AbsPathBuf::assert(std::env::current_dir()?.join(project_path));
        let root = ProjectManifest::discover_single(root.as_path())?;

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

    fn sysroot(&self) -> bool {
        match self {
            Self::Tree(_) => false,
            Self::Graph(options) => {
                // We only need to include sysroot if we include extern uses
                // and didn't explicitly request sysroot to be excluded:
                options.uses && options.externs
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

    fn selection_options(&self) -> &SelectionOptions {
        match self {
            Self::Tree(options) => &options.selection,
            Self::Graph(options) => &options.selection,
        }
    }

    #[allow(dead_code)]
    fn selection_options_mut(&mut self) -> &mut SelectionOptions {
        match self {
            Self::Tree(options) => &mut options.selection,
            Self::Graph(options) => &mut options.selection,
        }
    }
}
