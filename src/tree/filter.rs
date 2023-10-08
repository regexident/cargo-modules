// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use hir::HasAttrs;
use ra_ap_hir::{self as hir};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;

use crate::{
    graph::util,
    tree::{node::Node, Tree},
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Options {
    pub focus_on: Option<String>,
    pub max_depth: Option<usize>,
    pub acyclic: bool,
    pub types: bool,
    pub traits: bool,
    pub fns: bool,
    pub tests: bool,
    pub modules: bool,
    pub uses: bool,
    pub externs: bool,
}

#[derive(Debug)]
pub struct Filter<'a> {
    options: Options,
    db: &'a RootDatabase,
    krate: hir::Crate,
}

impl<'a> Filter<'a> {
    pub fn new(options: Options, db: &'a RootDatabase, krate: hir::Crate) -> Self {
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
        let use_tree: ast::UseTree = util::parse_ast(&syntax);

        let max_depth = self.options.max_depth.unwrap_or(usize::MAX);

        let root_node = self
            .filter_node(&tree.root_node, None, max_depth, &use_tree)
            .expect("root node");

        let tree = Tree::new(root_node);

        Ok(tree)
    }

    fn filter_node(
        &self,
        node: &Node,
        depth: Option<usize>,
        max_depth: usize,
        focus_tree: &ast::UseTree,
    ) -> Option<Node> {
        let is_focus_node = util::use_tree_matches_item(focus_tree, &node.item);

        let depth = if is_focus_node { Some(0) } else { depth };

        let should_be_retained = match node.item.hir {
            Some(moduledef_hir) => self.should_retain_moduledef(moduledef_hir),
            None => true,
        };

        let subnode_contains_focus_node = node
            .subnodes
            .iter()
            .map(|node| Self::is_or_contains_focus_node(node, focus_tree));

        let is_or_contains_focus_node =
            is_focus_node || subnode_contains_focus_node.clone().any(|flag| flag);

        let subnodes: Vec<Node> = node
            .subnodes
            .iter()
            .zip(subnode_contains_focus_node)
            .filter_map(|(node, is_or_contains_focus_node)| {
                let depth = if is_or_contains_focus_node {
                    Some(0)
                } else {
                    depth.map(|depth| depth + 1)
                };
                self.filter_node(node, depth, max_depth, focus_tree)
            })
            .collect();

        if !should_be_retained {
            return None;
        }

        if let Some(depth) = depth {
            if depth > max_depth {
                return None;
            }
        } else if !is_or_contains_focus_node {
            return None;
        }

        let item = node.item.clone();
        let node = Node::new(item, subnodes);

        Some(node)
    }

    fn is_or_contains_focus_node(node: &Node, focus_tree: &ast::UseTree) -> bool {
        if util::use_tree_matches_item(focus_tree, &node.item) {
            return true;
        }

        node.subnodes
            .iter()
            .any(|node| Self::is_or_contains_focus_node(node, focus_tree))
    }

    fn should_retain_moduledef(&self, moduledef_hir: hir::ModuleDef) -> bool {
        if !self.options.externs && self.is_extern(moduledef_hir) {
            return false;
        }

        match moduledef_hir {
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

    fn should_retain_module(&self, module_hir: hir::Module) -> bool {
        if !self.options.modules {
            // Always keep a crate's root module:
            return module_hir.is_crate_root();
        }
        true
    }

    fn should_retain_function(&self, function_hir: hir::Function) -> bool {
        if !self.options.fns {
            return false;
        }

        if !self.options.tests {
            let attrs = function_hir.attrs(self.db);
            if attrs.by_key("test").exists() {
                return false;
            }
        }

        true
    }

    fn should_retain_adt(&self, _adt_hir: hir::Adt) -> bool {
        if !self.options.types {
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
        if !self.options.traits {
            return false;
        }

        true
    }

    fn should_retain_trait_alias(&self, _trait_alias_hir: hir::TraitAlias) -> bool {
        if !self.options.traits {
            return false;
        }

        true
    }

    fn should_retain_type_alias(&self, _type_alias_hir: hir::TypeAlias) -> bool {
        if !self.options.types {
            return false;
        }

        true
    }

    fn should_retain_builtin_type(&self, _builtin_type_hir: hir::BuiltinType) -> bool {
        if !self.options.types {
            return false;
        }

        true
    }

    fn should_retain_macro(&self, _macro_hir: hir::Macro) -> bool {
        false
    }

    fn is_extern(&self, moduledef_hir: hir::ModuleDef) -> bool {
        let module = if let hir::ModuleDef::Module(module_hir) = moduledef_hir {
            Some(module_hir)
        } else {
            moduledef_hir.module(self.db)
        };

        let Some(import_krate) = module.map(|module| module.krate()) else {
            return true;
        };

        self.krate != import_krate
    }
}
