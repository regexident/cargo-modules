use log::trace;
use ra_ap_cfg::{CfgAtom, CfgExpr};
use ra_ap_hir::{self as hir, HasAttrs};
use ra_ap_ide_db::RootDatabase;

use crate::graph::{walker::GraphWalker, Graph, NodeIndex};

pub fn idx_of_node_with_path(
    graph: &Graph,
    path: &[String],
    _db: &RootDatabase,
) -> anyhow::Result<NodeIndex> {
    let mut node_indices = graph.node_indices();

    let node_idx = node_indices.find(|node_idx| {
        let node = &graph[*node_idx];
        node.path == path
    });

    node_idx.ok_or_else(|| anyhow::anyhow!("No node found with path {:?}", path))
}

pub fn shrink_graph(graph: &mut Graph, focus_node_idx: NodeIndex, max_depth: usize) {
    let mut walker = GraphWalker::new();

    trace!(
        "Walking graph from focus node up to depth {} ...",
        max_depth
    );

    walker.walk_graph(graph, focus_node_idx, max_depth);

    graph.retain_nodes(|_graph, node_idx| walker.nodes_visited.contains(&node_idx));
}

pub(crate) fn krate_name(krate: hir::Crate, db: &RootDatabase) -> String {
    // Obtain the crate's declaration name:
    let display_name = &krate.display_name(db).unwrap();

    // Since a crate's name may contain `-` we canonicalize it by replacing with `_`:
    display_name.replace("-", "_")
}

pub(crate) fn krate(module_def: hir::ModuleDef, db: &RootDatabase) -> Option<hir::Crate> {
    module(module_def, db).map(|module| module.krate())
}

pub(crate) fn module(module_def: hir::ModuleDef, db: &RootDatabase) -> Option<hir::Module> {
    match module_def {
        hir::ModuleDef::Module(module) => Some(module),
        module_def => module_def.module(db),
    }
}

pub(crate) fn path(module_def: hir::ModuleDef, db: &RootDatabase) -> Vec<String> {
    let mut path = vec![];

    let krate = krate(module_def, db);

    // Obtain the module's krate's name (unless it's a builtin type, which have no crate):
    if let Some(krate_name) = krate.map(|krate| krate_name(krate, db)) {
        path.push(krate_name);
    }

    // Obtain the module's canonicalized name:
    if let Some(relative_canonical_path) = module_def.canonical_path(db) {
        let components = relative_canonical_path
            .split("::")
            .map(|component| component.to_owned());
        path.extend(components);
    }

    assert!(!path.is_empty());

    path
}

// #[test] fn
// it_works() { … }
pub(crate) fn is_test_function(function: hir::Function, db: &RootDatabase) -> bool {
    let attrs = function.attrs(db);
    attrs.by_key("test").exists()
}

// #[cfg(test)]
// mod tests() { … }
pub(crate) fn is_test_module(module: hir::Module, db: &RootDatabase) -> bool {
    match module.attrs(db).cfg() {
        Some(cfg) => is_test_cfg(cfg),
        None => false,
    }
}

pub fn is_test_cfg(cfg: CfgExpr) -> bool {
    match cfg {
        CfgExpr::Invalid => false,
        CfgExpr::Atom(atom) => match atom {
            CfgAtom::Flag(flag) => flag == "test",
            CfgAtom::KeyValue { .. } => false,
        },
        CfgExpr::All(cfgs) => cfgs.into_iter().any(is_test_cfg),
        CfgExpr::Any(cfgs) => cfgs.into_iter().any(is_test_cfg),
        CfgExpr::Not(cfg) => is_test_cfg(*cfg),
    }
}

pub fn cfgs(hir: hir::ModuleDef, db: &RootDatabase) -> Vec<CfgExpr> {
    let cfg = match cfg(hir, db) {
        Some(cfg) => cfg,
        None => return vec![],
    };

    match cfg {
        CfgExpr::Invalid => vec![],
        cfg @ CfgExpr::Atom(_) => vec![cfg.clone()],
        CfgExpr::All(cfgs) => cfgs.clone(),
        cfg @ CfgExpr::Any(_) => vec![cfg.clone()],
        cfg @ CfgExpr::Not(_) => vec![cfg.clone()],
    }
}

pub fn cfg(hir: hir::ModuleDef, db: &RootDatabase) -> Option<CfgExpr> {
    match hir {
        hir::ModuleDef::Module(r#mod) => r#mod.attrs(db).cfg(),
        hir::ModuleDef::Function(r#fn) => r#fn.attrs(db).cfg(),
        hir::ModuleDef::Adt(r#adt) => r#adt.attrs(db).cfg(),
        hir::ModuleDef::Variant(r#variant) => r#variant.attrs(db).cfg(),
        hir::ModuleDef::Const(r#const) => r#const.attrs(db).cfg(),
        hir::ModuleDef::Static(r#static) => r#static.attrs(db).cfg(),
        hir::ModuleDef::Trait(r#trait) => r#trait.attrs(db).cfg(),
        hir::ModuleDef::TypeAlias(r#type) => r#type.attrs(db).cfg(),
        hir::ModuleDef::BuiltinType(_) => None,
    }
}
