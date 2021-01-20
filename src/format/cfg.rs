use std::fmt;

use ra_ap_cfg::{CfgAtom, CfgExpr};
use ra_ap_hir::{self as hir, HasAttrs};
use ra_ap_ide_db::RootDatabase;

#[derive(Clone, Debug)]
pub struct FormattedCfgExpr {
    cfg: CfgExpr,
}

impl FormattedCfgExpr {
    pub fn new(cfg: CfgExpr) -> Self {
        Self { cfg }
    }

    pub fn from_hir(hir: hir::ModuleDef, db: &RootDatabase) -> Option<Self> {
        let cfg = match hir {
            hir::ModuleDef::Module(r#mod) => r#mod.attrs(db).cfg(),
            hir::ModuleDef::Function(r#fn) => r#fn.attrs(db).cfg(),
            hir::ModuleDef::Adt(r#adt) => r#adt.attrs(db).cfg(),
            hir::ModuleDef::Variant(r#variant) => r#variant.attrs(db).cfg(),
            hir::ModuleDef::Const(r#const) => r#const.attrs(db).cfg(),
            hir::ModuleDef::Static(r#static) => r#static.attrs(db).cfg(),
            hir::ModuleDef::Trait(r#trait) => r#trait.attrs(db).cfg(),
            hir::ModuleDef::TypeAlias(r#type) => r#type.attrs(db).cfg(),
            hir::ModuleDef::BuiltinType(_) => None,
        };

        cfg.map(Self::new)
    }

    pub fn top_level(&self) -> Vec<CfgExpr> {
        match &self.cfg {
            CfgExpr::Invalid => vec![],
            cfg @ CfgExpr::Atom(_) => vec![cfg.clone()],
            CfgExpr::All(cfgs) => cfgs.clone(),
            cfg @ CfgExpr::Any(_) => vec![cfg.clone()],
            cfg @ CfgExpr::Not(_) => vec![cfg.clone()],
        }
    }
}

impl fmt::Display for FormattedCfgExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.cfg {
            CfgExpr::Invalid => write!(f, "INVALID"),
            CfgExpr::Atom(CfgAtom::Flag(flag)) => {
                write!(f, "{}", flag.to_string())
            }
            CfgExpr::Atom(CfgAtom::KeyValue { key, value }) => {
                write!(f, "{} = {:?}", key, value)
            }
            CfgExpr::All(cfgs) => {
                write!(f, "all(")?;
                for cfg in cfgs {
                    write!(f, "{}", Self::new(cfg.clone()))?;
                }
                write!(f, ")")
            }
            CfgExpr::Any(cfgs) => {
                write!(f, "any(")?;
                for cfg in cfgs {
                    write!(f, "{}", Self::new(cfg.clone()))?;
                }
                write!(f, ")")
            }
            CfgExpr::Not(cfg) => write!(f, "not({})", Self::new(cfg.as_ref().to_owned())),
        }
    }
}
