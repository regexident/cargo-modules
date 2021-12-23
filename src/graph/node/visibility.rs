// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;

use ra_ap_hir::{self as hir, HasVisibility};
use ra_ap_ide_db::RootDatabase;

#[derive(Clone, PartialEq, Debug)]
pub enum NodeVisibility {
    Crate,
    Module(String),
    Private,
    Public,
    Super,
}

impl<'a> NodeVisibility {
    pub fn new(hir: hir::ModuleDef, db: &RootDatabase) -> Self {
        let visibility = hir.visibility(db);

        let parent_module = match hir.module(db) {
            Some(module) => module,
            None => return Self::Public,
        };

        let grandparent_module = parent_module.parent(db);
        let krate_module = parent_module.krate().root_module(db);

        match visibility {
            hir::Visibility::Module(visibility_module_id) => {
                let visibility_module = hir::Module::from(visibility_module_id);
                if visibility_module == krate_module {
                    Self::Crate
                } else if Some(visibility_module) == grandparent_module {
                    // For some reason we actually have to match against the grandparent.
                    Self::Super
                } else if visibility_module == parent_module {
                    // For some reason we actually have to match against the parent.
                    Self::Private
                } else {
                    let visibility_module_def = hir::ModuleDef::Module(visibility_module);
                    let path = visibility_module_def.canonical_path(db).unwrap();
                    Self::Module(path)
                }
            }
            hir::Visibility::Public => Self::Public,
        }
    }
}

impl fmt::Display for NodeVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeVisibility::Crate => write!(f, "pub(crate)"),
            NodeVisibility::Module(path) => write!(f, "pub(in crate::{})", path),
            NodeVisibility::Private => write!(f, "pub(self)"),
            NodeVisibility::Public => write!(f, "pub"),
            NodeVisibility::Super => write!(f, "pub(super)"),
        }
    }
}
