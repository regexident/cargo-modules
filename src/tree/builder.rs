// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide, Edition};

use crate::{item::Item, tree::Tree};

type Node = Item;

#[derive(Debug)]
pub struct TreeBuilder<'a> {
    db: &'a ide::RootDatabase,
    krate: hir::Crate,
}

impl<'a> TreeBuilder<'a> {
    pub fn new(db: &'a ide::RootDatabase, krate: hir::Crate) -> Self {
        Self { db, krate }
    }

    pub fn build(mut self) -> anyhow::Result<Tree<Node>> {
        let _span = tracing::trace_span!("target").entered();

        let tree = self
            .process_crate(self.krate)
            .expect("Expected tree node for crate root module");

        Ok(tree)
    }

    fn process_crate(&mut self, crate_hir: hir::Crate) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "crate",
            crate = crate_hir
                .display_name(self.db)
                .map(|name| name.to_string())
                .unwrap_or_else(|| "<ANONYMOUS>".to_owned())
        )
        .entered();

        let module = crate_hir.root_module();

        self.process_module(module)
    }

    fn process_impl(&mut self, impl_hir: hir::Impl) -> Vec<Tree<Node>> {
        let _span = tracing::trace_span!("impl").entered();

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

    fn process_moduledef(&mut self, module_def_hir: hir::ModuleDef) -> Option<Tree<Node>> {
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

    fn process_module(&mut self, module_hir: hir::Module) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "module",
            module = module_hir
                .name(self.db)
                .map(|name| name.display(self.db, Edition::CURRENT).to_string())
                .unwrap_or_else(|| "<ROOT>".to_owned())
        )
        .entered();

        let item = Item::new(hir::ModuleDef::Module(module_hir));
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

    fn process_function(&mut self, function_hir: hir::Function) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "function",
            function = function_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        self.simple_node(hir::ModuleDef::Function(function_hir))
    }

    fn process_adt(&mut self, adt_hir: hir::Adt) -> Option<Tree<Node>> {
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

    fn process_struct(&mut self, struct_hir: hir::Struct) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "struct",
            struct = struct_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Struct(struct_hir)))
    }

    fn process_enum(&mut self, enum_hir: hir::Enum) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "enum",
            enum = enum_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Enum(enum_hir)))
    }

    fn process_union(&mut self, union_hir: hir::Union) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "union",
            union = union_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        self.simple_node(hir::ModuleDef::Adt(hir::Adt::Union(union_hir)))
    }

    fn process_variant(&mut self, variant_hir: hir::Variant) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "variant",
            variant = variant_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        None
    }

    fn process_const(&mut self, const_hir: hir::Const) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "const",
            const = const_hir
                .name(self.db)
                .map(|name| name.display(self.db, Edition::CURRENT).to_string())
                .unwrap_or_else(|| "_".to_owned())
        )
        .entered();

        None
    }

    fn process_static(&mut self, static_hir: hir::Static) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "static",
            static = static_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        self.simple_node(hir::ModuleDef::Static(static_hir))
    }

    fn process_trait(&mut self, trait_hir: hir::Trait) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!("trait",
            trait = trait_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string())
        .entered();

        self.simple_node(hir::ModuleDef::Trait(trait_hir))
    }

    fn process_trait_alias(&mut self, trait_alias_hir: hir::TraitAlias) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "trait alias",
            trait_alias = trait_alias_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        self.simple_node(hir::ModuleDef::TraitAlias(trait_alias_hir))
    }

    fn process_type_alias(&mut self, type_alias_hir: hir::TypeAlias) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "type alias",
            type_alias = type_alias_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        self.simple_node(hir::ModuleDef::TypeAlias(type_alias_hir))
    }

    fn process_builtin_type(&mut self, builtin_type_hir: hir::BuiltinType) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!(
            "builtin type",
            builtin_type = builtin_type_hir
                .name()
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        self.simple_node(hir::ModuleDef::BuiltinType(builtin_type_hir))
    }

    fn process_macro(&mut self, macro_hir: hir::Macro) -> Option<Tree<Node>> {
        let _span = tracing::trace_span!("macro",
            macro = macro_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string())
        .entered();

        let _ = macro_hir;

        None
    }

    fn simple_node(&mut self, module_def_hir: hir::ModuleDef) -> Option<Tree<Node>> {
        let item = Item::new(module_def_hir);
        Some(Tree::new(item, vec![]))
    }
}
