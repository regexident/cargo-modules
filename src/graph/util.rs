use ra_ap_hir::{self as hir};
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
        module_def @ _ => module_def.module(db),
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
