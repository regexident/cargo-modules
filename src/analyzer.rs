// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::{Path, PathBuf};

use log::{debug, trace};

use ra_ap_cfg::{self as cfg};
use ra_ap_hir::{self as hir, AsAssocItem as _, HasAttrs as _, HirFileIdExt as _};
use ra_ap_ide::{self as ide};
use ra_ap_ide_db::{self as ide_db};
use ra_ap_load_cargo::{self as load_cargo};
use ra_ap_paths::{self as paths};
use ra_ap_project_model::{self as project_model};
use ra_ap_syntax::{self as syntax, AstNode as _, ast};
use ra_ap_vfs::{self as vfs};

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
) -> anyhow::Result<(hir::Crate, ide::AnalysisHost, vfs::Vfs, ide::Edition)> {
    let project_path = project_options.manifest_path.as_path().canonicalize()?;

    // See: https://github.com/rust-lang/cargo/pull/13909
    // The `canonicalize` func on windows will return `r"\\?\"` verbatim prefix.
    // "The Cargo makes a core assumption that verbatim paths aren't used."
    // So we need to `simplify` the path as the non-verbatim path.
    let project_path = dunce::simplified(&project_path).to_path_buf();

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

    let edition = package.edition;

    if load_config.load_out_dirs_from_check {
        let build_scripts = project_workspace.run_build_scripts(&cargo_config, &progress)?;
        project_workspace.set_build_scripts(build_scripts)
    }

    let (db, vfs, _proc_macro_client) =
        ra_ap_load_cargo::load_workspace(project_workspace, &cargo_config.extra_env, &load_config)?;

    let host = ide::AnalysisHost::with_database(db);

    let krate = find_crate(host.raw_database(), &vfs, &target)?;

    Ok((krate, host, vfs, edition))
}

pub fn cargo_config(
    project_options: &ProjectOptions,
    load_options: &LoadOptions,
) -> project_model::CargoConfig {
    let all_targets = false;

    // Crates to enable/disable `#[cfg(test)]` on
    let cfg_overrides = match load_options.cfg_test {
        true => project_model::CfgOverrides {
            global: cfg::CfgDiff::new(
                vec![cfg::CfgAtom::Flag(hir::Symbol::intern("test"))],
                Vec::new(),
            ),
            selective: Default::default(),
        },
        false => project_model::CfgOverrides {
            global: cfg::CfgDiff::new(
                Vec::new(),
                vec![cfg::CfgAtom::Flag(hir::Symbol::intern("test"))],
            ),
            selective: Default::default(),
        },
    };

    let extra_args = vec![];

    // FIXME: support extra environment variables via CLI:
    let extra_env = ide_db::FxHashMap::default();

    let extra_includes = vec![];

    // List of features to activate (or deactivate).
    let features = if project_options.all_features {
        project_model::CargoFeatures::All
    } else {
        project_model::CargoFeatures::Selected {
            features: project_options.features.clone(),
            no_default_features: project_options.no_default_features,
        }
    };

    let invocation_strategy = project_model::InvocationStrategy::PerWorkspace;

    let run_build_script_command = None;

    // Rustc private crate source
    let rustc_source = None;

    let set_test = load_options.cfg_test;

    // Whether to load sysroot crates (`std`, `core` & friends).
    let sysroot = if load_options.sysroot {
        Some(project_model::RustLibSource::Discover)
    } else {
        None
    };

    let sysroot_src = None;

    // Target triple
    let target = project_options.target.clone();

    let target_dir = None;

    // Setup RUSTC_WRAPPER to point to `rust-analyzer` binary itself.
    // (We use that to compile only proc macros and build scripts
    // during the initial `cargo check`.)
    let wrap_rustc_in_build_scripts = true;

    project_model::CargoConfig {
        all_targets,
        cfg_overrides,
        extra_args,
        extra_env,
        extra_includes,
        features,
        invocation_strategy,
        run_build_script_command,
        rustc_source,
        set_test,
        sysroot_src,
        sysroot,
        target_dir,
        target,
        wrap_rustc_in_build_scripts,
    }
}

pub fn load_config() -> load_cargo::LoadCargoConfig {
    let load_out_dirs_from_check = true;
    let prefill_caches = false;
    let with_proc_macro_server = load_cargo::ProcMacroServerChoice::Sysroot;

    load_cargo::LoadCargoConfig {
        load_out_dirs_from_check,
        prefill_caches,
        with_proc_macro_server,
    }
}

pub fn load_project_workspace(
    project_path: &Path,
    cargo_config: &project_model::CargoConfig,
    progress: &dyn Fn(String),
) -> anyhow::Result<project_model::ProjectWorkspace> {
    let path_buf = std::env::current_dir()?.join(project_path);
    let utf8_path_buf = paths::Utf8PathBuf::from_path_buf(path_buf).unwrap();
    let root = paths::AbsPathBuf::assert(utf8_path_buf);
    let root = project_model::ProjectManifest::discover_single(root.as_path())?;

    project_model::ProjectWorkspace::load(root, cargo_config, &progress)
}

pub fn select_package_and_target(
    project_workspace: &project_model::ProjectWorkspace,
    options: &ProjectOptions,
) -> anyhow::Result<(project_model::PackageData, project_model::TargetData)> {
    let cargo_workspace = match project_workspace.kind {
        project_model::ProjectWorkspaceKind::Cargo { ref cargo, .. } => Ok(cargo),
        project_model::ProjectWorkspaceKind::Json { .. } => {
            Err(anyhow::anyhow!("Unexpected JSON workspace"))
        }
        project_model::ProjectWorkspaceKind::DetachedFile { .. } => {
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
    workspace: &project_model::CargoWorkspace,
    options: &ProjectOptions,
) -> anyhow::Result<project_model::Package> {
    let packages: Vec<_> = workspace
        .packages()
        .filter(|idx| workspace[*idx].is_member)
        .collect();

    let package_count = packages.len();

    // If project contains no packages, bail out:

    if package_count < 1 {
        anyhow::bail!("no packages found");
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
    workspace: &project_model::CargoWorkspace,
    package_idx: project_model::Package,
    options: &ProjectOptions,
) -> anyhow::Result<project_model::Target> {
    let package = &workspace[package_idx];

    // Retrieve list of indices for bin/lib targets:

    let targets: Vec<_> = package
        .targets
        .iter()
        .cloned()
        .filter(|target_idx| {
            let target = &workspace[*target_idx];
            match target.kind {
                project_model::TargetKind::Bin => true,
                project_model::TargetKind::Lib { .. } => true,
                project_model::TargetKind::Example => false,
                project_model::TargetKind::Test => false,
                project_model::TargetKind::Bench => false,
                project_model::TargetKind::Other => false,
                project_model::TargetKind::BuildScript => false,
            }
        })
        .collect();

    let target_count = targets.len();

    // If package contains no targets, bail out:

    if target_count < 1 {
        anyhow::bail!("no targets found");
    }

    // If no (or a non-existent) target was provided via options bail out:

    let target_list_items: Vec<_> = targets
        .iter()
        .map(|target_idx| {
            let target = &workspace[*target_idx];
            match target.kind {
                project_model::TargetKind::Bin => {
                    format!("- {} (--bin {})", target.name, target.name)
                }
                project_model::TargetKind::Lib { .. } => format!("- {} (--lib)", target.name),
                project_model::TargetKind::Example => unreachable!(),
                project_model::TargetKind::Test => unreachable!(),
                project_model::TargetKind::Bench => unreachable!(),
                project_model::TargetKind::Other => unreachable!(),
                project_model::TargetKind::BuildScript => unreachable!(),
            }
        })
        .collect();

    let target_list = target_list_items.join("\n");

    // If package contains multiple targets, select the one provided via options:

    if options.lib {
        let target = targets.into_iter().find(|target_idx| {
            let target = &workspace[*target_idx];
            matches!(target.kind, project_model::TargetKind::Lib { .. })
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
            (target.kind == project_model::TargetKind::Bin) && (target.name == bin_name[..])
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

pub fn find_crate(
    db: &ide::RootDatabase,
    vfs: &vfs::Vfs,
    target: &project_model::TargetData,
) -> anyhow::Result<hir::Crate> {
    let crates = hir::Crate::all(db);

    let target_root_path = target.root.as_path();

    let krate = crates.into_iter().find(|krate| {
        let vfs_path = vfs.file_path(krate.root_file(db));
        let crate_root_path = vfs_path.as_path().unwrap();

        crate_root_path == target_root_path
    });

    krate.ok_or_else(|| anyhow::anyhow!("Crate not found"))
}

pub(crate) fn crate_name(krate: hir::Crate, db: &ide::RootDatabase) -> String {
    // Obtain the crate's declaration name:
    let display_name = krate.display_name(db).unwrap().to_string();

    // Since a crate's name may contain `-` we canonicalize it by replacing with `_`:
    display_name.replace('-', "_")
}

pub(crate) fn krate(module_def_hir: hir::ModuleDef, db: &ide::RootDatabase) -> Option<hir::Crate> {
    module(module_def_hir, db).map(|module| module.krate())
}

pub(crate) fn module(
    module_def_hir: hir::ModuleDef,
    db: &ide::RootDatabase,
) -> Option<hir::Module> {
    match module_def_hir {
        hir::ModuleDef::Module(module) => Some(module),
        module_def_hir => module_def_hir.module(db),
    }
}

pub(crate) fn display_name(
    module_def_hir: hir::ModuleDef,
    db: &ide::RootDatabase,
    edition: ide::Edition,
) -> String {
    match module_def_hir {
        hir::ModuleDef::Module(module_hir) => {
            if module_hir.is_crate_root() {
                crate_name(module_hir.krate(), db)
            } else {
                module_hir
                    .name(db)
                    .map(|name| name.display(db, edition).to_string())
                    .expect("name")
            }
        }
        hir::ModuleDef::Const(const_hir) => {
            if let Some(name) = const_hir.name(db) {
                name.display(db, edition).to_string()
            } else {
                "_".to_owned()
            }
        }
        module_def_hir => module_def_hir
            .name(db)
            .map(|name| name.display(db, edition).to_string())
            .expect("name"),
    }
}

pub(crate) fn name(
    module_def_hir: hir::ModuleDef,
    db: &ide::RootDatabase,
    edition: ide::Edition,
) -> Option<String> {
    module_def_hir
        .name(db)
        .map(|name| name.display(db, edition).to_string())
}

pub(crate) fn display_path(
    module_def_hir: hir::ModuleDef,
    db: &ide::RootDatabase,
    edition: ide::Edition,
) -> String {
    path(module_def_hir, db, edition).unwrap_or_else(|| "<anonymous>".to_owned())
}

pub(crate) fn path(
    module_def_hir: hir::ModuleDef,
    db: &ide::RootDatabase,
    edition: ide::Edition,
) -> Option<String> {
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
                assoc_item_path(assoc_item_hir, db, edition)
            } else {
                hir::ModuleDef::Function(function_hir).canonical_path(db, edition)
            }
        }
        hir::ModuleDef::Const(const_hir) => {
            if let Some(assoc_item_hir) = const_hir.as_assoc_item(db) {
                assoc_item_path(assoc_item_hir, db, edition)
            } else {
                hir::ModuleDef::Const(const_hir).canonical_path(db, edition)
            }
        }
        hir::ModuleDef::TypeAlias(type_alias_hir) => {
            if let Some(assoc_item_hir) = type_alias_hir.as_assoc_item(db) {
                assoc_item_path(assoc_item_hir, db, edition)
            } else {
                hir::ModuleDef::TypeAlias(type_alias_hir).canonical_path(db, edition)
            }
        }
        hir::ModuleDef::BuiltinType(builtin_type_hir) => {
            Some(builtin_type_hir.name().display(db, edition).to_string())
        }
        module_def_hir => module_def_hir.canonical_path(db, edition),
    };

    if let Some(relative_path) = relative_path {
        if !path.is_empty() {
            path.push_str("::");
        }
        path.push_str(&relative_path);
    }

    if path.is_empty() { None } else { Some(path) }
}

fn assoc_item_path(
    assoc_item_hir: hir::AssocItem,
    db: &ide::RootDatabase,
    edition: ide::Edition,
) -> Option<String> {
    let name = match assoc_item_hir {
        hir::AssocItem::Function(function_hir) => hir::ModuleDef::Function(function_hir)
            .name(db)
            .map(|name| name.display(db, edition).to_string()),
        hir::AssocItem::Const(const_hir) => hir::ModuleDef::Const(const_hir)
            .name(db)
            .map(|name| name.display(db, edition).to_string()),
        hir::AssocItem::TypeAlias(type_alias_hir) => hir::ModuleDef::TypeAlias(type_alias_hir)
            .name(db)
            .map(|name| name.display(db, edition).to_string()),
    };

    let name = name?;

    let container_path = match assoc_item_hir.container(db) {
        hir::AssocItemContainer::Trait(trait_hir) => {
            hir::ModuleDef::Trait(trait_hir).canonical_path(db, edition)
        }
        hir::AssocItemContainer::Impl(impl_hir) => impl_hir
            .self_ty(db)
            .as_adt()
            .and_then(|adt_hir| hir::ModuleDef::Adt(adt_hir).canonical_path(db, edition)),
    };

    let container_path = container_path?;

    Some(format!("{container_path}::{name}"))
}

// https://github.com/rust-lang/rust-analyzer/blob/36a70b7435c48837018c71576d7bb4e8f763f501/crates/syntax/src/ast/make.rs#L821
pub(crate) fn parse_ast<N: syntax::AstNode>(text: &str) -> N {
    let parse = syntax::SourceFile::parse(text, ide::Edition::CURRENT);
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

pub(crate) fn parse_use_tree(path_expr: &str) -> ast::UseTree {
    parse_ast(&format!("use {path_expr};"))
}

pub(crate) fn parse_path_expr(path_expr: &str) -> ast::Path {
    parse_ast(path_expr)
}

pub(crate) fn use_tree_matches_item_path(use_tree: &ast::UseTree, item_path: &str) -> bool {
    if item_path.is_empty() {
        return false;
    }
    let node_path: ast::Path = parse_path_expr(item_path);
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

pub(crate) fn has_test_cfg(hir: hir::ModuleDef, db: &ide::RootDatabase) -> bool {
    let Some(attrs) = hir.attrs(db) else {
        return false;
    };

    let test_key = hir::Symbol::intern("test");
    let cfg_exprs: Vec<_> = attrs.cfgs().collect();

    cfg_exprs.into_iter().any(|cfg_expr| {
        cfg_expr
            .fold(&|cfg| {
                use ra_ap_cfg::CfgAtom;
                match cfg {
                    CfgAtom::Flag(symbol) => symbol == &test_key,
                    CfgAtom::KeyValue { .. } => false,
                }
            })
            .unwrap_or_default()
    })
}

pub(crate) fn is_test_function(function: hir::Function, db: &ide::RootDatabase) -> bool {
    let attrs = function.attrs(db);
    let key = hir::Symbol::intern("test");
    attrs.by_key(&key).exists()
}

pub fn cfgs(hir: hir::ModuleDef, db: &ide::RootDatabase) -> Vec<cfg::CfgExpr> {
    let cfg = match cfg(hir, db) {
        Some(cfg) => cfg,
        None => return vec![],
    };

    match cfg {
        cfg::CfgExpr::Invalid => vec![],
        cfg @ cfg::CfgExpr::Atom(_) => vec![cfg],
        cfg::CfgExpr::All(cfgs) => cfgs.to_vec(),
        cfg @ cfg::CfgExpr::Any(_) => vec![cfg],
        cfg @ cfg::CfgExpr::Not(_) => vec![cfg],
    }
}

pub fn cfg(hir: hir::ModuleDef, db: &ide::RootDatabase) -> Option<cfg::CfgExpr> {
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

pub fn cfg_attrs(module_def_hir: hir::ModuleDef, db: &ide::RootDatabase) -> Vec<ItemCfgAttr> {
    cfgs(module_def_hir, db)
        .iter()
        .filter_map(ItemCfgAttr::new)
        .collect()
}

pub fn test_attr(module_def_hir: hir::ModuleDef, db: &ide::RootDatabase) -> Option<ItemTestAttr> {
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

pub fn module_file(module: hir::Module, db: &ide::RootDatabase, vfs: &vfs::Vfs) -> Option<PathBuf> {
    let module_source = module.definition_source(db);
    let is_file_module: bool = match &module_source.value {
        hir::ModuleSource::SourceFile(_) => true,
        hir::ModuleSource::Module(_) => false,
        hir::ModuleSource::BlockExpr(_) => false,
    };

    if !is_file_module {
        return None;
    }

    let file_id = module_source.file_id.original_file(db);
    let vfs_path = vfs.file_path(file_id.into());
    let abs_path = vfs_path.as_path().expect("Could not convert to path");

    let path: &Path = abs_path.as_ref();

    let file_extension = path.extension().and_then(|ext| ext.to_str());

    if file_extension != Some("rs") {
        return None;
    }

    Some(path.to_owned())
}

pub fn moduledef_is_crate(module_def_hir: hir::ModuleDef, _db: &ide::RootDatabase) -> bool {
    let hir::ModuleDef::Module(module) = module_def_hir else {
        return false;
    };
    module.is_crate_root()
}
