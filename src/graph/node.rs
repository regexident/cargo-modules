// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use ra_ap_hir::{self as hir};
use ra_ap_ide_db::RootDatabase;

pub(crate) mod attr;
pub(crate) mod visibility;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FunctionNode {
    is_unsafe: bool,
    is_async: bool,
    is_const: bool,
}

impl FunctionNode {
    pub fn display_name(&self) -> Option<String> {
        let mut keywords = vec![];
        if self.is_const {
            keywords.push("const");
        }
        if self.is_async {
            keywords.push("async");
        }
        if self.is_unsafe {
            keywords.push("unsafe");
        }
        keywords.push("fn");
        Some(keywords.join(" "))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeNode {
    Struct,
    Union,
    Enum,
    BuiltinType,
}

impl TypeNode {
    pub fn display_name(&self) -> Option<String> {
        match self {
            Self::Struct => Some("struct".to_owned()),
            Self::Union => Some("union".to_owned()),
            Self::Enum => Some("enum".to_owned()),
            Self::BuiltinType => Some("builtin".to_owned()),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ValueNode {
    Const,
    Static,
}

impl ValueNode {
    pub fn display_name(&self) -> Option<String> {
        match self {
            Self::Const => Some("const".to_owned()),
            Self::Static => Some("static".to_owned()),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NodeKind {
    Crate,
    Function(FunctionNode),
    Module,
    Orphan,
    Trait,
    Type(TypeNode),
    TypeAlias,
    Value(ValueNode),
}

impl NodeKind {
    pub fn from(module_def: hir::ModuleDef, db: &RootDatabase) -> Option<Self> {
        match module_def {
            hir::ModuleDef::Module(module) => {
                if module == module.crate_root(db) {
                    Some(NodeKind::Crate)
                } else {
                    Some(NodeKind::Module)
                }
            }
            hir::ModuleDef::Function(function) => Some(NodeKind::Function(FunctionNode {
                is_unsafe: function.is_unsafe_to_call(db),
                is_async: function.is_async(db),
                is_const: function.is_const(db),
            })),
            hir::ModuleDef::Adt(hir::Adt::Struct(_)) => Some(NodeKind::Type(TypeNode::Struct)),
            hir::ModuleDef::Adt(hir::Adt::Union(_)) => Some(NodeKind::Type(TypeNode::Union)),
            hir::ModuleDef::Adt(hir::Adt::Enum(_)) => Some(NodeKind::Type(TypeNode::Enum)),
            hir::ModuleDef::Variant(_) => None,
            hir::ModuleDef::Const(_) => Some(NodeKind::Value(ValueNode::Const)),
            hir::ModuleDef::Static(_) => Some(NodeKind::Value(ValueNode::Static)),
            hir::ModuleDef::Trait(_) => Some(NodeKind::Trait),
            hir::ModuleDef::TypeAlias(_) => Some(NodeKind::TypeAlias),
            hir::ModuleDef::BuiltinType(_) => Some(NodeKind::Type(TypeNode::BuiltinType)),
            hir::ModuleDef::Macro(_) => None,
        }
    }

    pub fn display_name(&self) -> Option<String> {
        match self {
            Self::Crate => Some("crate".to_owned()),
            Self::Function(node) => node.display_name(),
            Self::Module => Some("mod".to_owned()),
            Self::Orphan => None,
            Self::Trait => Some("trait".to_owned()),
            Self::Type(node) => node.display_name(),
            Self::TypeAlias => Some("type".to_owned()),
            Self::Value(node) => node.display_name(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub krate: Option<String>,
    pub path: Vec<String>,
    pub file_path: Option<PathBuf>,
    pub kind: NodeKind,
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
}
