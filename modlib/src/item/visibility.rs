// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{cmp::Ordering, fmt};

use ra_ap_hir::{self as hir, HasVisibility as _};
use ra_ap_ide::{self as ide};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ItemVisibility {
    Crate,
    Module(String),
    Private,
    Public,
    Super,
}

impl ItemVisibility {
    pub fn new(hir: hir::ModuleDef, db: &ide::RootDatabase, edition: ide::Edition) -> Self {
        let visibility = hir.visibility(db);

        let parent_module = match hir.module(db) {
            Some(module) => module,
            None => return Self::Public,
        };

        let grandparent_module = parent_module.parent(db);
        let krate_module = parent_module.krate().root_module();

        match visibility {
            hir::Visibility::Module(visibility_module_id, _visibility_explicity) => {
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
                    let visibility_module_def_hir = hir::ModuleDef::Module(visibility_module);
                    let path = visibility_module_def_hir
                        .canonical_path(db, edition)
                        .unwrap();
                    Self::Module(path)
                }
            }
            hir::Visibility::Public => Self::Public,
        }
    }

    fn numerical_order(&self) -> isize {
        match self {
            ItemVisibility::Public => 0,
            ItemVisibility::Crate => 1,
            ItemVisibility::Module(_) => 2,
            ItemVisibility::Super => 3,
            ItemVisibility::Private => 4,
        }
    }
}

impl PartialOrd for ItemVisibility {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemVisibility {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.numerical_order().cmp(&other.numerical_order()) {
            ord @ Ordering::Less => ord,
            ord @ Ordering::Equal => match (self, other) {
                (ItemVisibility::Module(lhs), ItemVisibility::Module(rhs)) => lhs.cmp(rhs),
                _ => ord,
            },
            ord @ Ordering::Greater => ord,
        }
    }
}

impl fmt::Display for ItemVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemVisibility::Crate => write!(f, "pub(crate)"),
            ItemVisibility::Module(path) => write!(f, "pub(in crate::{path})"),
            ItemVisibility::Private => write!(f, "pub(self)"),
            ItemVisibility::Public => write!(f, "pub"),
            ItemVisibility::Super => write!(f, "pub(super)"),
        }
    }
}
