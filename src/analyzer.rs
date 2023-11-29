// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::{Path, PathBuf};

use hir::ModuleSource;
use ra_ap_cfg::CfgExpr;
use ra_ap_hir::{self as hir, HasAttrs};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{ast, AstNode, SourceFile};
use ra_ap_vfs::Vfs;

use crate::item::{
    attr::{ItemCfgAttr, ItemTestAttr},
    Item,
};

pub(crate) fn crate_name(krate: hir::Crate, db: &RootDatabase) -> String {
    // Obtain the crate's declaration name:
    let display_name = &krate.display_name(db).unwrap();

    // Since a crate's name may contain `-` we canonicalize it by replacing with `_`:
    display_name.replace('-', "_")
}

pub(crate) fn krate(module_def: hir::ModuleDef, db: &RootDatabase) -> Option<hir::Crate> {
    module(module_def, db).map(|module| module.krate())
}

pub(crate) fn module(module_def: hir::ModuleDef, db: &RootDatabase) -> Option<hir::Module> {
    match module_def {
        hir::ModuleDef::Module(module) => Some(module),
        module_def => module_def.module(db),
    }
}

pub(crate) fn path(module_def: hir::ModuleDef, db: &RootDatabase) -> Vec<String> {
    if let hir::ModuleDef::BuiltinType(builtin) = module_def {
        return vec![builtin.name().display(db).to_string()];
    }

    let mut path = vec![];

    let krate = krate(module_def, db);

    // Obtain the module's krate's name (unless it's a builtin type, which have no crate):
    if let Some(crate_name) = krate.map(|krate| crate_name(krate, db)) {
        path.push(crate_name);
    }

    // Obtain the module's canonicalized name:
    if let Some(relative_canonical_path) = module_def.canonical_path(db) {
        path.push(relative_canonical_path);
    }

    assert!(!path.is_empty());

    path
}

pub(crate) fn path_appending<T>(
    base_path: &[String],
    moduledef_hir: T,
    db: &RootDatabase,
) -> Vec<String>
where
    T: Into<hir::ModuleDef>,
{
    let moduledef_hir: hir::ModuleDef = moduledef_hir.into();

    let mut path = base_path.to_owned();

    let name = match moduledef_hir {
        hir::ModuleDef::Module(module_hir) => {
            if module_hir.is_crate_root() {
                crate_name(module_hir.krate(), db)
            } else {
                module_hir.name(db).expect("name").display(db).to_string()
            }
        }
        hir::ModuleDef::Function(function_hir) => function_hir.name(db).display(db).to_string(),
        hir::ModuleDef::Adt(adt_hir) => adt_hir.name(db).display(db).to_string(),
        hir::ModuleDef::Variant(variant_hir) => variant_hir.name(db).display(db).to_string(),
        hir::ModuleDef::Const(const_hir) => const_hir.name(db).map_or_else(
            || "<anonymous>".to_owned(),
            |name| name.display(db).to_string(),
        ),
        hir::ModuleDef::Static(static_hir) => static_hir.name(db).display(db).to_string(),
        hir::ModuleDef::Trait(trait_hir) => trait_hir.name(db).display(db).to_string(),
        hir::ModuleDef::TraitAlias(trait_alias_hir) => {
            trait_alias_hir.name(db).display(db).to_string()
        }
        hir::ModuleDef::TypeAlias(type_alias_hir) => {
            type_alias_hir.name(db).display(db).to_string()
        }
        hir::ModuleDef::BuiltinType(builtin_type_hir) => {
            builtin_type_hir.name().display(db).to_string()
        }
        hir::ModuleDef::Macro(macro_hir) => macro_hir.name(db).display(db).to_string(),
    };

    path.push(name);

    path
}

// https://github.com/rust-lang/rust-analyzer/blob/36a70b7435c48837018c71576d7bb4e8f763f501/crates/syntax/src/ast/make.rs#L821
pub(crate) fn parse_ast<N: AstNode>(text: &str) -> N {
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

pub(crate) fn use_tree_matches_item(use_tree: &ast::UseTree, item: &Item) -> bool {
    let node_path_segments = &item.path[..];
    if node_path_segments.is_empty() {
        return false;
    }
    let node_path: ast::Path = {
        let focus_on = node_path_segments.join("::");
        let syntax = format!("use {focus_on};");
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

pub fn cfg_attrs(moduledef_hir: hir::ModuleDef, db: &RootDatabase) -> Vec<ItemCfgAttr> {
    cfgs(moduledef_hir, db)
        .into_iter()
        .filter_map(ItemCfgAttr::new)
        .collect()
}

pub fn test_attr(moduledef_hir: hir::ModuleDef, db: &RootDatabase) -> Option<ItemTestAttr> {
    let function = match moduledef_hir {
        hir::ModuleDef::Function(function) => function,
        _ => return None,
    };

    if is_test_function(function, db) {
        Some(ItemTestAttr)
    } else {
        None
    }
}

pub fn module_file(
    module_source: hir::InFile<hir::ModuleSource>,
    db: &RootDatabase,
    vfs: &Vfs,
) -> Option<PathBuf> {
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
