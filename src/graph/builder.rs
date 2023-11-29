// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::{HashMap, HashSet};

use petgraph::graph::{EdgeIndex, NodeIndex};
use ra_ap_hir::{self as hir};
use ra_ap_ide_db::{self as ide_db};
use ra_ap_vfs::{self as vfs};

use crate::{
    graph::{
        edge::{Edge, EdgeKind},
        node::Node,
        util, Graph,
    },
    item::Item,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Options {}

#[derive(Debug, Hash, Eq, PartialEq)]
struct Dependency {
    source_idx: NodeIndex,
    target_hir: hir::ModuleDef,
}

#[derive(Debug)]
pub struct Builder<'a> {
    #[allow(dead_code)]
    options: Options,
    db: &'a ide_db::RootDatabase,
    vfs: &'a vfs::Vfs,
    krate: hir::Crate,
    graph: Graph,
    nodes: HashMap<String, NodeIndex>,
    edges: HashMap<(NodeIndex, EdgeKind, NodeIndex), EdgeIndex>,
}

impl<'a> Builder<'a> {
    pub fn new(
        options: Options,
        db: &'a ide_db::RootDatabase,
        vfs: &'a vfs::Vfs,
        krate: hir::Crate,
    ) -> Self {
        let graph = Graph::default();
        let nodes = HashMap::default();
        let edges = HashMap::default();

        Self {
            options,
            db,
            vfs,
            krate,
            graph,
            nodes,
            edges,
        }
    }

    pub fn build(mut self) -> anyhow::Result<(Graph, NodeIndex)> {
        let node_idx = self
            .process_crate(self.krate)
            .expect("Expected graph node for crate root module");

        Ok((self.graph, node_idx))
    }

    fn process_crate(&mut self, crate_hir: hir::Crate) -> Option<NodeIndex> {
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

            let impl_ty_idx = self
                .add_node_if_necessary(impl_ty_hir)
                .expect("impl type node");

            for impl_item_idx in self.process_impl(impl_hir, impl_ty_hir) {
                let edge = Edge {
                    kind: EdgeKind::Owns,
                };

                self.add_edge(impl_ty_idx, impl_item_idx, edge);
            }
        }

        node_idx
    }

    fn process_impl(&mut self, impl_hir: hir::Impl, impl_ty_hir: hir::ModuleDef) -> Vec<NodeIndex> {
        let impl_ty_path: Vec<_> = util::path(impl_ty_hir, self.db)
            .split("::")
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_owned())
                }
            })
            .collect();

        impl_hir
            .items(self.db)
            .into_iter()
            .filter_map(|item| {
                let mut dependencies: HashSet<_> = HashSet::default();

                let mut push_dependencies = |moduledef_hir| {
                    dependencies.insert(moduledef_hir);
                };

                let Some(name) = item.name(self.db) else {
                    return None;
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
                    let node = &mut self.graph[item_idx];
                    // Calling `util::path(hir)` or `hir.canonical_path(db)` on an item from an impl
                    // returns a path anchored to the implementing type's module, rather than the type itself.
                    // So we need to fix this by asking the `impl_ty_hir` for its path and appending
                    // the item's name to that path to get the expected type-anchored path.
                    let mut fixed_path = impl_ty_path.clone();
                    fixed_path.push(format!("{}", name.display(self.db)));
                    node.item.path = fixed_path;
                }

                if let Some(item_idx) = item_idx {
                    self.add_dependencies(item_idx, dependencies.clone());
                }

                item_idx
            })
            .collect()
    }

    fn process_moduledef(&mut self, moduledef_hir: hir::ModuleDef) -> Option<NodeIndex> {
        let mut dependencies: HashSet<_> = HashSet::default();

        let mut push_dependencies = |moduledef_hir| {
            dependencies.insert(moduledef_hir);
        };

        let node_idx = match moduledef_hir {
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

        if let Some(node_idx) = node_idx {
            self.add_dependencies(node_idx, dependencies);
        }

        node_idx
    }

    fn process_module(
        &mut self,
        module_hir: hir::Module,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = self.add_node_if_necessary(module_hir.into());

        if let Some(node_idx) = node_idx {
            // Process sub-items:
            for declaration in module_hir.declarations(self.db) {
                let Some(declaration_idx) = self.process_moduledef(declaration) else {
                    continue;
                };

                let edge = Edge {
                    kind: EdgeKind::Owns,
                };

                self.add_edge(node_idx, declaration_idx, edge);
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
        _dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = self.add_node_if_necessary(hir::ModuleDef::Function(function_hir));

        // TODO: scan function for dependencies

        #[allow(clippy::let_and_return)]
        node_idx
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
        let node_idx =
            self.add_node_if_necessary(hir::ModuleDef::Adt(hir::Adt::Struct(struct_hir)));

        for field_hir in struct_hir.fields(self.db) {
            Self::walk_and_push_type(
                field_hir.ty(self.db).strip_references(),
                self.db,
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
        let node_idx = self.add_node_if_necessary(hir::ModuleDef::Adt(hir::Adt::Enum(enum_hir)));

        for variant_hir in enum_hir.variants(self.db) {
            for field_hir in variant_hir.fields(self.db) {
                Self::walk_and_push_type(
                    field_hir.ty(self.db).strip_references(),
                    self.db,
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
        let node_idx = self.add_node_if_necessary(hir::ModuleDef::Adt(hir::Adt::Union(union_hir)));

        for field_hir in union_hir.fields(self.db) {
            Self::walk_and_push_type(
                field_hir.ty(self.db).strip_references(),
                self.db,
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
        let node_idx = None;

        for field_hir in variant_hir.fields(self.db) {
            Self::walk_and_push_type(field_hir.ty(self.db), self.db, dependencies_callback);
        }

        node_idx
    }

    fn process_const(
        &mut self,
        const_hir: hir::Const,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = None;

        Self::walk_and_push_type(const_hir.ty(self.db), self.db, dependencies_callback);

        node_idx
    }

    fn process_static(
        &mut self,
        static_hir: hir::Static,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = None;

        Self::walk_and_push_type(static_hir.ty(self.db), self.db, dependencies_callback);

        node_idx
    }

    fn process_trait(
        &mut self,
        trait_hir: hir::Trait,
        _dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
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
        let node_idx = self.add_node_if_necessary(hir::ModuleDef::TypeAlias(type_alias_hir));

        Self::walk_and_push_type(type_alias_hir.ty(self.db), self.db, dependencies_callback);

        node_idx
    }

    fn process_builtin_type(
        &mut self,
        builtin_type_hir: hir::BuiltinType,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = self.add_node_if_necessary(hir::ModuleDef::BuiltinType(builtin_type_hir));

        Self::walk_and_push_type(builtin_type_hir.ty(self.db), self.db, dependencies_callback);

        node_idx
    }

    fn process_macro(
        &mut self,
        _macro_hir: hir::Macro,
        _dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = None;

        // TODO: walk types?

        #[allow(clippy::let_and_return)]
        node_idx
    }

    pub(super) fn walk_and_push_type(
        ty: hir::Type,
        db: &ide_db::RootDatabase,
        visit: &mut dyn FnMut(hir::ModuleDef),
    ) {
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

    fn add_dependencies<I>(&mut self, depender_idx: NodeIndex, dependencies: I)
    where
        I: IntoIterator<Item = hir::ModuleDef>,
    {
        for dependency_hir in dependencies {
            let Some(dependency_hir) = self.add_node_if_necessary(dependency_hir) else {
                continue;
            };

            let edge = Edge {
                kind: EdgeKind::Uses,
            };

            self.add_edge(depender_idx, dependency_hir, edge);
        }
    }

    fn add_node_if_necessary(&mut self, moduledef_hir: hir::ModuleDef) -> Option<NodeIndex> {
        let node_id = util::path(moduledef_hir, self.db);

        // Check if we already added an equivalent node:
        match self.nodes.get(&node_id) {
            Some(node_idx) => {
                // If we did indeed already process it, then retrieve its index:
                Some(*node_idx)
            }
            None => {
                // Otherwise try to add a node:
                let node = Node::new(Item::new(moduledef_hir, self.db, self.vfs));
                let node_idx = self.graph.add_node(node);
                self.nodes.insert(node_id, node_idx);

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

        let edge_id = (source_idx, edge.kind, target_idx);

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
