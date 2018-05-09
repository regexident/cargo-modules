use std::cell::Cell;
use std::iter::repeat;
use std::collections::HashMap;

pub use tree::{Tree, Visibility, Visitor};

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
        print!("\t\"{}\" [label=\"{}\",", name, tree.name());

        let kind = match *tree {
            Tree::Crate { .. } => "color=green".to_string(),
            Tree::Module { ref visibility, .. } => match *visibility {
                Visibility::Public => "color=green".to_string(),
                Visibility::Private => "color=gold".to_string(),
            },
            Tree::Orphan { .. } => "color=red".to_string(),
        };

        print!("{}", kind);

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
            for use_name in uses {
                let use_name = fix_supers(use_name, &name);
                let use_name = fix_selfs(&use_name, &name);

                let use_data = UseModuleFinder::new(&use_name);
                self.tree.accept(&mut vec![], &mut vec![], &use_data);

                if let Some(module_name) = use_data.module.take() {
                    let (strip_str, module_name) = if module_name != "::" {
                        (module_name.clone() + "::", module_name)
                    } else {
                        (module_name, String::from("::") + parents[0])
                    };

                    let display_name = use_name.split(strip_str.as_str()).last().unwrap();

                    // Insert into used_modules hashmap
                    used_modules
                        .entry(module_name)
                        .or_insert_with(|| vec![])
                        .push(display_name.to_string());
                } else if self.config.show_external {
                    println!("\t\"{}\" -> \"{}\" [color=green];", name, use_name);
                }
            }

            for (key, val) in used_modules {
                let types = if self.config.show_types {
                    val.join("\\n")
                } else {
                    "".to_string()
                };

                println!(
                    "\t\"{}\" -> \"{}\" [color=green,penwidth={},label=\"{}\"];",
                    name,
                    key,
                    val.len(),
                    types
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
            .map(|s| *s)
            .chain(use_name_iter)
            .chain([opt_super].iter().map(|s| *s))
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
            .map(|s| *s)
            .chain(use_name_iter)
            .collect::<Vec<&str>>()
            .join("::")
    } else {
        use_name.to_string()
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
            name: name,
            module: Cell::new(None),
        }
    }
}

impl<'a> Visitor for UseModuleFinder<'a> {
    fn visit(&self, tree: &Tree, _: &[(usize, usize)], parents: &[&str]) {
        let tree_path: Vec<&str> = parents
            .iter()
            .chain([tree.name()].iter())
            .map(|s| *s)
            .collect();
        let last_index: usize = if parents.is_empty() { 0 } else { parents.len() };

        for (index, (segment, parent)) in self.name
            .split("::")
            .zip(tree_path.iter().chain(repeat(&"")))
            .skip(1)
            .enumerate()
        {
            if index == last_index {
                if &segment != parent {
                    if index != 0 || self.name.split("::").count() == 2 {
                        let module_name = "::".to_string() + &tree_path[1..].join("::");
                        self.module.set(Some(module_name));
                    }
                } else {
                    self.module
                        .set(Some("::".to_string() + &tree_path[1..].join("::")));
                }
            } else if &segment != parent {
                return;
            }
        }
    }
}
