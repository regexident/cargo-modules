use std::fmt;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum EdgeKind {
    Uses,
    Owns,
}

impl EdgeKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Uses => "uses",
            Self::Owns => "owns",
        }
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Uses => "Uses",
            Self::Owns => "Owns",
        };
        write!(f, "{}", name)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Edge {
    pub kind: EdgeKind,
}
