use std::fmt;

use ra_ap_cfg::{CfgAtom, CfgExpr};

#[derive(Clone, PartialEq, Debug)]
pub enum NodeCfgAttr {
    Flag(String),
    KeyValue(String, String),
    All(Vec<Self>),
    Any(Vec<Self>),
    Not(Box<Self>),
}

impl NodeCfgAttr {
    pub fn new(cfg: CfgExpr) -> Option<Self> {
        match cfg {
            CfgExpr::Invalid => None,
            CfgExpr::Atom(CfgAtom::Flag(flag)) => Some(Self::Flag(flag.to_string())),
            CfgExpr::Atom(CfgAtom::KeyValue { key, value }) => {
                Some(Self::KeyValue(key.to_string(), value.to_string()))
            }
            CfgExpr::All(cfgs) => Some(Self::All(cfgs.into_iter().filter_map(Self::new).collect())),
            CfgExpr::Any(cfgs) => Some(Self::Any(cfgs.into_iter().filter_map(Self::new).collect())),
            CfgExpr::Not(cfg) => Self::new(*cfg).map(|cfg| Self::Not(Box::new(cfg))),
        }
    }
}

impl fmt::Display for NodeCfgAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_cfgs(f: &mut fmt::Formatter<'_>, cfgs: &[NodeCfgAttr]) -> fmt::Result {
            let mut is_first = true;
            for cfg in cfgs {
                if !is_first {
                    write!(f, ", ")?;
                }
                is_first = false;
                write!(f, "{}", cfg)?;
            }
            Ok(())
        }

        match self {
            Self::Flag(content) => {
                write!(f, "{}", content)?;
            }
            Self::KeyValue(key, value) => {
                write!(f, "{} = {:?}", key, value)?;
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

#[derive(Clone, PartialEq, Debug)]
pub struct NodeTestAttr;

impl fmt::Display for NodeTestAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "test")
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct NodeAttrs {
    pub cfgs: Vec<NodeCfgAttr>,
    pub test: Option<NodeTestAttr>,
}

impl NodeAttrs {
    pub fn is_empty(&self) -> bool {
        self.test.is_none() && self.cfgs.is_empty()
    }
}
