// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_hir::{self as hir};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;

use crate::{
    analyzer,
    structure::{options::Options, tree::Tree},
};

#[derive(Debug)]
pub struct Filter<'a> {
    options: &'a Options,
    db: &'a RootDatabase,
    krate: hir::Crate,
}

impl<'a> Filter<'a> {
    pub fn new(options: &'a Options, db: &'a RootDatabase, krate: hir::Crate) -> Self {
        Self { options, db, krate }
    }

    pub fn filter(&self, tree: &Tree) -> anyhow::Result<Tree> {
        let focus_on = self
            .options
            .focus_on
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.krate.display_name(self.db).unwrap().to_string());

        let syntax = format!("use {focus_on};");
        let use_tree: ast::UseTree = analyzer::parse_ast(&syntax);

        let max_depth = self.options.max_depth.unwrap_or(usize::MAX);

        let tree = self
            .filter_tree(tree, None, max_depth, &use_tree)
            .expect("root tree");

        Ok(tree)
    }

    fn filter_tree(
        &self,
        tree: &Tree,
        depth: Option<usize>,
        max_depth: usize,
        focus_tree: &ast::UseTree,
    ) -> Option<Tree> {
        let path = tree.item.display_path(self.db);

        let is_focus_tree = analyzer::use_tree_matches_item_path(focus_tree, &path);

        let depth = if is_focus_tree { Some(0) } else { depth };

        let should_be_retained = self.should_retain_moduledef(tree.item.hir);

        let subtree_contains_focus_tree = tree
            .subtrees
            .iter()
            .map(|tree| self.is_or_contains_focus_tree(tree, focus_tree));

        let is_or_contains_focus_tree =
            is_focus_tree || subtree_contains_focus_tree.clone().any(|flag| flag);

        let subtrees: Vec<Tree> = tree
            .subtrees
            .iter()
            .zip(subtree_contains_focus_tree)
            .filter_map(|(tree, is_or_contains_focus_tree)| {
                let depth = if is_or_contains_focus_tree {
                    Some(0)
                } else {
                    depth.map(|depth| depth + 1)
                };
                self.filter_tree(tree, depth, max_depth, focus_tree)
            })
            .collect();

        if !should_be_retained {
            return None;
        }

        if let Some(depth) = depth {
            if depth > max_depth {
                return None;
            }
        } else if !is_or_contains_focus_tree {
            return None;
        }

        let item = tree.item.clone();
        let tree = Tree::new(item, subtrees);

        Some(tree)
    }

    fn is_or_contains_focus_tree(&self, tree: &Tree, focus_tree: &ast::UseTree) -> bool {
        let path = tree.item.display_path(self.db);

        if analyzer::use_tree_matches_item_path(focus_tree, &path) {
            return true;
        }

        tree.subtrees
            .iter()
            .any(|tree| self.is_or_contains_focus_tree(tree, focus_tree))
    }

    fn should_retain_moduledef(&self, module_def_hir: hir::ModuleDef) -> bool {
        if self.is_extern(module_def_hir) {
            return false;
        }

        match module_def_hir {
            hir::ModuleDef::Module(module_hir) => self.should_retain_module(module_hir),
            hir::ModuleDef::Function(function_hir) => self.should_retain_function(function_hir),
            hir::ModuleDef::Adt(adt_hir) => self.should_retain_adt(adt_hir),
            hir::ModuleDef::Variant(variant_hir) => self.should_retain_variant(variant_hir),
            hir::ModuleDef::Const(const_hir) => self.should_retain_const(const_hir),
            hir::ModuleDef::Static(static_hir) => self.should_retain_static(static_hir),
            hir::ModuleDef::Trait(trait_hir) => self.should_retain_trait(trait_hir),
            hir::ModuleDef::TraitAlias(trait_alias_hir) => {
                self.should_retain_trait_alias(trait_alias_hir)
            }
            hir::ModuleDef::TypeAlias(type_alias_hir) => {
                self.should_retain_type_alias(type_alias_hir)
            }
            hir::ModuleDef::BuiltinType(builtin_type_hir) => {
                self.should_retain_builtin_type(builtin_type_hir)
            }
            hir::ModuleDef::Macro(macro_hir) => self.should_retain_macro(macro_hir),
        }
    }

    fn should_retain_module(&self, _module_hir: hir::Module) -> bool {
        true
    }

    fn should_retain_function(&self, _function_hir: hir::Function) -> bool {
        if self.options.selection.no_fns {
            return false;
        }

        true
    }

    fn should_retain_adt(&self, _adt_hir: hir::Adt) -> bool {
        if self.options.selection.no_types {
            return false;
        }

        true
    }

    fn should_retain_variant(&self, _variant_hir: hir::Variant) -> bool {
        false
    }

    fn should_retain_const(&self, _const_hir: hir::Const) -> bool {
        false
    }

    fn should_retain_static(&self, _static_hir: hir::Static) -> bool {
        false
    }

    fn should_retain_trait(&self, _trait_hir: hir::Trait) -> bool {
        if self.options.selection.no_traits {
            return false;
        }

        true
    }

    fn should_retain_trait_alias(&self, _trait_alias_hir: hir::TraitAlias) -> bool {
        if self.options.selection.no_traits {
            return false;
        }

        true
    }

    fn should_retain_type_alias(&self, _type_alias_hir: hir::TypeAlias) -> bool {
        if self.options.selection.no_types {
            return false;
        }

        true
    }

    fn should_retain_builtin_type(&self, _builtin_type_hir: hir::BuiltinType) -> bool {
        if self.options.selection.no_types {
            return false;
        }

        true
    }

    fn should_retain_macro(&self, _macro_hir: hir::Macro) -> bool {
        false
    }

    fn is_extern(&self, module_def_hir: hir::ModuleDef) -> bool {
        let module = if let hir::ModuleDef::Module(module_hir) = module_def_hir {
            Some(module_hir)
        } else {
            module_def_hir.module(self.db)
        };

        let Some(import_krate) = module.map(|module| module.krate()) else {
            return true;
        };

        self.krate != import_krate
    }
}
