use arrayvec::ArrayString;
use petgraph::graphmap::DiGraphMap;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};

const MOD_PATH_SIZE: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edge {
    Child,
}

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

    pub fn add_mod(&mut self, path: &str, name: &str, visibility: Visibility) {
        let parent: Mod = self.find_mod(path).unwrap();
        let node = Mod::new(&[path, "::", name].concat(), visibility);
        self.graph.add_node(node);
        assert!(self.graph.add_edge(parent, node, Edge::Child).is_none());
    }

    pub fn build(self) -> Result<DiGraphMap<Mod, Edge>, GraphError> {
        Ok(self.graph)
    }

    fn find_mod(&self, path: &str) -> Option<Mod> {
        self.graph.nodes().find(|m| m.path() == path)
    }

    // TODO: add_dep();
}

#[derive(Debug)]
pub enum GraphError {}

#[derive(Clone, Copy, Debug)]
pub struct Mod {
    path: ArrayString<[u8; MOD_PATH_SIZE]>,
    name_ridx: usize,
    visibility: Visibility,
}

impl Mod {
    fn new(path: &str, visibility: Visibility) -> Self {
        Self {
            path: ArrayString::<[u8; MOD_PATH_SIZE]>::from(path)
                .unwrap_or_else(|_| panic!("Module path is too long")),
            name_ridx: path.rfind("::").unwrap_or(0),
            visibility,
        }
    }

    fn path(&self) -> &str {
        &self.path.as_str()
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

    #[test]
    fn add_mod_creates_an_association_with_parent() {
        let foo: Mod = Mod::new("foo", Visibility::Public);
        let bar: Mod = Mod::new("foo::bar", Visibility::Public);
        let baz: Mod = Mod::new("foo::bar::baz", Visibility::Private);
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("foo");
        builder.add_mod("foo", "bar", Visibility::Public);
        builder.add_mod("foo::bar", "baz", Visibility::Private);
        let graph: DiGraphMap<Mod, Edge> = builder.build().unwrap();
        assert_eq!(3, graph.node_count());
        assert_eq!(2, graph.edge_count());
        assert!(graph.contains_node(foo));
        assert!(graph.contains_node(bar));
        assert!(graph.contains_node(baz));
        assert_eq!(
            vec![(foo, bar, &Edge::Child), (bar, baz, &Edge::Child)],
            graph.all_edges().collect::<Vec<(Mod, Mod, &Edge)>>()
        );
    }

    #[test]
    #[should_panic(expected = "Module path is too long")]
    fn add_mod_panics_if_path_is_longer_than_limit() {
        let name = "foo";
        let path = std::str::from_utf8([114; MOD_PATH_SIZE - 4].as_ref()).unwrap(); // 'r'
        let concatenated = format!("{}::{}", path, name);
        assert!(concatenated.len() > MOD_PATH_SIZE);
        let mut builder = GraphBuilder::new();
        builder.add_crate_root(path);
        builder.add_mod(path, name, Visibility::Public);
    }

    // TODO: Add test where builder fails when non-existent parent node
}
