// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use log::trace;
use petgraph::graph::{EdgeIndex, NodeIndex};
use ra_ap_hir::{self as hir, Crate, ModuleSource};
use ra_ap_ide_db::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::{
    graph::{
        edge::{Edge, EdgeKind},
        node::Node,
        util, Graph,
    },
    item::{
        attr::{ItemAttrs, ItemCfgAttr, ItemTestAttr},
        visibility::ItemVisibility,
        Item,
    },
};

use super::orphans::add_orphan_nodes_to;

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
        self.process_module(module, true)
    }

    fn process_moduledef(
        &mut self,
        moduledef_hir: hir::ModuleDef,
        is_recursive: bool,
    ) -> Option<NodeIndex> {
        match moduledef_hir {
            hir::ModuleDef::Module(module_hir) => self.process_module(module_hir, is_recursive),
            hir::ModuleDef::Function(function_hir) => {
                self.process_function(function_hir, is_recursive)
            }
            hir::ModuleDef::Adt(adt_hir) => self.process_adt(adt_hir, is_recursive),
            hir::ModuleDef::Variant(variant_hir) => self.process_variant(variant_hir, is_recursive),
            hir::ModuleDef::Const(const_hir) => self.process_const(const_hir, is_recursive),
            hir::ModuleDef::Static(static_hir) => self.process_static(static_hir, is_recursive),
            hir::ModuleDef::Trait(trait_hir) => self.process_trait(trait_hir, is_recursive),
            hir::ModuleDef::TraitAlias(trait_alias_hir) => {
                self.process_trait_alias(trait_alias_hir, is_recursive)
            }
            hir::ModuleDef::TypeAlias(type_alias_hir) => {
                self.process_type_alias(type_alias_hir, is_recursive)
            }
            hir::ModuleDef::BuiltinType(builtin_type_hir) => {
                self.process_builtin_type(builtin_type_hir, is_recursive)
            }
            hir::ModuleDef::Macro(macro_hir) => self.process_macro(macro_hir, is_recursive),
        }
    }

    fn process_module(&mut self, module_hir: hir::Module, is_recursive: bool) -> Option<NodeIndex> {
        let module_idx = match self.add_node(module_hir.into()) {
            Some(owned_idx) => owned_idx,
            None => return None,
        };

        if !is_recursive {
            return Some(module_idx);
        }

        for declaration in module_hir.declarations(self.db) {
            let Some(declaration_idx) = self.process_moduledef(declaration, is_recursive) else {
                continue;
            };

            let edge = Edge {
                kind: EdgeKind::Owns,
            };

            self.add_edge(module_idx, declaration_idx, edge);
        }

        if self.options.orphans {
            add_orphan_nodes_to(&mut self.graph, module_idx);
        }

        self.add_dependencies(module_idx, self.dependencies_of_module(module_hir));

        Some(module_idx)
    }

    fn process_function(
        &mut self,
        function_hir: hir::Function,
        _is_recursive: bool,
    ) -> Option<NodeIndex> {
        self.add_node(hir::ModuleDef::Function(function_hir))
    }

    fn process_adt(&mut self, adt_hir: hir::Adt, is_recursive: bool) -> Option<NodeIndex> {
        match adt_hir {
            hir::Adt::Struct(struct_hir) => self.process_struct(struct_hir, is_recursive),
            hir::Adt::Union(union_hir) => self.process_union(union_hir, is_recursive),
            hir::Adt::Enum(enum_hir) => self.process_enum(enum_hir, is_recursive),
        }
    }

    fn process_struct(
        &mut self,
        struct_hir: hir::Struct,
        _is_recursive: bool,
    ) -> Option<NodeIndex> {
        self.add_node(hir::ModuleDef::Adt(hir::Adt::Struct(struct_hir)))
    }

    fn process_union(&mut self, union_hir: hir::Union, _is_recursive: bool) -> Option<NodeIndex> {
        self.add_node(hir::ModuleDef::Adt(hir::Adt::Union(union_hir)))
    }

    fn process_enum(&mut self, enum_hir: hir::Enum, _is_recursive: bool) -> Option<NodeIndex> {
        self.add_node(hir::ModuleDef::Adt(hir::Adt::Enum(enum_hir)))
    }

    fn process_variant(
        &mut self,
        _variant_hir: hir::Variant,
        _is_recursive: bool,
    ) -> Option<NodeIndex> {
        None
    }

    fn process_const(&mut self, _const_hir: hir::Const, _is_recursive: bool) -> Option<NodeIndex> {
        None
    }

    fn process_static(
        &mut self,
        static_hir: hir::Static,
        _is_recursive: bool,
    ) -> Option<NodeIndex> {
        self.add_node(hir::ModuleDef::Static(static_hir))
    }

    fn process_trait(&mut self, trait_hir: hir::Trait, _is_recursive: bool) -> Option<NodeIndex> {
        self.add_node(hir::ModuleDef::Trait(trait_hir))
    }

    fn process_trait_alias(
        &mut self,
        trait_alias_hir: hir::TraitAlias,
        _is_recursive: bool,
    ) -> Option<NodeIndex> {
        self.add_node(hir::ModuleDef::TraitAlias(trait_alias_hir))
    }

    fn process_type_alias(
        &mut self,
        type_alias_hir: hir::TypeAlias,
        _is_recursive: bool,
    ) -> Option<NodeIndex> {
        self.add_node(hir::ModuleDef::TypeAlias(type_alias_hir))
    }

    fn process_builtin_type(
        &mut self,
        builtin_type_hir: hir::BuiltinType,
        _is_recursive: bool,
    ) -> Option<NodeIndex> {
        self.add_node(hir::ModuleDef::BuiltinType(builtin_type_hir))
    }

    fn process_macro(&mut self, _macro_hir: hir::Macro, _is_recursive: bool) -> Option<NodeIndex> {
        None
    }

    fn add_dependencies(&mut self, depender_idx: NodeIndex, dependencies: Vec<hir::ModuleDef>) {
        for dependency_hir in dependencies {
            let Some(dependency_hir) = self.process_moduledef(dependency_hir, false) else {
                continue;
            };

            let edge = Edge {
                kind: EdgeKind::Uses,
            };

            self.add_edge(depender_idx, dependency_hir, edge);
        }
    }

    fn dependencies_of_module(&self, module_hir: hir::Module) -> Vec<hir::ModuleDef> {
        module_hir
            .scope(self.db, None)
            .into_iter()
            .filter_map(move |(_name, scope_hir)| {
                let hir::ScopeDef::ModuleDef(scope_module_hir) = scope_hir else {
                    // Skip everything but module-defs:
                    return None;
                };

                // Check if definition is a child of `module`:
                if scope_module_hir.module(self.db) == Some(module_hir) {
                    // Is a child, omit it:
                    return None;
                }

                Some(scope_module_hir)
            })
            .collect()
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
                let node = match self.node_weight(moduledef_hir) {
                    Some(node) => node,
                    None => return None,
                };

                let node_idx = self.graph.add_node(node);
                self.nodes.insert(node_id, node_idx);

                Some(node_idx)
            }
        }
    }

    fn add_edge(&mut self, source_idx: NodeIndex, target_idx: NodeIndex, edge: Edge) -> EdgeIndex {
        let edge_id = (source_idx, edge.kind, target_idx);

        // trace!(
        //     "Adding edge: {:?} --({:?})-> {:?}",
        //     edge_id.0,
        //     edge_id.1,
        //     edge_id.2
        // );

        // Check if we already added an equivalent edge:
        match self.edges.get(&edge_id) {
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
        }
    }

    fn node_weight(&self, moduledef_hir: hir::ModuleDef) -> Option<Node> {
        let krate = {
            let krate = util::krate(moduledef_hir, self.db);
            krate.map(|krate| util::krate_name(krate, self.db))
        };

        let path: Vec<_> = util::path(moduledef_hir, self.db)
            .split("::")
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_owned())
                }
            })
            .collect();

        let file_path = {
            match moduledef_hir {
                hir::ModuleDef::Module(module) => Some(module),
                _ => None,
            }
            .and_then(|module| {
                self.module_file(module.definition_source(self.db))
                    .map(Into::into)
            })
        };

        match moduledef_hir {
            hir::ModuleDef::Module(_) => {}
            hir::ModuleDef::Function(_) => {}
            hir::ModuleDef::Adt(_) => {}
            hir::ModuleDef::Variant(_) => return None,
            hir::ModuleDef::Const(_) => {}
            hir::ModuleDef::Static(_) => {}
            hir::ModuleDef::Trait(_) => {}
            hir::ModuleDef::TraitAlias(_) => {}
            hir::ModuleDef::TypeAlias(_) => {}
            hir::ModuleDef::BuiltinType(_) => {}
            hir::ModuleDef::Macro(_) => return None,
        };

        let hir = Some(moduledef_hir);

        let visibility = Some(ItemVisibility::new(moduledef_hir, self.db));

        let attrs = {
            let cfgs: Vec<_> = self.cfg_attrs(moduledef_hir);
            let test = self.test_attr(moduledef_hir);
            ItemAttrs { cfgs, test }
        };

        let item = Item {
            krate,
            path,
            file_path,
            hir,
            visibility,
            attrs,
        };

        Some(Node::new(item))
    }

    fn cfg_attrs(&self, moduledef_hir: hir::ModuleDef) -> Vec<ItemCfgAttr> {
        util::cfgs(moduledef_hir, self.db)
            .into_iter()
            .filter_map(ItemCfgAttr::new)
            .collect()
    }

    fn test_attr(&self, moduledef_hir: hir::ModuleDef) -> Option<ItemTestAttr> {
        let function = match moduledef_hir {
            hir::ModuleDef::Function(function) => function,
            _ => return None,
        };

        if util::is_test_function(function, self.db) {
            Some(ItemTestAttr)
        } else {
            None
        }
    }

    fn module_file(&self, module_source: hir::InFile<hir::ModuleSource>) -> Option<PathBuf> {
        let is_file_module: bool = match &module_source.value {
            ModuleSource::SourceFile(_) => true,
            ModuleSource::Module(_) => false,
            ModuleSource::BlockExpr(_) => false,
        };

        if !is_file_module {
            return None;
        }

        let file_id = module_source.file_id.original_file(self.db);
        let vfs_path = self.vfs.file_path(file_id);
        let abs_path = vfs_path.as_path().expect("Could not convert to path");

        let path: &Path = abs_path.as_ref();

        let file_extension = path.extension().and_then(|ext| ext.to_str());

        if file_extension != Some("rs") {
            return None;
        }

        Some(path.to_owned())
    }
}
