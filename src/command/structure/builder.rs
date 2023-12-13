// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use hir::ModuleDef;
use log::trace;
use ra_ap_hir::{self as hir, Crate};
use ra_ap_ide_db::RootDatabase;
use scopeguard::defer;

use crate::item::Item;

use super::{options::Options, tree::Tree};

#[derive(Debug)]
pub struct Builder<'a> {
    #[allow(dead_code)]
    options: &'a Options,
    db: &'a RootDatabase,
    krate: hir::Crate,
}

impl<'a> Builder<'a> {
    pub fn new(options: &'a Options, db: &'a RootDatabase, krate: hir::Crate) -> Self {
        Self { options, db, krate }
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

        self.process_module(module)
    }

    fn process_impl(&mut self, impl_hir: hir::Impl) -> Vec<Tree> {
        impl_hir
            .items(self.db)
            .into_iter()
            .filter_map(|item| match item {
                hir::AssocItem::Function(function_hir) => self.process_function(function_hir),
                hir::AssocItem::Const(const_hir) => self.process_const(const_hir),
                hir::AssocItem::TypeAlias(type_alias_hir) => {
                    self.process_type_alias(type_alias_hir)
                }
            })
            .collect()
    }

    fn process_moduledef(&mut self, module_def_hir: hir::ModuleDef) -> Option<Tree> {
        trace!("Processing moduledef {module_def_hir:?}...");

        defer! {
            trace!("Finished processing moduledef {module_def_hir:?}.");
        }

        match module_def_hir {
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

    fn process_module(&mut self, module_hir: hir::Module) -> Option<Tree> {
        trace!("Processing module {module_hir:?}...");

        defer! {
            trace!("Finished processing module {module_hir:?}.");
        }

        let item = Item::new(ModuleDef::Module(module_hir));
        let mut node = Tree::new(item, vec![]);

        let subtrees = module_hir
            .declarations(self.db)
            .into_iter()
            .filter_map(|module_def_hir| self.process_moduledef(module_def_hir));

        for subtree in subtrees {
            node.push_subtree(subtree);
        }

        Some(node)
    }

    fn process_function(&mut self, function_hir: hir::Function) -> Option<Tree> {
        trace!("Processing function {function_hir:?}...");

        defer! {
            trace!("Finished processing function {function_hir:?}.");
        }

        self.simple_node(hir::ModuleDef::Function(function_hir))
    }

    fn process_adt(&mut self, adt_hir: hir::Adt) -> Option<Tree> {
        trace!("Processing adt {adt_hir:?}...");

        defer! {
            trace!("Finished processing adt {adt_hir:?}.");
        }

        let mut node = match adt_hir {
            hir::Adt::Struct(struct_hir) => self.process_struct(struct_hir),
            hir::Adt::Union(union_hir) => self.process_union(union_hir),
            hir::Adt::Enum(enum_hir) => self.process_enum(enum_hir),
        };

        if let Some(node) = node.as_mut() {
            for impl_hir in hir::Impl::all_for_type(self.db, adt_hir.ty(self.db)) {
                for subtree in self.process_impl(impl_hir) {
                    node.push_subtree(subtree);
                }
            }
        }

        node
    }

    fn process_struct(&mut self, struct_hir: hir::Struct) -> Option<Tree> {
        trace!("Processing struct {struct_hir:?}...");

        defer! {
            trace!("Finished processing struct {struct_hir:?}.");
        }

        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Struct(struct_hir)))
    }

    fn process_enum(&mut self, enum_hir: hir::Enum) -> Option<Tree> {
        trace!("Processing enum {enum_hir:?}...");

        defer! {
            trace!("Finished processing enum {enum_hir:?}.");
        }

        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Enum(enum_hir)))
    }

    fn process_union(&mut self, union_hir: hir::Union) -> Option<Tree> {
        trace!("Processing union {union_hir:?}...");

        defer! {
            trace!("Finished processing union {union_hir:?}.");
        }

        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Union(union_hir)))
    }

    fn process_variant(&mut self, variant_hir: hir::Variant) -> Option<Tree> {
        trace!("Processing variant {variant_hir:?}...");

        defer! {
            trace!("Finished processing variant {variant_hir:?}.");
        }

        None
    }

    fn process_const(&mut self, const_hir: hir::Const) -> Option<Tree> {
        trace!("Processing const {const_hir:?}...");

        defer! {
            trace!("Finished processing const {const_hir:?}.");
        }

        None
    }

    fn process_static(&mut self, static_hir: hir::Static) -> Option<Tree> {
        trace!("Processing static {static_hir:?}...");

        defer! {
            trace!("Finished processing static {static_hir:?}.");
        }

        self.simple_node(hir::ModuleDef::Static(static_hir))
    }

    fn process_trait(&mut self, trait_hir: hir::Trait) -> Option<Tree> {
        trace!("Processing trait {trait_hir:?}...");

        defer! {
            trace!("Finished processing trait {trait_hir:?}.");
        }

        self.simple_node(hir::ModuleDef::Trait(trait_hir))
    }

    fn process_trait_alias(&mut self, trait_alias_hir: hir::TraitAlias) -> Option<Tree> {
        trace!("Processing trait alias {trait_alias_hir:?}...");

        defer! {
            trace!("Finished processing trait alias {trait_alias_hir:?}.");
        }

        self.simple_node(hir::ModuleDef::TraitAlias(trait_alias_hir))
    }

    fn process_type_alias(&mut self, type_alias_hir: hir::TypeAlias) -> Option<Tree> {
        trace!("Processing type alias {type_alias_hir:?}...");

        defer! {
            trace!("Finished processing type alias {type_alias_hir:?}.");
        }

        self.simple_node(hir::ModuleDef::TypeAlias(type_alias_hir))
    }

    fn process_builtin_type(&mut self, builtin_type_hir: hir::BuiltinType) -> Option<Tree> {
        trace!("Processing builtin type {builtin_type_hir:?}...");

        defer! {
            trace!("Finished processing builtin type {builtin_type_hir:?}.");
        }

        self.simple_node(hir::ModuleDef::BuiltinType(builtin_type_hir))
    }

    fn process_macro(&mut self, macro_hir: hir::Macro) -> Option<Tree> {
        trace!("Processing macro {macro_hir:?}...");

        defer! {
            trace!("Finished processing macro {macro_hir:?}.");
        }

        None
    }

    fn simple_node(&mut self, module_def_hir: hir::ModuleDef) -> Option<Tree> {
        let item = Item::new(module_def_hir);
        Some(Tree::new(item, vec![]))
    }
}
