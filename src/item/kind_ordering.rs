// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cmp::Ordering;

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide};

use super::Item;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum ItemKindOrdering {
    Module {
        is_crate_root: bool,
    },
    Function {
        is_const: bool,
        is_async: bool,
        is_unsafe_to_call: bool,
    },
    Struct,
    Union,
    Enum,
    Variant,
    Const,
    Static,
    Trait {
        is_unsafe: bool,
    },
    TraitAlias,
    TypeAlias,
    BuiltinType,
    Macro,
}

impl ItemKindOrdering {
    pub fn new(item: &Item, db: &ide::RootDatabase) -> Self {
        match item.hir {
            hir::ModuleDef::Module(module_def_hir) => Self::Module {
                is_crate_root: module_def_hir.is_crate_root(),
            },
            hir::ModuleDef::Function(function_def) => {
                let caller = None;
                // Technically this should be the caller's edition,
                // but for our purposes we should be fine with taking the
                // callee's edition instead:
                let edition = function_def.module(db).krate().edition(db);
                Self::Function {
                    is_const: function_def.is_const(db),
                    is_async: function_def.is_async(db),
                    is_unsafe_to_call: function_def.is_unsafe_to_call(db, caller, edition),
                }
            }
            hir::ModuleDef::Adt(adt_def) => match adt_def {
                hir::Adt::Struct(_) => Self::Struct,
                hir::Adt::Union(_) => Self::Union,
                hir::Adt::Enum(_) => Self::Enum,
            },
            hir::ModuleDef::Variant(_) => Self::Variant,
            hir::ModuleDef::Const(_) => Self::Const,
            hir::ModuleDef::Static(_) => Self::Static,
            hir::ModuleDef::Trait(trait_def) => Self::Trait {
                is_unsafe: trait_def.is_unsafe(db),
            },
            hir::ModuleDef::TraitAlias(_) => Self::TraitAlias,
            hir::ModuleDef::TypeAlias(_) => Self::TypeAlias,
            hir::ModuleDef::BuiltinType(_) => Self::BuiltinType,
            hir::ModuleDef::Macro(_) => Self::Macro,
        }
    }

    fn numerical_order(&self) -> isize {
        match self {
            Self::Module { .. } => 0,
            Self::Trait { .. } => 1,
            Self::TraitAlias => 1,
            Self::TypeAlias => 2,
            Self::Struct => 3,
            Self::Enum => 4,
            Self::Variant => 5,
            Self::Union => 6,
            Self::BuiltinType => 7,
            Self::Function { .. } => 8,
            Self::Const => 9,
            Self::Static => 10,
            Self::Macro => 11,
        }
    }
}

impl PartialOrd for ItemKindOrdering {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemKindOrdering {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.numerical_order().cmp(&other.numerical_order());

        if !ord.is_eq() {
            return ord;
        }

        match (self, other) {
            (
                Self::Module {
                    is_crate_root: lhs_is_crate_root,
                },
                Self::Module {
                    is_crate_root: rhs_is_crate_root,
                },
            ) => {
                // We want crates to be ordered first:
                let is_crate_root_ord = lhs_is_crate_root.cmp(rhs_is_crate_root).reverse();
                if !is_crate_root_ord.is_eq() {
                    return is_crate_root_ord;
                }

                Ordering::Equal
            }
            (
                Self::Function {
                    is_const: lhs_is_const,
                    is_async: lhs_is_async,
                    is_unsafe_to_call: lhs_is_unsafe_to_call,
                },
                Self::Function {
                    is_const: rhs_is_const,
                    is_async: rhs_is_async,
                    is_unsafe_to_call: rhs_is_unsafe_to_call,
                },
            ) => {
                // We want const functions to be ordered first:
                let is_const_ord = lhs_is_const.cmp(rhs_is_const).reverse();
                if !is_const_ord.is_eq() {
                    return is_const_ord;
                }

                // We want async functions to be ordered second:
                let is_async_ord = lhs_is_async.cmp(rhs_is_async).reverse();
                if !is_async_ord.is_eq() {
                    return is_async_ord;
                }

                // We want unsafe functions to be ordered third:
                let is_unsafe_to_call_ord =
                    lhs_is_unsafe_to_call.cmp(rhs_is_unsafe_to_call).reverse();
                if !is_unsafe_to_call_ord.is_eq() {
                    return is_unsafe_to_call_ord;
                }

                Ordering::Equal
            }
            (Self::Struct, _) => ord,
            (Self::Union, _) => ord,
            (Self::Enum, _) => ord,
            (Self::Variant, _) => ord,
            (Self::Const, _) => ord,
            (Self::Static, _) => ord,
            (
                Self::Trait {
                    is_unsafe: lhs_is_unsafe,
                },
                Self::Trait {
                    is_unsafe: rhs_is_unsafe,
                },
            ) => {
                // We want unsafe traits to be ordered last:
                let is_unsafe_ord = lhs_is_unsafe.cmp(rhs_is_unsafe).reverse();
                if !is_unsafe_ord.is_eq() {
                    return is_unsafe_ord;
                }

                Ordering::Equal
            }
            (Self::TraitAlias, _) => ord,
            (Self::TypeAlias, _) => ord,
            (Self::BuiltinType, _) => ord,
            (Self::Macro, _) => ord,
            _ => ord,
        }
    }
}
