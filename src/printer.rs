use colored::*;

use tree::{Tree, Visibility, Visitor};

pub struct Config {
    pub colored: bool,
}

pub struct Printer {
    config: Config,
}

impl Visitor for Printer {
    fn visit(&self, tree: &Tree, path: &[(usize, usize)], _parents: &[&str]) {
        // The "colored" crate doesn't support coloring for Windows:
        let colored = if cfg!(target_os = "windows") {
            false
        } else {
            self.config.colored
        };
        let mut branch = String::new();
        if let Some((last, others)) = path.split_last() {
            fn is_last(index_of: &(usize, usize)) -> bool {
                index_of.0 + 1 == index_of.1
            }
            for other in others {
                if is_last(other) {
                    branch.push_str("    ");
                } else {
                    branch.push_str(" │  ");
                }
            }
            if is_last(last) {
                branch.push_str(" └── ");
            } else {
                branch.push_str(" ├── ");
            };
        }
        if colored {
            print!("{}", branch.blue().bold());
        } else {
            print!("{}", branch);
        }
        let name = tree.name();
        if colored {
            let name = match *tree {
                Tree::Crate { ref name, .. } => name.green(),
                Tree::Module {
                    ref name,
                    ref visibility,
                    ..
                } => match *visibility {
                    Visibility::Public => name.green(),
                    Visibility::Private => name.yellow(),
                },
                Tree::Orphan { ref name } => name.red(),
            };
            print!("{}", name.bold());
        } else {
            print!("{}", name);
        }
        let kind = match *tree {
            Tree::Crate { .. } => "crate".to_string(),
            Tree::Module { ref visibility, .. } => match *visibility {
                Visibility::Public => "public".to_string(),
                Visibility::Private => "private".to_string(),
            },
            Tree::Orphan { .. } => "orphan".to_string(),
        };
        print!(" : ");
        if colored {
            print!("{}", kind.cyan().bold());
        } else {
            print!("{}", kind);
        }
        if let Tree::Module { ref condition, .. } = *tree {
            if let Some(ref condition) = *condition {
                let condition = if colored {
                    condition.as_str().magenta().bold()
                } else {
                    condition.as_str().clear()
                };
                print!(" @ ");
                print!("{}", condition);
            }
        }
        println!();
    }
}

impl Printer {
    pub fn new(config: Config) -> Self {
        Printer { config }
    }
}
