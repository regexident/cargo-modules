// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use ra_ap_hir::{self as hir};
use ra_ap_ide_db::RootDatabase;

pub(crate) mod attr;
pub(crate) mod visibility;

#[derive(Clone, PartialEq, Debug)]
pub enum TypeNode {
    Struct,
    Union,
    Enum,
    BuiltinType,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ValueNode {
    Const,
    Static,
}

#[derive(Clone, PartialEq, Debug)]
pub enum NodeKind {
    Crate,
    Function,
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
            hir::ModuleDef::Function(_) => Some(NodeKind::Function),
            hir::ModuleDef::Adt(hir::Adt::Struct(_)) => Some(NodeKind::Type(TypeNode::Struct)),
            hir::ModuleDef::Adt(hir::Adt::Union(_)) => Some(NodeKind::Type(TypeNode::Union)),
            hir::ModuleDef::Adt(hir::Adt::Enum(_)) => Some(NodeKind::Type(TypeNode::Enum)),
            hir::ModuleDef::Variant(_) => None,
            hir::ModuleDef::Const(_) => Some(NodeKind::Value(ValueNode::Const)),
            hir::ModuleDef::Static(_) => Some(NodeKind::Value(ValueNode::Static)),
            hir::ModuleDef::Trait(_) => Some(NodeKind::Trait),
            hir::ModuleDef::TypeAlias(_) => Some(NodeKind::TypeAlias),
            hir::ModuleDef::BuiltinType(_) => Some(NodeKind::Type(TypeNode::BuiltinType)),
        }
    }

    pub fn display_name(&self) -> Option<&'static str> {
        match self {
            Self::Crate => Some("crate"),
            Self::Function => Some("fn"),
            Self::Module => Some("mod"),
            Self::Orphan => None,
            Self::Trait => Some("trait"),
            Self::Type(TypeNode::Struct) => Some("struct"),
            Self::Type(TypeNode::Union) => Some("union"),
            Self::Type(TypeNode::Enum) => Some("enum"),
            Self::Type(TypeNode::BuiltinType) => Some("builtin"),
            Self::TypeAlias => Some("type"),
            Self::Value(ValueNode::Const) => Some("const"),
            Self::Value(ValueNode::Static) => Some("static"),
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
