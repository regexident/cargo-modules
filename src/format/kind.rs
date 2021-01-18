use std::fmt;

use ra_ap_hir as hir;

#[derive(Clone, Debug)]
pub enum FormattedKind {
    Crate,
    Module,
    Function,
    Struct,
    Union,
    Enum,
    Variant,
    Const,
    Static,
    Trait,
    Type,
    Primitive,
}

impl<'a> FormattedKind {
    pub fn new(hir: hir::ModuleDef) -> Self {
        match hir {
            hir::ModuleDef::Module(_) => Self::Module,
            hir::ModuleDef::Function(_) => Self::Function,
            hir::ModuleDef::Adt(hir::Adt::Struct(_)) => Self::Struct,
            hir::ModuleDef::Adt(hir::Adt::Union(_)) => Self::Union,
            hir::ModuleDef::Adt(hir::Adt::Enum(_)) => Self::Enum,
            hir::ModuleDef::Variant(_) => Self::Variant,
            hir::ModuleDef::Const(_) => Self::Const,
            hir::ModuleDef::Static(_) => Self::Static,
            hir::ModuleDef::Trait(_) => Self::Trait,
            hir::ModuleDef::TypeAlias(_) => Self::Type,
            hir::ModuleDef::BuiltinType(_) => Self::Primitive,
        }
    }
}

impl fmt::Display for FormattedKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormattedKind::Crate => write!(f, "crate"),
            FormattedKind::Module => write!(f, "mod"),
            FormattedKind::Function => write!(f, "fn"),
            FormattedKind::Struct => write!(f, "struct"),
            FormattedKind::Union => write!(f, "union"),
            FormattedKind::Enum => write!(f, "enum"),
            FormattedKind::Variant => write!(f, "variant"),
            FormattedKind::Const => write!(f, "const"),
            FormattedKind::Static => write!(f, "static"),
            FormattedKind::Trait => write!(f, "trait"),
            FormattedKind::Type => write!(f, "type"),
            FormattedKind::Primitive => write!(f, "primitive"),
        }
    }
}
