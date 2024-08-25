// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;

use ra_ap_cfg::{self as cfg};
use ra_ap_ide::{self as ide};

use crate::{analyzer, item::Item};

#[derive(Clone, PartialEq, Debug)]
pub enum ItemCfgAttr {
    Flag(String),
    KeyValue(String, String),
    All(Vec<Self>),
    Any(Vec<Self>),
    Not(Box<Self>),
}

impl ItemCfgAttr {
    pub fn new(cfg: &cfg::CfgExpr) -> Option<Self> {
        match cfg {
            cfg::CfgExpr::Invalid => None,
            cfg::CfgExpr::Atom(cfg::CfgAtom::Flag(flag)) => Some(Self::Flag(flag.to_string())),
            cfg::CfgExpr::Atom(cfg::CfgAtom::KeyValue { key, value }) => {
                Some(Self::KeyValue(key.to_string(), value.to_string()))
            }
            cfg::CfgExpr::All(cfgs) => Some(Self::All(cfgs.iter().filter_map(Self::new).collect())),
            cfg::CfgExpr::Any(cfgs) => Some(Self::Any(cfgs.iter().filter_map(Self::new).collect())),
            cfg::CfgExpr::Not(cfg) => Self::new(cfg).map(|cfg| Self::Not(Box::new(cfg))),
        }
    }
}

impl fmt::Display for ItemCfgAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_cfgs(f: &mut fmt::Formatter<'_>, cfgs: &[ItemCfgAttr]) -> fmt::Result {
            let mut is_first = true;
            for cfg in cfgs {
                if !is_first {
                    write!(f, ", ")?;
                }
                is_first = false;
                write!(f, "{cfg}")?;
            }
            Ok(())
        }

        match self {
            Self::Flag(content) => {
                write!(f, "{content}")?;
            }
            Self::KeyValue(key, value) => {
                write!(f, "{key} = {value:?}")?;
            }
            Self::All(cfgs) => {
                write!(f, "all(")?;
                fmt_cfgs(f, cfgs)?;
                write!(f, ")")?;
            }
            Self::Any(cfgs) => {
                write!(f, "any(")?;
                fmt_cfgs(f, cfgs)?;
                write!(f, ")")?;
            }
            Self::Not(cfg) => {
                write!(f, "not(")?;
                write!(f, "{}", *cfg)?;
                write!(f, ")")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ItemTestAttr;

impl fmt::Display for ItemTestAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "test")
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ItemAttrs {
    pub cfgs: Vec<ItemCfgAttr>,
    pub test: Option<ItemTestAttr>,
}

impl ItemAttrs {
    pub fn new(item: &Item, db: &ide::RootDatabase) -> ItemAttrs {
        let cfgs: Vec<_> = analyzer::cfg_attrs(item.hir, db);
        let test = analyzer::test_attr(item.hir, db);
        Self { cfgs, test }
    }

    pub fn is_empty(&self) -> bool {
        self.test.is_none() && self.cfgs.is_empty()
    }
}
