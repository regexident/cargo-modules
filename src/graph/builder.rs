// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::{HashMap, HashSet};

use ra_ap_hir::{self as hir};
use ra_ap_hir_def::{self as hir_def};
use ra_ap_hir_ty::{self as hir_ty, TyExt as _, db::HirDatabase as _, from_assoc_type_id};
use ra_ap_ide::{self as ide, Edition};

use petgraph::graph::{EdgeIndex, NodeIndex};

use crate::{
    analyzer,
    graph::{Edge, Graph, Node, Relationship},
    item::Item,
};

#[allow(unused)]
#[derive(Debug, Hash, Eq, PartialEq)]
struct Dependency {
    source_idx: NodeIndex,
    target_hir: hir::ModuleDef,
}

#[derive(Debug)]
pub struct GraphBuilder<'a> {
    db: &'a ide::RootDatabase,
    edition: ide::Edition,
    krate: hir::Crate,
    graph: Graph<Node, Edge>,
    nodes: HashMap<hir::ModuleDef, NodeIndex>,
    edges: HashMap<(NodeIndex, Relationship, NodeIndex), EdgeIndex>,
}

impl<'a> GraphBuilder<'a> {
    pub fn new(db: &'a ide::RootDatabase, edition: ide::Edition, krate: hir::Crate) -> Self {
        let graph = Graph::default();
        let nodes = HashMap::default();
        let edges = HashMap::default();

        Self {
            db,
            edition,
            krate,
            graph,
            nodes,
            edges,
        }
    }

    pub fn build(mut self) -> anyhow::Result<(Graph<Node, Edge>, NodeIndex)> {
        let _span = tracing::trace_span!("Scanning project...").entered();

        let node_idx = self
            .process_crate(self.krate)
            .expect("graph node for crate root module");

        Ok((self.graph, node_idx))
    }

    fn process_crate(&mut self, crate_hir: hir::Crate) -> Option<NodeIndex> {
        let _span = tracing::trace_span!(
            "crate",
            crate = crate_hir
                .display_name(self.db)
                .map(|name| name.to_string())
                .unwrap_or_else(|| "<ANONYMOUS>".to_owned())
        )
        .entered();

        let module = crate_hir.root_module();

        let node_idx = self.process_moduledef(module.into());

        for impl_hir in hir::Impl::all_in_crate(self.db, crate_hir) {
            let impl_ty = impl_hir.self_ty(self.db);

            let impl_ty_hir = if let Some(adt_hir) = impl_ty.as_adt() {
                Some(hir::ModuleDef::Adt(adt_hir))
            } else {
                impl_ty.as_builtin().map(hir::ModuleDef::BuiltinType)
            };

            let Some(impl_ty_hir) = impl_ty_hir else {
                continue;
            };

            let Some(&impl_ty_idx) = self.nodes.get(&impl_ty_hir) else {
                let ty_path = analyzer::display_path(impl_ty_hir, self.db, self.edition);
                tracing::debug!("Could not find node for type {ty_path:?}, skipping impl.");
                continue;
            };

            for impl_item_idx in self.process_impl(impl_hir) {
                self.add_edge(impl_ty_idx, impl_item_idx, Edge::Owns);
            }
        }

        node_idx
    }

    fn process_impl(&mut self, impl_hir: hir::Impl) -> Vec<NodeIndex> {
        let _span = tracing::trace_span!("impl").entered();

        impl_hir
            .items(self.db)
            .into_iter()
            .filter_map(|item| {
                let mut dependencies: HashSet<_> = HashSet::default();

                let mut push_dependencies = |module_def_hir| {
                    dependencies.insert(module_def_hir);
                };

                let item_idx = match item {
                    hir::AssocItem::Function(function_hir) => {
                        self.process_function(function_hir, &mut push_dependencies)
                    }
                    hir::AssocItem::Const(const_hir) => {
                        self.process_const(const_hir, &mut push_dependencies)
                    }
                    hir::AssocItem::TypeAlias(type_alias_hir) => {
                        self.process_type_alias(type_alias_hir, &mut push_dependencies)
                    }
                };

                if let Some(item_idx) = item_idx {
                    self.add_dependencies(item_idx, dependencies.clone());
                }

                item_idx
            })
            .collect()
    }

    fn process_moduledef(&mut self, module_def_hir: hir::ModuleDef) -> Option<NodeIndex> {
        let mut dependencies: HashSet<_> = HashSet::default();

        let mut push_dependencies = |module_def_hir| {
            dependencies.insert(module_def_hir);
        };

        let node_idx = match module_def_hir {
            hir::ModuleDef::Module(module_hir) => {
                self.process_module(module_hir, &mut push_dependencies)
            }
            hir::ModuleDef::Function(function_hir) => {
                self.process_function(function_hir, &mut push_dependencies)
            }
            hir::ModuleDef::Adt(adt_hir) => self.process_adt(adt_hir, &mut push_dependencies),
            hir::ModuleDef::Variant(variant_hir) => {
                self.process_variant(variant_hir, &mut push_dependencies)
            }
            hir::ModuleDef::Const(const_hir) => {
                self.process_const(const_hir, &mut push_dependencies)
            }
            hir::ModuleDef::Static(static_hir) => {
                self.process_static(static_hir, &mut push_dependencies)
            }
            hir::ModuleDef::Trait(trait_hir) => {
                self.process_trait(trait_hir, &mut push_dependencies)
            }
            hir::ModuleDef::TraitAlias(trait_alias_hir) => {
                self.process_trait_alias(trait_alias_hir, &mut push_dependencies)
            }
            hir::ModuleDef::TypeAlias(type_alias_hir) => {
                self.process_type_alias(type_alias_hir, &mut push_dependencies)
            }
            hir::ModuleDef::BuiltinType(builtin_type_hir) => {
                self.process_builtin_type(builtin_type_hir, &mut push_dependencies)
            }
            hir::ModuleDef::Macro(macro_hir) => {
                self.process_macro(macro_hir, &mut push_dependencies)
            }
        };

        if let Some(node_idx) = node_idx.as_ref() {
            self.add_dependencies(*node_idx, dependencies.clone());
        }

        node_idx
    }

    fn process_module(
        &mut self,
        module_hir: hir::Module,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!(
            "module",
            module = module_hir
                .name(self.db)
                .map(|name| name.display(self.db, Edition::CURRENT).to_string())
                .unwrap_or_else(|| "<ROOT>".to_owned())
        )
        .entered();

        let node_idx = self.add_node_if_necessary(module_hir.into());

        if let Some(node_idx) = node_idx {
            // Process sub-items:
            for declaration in module_hir.declarations(self.db) {
                let Some(declaration_idx) = self.process_moduledef(declaration) else {
                    continue;
                };

                self.add_edge(node_idx, declaration_idx, Edge::Owns);
            }
        }

        for (_name, scope_hir) in module_hir.scope(self.db, None) {
            let hir::ScopeDef::ModuleDef(scope_module_hir) = scope_hir else {
                // Skip everything but module-defs:
                continue;
            };

            // Check if definition is a child of `module`:
            if scope_module_hir.module(self.db) == Some(module_hir) {
                // Is a child, omit it:
                continue;
            }

            dependencies_callback(scope_module_hir);
        }

        node_idx
    }

    fn process_function(
        &mut self,
        function_hir: hir::Function,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!(
            "function",
            function = function_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        let node_idx = self.add_node_if_necessary(hir::ModuleDef::Function(function_hir))?;

        for param in function_hir.params_without_self(self.db) {
            Self::walk_and_push_type(
                param.ty().strip_references(),
                self.db,
                self.edition,
                dependencies_callback,
            );
        }

        for param in function_hir.assoc_fn_params(self.db) {
            Self::walk_and_push_type(
                param.ty().strip_references(),
                self.db,
                self.edition,
                dependencies_callback,
            );
        }

        let return_type = function_hir.ret_type(self.db);
        Self::walk_and_push_type(
            return_type.strip_references(),
            self.db,
            self.edition,
            dependencies_callback,
        );

        let def_with_body = hir::DefWithBody::from(function_hir);
        let def_with_body_id: hir_def::DefWithBodyId = def_with_body.into();
        let inference_result = self.db.infer(def_with_body_id);

        for (_id, ty) in inference_result.type_of_binding.iter() {
            Self::walk_and_push_ty(ty.clone(), self.db, self.edition, dependencies_callback);
        }

        for (_id, ty) in inference_result.type_of_expr.iter() {
            Self::walk_and_push_ty(ty.clone(), self.db, self.edition, dependencies_callback);
        }

        for (_id, ty) in inference_result.type_of_for_iterator.iter() {
            Self::walk_and_push_ty(ty.clone(), self.db, self.edition, dependencies_callback);
        }

        for (_id, ty) in inference_result.type_of_pat.iter() {
            Self::walk_and_push_ty(ty.clone(), self.db, self.edition, dependencies_callback);
        }

        for (_id, ty) in inference_result.type_of_rpit.iter() {
            Self::walk_and_push_ty(ty.clone(), self.db, self.edition, dependencies_callback);
        }

        Some(node_idx)
    }

    fn process_adt(
        &mut self,
        adt_hir: hir::Adt,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        match adt_hir {
            hir::Adt::Struct(struct_hir) => self.process_struct(struct_hir, dependencies_callback),
            hir::Adt::Enum(enum_hir) => self.process_enum(enum_hir, dependencies_callback),
            hir::Adt::Union(union_hir) => self.process_union(union_hir, dependencies_callback),
        }
    }

    fn process_struct(
        &mut self,
        struct_hir: hir::Struct,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!("struct",
            struct = struct_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string())
        .entered();

        let node_idx =
            self.add_node_if_necessary(hir::ModuleDef::Adt(hir::Adt::Struct(struct_hir)));

        for field_hir in struct_hir.fields(self.db) {
            Self::walk_and_push_type(
                field_hir.ty(self.db).strip_references(),
                self.db,
                self.edition,
                dependencies_callback,
            );
        }

        node_idx
    }

    fn process_enum(
        &mut self,
        enum_hir: hir::Enum,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!("enum",
            enum = enum_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string())
        .entered();

        let node_idx = self.add_node_if_necessary(hir::ModuleDef::Adt(hir::Adt::Enum(enum_hir)));

        for variant_hir in enum_hir.variants(self.db) {
            for field_hir in variant_hir.fields(self.db) {
                Self::walk_and_push_type(
                    field_hir.ty(self.db).strip_references(),
                    self.db,
                    self.edition,
                    dependencies_callback,
                );
            }
        }

        node_idx
    }

    fn process_union(
        &mut self,
        union_hir: hir::Union,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!(
            "union",
            union = union_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        let node_idx = self.add_node_if_necessary(hir::ModuleDef::Adt(hir::Adt::Union(union_hir)));

        for field_hir in union_hir.fields(self.db) {
            Self::walk_and_push_type(
                field_hir.ty(self.db).strip_references(),
                self.db,
                self.edition,
                dependencies_callback,
            );
        }

        node_idx
    }

    fn process_variant(
        &mut self,
        variant_hir: hir::Variant,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!(
            "variant",
            variant = variant_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        for field_hir in variant_hir.fields(self.db) {
            Self::walk_and_push_type(
                field_hir.ty(self.db),
                self.db,
                self.edition,
                dependencies_callback,
            );
        }

        None
    }

    fn process_const(
        &mut self,
        const_hir: hir::Const,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!("const",
            const = const_hir
                .name(self.db)
                .map(|name| name.display(self.db, Edition::CURRENT).to_string())
                .unwrap_or_else(|| "_".to_owned()))
        .entered();

        Self::walk_and_push_type(
            const_hir.ty(self.db),
            self.db,
            self.edition,
            dependencies_callback,
        );

        None
    }

    fn process_static(
        &mut self,
        static_hir: hir::Static,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!("static",
            static = static_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string())
        .entered();

        Self::walk_and_push_type(
            static_hir.ty(self.db),
            self.db,
            self.edition,
            dependencies_callback,
        );

        None
    }

    fn process_trait(
        &mut self,
        trait_hir: hir::Trait,
        _dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!("trait",
            trait = trait_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string())
        .entered();

        let node_idx = self.add_node_if_necessary(hir::ModuleDef::Trait(trait_hir));

        // TODO: walk types?

        #[allow(clippy::let_and_return)]
        node_idx
    }

    fn process_trait_alias(
        &mut self,
        trait_alias_hir: hir::TraitAlias,
        _dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!(
            "trait alias",
            trait_alias = trait_alias_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        let node_idx = self.add_node_if_necessary(hir::ModuleDef::TraitAlias(trait_alias_hir));

        // TODO: walk types?

        #[allow(clippy::let_and_return)]
        node_idx
    }

    fn process_type_alias(
        &mut self,
        type_alias_hir: hir::TypeAlias,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!(
            "type alias",
            type_alias = type_alias_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        let node_idx = self.add_node_if_necessary(hir::ModuleDef::TypeAlias(type_alias_hir));

        Self::walk_and_push_type(
            type_alias_hir.ty(self.db),
            self.db,
            self.edition,
            dependencies_callback,
        );

        node_idx
    }

    fn process_builtin_type(
        &mut self,
        builtin_type_hir: hir::BuiltinType,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!(
            "builtin type",
            builtin_type = builtin_type_hir
                .name()
                .display(self.db, Edition::CURRENT)
                .to_string()
        )
        .entered();

        let node_idx = self.add_node_if_necessary(hir::ModuleDef::BuiltinType(builtin_type_hir));

        Self::walk_and_push_type(
            builtin_type_hir.ty(self.db),
            self.db,
            self.edition,
            dependencies_callback,
        );

        node_idx
    }

    fn process_macro(
        &mut self,
        macro_hir: hir::Macro,
        _dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let _span = tracing::trace_span!("macro",
            macro = macro_hir
                .name(self.db)
                .display(self.db, Edition::CURRENT)
                .to_string())
        .entered();

        // TODO: should the macro be walked, somehow?

        None
    }

    pub(super) fn walk_and_push_type(
        ty: hir::Type,
        db: &ide::RootDatabase,
        _edition: ide::Edition,
        visit: &mut dyn FnMut(hir::ModuleDef),
    ) {
        // tracing::trace!(
        //     "Walking type {ty}...",
        //     ty = ty.display(db, edition).to_string()
        // );

        ty.walk(db, |ty| {
            if let Some(adt) = ty.as_adt() {
                visit(adt.into());
            } else if let Some(trait_) = ty.as_dyn_trait() {
                visit(trait_.into());
            } else if let Some(traits) = ty.as_impl_traits(db) {
                traits.for_each(|it| visit(it.into()));
            } else if let Some(trait_) = ty.as_associated_type_parent_trait(db) {
                visit(trait_.into());
            }
        });
    }

    fn walk_and_push_ty(
        ty: hir_ty::Ty,
        db: &ide::RootDatabase,
        edition: ide::Edition,
        visit: &mut dyn FnMut(hir::ModuleDef),
    ) {
        // tracing::trace!(
        //     "Walking type {ty}...",
        //     ty = ty.display(db, edition).to_string()
        // );

        use hir_ty::TyKind;

        match ty.kind(hir_ty::Interner) {
            TyKind::Adt(adt_id, substitution) => {
                let adt_hir = hir::Adt::from(adt_id.0);
                visit(hir::ModuleDef::Adt(adt_hir));
                Self::walk_and_push_substitution(substitution.clone(), db, edition, visit);
            }
            TyKind::AssociatedType(assoc_type_id, substitution) => {
                let associated_ty = db.associated_ty_data(from_assoc_type_id(*assoc_type_id));
                Self::walk_and_push_binders(
                    associated_ty.binders.binders.iter(hir_ty::Interner),
                    db,
                    edition,
                    visit,
                );
                Self::walk_and_push_substitution(substitution.clone(), db, edition, visit);
            }
            TyKind::Scalar(_scalar) => {
                let builtin = ty.as_builtin().expect("builtin type");
                let builtin_hir = hir::BuiltinType::from(builtin);
                visit(hir::ModuleDef::BuiltinType(builtin_hir));
            }
            TyKind::Tuple(_usize, substitution) => {
                Self::walk_and_push_substitution(substitution.clone(), db, edition, visit);
            }
            TyKind::Array(ty, konst) => {
                Self::walk_and_push_ty(ty.clone(), db, edition, visit);
                Self::walk_and_push_ty(konst.data(hir_ty::Interner).ty.clone(), db, edition, visit);
            }
            TyKind::Slice(ty) => {
                Self::walk_and_push_ty(ty.clone(), db, edition, visit);
            }
            TyKind::Raw(_mutability, ty) => {
                Self::walk_and_push_ty(ty.clone(), db, edition, visit);
            }
            TyKind::Ref(_mutability, _lifetime, ty) => {
                Self::walk_and_push_ty(ty.clone(), db, edition, visit);
            }
            TyKind::OpaqueType(_opaque_ty_id, substitution) => {
                Self::walk_and_push_substitution(substitution.clone(), db, edition, visit);
            }
            TyKind::FnDef(_fn_def_id, substitution) => {
                Self::walk_and_push_substitution(substitution.clone(), db, edition, visit);
            }
            TyKind::Str => {
                let builtin_hir = hir::BuiltinType::str();
                visit(hir::ModuleDef::BuiltinType(builtin_hir));
            }
            TyKind::Never => {
                // nothing to do here
            }
            TyKind::Closure(_closure_id, substitution) => {
                Self::walk_and_push_substitution(substitution.clone(), db, edition, visit);
            }
            TyKind::Coroutine(_coroutine_id, substitution) => {
                Self::walk_and_push_substitution(substitution.clone(), db, edition, visit);
            }
            TyKind::CoroutineWitness(_generator_id, substitution) => {
                Self::walk_and_push_substitution(substitution.clone(), db, edition, visit);
            }
            TyKind::Foreign(_foreign_def_id) => {
                // FIXME: Anything to do here?
            }
            TyKind::Error => {
                // nothing to do here
            }
            TyKind::Placeholder(_placeholder_index) => {
                // Do not walk the placeholder or the stack overflows in an infinite loop!
            }
            TyKind::Dyn(dyn_ty) => {
                Self::walk_and_push_binders(
                    dyn_ty.bounds.binders.iter(hir_ty::Interner),
                    db,
                    edition,
                    visit,
                );
            }
            TyKind::Alias(alias_ty) => match alias_ty {
                hir_ty::AliasTy::Projection(projection) => {
                    Self::walk_and_push_substitution(
                        projection.substitution.clone(),
                        db,
                        edition,
                        visit,
                    );
                }
                hir_ty::AliasTy::Opaque(opaque) => {
                    Self::walk_and_push_substitution(
                        opaque.substitution.clone(),
                        db,
                        edition,
                        visit,
                    );
                }
            },
            TyKind::Function(fn_pointer) => {
                Self::walk_and_push_substitution(
                    fn_pointer.substitution.0.clone(),
                    db,
                    edition,
                    visit,
                );
            }
            TyKind::BoundVar(bound_var) => {
                Self::walk_and_push_ty(bound_var.to_ty(hir_ty::Interner), db, edition, visit);
            }
            TyKind::InferenceVar(inference_var, _ty_variable_kind) => {
                Self::walk_and_push_ty(
                    inference_var.to_ty(hir_ty::Interner, hir_ty::TyVariableKind::General),
                    db,
                    edition,
                    visit,
                );
            }
        }
    }

    fn walk_and_push_substitution(
        substitution: hir_ty::Substitution,
        db: &ide::RootDatabase,
        edition: ide::Edition,
        visit: &mut dyn FnMut(hir::ModuleDef),
    ) {
        // tracing::trace!("Walking substitution {substitution:?}...");

        for ty in substitution
            .iter(hir_ty::Interner)
            .filter_map(|a| a.ty(hir_ty::Interner))
        {
            Self::walk_and_push_ty(ty.clone(), db, edition, visit);
        }
    }

    fn walk_and_push_binders<'b>(
        binders: impl Iterator<Item = &'b hir_ty::VariableKind>,
        db: &ide::RootDatabase,
        edition: ide::Edition,
        visit: &mut dyn FnMut(hir::ModuleDef),
    ) {
        for binder in binders {
            match binder {
                hir_ty::VariableKind::Ty(_ty_variable_kind) => {}
                hir_ty::VariableKind::Lifetime => {}
                hir_ty::VariableKind::Const(ty) => {
                    Self::walk_and_push_ty(ty.clone(), db, edition, visit);
                }
            }
        }
    }

    fn add_dependencies<I>(&mut self, depender_idx: NodeIndex, dependencies: I)
    where
        I: IntoIterator<Item = hir::ModuleDef>,
    {
        // tracing::trace!("Adding outgoing 'use' edges for node {depender_idx:?}...");

        for dependency_hir in dependencies {
            let Some(dependency_hir) = self.add_node_if_necessary(dependency_hir) else {
                continue;
            };

            self.add_edge(depender_idx, dependency_hir, Edge::Uses);
        }
    }

    fn add_node_if_necessary(&mut self, module_def_hir: hir::ModuleDef) -> Option<NodeIndex> {
        // tracing::trace!(
        //     "Adding node {name}...",
        //     name = module_def_hir
        //         .name(self.db)
        //         .map(|name| name.display(self.db, self.edition).to_string())
        //         .unwrap_or_default()
        // );

        // Check if we already added an equivalent node:
        match self.nodes.get(&module_def_hir) {
            Some(node_idx) => {
                // If we did indeed already process it, then retrieve its index:
                Some(*node_idx)
            }
            None => {
                // Otherwise try to add a node:
                let node = Item::new(module_def_hir);
                let node_idx = self.graph.add_node(node);
                self.nodes.insert(module_def_hir, node_idx);

                Some(node_idx)
            }
        }
    }

    fn add_edge(
        &mut self,
        source_idx: NodeIndex,
        target_idx: NodeIndex,
        edge: Edge,
    ) -> Option<EdgeIndex> {
        if source_idx == target_idx {
            return None;
        }

        let edge_id = (source_idx, edge, target_idx);

        // tracing::trace!(
        //     "Adding edge: {source_path} --({edge_name})-> {target_path}...",
        //     source_path = self.graph[source_idx].display_path(self.db, self.edition),
        //     target_path = self.graph[target_idx].display_path(self.db, self.edition),
        //     edge_name = edge.display_name(),
        // );

        // Check if we already added an equivalent edge:
        let edge_idx = match self.edges.get(&edge_id) {
            Some(edge_idx) => {
                // If we did indeed already process it, then retrieve its index:
                *edge_idx
            }
            None => {
                // Otherwise add an edge:
                let edge_idx = self.graph.add_edge(source_idx, target_idx, edge);
                self.edges.insert(edge_id, edge_idx);

                edge_idx
            }
        };

        Some(edge_idx)
    }
}
