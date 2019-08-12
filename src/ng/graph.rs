//! Data structures that represent module hierarchy and dependencies.
use arrayvec::ArrayString;
use petgraph::graphmap::DiGraphMap;
use std::cmp::{Ord, Ordering};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Add;

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

/// Represents the scope of dependencies between modules.
///
/// See also [`Edge`].
#[derive(Clone, Debug, PartialEq)]
pub struct Dependency {
    /// Eg: `use path::to::mod::*;`
    refers_to_all: bool,
    /// Eg: `use path::to::mod;` or `use path::to::mod::{self, ...}`
    refers_to_mod: bool,
    /// Eg: `use path::to::mod::{some_function, SomeStruct};`
    referred_members: HashSet<String>,
}

impl Dependency {
    pub fn module() -> Self {
        Self {
            refers_to_all: false,
            refers_to_mod: true,
            referred_members: HashSet::new(),
        }
    }

    pub fn all() -> Self {
        Self {
            refers_to_all: true,
            refers_to_mod: false,
            referred_members: HashSet::new(),
        }
    }

    pub fn members(mut members: Vec<String>) -> Self {
        let refers_to_mod: bool = members.contains(&String::from(SELF_KEYWORD));
        let referred_members: HashSet<String> =
            members.drain(..).filter(|m| m != SELF_KEYWORD).collect();
        Self {
            refers_to_all: false,
            refers_to_mod,
            referred_members,
        }
    }
}

impl Add for Dependency {
    type Output = Dependency;

    fn add(self, other: Dependency) -> Dependency {
        Dependency {
            refers_to_all: self.refers_to_all || other.refers_to_all,
            refers_to_mod: self.refers_to_mod || other.refers_to_mod,
            referred_members: self
                .referred_members
                .union(&other.referred_members)
                .cloned()
                .collect(),
        }
    }
}

/// Represents an association between modules.
///
/// This enum is intended to be used with directed graphs because
/// relationships represented are asymmetric.
#[derive(Clone, Debug, PartialEq)]
pub enum Edge {
    /// Represents a **parent ⇒ child** relationship.
    Child,
    /// Represents a **dependent ⇒ dependency** relationship created by `use`.
    Dependency(Dependency),
    /// Like `Child` but the **child** module is an orphan.  So this
    /// represents a _lack of_ semantic relationship.
    Unconnected,
}

/// Builds a graph, `DiGraphMap<Mod, Edge>` to be specific using domain
/// specific operations.
pub struct GraphBuilder {
    graph: Graph,
    uses: HashMap<String, HashSet<String>>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: DiGraphMap::new(),
            uses: HashMap::new(),
        }
    }

    /// Define a public module that has no parent.
    ///
    /// # Panics
    /// If `name` contains `"::"`.
    pub fn add_crate_root(&mut self, name: &str) {
        assert!(!name.contains(SEP));
        self.graph.add_node(Module::new_root(name));
    }

    /// Define a sub-modules and associate it with its parent.
    ///
    /// # Panics
    /// - If the parent is not already defined.
    /// - If parent-child relationship for this pair is already defined.
    pub fn add_mod(
        &mut self,
        path: &str,
        name: &str,
        visibility: Visibility,
        conditions: Option<&str>,
    ) {
        let parent: Module = self.find(path).unwrap();
        let node = Module::new(&[path, SEP, name].concat(), Some(visibility), conditions);
        self.graph.add_node(node);
        assert!(self.graph.add_edge(parent, node, Edge::Child).is_none());
    }

    pub fn add_orphan(&mut self, path: &str, name: &str) {
        let parent: Module = self.find(path).unwrap();
        let node = Module::new(&[path, SEP, name].concat(), None, None);
        self.graph.add_node(node);
        assert!(self
            .graph
            .add_edge(parent, node, Edge::Unconnected)
            .is_none());
    }

    pub fn add_use(&mut self, path: &str, use_: &str) {
        if !self.uses.contains_key(path) {
            self.uses.insert(path.to_owned(), HashSet::new());
        }
        self.uses.get_mut(path).unwrap().insert(use_.to_owned());
    }

    /// Build the graph, consuming this builder, or return an error.
    pub fn build(mut self) -> Result<Graph, GraphError> {
        // Take ownership of uses separately so that we can still
        // call the builder struct as mutable.
        let uses: HashMap<String, HashSet<String>> =
            std::mem::replace(&mut self.uses, HashMap::new());

        uses.iter()
            .fold(Ok(()), |result, (from, uses)| {
                // Go through each module (from) that has uses recorded
                // but break as soon as a step results in an error.
                result.and_then(|_| {
                    // We can safely unwrap here since we expect the module
                    // to be visited before any use can be recorded.
                    let from_module = self
                        .find(&from)
                        .expect("Trying to add dependency from an unknown module");
                    // Go through uses for from_module, break
                    // as soon as a error is encountered.
                    uses.iter().fold(Ok(()), |result, to| {
                        result.and_then(|_| self.add_dependency(from_module, &to))
                    })
                })
            })
            // If all went right, return Ok(self.graph)
            .map(|_| self.graph)
    }

    pub fn find(&self, path: &str) -> Option<Module> {
        self.graph.nodes().find(|m| m.path() == path)
    }

    fn add_dependency(&mut self, from: Module, use_: &str) -> Result<(), GraphError> {
        match self.find(use_) {
            Some(to) => {
                // TODO: Don't bash on existing dependency.
                // TODO: Actually add uses (see analysis.rs)
                // TODO: Support self & super prefixed imports (E2015)
                // TODO: Support crate prefixed imports (E2018)
                assert!(self
                    .graph
                    .add_edge(from, to, Edge::Dependency(Dependency::module()))
                    .is_none());
                Ok(())
            }
            None => Err(GraphError::UnknownModule(use_.to_owned())),
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
    visibility: Option<Visibility>,
    /// This needs to be `Copy` for the same reason as `path`.
    conditions: Option<ArrayString<[u8; CONDITIONS_SIZE]>>,
}

impl Module {
    pub fn new(path: &str, visibility: Option<Visibility>, conditions: Option<&str>) -> Self {
        Self {
            is_root: false,
            path: ArrayString::<[u8; MOD_PATH_SIZE]>::from(path)
                .unwrap_or_else(|_| panic!("Module path is too long")),
            name_ridx: path.rfind(SEP).map(|i| i + SEP.len()).unwrap_or(0),
            visibility,
            conditions: conditions.map(|c| {
                ArrayString::<[u8; CONDITIONS_SIZE]>::from(c)
                    .unwrap_or_else(|_| panic!("Conditions are too long"))
            }),
        }
    }

    pub fn new_root(path: &str) -> Self {
        let mut m = Self::new(path, Some(Visibility::Public), None);
        m.is_root = true;
        m
    }

    pub fn conditions(&self) -> Option<&str> {
        self.conditions.as_ref().map(|c| &c[..])
    }

    pub fn name(&self) -> &str {
        &self.path[self.name_ridx..]
    }

    pub fn is_orphan(&self) -> bool {
        self.visibility.is_none()
    }

    pub fn is_root(&self) -> bool {
        self.is_root
    }

    pub fn path(&self) -> &str {
        &self.path.as_str()
    }

    pub fn visibility(&self) -> Option<Visibility> {
        self.visibility
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

// TODO: We should support more kinds of visibility.
//
// Defined kinds are:
//
// - Public
// - Crate
// - Restricted
// - Inherited
//
// See: https://doc.rust-lang.org/nightly/nightly-rustc/syntax/ast/enum.VisibilityKind.html
#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    Public,
    Private,
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
        assert!(graph.contains_node(Module::new("crate-root", Some(Visibility::Public), None)));
    }

    #[test]
    fn add_mod_creates_an_association_with_parent() {
        let foo: Module = Module::new("foo", Some(Visibility::Public), None);
        let bar: Module = Module::new("foo::bar", Some(Visibility::Public), None);
        let baz: Module = Module::new("foo::bar::baz", Some(Visibility::Private), None);
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("foo");
        builder.add_mod("foo", "bar", Visibility::Public, None);
        builder.add_mod("foo::bar", "baz", Visibility::Private, None);
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
        builder.add_mod(path, name, Visibility::Public, None);
    }

    #[test]
    fn adding_a_use_is_idempotent() {
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("foo");
        builder.add_mod("foo", "bar", Visibility::Public, None);
        builder.add_mod("foo", "baz", Visibility::Private, None);
        builder.add_use("foo::bar", "foo::baz");
        builder.add_use("foo::bar", "foo::baz");
        let graph = builder.build().unwrap();
        assert_eq!(3, graph.edge_count());
    }

    #[test]
    fn add_dep_requires_both_modules_to_be_defined() {
        {
            let mut builder = GraphBuilder::new();
            builder.add_crate_root("foo");
            builder.add_mod("foo", "bar", Visibility::Public, None);
            builder.add_use("foo::bar", "foo::baz");
            assert_eq!(
                Some(GraphError::UnknownModule(String::from("foo::baz"))),
                builder.build().err()
            );
        }
        {
            let mut builder = GraphBuilder::new();
            builder.add_crate_root("foo");
            builder.add_mod("foo", "bar", Visibility::Private, None);
            builder.add_use("foo::bar", "foo::baz");
            builder.add_use("foo::bar", "foo::fubar");
            builder.add_mod("foo", "baz", Visibility::Private, None);
            assert_eq!(
                Some(GraphError::UnknownModule(String::from("foo::fubar"))),
                builder.build().err()
            );
        }
    }

    #[test]
    #[should_panic(expected = "Trying to add dependency from an unknown module")]
    #[allow(unused_must_use)]
    fn dependency_from_unknown_module_panics() {
        {
            let mut builder = GraphBuilder::new();
            builder.add_crate_root("foo");
            builder.add_use("foo::bar", "foo::baz");
            builder.build();
        }
    }

    #[test]
    fn orphaned_modules_are_linked_to_their_parent_via_unconnected_edges() {
        let mut builder = GraphBuilder::new();
        builder.add_crate_root("foo");
        builder.add_mod("foo", "bar", Visibility::Public, None);
        builder.add_orphan("foo", "baz");
        builder.add_orphan("foo::bar", "bat");
        let graph = builder.build().unwrap();
        assert_eq!(3, graph.edge_count());

        // Check edge `foo -> baz`
        assert!(graph.contains_edge(
            Module::new_root("foo"),
            Module::new("foo::bar", Some(Visibility::Public), None)
        ));
        assert_eq!(
            Some(&Edge::Unconnected),
            graph.edge_weight(
                Module::new_root("foo"),
                Module::new("foo::baz", Some(Visibility::Public), None)
            )
        );

        // Check edge `bar -> bat`
        assert!(graph.contains_edge(
            Module::new("foo::bar", Some(Visibility::Public), None),
            Module::new("foo::bar::bat", Some(Visibility::Public), None)
        ));
        assert_eq!(
            Some(&Edge::Unconnected),
            graph.edge_weight(
                Module::new("foo::bar", Some(Visibility::Public), None),
                Module::new("foo::bar::bat", Some(Visibility::Public), None)
            )
        );
    }

    #[test]
    fn module_name() {
        assert_eq!(Module::new_root("foo").name(), "foo");
        assert_eq!(
            Module::new("foo::bar", Some(Visibility::Public), None).name(),
            "bar"
        );
        assert_eq!(
            Module::new("foo::bar::baz::bat::quux", Some(Visibility::Private), None).name(),
            "quux"
        );
    }
}
