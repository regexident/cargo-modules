// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{cmp::Ordering, fmt};

use ra_ap_hir::{self as hir, ModuleDef};
use ra_ap_ide_db::RootDatabase;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ItemKind {
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

impl ItemKind {
    pub fn new(hir: hir::ModuleDef, db: &RootDatabase) -> Self {
        match hir {
            ModuleDef::Module(module_def) => Self::Module {
                is_crate_root: module_def.is_crate_root(),
            },
            ModuleDef::Function(function_def) => Self::Function {
                is_const: function_def.is_const(db),
                is_async: function_def.is_async(db),
                is_unsafe_to_call: function_def.is_unsafe_to_call(db),
            },
            ModuleDef::Adt(adt_def) => match adt_def {
                hir::Adt::Struct(_) => Self::Struct,
                hir::Adt::Union(_) => Self::Union,
                hir::Adt::Enum(_) => Self::Enum,
            },
            ModuleDef::Variant(_) => Self::Variant,
            ModuleDef::Const(_) => Self::Const,
            ModuleDef::Static(_) => Self::Static,
            ModuleDef::Trait(trait_def) => Self::Trait {
                is_unsafe: trait_def.is_unsafe(db),
            },
            ModuleDef::TraitAlias(_) => Self::TraitAlias,
            ModuleDef::TypeAlias(_) => Self::TypeAlias,
            ModuleDef::BuiltinType(_) => Self::BuiltinType,
            ModuleDef::Macro(_) => Self::Macro,
        }
    }

    fn numerical_order(&self) -> isize {
        match self {
            ItemKind::Module { .. } => 0,
            ItemKind::Trait { .. } => 1,
            ItemKind::TraitAlias => 1,
            ItemKind::TypeAlias => 2,
            ItemKind::Struct => 3,
            ItemKind::Enum => 4,
            ItemKind::Variant => 5,
            ItemKind::Union => 6,
            ItemKind::BuiltinType => 7,
            ItemKind::Function { .. } => 8,
            ItemKind::Const => 9,
            ItemKind::Static => 10,
            ItemKind::Macro => 11,
        }
    }
}

impl PartialOrd for ItemKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemKind {
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

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Module { is_crate_root } => {
                if *is_crate_root {
                    write!(f, "crate")
                } else {
                    write!(f, "mod")
                }
            }
            Self::Function {
                is_const,
                is_async,
                is_unsafe_to_call,
            } => {
                let mut keywords = vec![];

                if *is_const {
                    keywords.push("const");
                }
                if *is_async {
                    keywords.push("async");
                }
                if *is_unsafe_to_call {
                    keywords.push("unsafe");
                }

                keywords.push("fn");

                write!(f, "{}", keywords.join(" "))
            }
            Self::Struct => write!(f, "struct"),
            Self::Union => write!(f, "union"),
            Self::Enum => write!(f, "enum"),
            Self::Variant => write!(f, "variant"),
            Self::Const => write!(f, "const"),
            Self::Static => write!(f, "static"),
            Self::Trait { is_unsafe } => {
                let mut keywords = vec![];
                if *is_unsafe {
                    keywords.push("unsafe");
                }
                keywords.push("trait");
                write!(f, "{}", keywords.join(" "))
            }
            Self::TraitAlias => write!(f, "trait"),
            Self::TypeAlias => write!(f, "type"),
            Self::BuiltinType => write!(f, "builtin"),
            Self::Macro => write!(f, "macro"),
        }
    }
}
