// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide};

use super::Item;

#[derive(Clone, Eq, PartialEq)]
pub struct ItemKindDisplayName(String);

impl ItemKindDisplayName {
    pub fn new(item: &Item, db: &ide::RootDatabase) -> Self {
        Self(match item.hir {
            hir::ModuleDef::Module(hir) => {
                if hir.is_crate_root() {
                    "crate".to_owned()
                } else {
                    "mod".to_owned()
                }
            }
            hir::ModuleDef::Function(hir) => {
                let mut keywords = vec![];
                if hir.is_const(db) {
                    keywords.push("const");
                }
                if hir.is_async(db) {
                    keywords.push("async");
                }
                let caller = None;
                // Technically this should be the caller's edition,
                // but for our purposes we should be fine with taking the
                // callee's edition instead:
                let edition = hir.module(db).krate().edition(db);
                if hir.is_unsafe_to_call(db, caller, edition) {
                    keywords.push("unsafe");
                }
                keywords.push("fn");
                keywords.join(" ")
            }
            hir::ModuleDef::Adt(hir::Adt::Struct(_hir)) => "struct".to_owned(),
            hir::ModuleDef::Adt(hir::Adt::Union(_hir)) => "union".to_owned(),
            hir::ModuleDef::Adt(hir::Adt::Enum(_hir)) => "enum".to_owned(),
            hir::ModuleDef::Variant(_hir) => "variant".to_owned(),
            hir::ModuleDef::Const(_hir) => "const".to_owned(),
            hir::ModuleDef::Static(_hir) => "static".to_owned(),
            hir::ModuleDef::Trait(hir) => {
                let mut keywords = vec![];
                if hir.is_unsafe(db) {
                    keywords.push("unsafe");
                }
                keywords.push("trait");
                keywords.join(" ")
            }
            hir::ModuleDef::TraitAlias(_hir) => "trait".to_owned(),
            hir::ModuleDef::TypeAlias(_hir) => "type".to_owned(),
            hir::ModuleDef::BuiltinType(_hir) => "builtin".to_owned(),
            hir::ModuleDef::Macro(_hir) => "macro".to_owned(),
        })
    }
}

impl fmt::Display for ItemKindDisplayName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for ItemKindDisplayName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
