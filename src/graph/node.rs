// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use hir::ModuleDef;
use ra_ap_hir::{self as hir};
use ra_ap_ide_db::RootDatabase;

pub(crate) mod attr;
pub(crate) mod visibility;

#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub krate: Option<String>,
    pub path: Vec<String>,
    pub file_path: Option<PathBuf>,
    pub hir: Option<hir::ModuleDef>,
    pub visibility: Option<visibility::NodeVisibility>,
    pub attrs: attr::NodeAttrs,
}

impl Node {
    pub fn display_name(&self) -> String {
        self.path
            .last()
            .expect("Expected path with at least one component")
            .clone()
    }

    pub fn display_path(&self) -> String {
        self.path.join("::")
    }

    pub fn crate_display_name(&self) -> String {
        self.path
            .first()
            .expect("Expected path with at least one component")
            .clone()
    }

    pub fn kind_display_name(&self, db: &RootDatabase) -> Option<String> {
        let Some(module_def) = self.hir else {
            return None;
        };

        match module_def {
            ModuleDef::Module(module_def) => {
                if module_def.is_crate_root() {
                    Some("crate".to_owned())
                } else {
                    Some("mod".to_owned())
                }
            }
            ModuleDef::Function(function_def) => {
                let mut keywords = vec![];

                if function_def.is_const(db) {
                    keywords.push("const");
                }
                if function_def.is_async(db) {
                    keywords.push("async");
                }
                if function_def.is_unsafe_to_call(db) {
                    keywords.push("unsafe");
                }

                keywords.push("fn");

                Some(keywords.join(" "))
            }
            ModuleDef::Adt(adt_def) => match adt_def {
                hir::Adt::Struct(_) => Some("struct".to_owned()),
                hir::Adt::Union(_) => Some("union".to_owned()),
                hir::Adt::Enum(_) => Some("enum".to_owned()),
            },
            ModuleDef::Variant(_) => Some("variant".to_owned()),
            ModuleDef::Const(_) => Some("const".to_owned()),
            ModuleDef::Static(_) => Some("static".to_owned()),
            ModuleDef::Trait(trait_def) => {
                let mut keywords = vec![];
                if trait_def.is_unsafe(db) {
                    keywords.push("unsafe");
                }
                keywords.push("trait");
                Some(keywords.join(" "))
            }
            ModuleDef::TraitAlias(_) => Some("trait".to_owned()),
            ModuleDef::TypeAlias(_) => Some("type".to_owned()),
            ModuleDef::BuiltinType(_) => Some("builtin".to_owned()),
            ModuleDef::Macro(_) => Some("macro".to_owned()),
        }
    }

    pub(crate) fn is_crate(&self, _db: &RootDatabase) -> bool {
        let Some(hir::ModuleDef::Module(module)) = self.hir else {
            return false;
        };

        module.is_crate_root()
    }
}
