use std::path::PathBuf;

use log::trace;
use ra_ap_hir::{self as hir, Crate};
use ra_ap_ide_db::RootDatabase;
use ra_ap_paths::AbsPathBuf;
use ra_ap_project_model::{
    CargoConfig, CargoWorkspace, Package, ProjectManifest, ProjectWorkspace, Target, TargetKind,
};
use ra_ap_vfs::Vfs;

use crate::options::project::Options;

pub struct Runner<'a> {
    project_dir: PathBuf,
    options: Options,
    db: &'a RootDatabase,
    vfs: &'a Vfs,
}

impl<'a> Runner<'a> {
    pub fn new(project_dir: PathBuf, options: Options, db: &'a RootDatabase, vfs: &'a Vfs) -> Self {
        Self {
            project_dir,
            options,
            db,
            vfs,
        }
    }

    pub fn run<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(Crate) -> anyhow::Result<()>,
    {
        let project_dir = std::env::current_dir()?.join(&self.project_dir);
        let abs_project_dir = AbsPathBuf::assert(project_dir);
        let manifest = ProjectManifest::discover_single(abs_project_dir.as_path())?;

        let config = CargoConfig {
            // Do not activate the `default` feature.
            no_default_features: false,

            // Activate all available features
            all_features: false,

            // List of features to activate.
            // This will be ignored if `cargo_all_features` is true.
            features: vec![],

            // Runs cargo check on launch to figure out the correct values of OUT_DIR
            load_out_dirs_from_check: false,

            // rustc target
            target: None,

            // Don't load sysroot crates (`std`, `core` & friends). Might be useful
            // when debugging isolated issues.
            no_sysroot: true,

            // rustc private crate source
            rustc_source: None,
        };

        let project_workspace = ProjectWorkspace::load(manifest, &config)?;

        let workspace = match project_workspace {
            ProjectWorkspace::Cargo { cargo, .. } => cargo,
            ProjectWorkspace::Json { .. } => {
                unreachable!();
            }
        };

        let package_idx = self.package(&workspace)?;
        let package = &workspace[package_idx];
        trace!("Selected package: {:#?}", package.name);

        let target_idx = self.target(&workspace, package_idx)?;
        let target = &workspace[target_idx];
        trace!("Selected target: {:#?}", target.name);

        let target_root_path = target.root.as_path();

        let crates = hir::Crate::all(self.db);

        let krate = crates.into_iter().find(|krate| {
            let vfs_path = self.vfs.file_path(krate.root_file(self.db));
            let crate_root_path = vfs_path.as_path().unwrap();

            crate_root_path == target_root_path
        });

        let krate = match krate {
            Some(krate) => krate,
            None => panic!("Crate not found"),
        };

        let crate_name = krate.display_name(self.db).unwrap().to_string();
        trace!("Selected crate: {:#?}", crate_name);

        f(krate)
    }

    fn package(&self, workspace: &CargoWorkspace) -> anyhow::Result<Package> {
        let packages: Vec<_> = workspace
            .packages()
            .filter(|idx| workspace[*idx].is_member)
            .collect();

        let package_count = packages.len();

        // If project contains no packages, bail out:

        if package_count < 1 {
            anyhow::bail!("No packages found");
        }

        // If project contains a single packages, just pick it:

        if package_count == 1 {
            return Ok(packages[0]);
        }

        // If project contains multiple packages, select the one provided via options:

        if let Some(package_name) = &self.options.package {
            let package_idx = packages
                .into_iter()
                .find(|package_idx| {
                    let package = &workspace[*package_idx];
                    package.name == *package_name
                })
                .expect(&format!("No package with name {:?}", package_name));
            return Ok(package_idx);
        }

        // If no package was provided via options bail out:

        let package_list_items: Vec<_> = packages
            .into_iter()
            .map(|package_idx| {
                let package = &workspace[package_idx];
                format!("- {}", package.name)
            })
            .collect();

        let package_list = package_list_items.join("\n");

        Err(anyhow::anyhow!(
            indoc::indoc! {
                "Multiple packages present in workspace,
                please explicitly select one via --package flag.

                Packages present in workspace:
                {}
                "
            },
            package_list
        ))
    }

    fn target(&self, workspace: &CargoWorkspace, package_idx: Package) -> anyhow::Result<Target> {
        let package = &workspace[package_idx];

        // Retrieve list of indices for bin/lib targets:

        let targets: Vec<_> = package
            .targets
            .iter()
            .cloned()
            .filter(|target_idx| {
                let target = &workspace[*target_idx];
                match target.kind {
                    TargetKind::Bin => true,
                    TargetKind::Lib => true,
                    TargetKind::Example => false,
                    TargetKind::Test => false,
                    TargetKind::Bench => false,
                    TargetKind::Other => false,
                }
            })
            .collect();

        let target_count = targets.len();

        // If package contains no targets, bail out:

        if target_count < 1 {
            anyhow::bail!("No targets found");
        }

        // If project contains a single target, just pick it:

        if target_count == 1 {
            return Ok(targets[0]);
        }

        // If package contains multiple targets, select the one provided via options:

        if self.options.lib {
            let target = targets.into_iter().find(|target_idx| {
                let target = &workspace[*target_idx];
                target.kind == TargetKind::Lib
            });

            return target.ok_or_else(|| anyhow::anyhow!("No library target found"));
        }

        if let Some(bin_name) = &self.options.bin {
            let target = targets.into_iter().find(|target_idx| {
                let target = &workspace[*target_idx];
                (target.kind == TargetKind::Bin) && (target.name == &bin_name[..])
            });

            return target
                .ok_or_else(|| anyhow::anyhow!("No binary target found with name {:?}", bin_name));
        }

        // If no target was provided via options bail out:

        let target_list_items: Vec<_> = targets
            .into_iter()
            .map(|target_idx| {
                let target = &workspace[target_idx];
                match target.kind {
                    TargetKind::Bin => format!("- {} (--bin {})", target.name, target.name),
                    TargetKind::Lib => format!("- {} (--lib)", target.name),
                    TargetKind::Example => unreachable!(),
                    TargetKind::Test => unreachable!(),
                    TargetKind::Bench => unreachable!(),
                    TargetKind::Other => unreachable!(),
                }
            })
            .collect();

        let target_list = target_list_items.join("\n");

        Err(anyhow::anyhow!(
            indoc::indoc! {
                "Multiple targets present in package,
                please explicitly select one via --lib or --bin flag.

                Targets present in package:
                {}
                "
            },
            target_list
        ))
    }
}
