use arrayvec::ArrayString;
use petgraph::graphmap::DiGraphMap;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};

// TODO: Add support to represent use's of individual members (fn's, trairs
//       etc.).  This would require using a union type (of Mod + member?) as
//       node type.

/// Determines the maximum length of a module's path.
///
/// eg: `"my_crate::foo::bar::baz"`.
const MOD_PATH_SIZE: usize = 200;

static SEP: &'static str = "::";

/// Represents an association between modules.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edge {
    Child,
    Dependency,
}

/// Builds a graph, `DiGraphMap<Mod, Edge>` to be specific using domain
/// specific operations.
pub struct GraphBuilder {
    graph: DiGraphMap<Mod, Edge>,
    deferred_deps: Vec<(String, String)>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: DiGraphMap::new(),
            deferred_deps: vec![],
        }
    }

    /// Define a public module that has no parent.
    ///
    /// # Panics
    /// If `name` contains `"::"`.
    pub fn add_crate_root(&mut self, name: &str) {
        assert!(!name.contains(SEP));
        self.graph.add_node(Mod::new(name, Visibility::Public));
    }

    /// Define a sub-modules and associate it with its parent.
    ///
    /// # Panics
    /// - If the parent is not already defined.
    /// - If parent-child relationship for this pair is already defined.
    pub fn add_mod(&mut self, path: &str, name: &str, visibility: Visibility) {
        let parent: Mod = self.find_mod(path).unwrap();
        let node = Mod::new(&[path, SEP, name].concat(), visibility);
        self.graph.add_node(node);
        assert!(self.graph.add_edge(parent, node, Edge::Child).is_none());
    }

    pub fn add_dep(&mut self, from: &str, to: &str) {
        assert!(from != to, "Module cannot depend on itself");
        match (self.find_mod(from), self.find_mod(to)) {
            (Some(from), Some(to)) => {
                self.graph.add_edge(from, to, Edge::Dependency);
                ()
            }
            // Defer creating dependency link if
            // one of the nodes are not defined yet.
            (_, _) => self.deferred_deps.push((from.to_owned(), to.to_owned())),
        };
    }

    /// Build the graph, consuming this builder, or return an error.
    pub fn build(mut self) -> Result<DiGraphMap<Mod, Edge>, GraphError> {
        if self.deferred_deps.is_empty() {
            Ok(self.graph)
        } else {
            let (from, to) = self.deferred_deps.remove(0);
            match self.find_mod(&from) {
                Some(_) => Err(GraphError::UnknownModule(to)),
                None => Err(GraphError::UnknownModule(from)),
            }
        }
    }

    fn find_mod(&self, path: &str) -> Option<Mod> {
        self.graph.nodes().find(|m| m.path() == path)
    }
}

#[derive(Debug, PartialEq)]
pub enum GraphError {
    UnknownModule(String),
}

/// Represents a node that is a module in the graph.
#[derive(Clone, Copy, Debug)]
pub struct Mod {
    /// Because this struct needs to be `Copy`, using `String` or `&str` was
    /// out of the question.  `ArrayString` provides a sized and owned string
    /// that is backed by a byte array.
    ///
    /// See also [MOD_PATH_SIZE]
    path: ArrayString<[u8; MOD_PATH_SIZE]>,
    name_ridx: usize,
    visibility: Visibility,
}

impl Mod {
    fn new(path: &str, visibility: Visibility) -> Self {
        Self {
            path: ArrayString::<[u8; MOD_PATH_SIZE]>::from(path)
                .unwrap_or_else(|_| panic!("Module path is too long")),
            name_ridx: path.rfind(SEP).unwrap_or(0),
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

    #[test]
    #[should_panic(expected = "Module cannot depend on itself")]
    fn adding_a_dependency_to_the_same_module_panics() {
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("root");
        builder.add_mod("root", "sub", Visibility::Public);
        builder.add_dep("root::sub", "root::sub");
    }

    #[test]
    fn adding_a_dependency_is_idempotent() {
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("foo");
        builder.add_mod("foo", "bar", Visibility::Public);
        builder.add_mod("foo", "baz", Visibility::Private);
        builder.add_dep("foo::bar", "foo::baz");
        builder.add_dep("foo::bar", "foo::baz");
        let graph = builder.build().unwrap();
        assert_eq!(3, graph.edge_count());
    }

    #[test]
    fn add_dep_requires_both_modules_to_be_defined() {
        {
            let mut builder = GraphBuilder::new();
            builder.add_crate_root("foo");
            builder.add_mod("foo", "bar", Visibility::Public);
            builder.add_dep("foo::bar", "foo::baz");
            assert_eq!(
                Some(GraphError::UnknownModule(String::from("foo::baz"))),
                builder.build().err()
            );
        }
        {
            let mut builder = GraphBuilder::new();
            builder.add_crate_root("foo");
            builder.add_mod("foo", "baz", Visibility::Private);
            builder.add_dep("foo::bar", "foo::baz");
            assert_eq!(
                Some(GraphError::UnknownModule(String::from("foo::bar"))),
                builder.build().err()
            );
        }
        {
            let mut builder = GraphBuilder::new();
            builder.add_crate_root("foo");
            builder.add_dep("foo::bar", "foo::baz");
            assert_eq!(
                Some(GraphError::UnknownModule(String::from("foo::bar"))),
                builder.build().err()
            );
        }
    }

    // TODO: Add test for deferring dependency
}
