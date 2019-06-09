use arrayvec::ArrayString;
use petgraph::graphmap::DiGraphMap;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};

/// Determines the maximum length of the #cfg that
/// can be attached to a mod definition.
const CONDITIONS_SIZE: usize = 100;

/// Determines the maximum length of a module's path.
///
/// eg: `"my_crate::foo::bar::baz"`.
const MOD_PATH_SIZE: usize = 200;

const SELF_KEYWORD: &str = "self";

pub const SEP: &str = "::";

pub type Graph = DiGraphMap<Module, Edge>;

#[derive(Clone, Debug, PartialEq)]
pub struct Dependency {
    refers_to_all: bool,
    refers_to_mod: bool,
    referred_members: Vec<String>,
}

impl Dependency {
    pub fn module() -> Self {
        Self {
            refers_to_all: false,
            refers_to_mod: true,
            referred_members: vec![],
        }
    }

    pub fn all() -> Self {
        Self {
            refers_to_all: true,
            refers_to_mod: false,
            referred_members: vec![],
        }
    }

    pub fn members(mut members: Vec<String>) -> Self {
        let refers_to_mod: bool = members.contains(&String::from(SELF_KEYWORD));
        let referred_members: Vec<String> =
            members.drain(..).filter(|m| m != SELF_KEYWORD).collect();
        Self {
            refers_to_all: false,
            refers_to_mod,
            referred_members,
        }
    }
}

/// Represents an association between modules.
#[derive(Clone, Debug, PartialEq)]
pub enum Edge {
    Child,
    Dependency(Dependency),
    Unconnected,
}

/// Builds a graph, `DiGraphMap<Mod, Edge>` to be specific using domain
/// specific operations.
pub struct GraphBuilder {
    graph: Graph,
    deferred_deps: Vec<(String, String, Dependency)>,
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
        self.graph
            .add_node(Module::new_root(name, Visibility::Public, None));
    }

    /// Define a sub-modules and associate it with its parent.
    ///
    /// # Panics
    /// - If the parent is not already defined.
    /// - If parent-child relationship for this pair is already defined.
    pub fn add_mod(&mut self, path: &str, name: &str, visibility: Visibility) {
        let parent: Module = find_mod(&self.graph, path).unwrap();
        let node = Module::new(&[path, SEP, name].concat(), visibility, None);
        self.graph.add_node(node);
        assert!(self.graph.add_edge(parent, node, Edge::Child).is_none());
        self.apply_deferred();
    }

    pub fn add_orphan(&mut self, path: &str, name: &str, visibility: Visibility) {
        let parent: Module = find_mod(&self.graph, path).unwrap();
        let node = Module::new(&[path, SEP, name].concat(), visibility, None);
        self.graph.add_node(node);
        assert!(self
            .graph
            .add_edge(parent, node, Edge::Unconnected)
            .is_none());
        self.apply_deferred();
    }

    pub fn add_dep(&mut self, from: &str, to: &str, dependency: Dependency) {
        assert!(from != to, "Module cannot depend on itself");
        match (find_mod(&self.graph, from), find_mod(&self.graph, to)) {
            (Some(from), Some(to)) => {
                self.graph.add_edge(from, to, Edge::Dependency(dependency));
            }
            (None, _) => panic!("Trying to add dependency from an unknown module"),
            // Defer creating dependency link if
            // one of the nodes are not defined yet.
            (_, _) => self
                .deferred_deps
                .push((from.to_owned(), to.to_owned(), dependency)),
        };
    }

    /// Build the graph, consuming this builder, or return an error.
    pub fn build(mut self) -> Result<Graph, GraphError> {
        if self.deferred_deps.is_empty() {
            Ok(self.graph)
        } else {
            let (from, to, _) = self.deferred_deps.remove(0);
            match find_mod(&self.graph, &from) {
                Some(_) => Err(GraphError::UnknownModule(to)),
                None => Err(GraphError::UnknownModule(from)),
            }
        }
    }

    fn apply_deferred(&mut self) {
        let deferred: Vec<(String, String, Dependency)> = self.deferred_deps.drain(..).collect();
        for (from, to, dep) in deferred {
            // We are only checking `to` because a dependent module
            // needs to be defined before the dependency is defined.
            if find_mod(&self.graph, &to).is_some() {
                self.add_dep(&from, &to, dep);
            } else {
                self.deferred_deps.push((from, to, dep));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GraphError {
    UnknownModule(String),
}

/// Represents a node that is a module in the graph.
#[derive(Clone, Copy, Debug)]
pub struct Module {
    is_root: bool,
    /// Because this struct needs to be `Copy`, using `String` or `&str` was
    /// out of the question.  `ArrayString` provides a sized and owned string
    /// that is backed by a byte array.
    ///
    /// See also [MOD_PATH_SIZE]
    path: ArrayString<[u8; MOD_PATH_SIZE]>,
    name_ridx: usize,
    visibility: Visibility,
    /// This needs to be `Copy` for the same reason as `path`.
    conditions: Option<ArrayString<[u8; CONDITIONS_SIZE]>>,
}

impl Module {
    pub fn new(path: &str, visibility: Visibility, conditions: Option<&str>) -> Self {
        Self {
            is_root: false,
            path: ArrayString::<[u8; MOD_PATH_SIZE]>::from(path)
                .unwrap_or_else(|_| panic!("Module path is too long")),
            name_ridx: path.rfind(SEP).unwrap_or(0),
            visibility,
            conditions: conditions.map(|c| {
                ArrayString::<[u8; CONDITIONS_SIZE]>::from(c)
                    .unwrap_or_else(|_| panic!("Conditions are too long"))
            }),
        }
    }

    pub fn new_root(path: &str, visibility: Visibility, conditions: Option<&str>) -> Self {
        let mut m = Self::new(path, visibility, conditions);
        m.is_root = true;
        m
    }

    pub fn is_root(&self) -> bool {
        self.is_root
    }

    pub fn path(&self) -> &str {
        &self.path.as_str()
    }
}

impl Eq for Module {}

impl Hash for Module {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path().hash(state);
    }
}

impl Ord for Module {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path().cmp(&other.path())
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    Public,
    Private,
}

pub fn find_mod(graph: &Graph, path: &str) -> Option<Module> {
    graph.nodes().find(|m| m.path() == path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builder_produces_an_empty_directed_graph() {
        let builder = GraphBuilder::new();
        let graph: Graph = builder.build().unwrap();
        assert_eq!(0, graph.node_count());
        assert_eq!(0, graph.edge_count());
    }

    #[test]
    fn add_crate_root_adds_a_node() {
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("crate-root");
        let graph: Graph = builder.build().unwrap();
        assert_eq!(1, graph.node_count());
        assert_eq!(0, graph.edge_count());
        assert!(graph.contains_node(Module::new("crate-root", Visibility::Public, None)));
    }

    #[test]
    fn add_mod_creates_an_association_with_parent() {
        let foo: Module = Module::new("foo", Visibility::Public, None);
        let bar: Module = Module::new("foo::bar", Visibility::Public, None);
        let baz: Module = Module::new("foo::bar::baz", Visibility::Private, None);
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("foo");
        builder.add_mod("foo", "bar", Visibility::Public);
        builder.add_mod("foo::bar", "baz", Visibility::Private);
        let graph: Graph = builder.build().unwrap();
        assert_eq!(3, graph.node_count());
        assert_eq!(2, graph.edge_count());
        assert!(graph.contains_node(foo));
        assert!(graph.contains_node(bar));
        assert!(graph.contains_node(baz));
        assert_eq!(
            vec![(foo, bar, &Edge::Child), (bar, baz, &Edge::Child)],
            graph.all_edges().collect::<Vec<(Module, Module, &Edge)>>()
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
        builder.add_dep("root::sub", "root::sub", Dependency::module());
    }

    #[test]
    fn adding_a_dependency_is_idempotent() {
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("foo");
        builder.add_mod("foo", "bar", Visibility::Public);
        builder.add_mod("foo", "baz", Visibility::Private);
        builder.add_dep("foo::bar", "foo::baz", Dependency::module());
        builder.add_dep("foo::bar", "foo::baz", Dependency::module());
        let graph = builder.build().unwrap();
        assert_eq!(3, graph.edge_count());
    }

    #[test]
    fn add_dep_requires_both_modules_to_be_defined() {
        {
            let mut builder = GraphBuilder::new();
            builder.add_crate_root("foo");
            builder.add_mod("foo", "bar", Visibility::Public);
            builder.add_dep("foo::bar", "foo::baz", Dependency::module());
            assert_eq!(
                Some(GraphError::UnknownModule(String::from("foo::baz"))),
                builder.build().err()
            );
        }
        {
            let mut builder = GraphBuilder::new();
            builder.add_crate_root("foo");
            builder.add_mod("foo", "bar", Visibility::Private);
            builder.add_dep("foo::bar", "foo::baz", Dependency::module());
            builder.add_dep("foo::bar", "foo::fubar", Dependency::module());
            builder.add_mod("foo", "baz", Visibility::Private);
            assert_eq!(
                Some(GraphError::UnknownModule(String::from("foo::fubar"))),
                builder.build().err()
            );
        }
    }

    #[test]
    #[should_panic(expected = "Trying to add dependency from an unknown module")]
    fn dependency_from_unknown_module_panics() {
        {
            let mut builder = GraphBuilder::new();
            builder.add_crate_root("foo");
            builder.add_dep("foo::bar", "foo::baz", Dependency::module());
        }
    }

    #[test]
    fn dependency_can_be_added_before_dependent_module_is_added() {
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("foo");
        builder.add_mod("foo", "bar", Visibility::Public);
        builder.add_dep("foo::bar", "foo::baz", Dependency::module());
        builder.add_mod("foo", "baz", Visibility::Public);
        assert_eq!(3, builder.build().unwrap().edge_count());
    }

    #[test]
    fn orphaned_modules_are_linked_to_their_parent_via_unconnected_edges() {
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("foo");
        builder.add_mod("foo", "bar", Visibility::Public);
        builder.add_orphan("foo", "baz", Visibility::Public);
        builder.add_orphan("foo::bar", "bat", Visibility::Public);
        let graph = builder.build().unwrap();
        assert_eq!(3, graph.edge_count());

        // Check edge `foo -> baz`
        assert!(graph.contains_edge(
            find_mod(&graph, "foo").unwrap(),
            find_mod(&graph, "foo::baz").unwrap()
        ));
        assert_eq!(
            Some(&Edge::Unconnected),
            graph.edge_weight(
                find_mod(&graph, "foo").unwrap(),
                find_mod(&graph, "foo::baz").unwrap()
            )
        );

        // Check edge `bar -> bat`
        assert!(graph.contains_edge(
            find_mod(&graph, "foo::bar").unwrap(),
            find_mod(&graph, "foo::bar::bat").unwrap()
        ));
        assert_eq!(
            Some(&Edge::Unconnected),
            graph.edge_weight(
                find_mod(&graph, "foo::bar").unwrap(),
                find_mod(&graph, "foo::bar::bat").unwrap()
            )
        );
    }
}
