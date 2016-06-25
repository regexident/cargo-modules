use std::cmp::Ordering;

use colored::*;

#[derive(PartialEq, Eq, Debug)]
pub enum Kind {
    Test,
    Public,
    Private,
    Orphaned,
}

#[derive(Debug)]
pub struct Module {
    name: String,
    kind: Kind,
    submodules: Vec<Module>,
}

impl PartialEq<Module> for Module {
    fn eq(&self, other: &Module) -> bool {
        self.name == other.name
    }
}

impl Eq for Module {}

impl PartialOrd<Module> for Module {
    fn partial_cmp(&self, other: &Module) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for Module {
    fn cmp(&self, other: &Module) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl Module {
    pub fn new(name: String, kind: Kind) -> Self {
        Module {
            name: name,
            kind: kind,
            submodules: Vec::new(),
        }
    }

    pub fn submodule_at_path(&mut self, path: &[String]) -> Option<&mut Module> {
        if let Some((name, sub_path)) = path.split_first() {
            if let Some(submodule) = self.submodules.iter_mut().find(|m| m.name == *name) {
                return submodule.submodule_at_path(sub_path);
            } else {
                return None;
            }
        }
        Some(self)
    }

    pub fn insert(&mut self, name: String, kind: Kind) {
        let kind = if self.kind == Kind::Test {
            Kind::Test
        } else if self.kind == Kind::Private {
            Kind::Private
        } else {
            kind
        };
        let submodule = Module::new(name, kind);
        self.submodules.push(submodule);
        self.submodules.sort();
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn submodule_names(&self) -> Vec<String> {
        self.submodules.iter().map(|m| m.name().to_string()).collect()
    }

    pub fn print_tree(&self, path_mask: &mut Vec<bool>) {
        if let Some((last_flag, other_flags)) = path_mask.split_last() {
            let mut prefix = String::new();
            for other_flag in other_flags {
                if *other_flag {
                    prefix.push_str("    ");
                } else {
                    prefix.push_str(" │  ");
                }
            }
            if *last_flag {
                prefix.push_str(" └── ");
            } else {
                prefix.push_str(" ├── ");
            };
            print!("{}", prefix.blue().bold());
        }
        let name = match self.kind {
            Kind::Test => self.name.cyan(),
            Kind::Public => self.name.green(),
            Kind::Private => self.name.yellow(),
            Kind::Orphaned => self.name.red(),
        };
        println!("{}", name.bold());
        if let Some((_, submodules)) = self.submodules.split_last() {
            for submodule in submodules {
                path_mask.push(false);
                submodule.print_tree(path_mask);
                let _ = path_mask.pop();
            }
        }
        if let Some(last) = self.submodules.last() {
            path_mask.push(true);
            last.print_tree(path_mask);
            let _ = path_mask.pop();
        }
    }
}
