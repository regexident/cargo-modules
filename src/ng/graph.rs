use arrayvec::ArrayString;
use petgraph::graphmap::DiGraphMap;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};

const MOD_NAME_SIZE: usize = 30;

pub struct Edge;

pub struct GraphBuilder {
    graph: DiGraphMap<Mod, Edge>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: DiGraphMap::new(),
        }
    }

    pub fn add_crate_root(&mut self, name: &str) {
        self.graph.add_node(Mod::new(name, Visibility::Public));
    }

    pub fn build(self) -> Result<DiGraphMap<Mod, Edge>, GraphError> {
        Ok(self.graph)
    }

    // TODO: add_mod();
    // TODO: add_dep();
}

#[derive(Debug)]
pub enum GraphError {}

#[derive(Clone, Copy, Debug)]
pub struct Mod {
    name: ArrayString<[u8; MOD_NAME_SIZE]>,
    visibility: Visibility,
}

impl Mod {
    fn new(name: &str, visibility: Visibility) -> Self {
        Self {
            name: ArrayString::<[u8; MOD_NAME_SIZE]>::from(name).unwrap(),
            visibility,
        }
    }

    fn path(&self) -> &str {
        // FIXME: Use full mod path.
        &self.name.as_str()
    }
}

impl Eq for Mod {}

impl Hash for Mod {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path().hash(state);
    }
}

impl Ord for Mod {
    fn cmp(&self, other: &Mod) -> Ordering {
        self.path().cmp(&other.path())
    }
}

impl PartialEq for Mod {
    fn eq(&self, other: &Mod) -> bool {
        self.path() == other.path()
    }
}

impl PartialOrd for Mod {
    fn partial_cmp(&self, other: &Mod) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    Public,
    Private,
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graphmap::DiGraphMap;

    #[test]
    fn new_builder_produces_an_empty_directed_graph() {
        let builder = GraphBuilder::new();
        let graph: DiGraphMap<Mod, Edge> = builder.build().unwrap();
        assert_eq!(0, graph.node_count());
        assert_eq!(0, graph.edge_count());
    }

    #[test]
    fn add_crate_root_adds_a_node() {
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("crate-root");
        let graph: DiGraphMap<Mod, Edge> = builder.build().unwrap();
        assert_eq!(1, graph.node_count());
        assert_eq!(0, graph.edge_count());
        assert!(graph.contains_node(Mod::new("crate-root", Visibility::Public)));
    }
}
