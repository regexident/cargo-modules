// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use hir::ModuleDef;
use log::trace;
use ra_ap_hir::{self as hir, Crate};
use ra_ap_ide_db::RootDatabase;
use ra_ap_vfs::Vfs;
use scopeguard::defer;

use crate::{
    analyzer,
    item::Item,
    structure::{
        options::Options,
        tree::{Node, Tree},
    },
};

#[derive(Debug)]
pub struct Builder<'a> {
    #[allow(dead_code)]
    options: &'a Options,
    db: &'a RootDatabase,
    vfs: &'a Vfs,
    krate: hir::Crate,
}

impl<'a> Builder<'a> {
    pub fn new(
        options: &'a Options,
        db: &'a RootDatabase,
        vfs: &'a Vfs,
        krate: hir::Crate,
    ) -> Self {
        Self {
            options,
            db,
            vfs,
            krate,
        }
    }

    pub fn build(mut self) -> anyhow::Result<Tree> {
        trace!("Scanning project...");

        defer! {
            trace!("Finished canning project.");
        }

        let tree = self
            .process_crate(self.krate)
            .expect("Expected tree node for crate root module");

        Ok(tree)
    }

    fn process_crate(&mut self, crate_hir: Crate) -> Option<Tree> {
        trace!("Processing crate {crate_hir:?}...");

        defer! {
            trace!("Finished processing impl {crate_hir:?}.");
        }

        let module = crate_hir.root_module();

        let root_node = self.process_module(module, &[]);

        root_node.map(Tree::new)
    }

    fn process_impl(&mut self, impl_hir: hir::Impl, owner_path: &[String]) -> Vec<Node> {
        impl_hir
            .items(self.db)
            .into_iter()
            .filter_map(|item| match item {
                hir::AssocItem::Function(function_hir) => {
                    self.process_function(function_hir, owner_path)
                }
                hir::AssocItem::Const(const_hir) => self.process_const(const_hir, owner_path),
                hir::AssocItem::TypeAlias(type_alias_hir) => {
                    self.process_type_alias(type_alias_hir, owner_path)
                }
            })
            .collect()
    }

    fn process_moduledef(
        &mut self,
        moduledef_hir: hir::ModuleDef,
        owner_path: &[String],
    ) -> Option<Node> {
        trace!("Processing moduledef {moduledef_hir:?}...");

        defer! {
            trace!("Finished processing moduledef {moduledef_hir:?}.");
        }

        match moduledef_hir {
            hir::ModuleDef::Module(module_hir) => self.process_module(module_hir, owner_path),
            hir::ModuleDef::Function(function_hir) => {
                self.process_function(function_hir, owner_path)
            }
            hir::ModuleDef::Adt(adt_hir) => self.process_adt(adt_hir, owner_path),
            hir::ModuleDef::Variant(variant_hir) => self.process_variant(variant_hir, owner_path),
            hir::ModuleDef::Const(const_hir) => self.process_const(const_hir, owner_path),
            hir::ModuleDef::Static(static_hir) => self.process_static(static_hir, owner_path),
            hir::ModuleDef::Trait(trait_hir) => self.process_trait(trait_hir, owner_path),
            hir::ModuleDef::TraitAlias(trait_alias_hir) => {
                self.process_trait_alias(trait_alias_hir, owner_path)
            }
            hir::ModuleDef::TypeAlias(type_alias_hir) => {
                self.process_type_alias(type_alias_hir, owner_path)
            }
            hir::ModuleDef::BuiltinType(builtin_type_hir) => {
                self.process_builtin_type(builtin_type_hir, owner_path)
            }
            hir::ModuleDef::Macro(macro_hir) => self.process_macro(macro_hir, owner_path),
        }
    }

    fn process_module(&mut self, module_hir: hir::Module, owner_path: &[String]) -> Option<Node> {
        trace!("Processing module {module_hir:?}...");

        defer! {
            trace!("Finished processing module {module_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, module_hir, self.db);

        let item = Item::new(
            ModuleDef::Module(module_hir),
            path.clone(),
            self.db,
            self.vfs,
        );
        let mut node = Node::new(item, vec![]);

        let subnodes = module_hir
            .declarations(self.db)
            .into_iter()
            .filter_map(|moduledef_hir| self.process_moduledef(moduledef_hir, &path[..]));

        for subnode in subnodes {
            node.push_subnode(subnode);
        }

        Some(node)
    }

    fn process_function(
        &mut self,
        function_hir: hir::Function,
        owner_path: &[String],
    ) -> Option<Node> {
        trace!("Processing function {function_hir:?}...");

        defer! {
            trace!("Finished processing function {function_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, function_hir, self.db);

        self.simple_node(hir::ModuleDef::Function(function_hir), path)
    }

    fn process_adt(&mut self, adt_hir: hir::Adt, owner_path: &[String]) -> Option<Node> {
        trace!("Processing adt {adt_hir:?}...");

        defer! {
            trace!("Finished processing adt {adt_hir:?}.");
        }

        let mut node = match adt_hir {
            hir::Adt::Struct(struct_hir) => self.process_struct(struct_hir, owner_path),
            hir::Adt::Union(union_hir) => self.process_union(union_hir, owner_path),
            hir::Adt::Enum(enum_hir) => self.process_enum(enum_hir, owner_path),
        };

        let path = analyzer::path_appending(owner_path, adt_hir, self.db);

        if let Some(node) = node.as_mut() {
            for impl_hir in hir::Impl::all_for_type(self.db, adt_hir.ty(self.db)) {
                for subnode in self.process_impl(impl_hir, &path[..]) {
                    node.push_subnode(subnode);
                }
            }
        }

        node
    }

    fn process_struct(&mut self, struct_hir: hir::Struct, owner_path: &[String]) -> Option<Node> {
        trace!("Processing struct {struct_hir:?}...");

        defer! {
            trace!("Finished processing struct {struct_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, struct_hir, self.db);

        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Struct(struct_hir)), path)
    }

    fn process_enum(&mut self, enum_hir: hir::Enum, owner_path: &[String]) -> Option<Node> {
        trace!("Processing enum {enum_hir:?}...");

        defer! {
            trace!("Finished processing enum {enum_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, enum_hir, self.db);

        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Enum(enum_hir)), path)
    }

    fn process_union(&mut self, union_hir: hir::Union, owner_path: &[String]) -> Option<Node> {
        trace!("Processing union {union_hir:?}...");

        defer! {
            trace!("Finished processing union {union_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, union_hir, self.db);

        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Union(union_hir)), path)
    }

    fn process_variant(
        &mut self,
        variant_hir: hir::Variant,
        _owner_path: &[String],
    ) -> Option<Node> {
        trace!("Processing variant {variant_hir:?}...");

        defer! {
            trace!("Finished processing variant {variant_hir:?}.");
        }

        None
    }

    fn process_const(&mut self, const_hir: hir::Const, _owner_path: &[String]) -> Option<Node> {
        trace!("Processing const {const_hir:?}...");

        defer! {
            trace!("Finished processing const {const_hir:?}.");
        }

        None
    }

    fn process_static(&mut self, static_hir: hir::Static, owner_path: &[String]) -> Option<Node> {
        trace!("Processing static {static_hir:?}...");

        defer! {
            trace!("Finished processing static {static_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, static_hir, self.db);

        self.simple_node(hir::ModuleDef::Static(static_hir), path)
    }

    fn process_trait(&mut self, trait_hir: hir::Trait, owner_path: &[String]) -> Option<Node> {
        trace!("Processing trait {trait_hir:?}...");

        defer! {
            trace!("Finished processing trait {trait_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, trait_hir, self.db);

        self.simple_node(hir::ModuleDef::Trait(trait_hir), path)
    }

    fn process_trait_alias(
        &mut self,
        trait_alias_hir: hir::TraitAlias,
        owner_path: &[String],
    ) -> Option<Node> {
        trace!("Processing trait alias {trait_alias_hir:?}...");

        defer! {
            trace!("Finished processing trait alias {trait_alias_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, trait_alias_hir, self.db);

        self.simple_node(hir::ModuleDef::TraitAlias(trait_alias_hir), path)
    }

    fn process_type_alias(
        &mut self,
        type_alias_hir: hir::TypeAlias,
        owner_path: &[String],
    ) -> Option<Node> {
        trace!("Processing type alias {type_alias_hir:?}...");

        defer! {
            trace!("Finished processing type alias {type_alias_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, type_alias_hir, self.db);

        self.simple_node(hir::ModuleDef::TypeAlias(type_alias_hir), path)
    }

    fn process_builtin_type(
        &mut self,
        builtin_type_hir: hir::BuiltinType,
        owner_path: &[String],
    ) -> Option<Node> {
        trace!("Processing builtin type {builtin_type_hir:?}...");

        defer! {
            trace!("Finished processing builtin type {builtin_type_hir:?}.");
        }

        let path = analyzer::path_appending(owner_path, builtin_type_hir, self.db);

        self.simple_node(hir::ModuleDef::BuiltinType(builtin_type_hir), path)
    }

    fn process_macro(&mut self, macro_hir: hir::Macro, _owner_path: &[String]) -> Option<Node> {
        trace!("Processing macro {macro_hir:?}...");

        defer! {
            trace!("Finished processing macro {macro_hir:?}.");
        }

        None
    }

    fn simple_node(&mut self, moduledef_hir: hir::ModuleDef, path: Vec<String>) -> Option<Node> {
        let item = Item::new(moduledef_hir, path, self.db, self.vfs);
        Some(Node::new(item, vec![]))
    }
}
