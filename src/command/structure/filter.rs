// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide};
use ra_ap_syntax::ast;

use crate::{
    analyzer::{self, has_test_cfg, is_test_function},
    tree::Tree,
};

use super::{Node, options::Options};

#[derive(Debug)]
pub struct Filter<'a> {
    options: &'a Options,
    krate: hir::Crate,
    db: &'a ide::RootDatabase,
    edition: ide::Edition,
}

impl<'a> Filter<'a> {
    pub fn new(
        options: &'a Options,
        krate: hir::Crate,
        db: &'a ide::RootDatabase,
        edition: ide::Edition,
    ) -> Self {
        Self {
            options,
            krate,
            db,
            edition,
        }
    }

    pub fn filter(&self, tree: &Tree<Node>) -> anyhow::Result<Tree<Node>> {
        let crate_name = self.krate.display_name(self.db).unwrap().to_string();
        let focus_on = self.options.focus_on.as_deref();
        let use_tree: ast::UseTree = crate::utils::sanitized_use_tree(focus_on, &crate_name)?;

        let max_depth = self.options.max_depth.unwrap_or(usize::MAX);

        if let Some(tree) = self.filter_tree(tree, 0, max_depth, false, &use_tree) {
            Ok(tree)
        } else {
            anyhow::bail!(
                "no items found matching use tree {:?}",
                focus_on.unwrap_or("crate")
            );
        }
    }

    fn filter_tree(
        &self,
        tree: &Tree<Node>,
        depth: usize,
        max_depth: usize,
        ancestor_is_focus: bool,
        focus_tree: &ast::UseTree,
    ) -> Option<Tree<Node>> {
        let is_focus = self.node_is_focus(tree, focus_tree);

        let has_children = !tree.subtrees.is_empty();

        let subtrees: Vec<Tree<Node>> = tree
            .subtrees
            .iter()
            .filter_map(|tree| {
                let depth = if is_focus { 1 } else { depth + 1 };
                let ancestor_is_focus = ancestor_is_focus || is_focus;
                self.filter_tree(tree, depth, max_depth, ancestor_is_focus, focus_tree)
            })
            .collect();

        let children_contain_focus = has_children && !subtrees.is_empty();

        if !self.should_retain_moduledef(tree.node.hir) {
            return None;
        }

        let mut should_retain_node = false;

        // Keep the node if it is a sub-node of a node matched by `--focus-on`,
        // as long as it is within the specified `--max-depth`:
        should_retain_node |= ancestor_is_focus && depth <= max_depth;

        // Keep the node if it is matched by `--focus-on`:
        should_retain_node |= is_focus;

        // Keep the node if one of its children is matched by `--focus-on`:
        should_retain_node |= children_contain_focus;

        if !should_retain_node {
            return None;
        }

        let item = tree.node.clone();
        let tree = Tree::new(item, subtrees);

        Some(tree)
    }

    fn node_is_focus(&self, tree: &Tree<Node>, focus_tree: &ast::UseTree) -> bool {
        let path = tree.node.display_path(self.db, self.edition);
        analyzer::use_tree_matches_item_path(focus_tree, &path)
    }

    fn should_retain_moduledef(&self, module_def_hir: hir::ModuleDef) -> bool {
        if self.is_extern(module_def_hir) {
            return false;
        }

        if !self.options.cfg_test && has_test_cfg(module_def_hir, self.db) {
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

    fn should_retain_function(&self, function_hir: hir::Function) -> bool {
        if self.options.selection.no_fns {
            return false;
        }

        if !self.options.cfg_test && is_test_function(function_hir, self.db) {
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
