// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::{Path, PathBuf};

use log::{debug, trace};

use ra_ap_cfg::{CfgAtom, CfgDiff, CfgExpr};
use ra_ap_hir::{self as hir, AsAssocItem, Crate, HasAttrs, HirFileIdExt as _, ModuleSource};
use ra_ap_ide::{AnalysisHost, Edition, RootDatabase};
use ra_ap_ide_db::FxHashMap;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_paths::{AbsPathBuf, Utf8PathBuf};
use ra_ap_project_model::{
    CargoConfig, CargoFeatures, CfgOverrides, InvocationLocation, InvocationStrategy, PackageData,
    ProjectManifest, ProjectWorkspace, ProjectWorkspaceKind, RustLibSource, TargetData,
};
use ra_ap_project_model::{CargoWorkspace, Package, Target, TargetKind};
use ra_ap_syntax::{ast, AstNode, SourceFile};
use ra_ap_vfs::Vfs;

use crate::{
    item::{ItemCfgAttr, ItemTestAttr},
    options::{GeneralOptions, ProjectOptions},
};

pub struct LoadOptions {
    /// Analyze with `#[cfg(test)]` enabled (i.e as if built via `cargo test`).
    pub cfg_test: bool,

    /// Include sysroot crates (`std`, `core` & friends) in analysis.
    pub sysroot: bool,
}

pub fn load_workspace(
    general_options: &GeneralOptions,
    project_options: &ProjectOptions,
    load_options: &LoadOptions,
) -> anyhow::Result<(Crate, AnalysisHost, Vfs)> {
    let project_path = {
        let project_path = project_options.manifest_path.as_path();

        // See: https://github.com/rust-lang/cargo/pull/13909
        // The `canonicalize` func on windows will return `r"\\?\"` verbatim prefix.
        // "The Cargo makes a core assumption that verbatim paths aren't used."
        // So we cannot use this func on windows.
        if cfg!(windows) {
            project_path.to_path_buf()
        } else {
            project_path.canonicalize()?
        }
    };

    let cargo_config = cargo_config(project_options, load_options);
    let load_config = load_config();

    let progress = |string| {
        trace!("Progress: {}", string);
    };

    let mut project_workspace = load_project_workspace(&project_path, &cargo_config, &progress)?;

    let (package, target) = select_package_and_target(&project_workspace, project_options)?;

    if general_options.verbose {
        eprintln!();
        eprintln!("crate");
        eprintln!("└── package: {}", package.name);
        eprintln!("    └── target: {}", target.name);
        eprintln!();
    }

    if load_config.load_out_dirs_from_check {
        let build_scripts = project_workspace.run_build_scripts(&cargo_config, &progress)?;
        project_workspace.set_build_scripts(build_scripts)
    }

    let (db, vfs, _proc_macro_client) =
        ra_ap_load_cargo::load_workspace(project_workspace, &cargo_config.extra_env, &load_config)?;

    let host = AnalysisHost::with_database(db);

    let krate = find_crate(host.raw_database(), &vfs, &target)?;

    Ok((krate, host, vfs))
}

pub fn cargo_config(project_options: &ProjectOptions, load_options: &LoadOptions) -> CargoConfig {
    let all_targets = false;

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

    // Whether to invoke cargo metadata on the sysroot crate
    let sysroot_query_metadata = false;

    // Whether to load sysroot crates (`std`, `core` & friends).
    let sysroot = if load_options.sysroot {
        Some(RustLibSource::Discover)
    } else {
        None
    };

    // Rustc private crate source
    let rustc_source = None;

    // Crates to enable/disable `#[cfg(test)]` on
    let cfg_overrides = match load_options.cfg_test {
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
        all_targets,
        features,
        target,
        sysroot_query_metadata,
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

pub fn load_config() -> LoadCargoConfig {
    let load_out_dirs_from_check = true;
    let prefill_caches = false;
    let with_proc_macro_server = ProcMacroServerChoice::Sysroot;

    LoadCargoConfig {
        load_out_dirs_from_check,
        prefill_caches,
        with_proc_macro_server,
    }
}

pub fn load_project_workspace(
    project_path: &Path,
    cargo_config: &CargoConfig,
    progress: &dyn Fn(String),
) -> anyhow::Result<ProjectWorkspace> {
    let path_buf = std::env::current_dir()?.join(project_path);
    let utf8_path_buf = Utf8PathBuf::from_path_buf(path_buf).unwrap();
    let root = AbsPathBuf::assert(utf8_path_buf);
    let root = ProjectManifest::discover_single(root.as_path())?;

    ProjectWorkspace::load(root, cargo_config, &progress)
}

pub fn select_package_and_target(
    project_workspace: &ProjectWorkspace,
    options: &ProjectOptions,
) -> anyhow::Result<(PackageData, TargetData)> {
    let cargo_workspace = match project_workspace.kind {
        ProjectWorkspaceKind::Cargo { ref cargo, .. } => Ok(cargo),
        ProjectWorkspaceKind::Json { .. } => Err(anyhow::anyhow!("Unexpected JSON workspace")),
        ProjectWorkspaceKind::DetachedFile { .. } => {
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

pub fn select_package(
    workspace: &CargoWorkspace,
    options: &ProjectOptions,
) -> anyhow::Result<Package> {
    let packages: Vec<_> = workspace
        .packages()
        .filter(|idx| workspace[*idx].is_member)
        .collect();

    let package_count = packages.len();

    // If project contains no packages, bail out:

    if package_count < 1 {
        anyhow::bail!("No packages found");
    }

    // If no (or a non-existent) package was provided via options bail out:

    let package_list_items: Vec<_> = packages
        .iter()
        .map(|package_idx| {
            let package = &workspace[*package_idx];
            format!("- {}", package.name)
        })
        .collect();

    let package_list = package_list_items.join("\n");

    // If project contains multiple packages, select the one provided via options:

    if let Some(package_name) = &options.package {
        let package_idx = packages.into_iter().find(|package_idx| {
            let package = &workspace[*package_idx];
            package.name == *package_name
        });

        return package_idx.ok_or_else(|| {
            anyhow::anyhow!(
                indoc::indoc! {
                    "No package found with name {:?}.

                        Packages present in workspace:
                        {}
                        "
                },
                package_name,
                package_list,
            )
        });
    }

    // If project contains a single packages, just pick it:

    if package_count == 1 {
        return Ok(packages[0]);
    }

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

pub fn select_target(
    workspace: &CargoWorkspace,
    package_idx: Package,
    options: &ProjectOptions,
) -> anyhow::Result<Target> {
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
                TargetKind::Lib { .. } => true,
                TargetKind::Example => false,
                TargetKind::Test => false,
                TargetKind::Bench => false,
                TargetKind::Other => false,
                TargetKind::BuildScript => false,
            }
        })
        .collect();

    let target_count = targets.len();

    // If package contains no targets, bail out:

    if target_count < 1 {
        anyhow::bail!("No targets found");
    }

    // If no (or a non-existent) target was provided via options bail out:

    let target_list_items: Vec<_> = targets
        .iter()
        .map(|target_idx| {
            let target = &workspace[*target_idx];
            match target.kind {
                TargetKind::Bin => format!("- {} (--bin {})", target.name, target.name),
                TargetKind::Lib { .. } => format!("- {} (--lib)", target.name),
                TargetKind::Example => unreachable!(),
                TargetKind::Test => unreachable!(),
                TargetKind::Bench => unreachable!(),
                TargetKind::Other => unreachable!(),
                TargetKind::BuildScript => unreachable!(),
            }
        })
        .collect();

    let target_list = target_list_items.join("\n");

    // If package contains multiple targets, select the one provided via options:

    if options.lib {
        let target = targets.into_iter().find(|target_idx| {
            let target = &workspace[*target_idx];
            matches!(target.kind, TargetKind::Lib { .. })
        });

        return target.ok_or_else(|| {
            anyhow::anyhow!(
                indoc::indoc! {
                    "No library target found.

                        Targets present in package:
                        {}
                        "
                },
                target_list,
            )
        });
    }

    if let Some(bin_name) = &options.bin {
        let target = targets.into_iter().find(|target_idx| {
            let target = &workspace[*target_idx];
            (target.kind == TargetKind::Bin) && (target.name == bin_name[..])
        });

        return target.ok_or_else(|| {
            anyhow::anyhow!(
                indoc::indoc! {
                    "No binary target found with name {:?}.

                        Targets present in package:
                        {}
                        "
                },
                bin_name,
                target_list,
            )
        });
    }

    // If project contains a single target, just pick it:

    if target_count == 1 {
        return Ok(targets[0]);
    }

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

pub fn find_crate(db: &RootDatabase, vfs: &Vfs, target: &TargetData) -> anyhow::Result<Crate> {
    let crates = Crate::all(db);

    let target_root_path = target.root.as_path();

    let krate = crates.into_iter().find(|krate| {
        let vfs_path = vfs.file_path(krate.root_file(db));
        let crate_root_path = vfs_path.as_path().unwrap();

        crate_root_path == target_root_path
    });

    krate.ok_or_else(|| anyhow::anyhow!("Crate not found"))
}

pub(crate) fn crate_name(krate: hir::Crate, db: &RootDatabase) -> String {
    // Obtain the crate's declaration name:
    let display_name = &krate.display_name(db).unwrap();

    // Since a crate's name may contain `-` we canonicalize it by replacing with `_`:
    display_name.replace('-', "_")
}

pub(crate) fn krate(module_def_hir: hir::ModuleDef, db: &RootDatabase) -> Option<hir::Crate> {
    module(module_def_hir, db).map(|module| module.krate())
}

pub(crate) fn module(module_def_hir: hir::ModuleDef, db: &RootDatabase) -> Option<hir::Module> {
    match module_def_hir {
        hir::ModuleDef::Module(module) => Some(module),
        module_def_hir => module_def_hir.module(db),
    }
}

pub(crate) fn display_name(module_def_hir: hir::ModuleDef, db: &RootDatabase) -> String {
    match module_def_hir {
        hir::ModuleDef::Module(module_hir) => {
            if module_hir.is_crate_root() {
                crate_name(module_hir.krate(), db)
            } else {
                module_hir
                    .name(db)
                    .map(|name| name.display(db).to_string())
                    .expect("name")
            }
        }
        hir::ModuleDef::Const(const_hir) => {
            if let Some(name) = const_hir.name(db) {
                name.display(db).to_string()
            } else {
                "_".to_owned()
            }
        }
        module_def_hir => module_def_hir
            .name(db)
            .map(|name| name.display(db).to_string())
            .expect("name"),
    }
}

pub(crate) fn name(module_def_hir: hir::ModuleDef, db: &RootDatabase) -> Option<String> {
    module_def_hir
        .name(db)
        .map(|name| name.display(db).to_string())
}

pub(crate) fn display_path(module_def_hir: hir::ModuleDef, db: &RootDatabase) -> String {
    path(module_def_hir, db).unwrap_or_else(|| "<anonymous>".to_owned())
}

pub(crate) fn path(module_def_hir: hir::ModuleDef, db: &RootDatabase) -> Option<String> {
    let mut path = String::new();

    let krate = krate(module_def_hir, db);

    // Obtain the crate's name (unless it's a builtin type, which have no crate):
    if let Some(crate_name) = krate.map(|krate| crate_name(krate, db)) {
        path.push_str(&crate_name);
    }

    // Obtain the item's canonicalized name:
    let relative_path = match module_def_hir {
        hir::ModuleDef::Function(function_hir) => {
            if let Some(assoc_item_hir) = function_hir.as_assoc_item(db) {
                assoc_item_path(assoc_item_hir, db)
            } else {
                hir::ModuleDef::Function(function_hir).canonical_path(db)
            }
        }
        hir::ModuleDef::Const(const_hir) => {
            if let Some(assoc_item_hir) = const_hir.as_assoc_item(db) {
                assoc_item_path(assoc_item_hir, db)
            } else {
                hir::ModuleDef::Const(const_hir).canonical_path(db)
            }
        }
        hir::ModuleDef::TypeAlias(type_alias_hir) => {
            if let Some(assoc_item_hir) = type_alias_hir.as_assoc_item(db) {
                assoc_item_path(assoc_item_hir, db)
            } else {
                hir::ModuleDef::TypeAlias(type_alias_hir).canonical_path(db)
            }
        }
        hir::ModuleDef::BuiltinType(builtin_type_hir) => {
            Some(builtin_type_hir.name().display(db).to_string())
        }
        module_def_hir => module_def_hir.canonical_path(db),
    };

    if let Some(relative_path) = relative_path {
        if !path.is_empty() {
            path.push_str("::");
        }
        path.push_str(&relative_path);
    }

    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

fn assoc_item_path(assoc_item_hir: hir::AssocItem, db: &RootDatabase) -> Option<String> {
    let name = match assoc_item_hir {
        hir::AssocItem::Function(function_hir) => hir::ModuleDef::Function(function_hir)
            .name(db)
            .map(|name| name.display(db).to_string()),
        hir::AssocItem::Const(const_hir) => hir::ModuleDef::Const(const_hir)
            .name(db)
            .map(|name| name.display(db).to_string()),
        hir::AssocItem::TypeAlias(type_alias_hir) => hir::ModuleDef::TypeAlias(type_alias_hir)
            .name(db)
            .map(|name| name.display(db).to_string()),
    };

    let name = name?;

    let container_path = match assoc_item_hir.container(db) {
        hir::AssocItemContainer::Trait(trait_hir) => {
            hir::ModuleDef::Trait(trait_hir).canonical_path(db)
        }
        hir::AssocItemContainer::Impl(impl_hir) => impl_hir
            .self_ty(db)
            .as_adt()
            .and_then(|adt_hir| hir::ModuleDef::Adt(adt_hir).canonical_path(db)),
    };

    let container_path = container_path?;

    Some(format!("{container_path}::{name}"))
}

// https://github.com/rust-lang/rust-analyzer/blob/36a70b7435c48837018c71576d7bb4e8f763f501/crates/syntax/src/ast/make.rs#L821
pub(crate) fn parse_ast<N: AstNode>(text: &str) -> N {
    let parse = SourceFile::parse(text, Edition::CURRENT);
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

pub(crate) fn use_tree_matches_item_path(use_tree: &ast::UseTree, item_path: &str) -> bool {
    if item_path.is_empty() {
        return false;
    }
    let node_path: ast::Path = {
        let syntax = format!("use {item_path};");
        parse_ast(&syntax)
    };
    use_tree_matches_path(use_tree, &node_path)
}

pub(crate) fn use_tree_matches_path(use_tree: &ast::UseTree, path: &ast::Path) -> bool {
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
        return use_tree.is_simple_path() || tree_contains_self(use_tree);
    }

    if use_tree.star_token().is_some() {
        return path_segments.len() == 1;
    }

    let path_suffix = ast::make::path_from_segments(path_segments, false);

    use_tree
        .use_tree_list()
        .into_iter()
        .flat_map(|list| list.use_trees())
        .any(|use_tree| use_tree_matches_path(&use_tree, &path_suffix))
}

fn path_is_self(path: &ast::Path) -> bool {
    path.segment().and_then(|seg| seg.self_token()).is_some() && path.qualifier().is_none()
}

fn tree_is_self(tree: &ast::UseTree) -> bool {
    tree.path().as_ref().map(path_is_self).unwrap_or(false)
}

fn tree_contains_self(tree: &ast::UseTree) -> bool {
    tree.use_tree_list()
        .map(|tree_list| tree_list.use_trees().any(|tree| tree_is_self(&tree)))
        .unwrap_or(false)
}

pub(crate) fn is_test_function(function: hir::Function, db: &RootDatabase) -> bool {
    let attrs = function.attrs(db);
    attrs.by_key("test").exists()
}

pub fn cfgs(hir: hir::ModuleDef, db: &RootDatabase) -> Vec<CfgExpr> {
    let cfg = match cfg(hir, db) {
        Some(cfg) => cfg,
        None => return vec![],
    };

    match cfg {
        CfgExpr::Invalid => vec![],
        cfg @ CfgExpr::Atom(_) => vec![cfg],
        CfgExpr::All(cfgs) => cfgs,
        cfg @ CfgExpr::Any(_) => vec![cfg],
        cfg @ CfgExpr::Not(_) => vec![cfg],
    }
}

pub fn cfg(hir: hir::ModuleDef, db: &RootDatabase) -> Option<CfgExpr> {
    match hir {
        hir::ModuleDef::Module(r#mod) => r#mod.attrs(db).cfg(),
        hir::ModuleDef::Function(r#fn) => r#fn.attrs(db).cfg(),
        hir::ModuleDef::Adt(adt) => adt.attrs(db).cfg(),
        hir::ModuleDef::Variant(r#variant) => r#variant.attrs(db).cfg(),
        hir::ModuleDef::Const(r#const) => r#const.attrs(db).cfg(),
        hir::ModuleDef::Static(r#static) => r#static.attrs(db).cfg(),
        hir::ModuleDef::Trait(r#trait) => r#trait.attrs(db).cfg(),
        hir::ModuleDef::TraitAlias(trait_type) => trait_type.attrs(db).cfg(),
        hir::ModuleDef::TypeAlias(type_alias) => type_alias.attrs(db).cfg(),
        hir::ModuleDef::BuiltinType(_builtin_type) => None,
        hir::ModuleDef::Macro(_) => None,
    }
}

pub fn cfg_attrs(module_def_hir: hir::ModuleDef, db: &RootDatabase) -> Vec<ItemCfgAttr> {
    cfgs(module_def_hir, db)
        .into_iter()
        .filter_map(ItemCfgAttr::new)
        .collect()
}

pub fn test_attr(module_def_hir: hir::ModuleDef, db: &RootDatabase) -> Option<ItemTestAttr> {
    let function = match module_def_hir {
        hir::ModuleDef::Function(function) => function,
        _ => return None,
    };

    if is_test_function(function, db) {
        Some(ItemTestAttr)
    } else {
        None
    }
}

pub fn module_file(module: hir::Module, db: &RootDatabase, vfs: &Vfs) -> Option<PathBuf> {
    let module_source = module.definition_source(db);
    let is_file_module: bool = match &module_source.value {
        ModuleSource::SourceFile(_) => true,
        ModuleSource::Module(_) => false,
        ModuleSource::BlockExpr(_) => false,
    };

    if !is_file_module {
        return None;
    }

    let file_id = module_source.file_id.original_file(db);
    let vfs_path = vfs.file_path(file_id);
    let abs_path = vfs_path.as_path().expect("Could not convert to path");

    let path: &Path = abs_path.as_ref();

    let file_extension = path.extension().and_then(|ext| ext.to_str());

    if file_extension != Some("rs") {
        return None;
    }

    Some(path.to_owned())
}

pub fn moduledef_is_crate(module_def_hir: hir::ModuleDef, _db: &RootDatabase) -> bool {
    let hir::ModuleDef::Module(module) = module_def_hir else {
        return false;
    };
    module.is_crate_root()
}
