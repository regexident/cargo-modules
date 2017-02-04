use std::cmp::Ordering;

pub trait Visitor {
    fn visit(&self, module: &Tree, path: &[(usize, usize)]);
}

#[derive(Debug)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug)]
pub enum Tree {
    Crate { name: String, subtrees: Vec<Tree> },
    Module {
        name: String,
        visibility: Visibility,
        condition: Option<String>,
        subtrees: Vec<Tree>,
    },
    Orphan { name: String },
}

impl PartialEq<Tree> for Tree {
    fn eq(&self, other: &Tree) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Tree {}

impl PartialOrd<Tree> for Tree {
    fn partial_cmp(&self, other: &Tree) -> Option<Ordering> {
        self.name().partial_cmp(other.name())
    }
}

impl Ord for Tree {
    fn cmp(&self, other: &Tree) -> Ordering {
        self.name().cmp(other.name())
    }
}

impl Tree {
    pub fn new_crate(name: String) -> Self {
        Tree::Crate {
            name: name,
            subtrees: vec![],
        }
    }

    pub fn new_module(name: String, visibility: Visibility, condition: Option<String>) -> Self {
        Tree::Module {
            name: name,
            visibility: visibility,
            condition: condition,
            subtrees: vec![],
        }
    }

    pub fn new_orphan(name: String) -> Self {
        Tree::Orphan { name: name }
    }

    pub fn subtree_at_path(&mut self, path: &[String]) -> Option<&mut Tree> {
        if let Some((name, sub_path)) = path.split_first() {
            match *self {
                Tree::Crate { ref mut subtrees, .. } |
                Tree::Module { ref mut subtrees, .. } => {
                    subtrees.iter_mut()
                        .find(|m| m.name() == *name)
                        .and_then(|subtree| subtree.subtree_at_path(sub_path))
                }
                Tree::Orphan { .. } => None,
            }
        } else {
            Some(self)
        }
    }

    pub fn insert(&mut self, subtree: Tree) {
        match *self {
            Tree::Crate { ref mut subtrees, .. } |
            Tree::Module { ref mut subtrees, .. } => {
                subtrees.push(subtree);
                subtrees.sort();
            }
            Tree::Orphan { .. } => {}
        }
    }

    pub fn name(&self) -> &str {
        match *self {
            Tree::Crate { ref name, .. } |
            Tree::Module { ref name, .. } |
            Tree::Orphan { ref name } => name,
        }
    }

    pub fn subtree_names(&self) -> Vec<String> {
        match *self {
            Tree::Crate { ref subtrees, .. } |
            Tree::Module { ref subtrees, .. } => {
                subtrees.iter().map(|m| m.name().to_string()).collect()
            }
            Tree::Orphan { .. } => vec![],
        }
    }

    pub fn accept<V>(&self, path: &mut Vec<(usize, usize)>, visitor: &V)
        where V: Visitor
    {
        visitor.visit(self, path);
        match *self {
            Tree::Crate { ref subtrees, .. } |
            Tree::Module { ref subtrees, .. } => {
                let count = subtrees.len();
                for (index, subtree) in subtrees.iter().enumerate() {
                    path.push((index, count));
                    subtree.accept(path, visitor);
                    let _ = path.pop();
                }
            }
            Tree::Orphan { .. } => {}
        }
    }
}
