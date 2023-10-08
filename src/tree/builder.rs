// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use hir::ModuleDef;
use log::trace;
use ra_ap_hir::{self as hir, Crate};
use ra_ap_ide_db::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::{
    item::Item,
    tree::{node::Node, Tree},
};

use super::orphans::orphan_nodes_for;

// use super::orphans::add_orphan_nodes_to;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Options {
    pub orphans: bool,
}

#[derive(Debug)]
pub struct Builder<'a> {
    options: Options,
    db: &'a RootDatabase,
    vfs: &'a Vfs,
    krate: hir::Crate,
}

impl<'a> Builder<'a> {
    pub fn new(options: Options, db: &'a RootDatabase, vfs: &'a Vfs, krate: hir::Crate) -> Self {
        Self {
            options,
            db,
            vfs,
            krate,
        }
    }

    pub fn build(mut self) -> anyhow::Result<Tree> {
        trace!("Scanning project ...");

        let tree = self
            .process_crate(self.krate)
            .expect("Expected tree node for crate root module");

        Ok(tree)
    }

    fn process_crate(&mut self, krate: Crate) -> Option<Tree> {
        let root_module = krate.root_module();
        let root_node = self.process_module(root_module);

        root_node.map(Tree::new)
    }

    fn process_moduledef(&mut self, moduledef_hir: hir::ModuleDef) -> Option<Node> {
        match moduledef_hir {
            hir::ModuleDef::Module(module_hir) => self.process_module(module_hir),
            hir::ModuleDef::Function(function_hir) => self.process_function(function_hir),
            hir::ModuleDef::Adt(adt_hir) => self.process_adt(adt_hir),
            hir::ModuleDef::Variant(variant_hir) => self.process_variant(variant_hir),
            hir::ModuleDef::Const(const_hir) => self.process_const(const_hir),
            hir::ModuleDef::Static(static_hir) => self.process_static(static_hir),
            hir::ModuleDef::Trait(trait_hir) => self.process_trait(trait_hir),
            hir::ModuleDef::TraitAlias(trait_alias_hir) => {
                self.process_trait_alias(trait_alias_hir)
            }
            hir::ModuleDef::TypeAlias(type_alias_hir) => self.process_type_alias(type_alias_hir),
            hir::ModuleDef::BuiltinType(builtin_type_hir) => {
                self.process_builtin_type(builtin_type_hir)
            }
            hir::ModuleDef::Macro(macro_hir) => self.process_macro(macro_hir),
        }
    }

    fn process_module(&mut self, module_hir: hir::Module) -> Option<Node> {
        let item = Item::new(ModuleDef::Module(module_hir), self.db, self.vfs);
        let mut node = Node::new(item, vec![]);

        // eprintln!("node: {:?}", node.item.path);

        let subnodes = module_hir
            .declarations(self.db)
            .into_iter()
            .filter_map(|moduledef_hir| self.process_moduledef(moduledef_hir));

        for subnode in subnodes {
            // eprintln!("- subnode: {:?}", subnode.item.path);
            node.push_subnode(subnode);
        }

        if self.options.orphans && node.item.is_file() {
            for subnode in orphan_nodes_for(&node) {
                // eprintln!("- orphan: {:?}", subnode.item.path);
                node.push_subnode(subnode);
            }
        }

        Some(node)
    }

    fn process_function(&mut self, function_hir: hir::Function) -> Option<Node> {
        self.simple_node(hir::ModuleDef::Function(function_hir))
    }

    fn process_adt(&mut self, adt_hir: hir::Adt) -> Option<Node> {
        match adt_hir {
            hir::Adt::Struct(struct_hir) => self.process_struct(struct_hir),
            hir::Adt::Union(union_hir) => self.process_union(union_hir),
            hir::Adt::Enum(enum_hir) => self.process_enum(enum_hir),
        }
    }

    fn process_struct(&mut self, struct_hir: hir::Struct) -> Option<Node> {
        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Struct(struct_hir)))
    }

    fn process_union(&mut self, union_hir: hir::Union) -> Option<Node> {
        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Union(union_hir)))
    }

    fn process_enum(&mut self, enum_hir: hir::Enum) -> Option<Node> {
        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Enum(enum_hir)))
    }

    fn process_variant(&mut self, _variant_hir: hir::Variant) -> Option<Node> {
        None
    }

    fn process_const(&mut self, _const_hir: hir::Const) -> Option<Node> {
        None
    }

    fn process_static(&mut self, static_hir: hir::Static) -> Option<Node> {
        self.simple_node(hir::ModuleDef::Static(static_hir))
    }

    fn process_trait(&mut self, trait_hir: hir::Trait) -> Option<Node> {
        self.simple_node(hir::ModuleDef::Trait(trait_hir))
    }

    fn process_trait_alias(&mut self, trait_alias_hir: hir::TraitAlias) -> Option<Node> {
        self.simple_node(hir::ModuleDef::TraitAlias(trait_alias_hir))
    }

    fn process_type_alias(&mut self, type_alias_hir: hir::TypeAlias) -> Option<Node> {
        self.simple_node(hir::ModuleDef::TypeAlias(type_alias_hir))
    }

    fn process_builtin_type(&mut self, builtin_type_hir: hir::BuiltinType) -> Option<Node> {
        self.simple_node(hir::ModuleDef::BuiltinType(builtin_type_hir))
    }

    fn process_macro(&mut self, _macro_hir: hir::Macro) -> Option<Node> {
        None
    }

    fn simple_node(&mut self, moduledef_hir: hir::ModuleDef) -> Option<Node> {
        let item = Item::new(moduledef_hir, self.db, self.vfs);
        Some(Node::new(item, vec![]))
    }
}
