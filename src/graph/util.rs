use ra_ap_cfg::{CfgAtom, CfgExpr};
use ra_ap_hir::{self as hir, HasAttrs};
use ra_ap_ide_db::RootDatabase;

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

pub(crate) fn path(module_def: hir::ModuleDef, db: &RootDatabase) -> String {
    let krate = krate(module_def, db);

    // Obtain the module's krate's name (unless it's a builtin type, which have no crate):
    let krate_name = krate.map(|krate| krate_name(krate, db));

    // Obtain the module's canonicalized name:
    let relative_canonical_path = module_def.canonical_path(db);

    match (krate_name, relative_canonical_path) {
        (Some(krate_name), Some(relative_canonical_path)) => {
            format!("{}::{}", krate_name, relative_canonical_path)
        }
        (None, Some(relative_canonical_path)) => relative_canonical_path,
        (Some(krate_name), None) => krate_name,
        (None, None) => unreachable!(),
    }
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

fn is_test_cfg(cfg: CfgExpr) -> bool {
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
