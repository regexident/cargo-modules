// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Edge {
    pub kind: EdgeKind,
}

impl Edge {
    pub fn merge_with(&mut self, other: &Edge) {
        match (self.kind, other.kind) {
            (EdgeKind::Uses, EdgeKind::Uses) => {
                self.kind = EdgeKind::Uses;
            }
            (EdgeKind::Uses, EdgeKind::Owns)
            | (EdgeKind::Owns, EdgeKind::Uses)
            | (EdgeKind::Owns, EdgeKind::Owns) => {
                self.kind = EdgeKind::Owns;
            }
        }
    }

    pub fn merged_with(&self, other: &Edge) -> Self {
        let mut clone = self.clone();
        clone.merge_with(other);
        clone
    }
}
