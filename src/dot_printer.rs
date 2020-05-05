use std::{
    cell::Cell,
    collections::HashMap,
    iter::repeat,
};

use crate::tree::{Tree, Visibility, Visitor};

pub struct Config {
    pub colored: bool,
    pub show_conditional: bool,
    pub show_external: bool,
    pub show_types: bool,
}

pub struct DotPrinter<'a> {
    config: Config,
    tree: &'a Tree,
}

impl<'a> Visitor for DotPrinter<'a> {
    fn visit(&self, tree: &Tree, _: &[(usize, usize)], parents: &[&str]) {
        if !self.config.show_conditional {
            if let Tree::Module { ref condition, .. } = *tree {
                if condition.is_some() {
                    return;
                }
            }
        }

        let mut parent_name = "".to_string();

        for parent in parents.iter().skip(1) {
            parent_name += "::";
            parent_name += parent;
        }
        let name = parent_name.clone() + "::" + tree.name();
        print!("\t\"{}\" [label=\"{}\"", name, tree.name());

        if self.config.colored {
            let kind = match *tree {
                Tree::Crate { .. } => "color=green".to_string(),
                Tree::Module { ref visibility, .. } => match *visibility {
                    Visibility::Public => "color=green".to_string(),
                    Visibility::Private => "color=gold".to_string(),
                },
                Tree::Orphan { .. } => "color=red".to_string(),
            };
            print!(",{}", kind);
        }

        if let Tree::Module { ref condition, .. } = *tree {
            if condition.is_some() {
                print!(",style=dotted");
            }
        }

        println!("];");

        if !parents.is_empty() {
            if parent_name == "" {
                println!("\t\"::{}\" -> \"{}\" [weight=100,len=1];", parents[0], name);
            } else {
                println!("\t\"{}\" -> \"{}\" [weight=100,len=1];", parent_name, name);
            }
        }

        if let Tree::Module { ref uses, .. } = *tree {
            let mut used_modules = HashMap::new();
            for (visibility, use_name) in uses {
                let use_name = fix_supers(&use_name, &name);
                let use_name = fix_selfs(&use_name, &name);
                let use_name = make_absolute(&use_name);

                let use_data = UseModuleFinder::new(&use_name);
                self.tree.accept(&mut vec![], &mut vec![], &use_data);

                if let Some(module_name) = use_data.module.take() {
                    let (strip_str, module_name) = if module_name != "::" {
                        (module_name.clone() + "::", module_name)
                    } else {
                        (module_name, String::from("::") + parents[0])
                    };

                    let display_name = if use_name.to_string() + "::" == strip_str {
                        "self"
                    } else {
                        use_name.split(strip_str.as_str()).last().unwrap()
                    };
                    // Insert into used_modules hashmap
                    used_modules
                        .entry(module_name)
                        .or_insert_with(|| vec![])
                        .push((*visibility, display_name.to_string()));
                } else if self.config.show_external {
                    let attrs = if self.config.colored {
                        match visibility {
                            Visibility::Public => "color=green".to_string(),
                            Visibility::Private => "color=gold".to_string(),
                        }
                    } else {
                        "".to_string()
                    };
                    println!("\t\"{}\" -> \"{}\" [{}];", name, &use_name[2..], attrs);
                }
            }

            for (key, val) in used_modules {
                let types = if self.config.show_types {
                    val.iter()
                        .map(|(_, name)| name.as_ref())
                        .collect::<Vec<&str>>()
                        .join("\\n")
                } else {
                    "".to_string()
                };

                let attrs = if self.config.colored {
                    if val.iter().any(|(vis, _)| &Visibility::Public == vis) {
                        ",color=green"
                    } else {
                        ",color=gold"
                    }
                } else {
                    ""
                };

                println!(
                    "\t\"{}\" -> \"{}\" [penwidth={},label=\"{}\"{}];",
                    name,
                    key,
                    val.len(),
                    types,
                    attrs,
                );
            }
        }
    }
}

/// This method resolves relative paths that start with `super`
fn fix_supers(use_name: &str, name: &str) -> String {
    let mut use_name_iter = use_name.split("::");
    let mut opt_super = use_name_iter.next().unwrap();
    if opt_super == "super" {
        let name_segments = name.split("::").collect::<Vec<&str>>();
        let mut name_iter = name_segments.iter().rev();

        while opt_super == "super" {
            name_iter.next();

            if let Some(segment) = use_name_iter.next() {
                opt_super = segment;
            } else {
                opt_super = "";
            }
        }

        name_iter
            .rev()
            .cloned()
            .chain([opt_super].iter().cloned())
            .chain(use_name_iter)
            .collect::<Vec<&str>>()
            .join("::")
    } else {
        use_name.to_string()
    }
}

/// This method resolves relative paths that start with `self`
fn fix_selfs(use_name: &str, name: &str) -> String {
    let mut use_name_iter = use_name.split("::");
    let opt_self = use_name_iter.next().unwrap();
    if opt_self == "self" {
        [name]
            .iter()
            .cloned()
            .chain(use_name_iter)
            .collect::<Vec<&str>>()
            .join("::")
    } else {
        use_name.to_string()
    }
}

fn make_absolute(use_name: &str) -> String {
    if use_name.starts_with("::") {
        use_name.to_string()
    } else {
        "::".to_string() + use_name
    }
}

impl<'a> DotPrinter<'a> {
    pub fn new(config: Config, tree: &'a Tree) -> Self {
        DotPrinter { config, tree }
    }
}

/// Bit of a hack to determine if this use string is pointing to some internal type
/// or to some external type. And if it is internal we also find out the module it belongs to
struct UseModuleFinder<'a> {
    pub name: &'a str,
    pub module: Cell<Option<String>>,
}

impl<'a> UseModuleFinder<'a> {
    pub fn new(name: &'a str) -> Self {
        UseModuleFinder {
            name,
            module: Cell::new(None),
        }
    }
}

impl<'a> Visitor for UseModuleFinder<'a> {
    fn visit(&self, tree: &Tree, _: &[(usize, usize)], parents: &[&str]) {
        let tree_path: Vec<&str> = parents
            .iter()
            .chain([tree.name()].iter())
            .cloned()
            .collect();
        let last_index: usize = if parents.is_empty() {
            0
        } else {
            parents.len() - 1
        };
        let module_name = "::".to_string() + &tree_path[1..].join("::");

        for (index, (segment, parent)) in self
            .name
            .split("::")
            .zip(tree_path.iter().chain(repeat(&"")))
            .skip(1) // Skip the crate name
            .enumerate()
        {
            if index == last_index {
                if &segment != parent {
                    if index != 0 {
                        let old_value = self.module.take();

                        if old_value.is_none()
                            || (old_value.is_some()
                                && old_value.as_ref().unwrap().split("::").count() < index)
                        {
                            self.module.set(Some(module_name.to_string()));
                        } else {
                            self.module.set(old_value);
                        }
                    }
                } else {
                    self.module.set(Some(module_name.to_string()));
                }
            } else if &segment != parent {
                return;
            }
        }
    }
}
