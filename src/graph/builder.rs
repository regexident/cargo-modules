// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::{HashMap, HashSet};

use log::trace;
use petgraph::graph::{EdgeIndex, NodeIndex};
use ra_ap_hir::{self as hir, Crate};
use ra_ap_ide_db::RootDatabase;
use ra_ap_vfs::Vfs;

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
    db: &'a RootDatabase,
    vfs: &'a Vfs,
    krate: hir::Crate,
    graph: Graph,
    nodes: HashMap<String, NodeIndex>,
    edges: HashMap<(NodeIndex, EdgeKind, NodeIndex), EdgeIndex>,
}

impl<'a> Builder<'a> {
    pub fn new(options: Options, db: &'a RootDatabase, vfs: &'a Vfs, krate: hir::Crate) -> Self {
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
        trace!("Scanning project ...");

        let node_idx = self
            .process_crate(self.krate)
            .expect("Expected graph node for crate root module");

        Ok((self.graph, node_idx))
    }

    fn process_crate(&mut self, krate: Crate) -> Option<NodeIndex> {
        let module = krate.root_module();

        self.process_moduledef(module.into())
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
        let node_idx = self.add_node(module_hir.into());

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
        let node_idx = self.add_node(hir::ModuleDef::Function(function_hir));

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
        let node_idx = self.add_node(hir::ModuleDef::Adt(hir::Adt::Struct(struct_hir)));

        for field_hir in struct_hir.fields(self.db) {
            util::walk_and_push_ty(
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
        let node_idx = self.add_node(hir::ModuleDef::Adt(hir::Adt::Enum(enum_hir)));

        for variant_hir in enum_hir.variants(self.db) {
            for field_hir in variant_hir.fields(self.db) {
                util::walk_and_push_ty(
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
        let node_idx = self.add_node(hir::ModuleDef::Adt(hir::Adt::Union(union_hir)));

        for field_hir in union_hir.fields(self.db) {
            util::walk_and_push_ty(
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
            util::walk_and_push_ty(field_hir.ty(self.db), self.db, dependencies_callback);
        }

        node_idx
    }

    fn process_const(
        &mut self,
        const_hir: hir::Const,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = None;

        util::walk_and_push_ty(const_hir.ty(self.db), self.db, dependencies_callback);

        node_idx
    }

    fn process_static(
        &mut self,
        static_hir: hir::Static,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = None;

        util::walk_and_push_ty(static_hir.ty(self.db), self.db, dependencies_callback);

        node_idx
    }

    fn process_trait(
        &mut self,
        trait_hir: hir::Trait,
        _dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = self.add_node(hir::ModuleDef::Trait(trait_hir));

        // TODO: walk types?

        #[allow(clippy::let_and_return)]
        node_idx
    }

    fn process_trait_alias(
        &mut self,
        trait_alias_hir: hir::TraitAlias,
        _dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = self.add_node(hir::ModuleDef::TraitAlias(trait_alias_hir));

        // TODO: walk types?

        #[allow(clippy::let_and_return)]
        node_idx
    }

    fn process_type_alias(
        &mut self,
        type_alias_hir: hir::TypeAlias,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = self.add_node(hir::ModuleDef::TypeAlias(type_alias_hir));

        util::walk_and_push_ty(type_alias_hir.ty(self.db), self.db, dependencies_callback);

        node_idx
    }

    fn process_builtin_type(
        &mut self,
        builtin_type_hir: hir::BuiltinType,
        dependencies_callback: &mut dyn FnMut(hir::ModuleDef),
    ) -> Option<NodeIndex> {
        let node_idx = self.add_node(hir::ModuleDef::BuiltinType(builtin_type_hir));

        util::walk_and_push_ty(builtin_type_hir.ty(self.db), self.db, dependencies_callback);

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

    fn add_dependencies<I>(&mut self, depender_idx: NodeIndex, dependencies: I)
    where
        I: IntoIterator<Item = hir::ModuleDef>,
    {
        for dependency_hir in dependencies {
            let Some(dependency_hir) = self.add_node(dependency_hir) else {
                continue;
            };

            let edge = Edge {
                kind: EdgeKind::Uses,
            };

            self.add_edge(depender_idx, dependency_hir, edge);
        }
    }

    fn add_node(&mut self, moduledef_hir: hir::ModuleDef) -> Option<NodeIndex> {
        let node_id = util::path(moduledef_hir, self.db);

        // trace!("Adding module node: {:?}", node_id);

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

        // trace!(
        //     "Adding edge: {:?} --({:?})-> {:?}",
        //     edge_id.0,
        //     edge_id.1,
        //     edge_id.2
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
