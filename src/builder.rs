use std::{ffi::OsStr, fs, io, path, result::Result, string::ToString};

use syntax::print::pprust;
use syntax::{ast, source_map, visit};

use tree::{Tree, Visibility};

pub struct Config {
    pub include_orphans: bool,
    pub ignored_files: Vec<path::PathBuf>,
}

pub struct Builder<'a> {
    tree: Tree,
    path: Vec<String>,
    config: Config,
    source_map: &'a source_map::SourceMap,
}

impl<'a> Builder<'a> {
    pub fn new(config: Config, crate_name: String, source_map: &'a source_map::SourceMap) -> Self {
        let tree = Tree::new_crate(crate_name);
        Builder {
            tree,
            path: vec![],
            config,
            source_map,
        }
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    fn find_orphan_candidates(
        path: &[String],
        ignored_paths: &[path::PathBuf],
    ) -> Result<Vec<String>, io::Error> {
        let mut dir_path = "./src/".to_string();
        for name in path.iter() {
            dir_path.push_str(name);
            dir_path.push_str("/");
        }
        fn is_mod(entry: &fs::DirEntry) -> bool {
            let path = entry.path();
            if path.is_file() {
                if path.extension().map_or(false, |ext| ext == "rs") {
                    return file_name(entry)
                        .map_or(false, |s| (s != "mod") && (s != "lib") && (s != "main"));
                }
            } else if path.join("mod.rs").exists() {
                return true;
            }
            false
        }
        fn file_name(entry: &fs::DirEntry) -> Option<String> {
            entry
                .path()
                .file_stem()
                .and_then(OsStr::to_str)
                .map(ToString::to_string)
        }
        Ok(fs::read_dir(&dir_path)?
            .filter_map(Result::ok)
            .filter(is_mod)
            .filter(|e| !ignored_paths.contains(&e.path()))
            .filter_map(|e| file_name(&e))
            .collect::<Vec<_>>())
    }

    fn sanitize_condition(condition: String) -> String {
        let words: Vec<&str> = condition.split_whitespace().collect();
        words.join(" ")
    }
}

impl<'a> visit::Visitor<'a> for Builder<'a> {
    fn visit_item(&mut self, item: &'a ast::Item) {
        let condition = item
            .attrs
            .iter()
            .find(|attr| attr.check_name("cfg"))
            .map(|attr| {
                self.source_map
                    .span_to_snippet(attr.span)
                    .map(Builder::sanitize_condition)
                    .unwrap_or_else(|_| "".to_string())
            });

        match item.node {
            ast::ItemKind::Mod(_) => {
                let name = item.ident.to_string();
                {
                    let tree = self.tree.subtree_at_path(&self.path).unwrap();
                    let visibility = if let ast::VisibilityKind::Public = item.vis.node {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };
                    tree.insert(Tree::new_module(name.clone(), visibility, condition));
                }
                self.path.push(name);
                visit::walk_item(self, item);
                let _ = self.path.pop();
            }
            ast::ItemKind::Use(ref use_tree) => {
                {
                    let tree = self.tree.subtree_at_path(&self.path).unwrap();
                    let visibility = if let ast::VisibilityKind::Public = item.vis.node {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };

                    add_use_tree(tree, visibility, "", use_tree);
                }

                visit::walk_item(self, item);
            }
            _ => {
                visit::walk_item(self, item);
            }
        }
    }

    fn visit_mod(
        &mut self,
        m: &'a ast::Mod,
        _s: source_map::Span,
        _attrs: &[ast::Attribute],
        _n: ast::NodeId,
    ) {
        visit::walk_mod(self, m);
        if !self.config.include_orphans {
            return;
        }
        let path = &self.path;
        if let Ok(candidates) = Builder::find_orphan_candidates(path, &self.config.ignored_files) {
            let tree = self.tree.subtree_at_path(path).unwrap();
            let names: Vec<_> = tree.subtree_names();
            for candidate in candidates {
                if !names.contains(&candidate) {
                    tree.insert(Tree::new_orphan(candidate.clone()));
                }
            }
        }
    }

    fn visit_mac(&mut self, mac: &ast::Mac) {
        visit::walk_mac(self, mac);
    }
}

fn add_use_tree(tree: &mut Tree, visibility: Visibility, prefix: &str, use_tree: &ast::UseTree) {
    let prefix = if prefix.is_empty() {
        String::new()
    } else {
        prefix.to_string()
    };

    match use_tree.kind {
        ast::UseTreeKind::Simple(_, ..) => {
            let path = pprust::path_to_string(&use_tree.prefix);
            if path != "self" {
                tree.insert_use((visibility, prefix + &path));
            } else {
                let mut prefix = prefix;
                let new_len = prefix.len() - 2;
                prefix.truncate(new_len);
                tree.insert_use((visibility, prefix));
            }
        }
        ast::UseTreeKind::Glob => {
            let use_path = if use_tree.prefix.segments.is_empty() {
                prefix + "*"
            } else {
                prefix + &pprust::path_to_string(&use_tree.prefix) + "::*"
            };
            tree.insert_use((visibility, use_path));
        }
        ast::UseTreeKind::Nested(ref items) => {
            let prefix_ = if use_tree.prefix.segments.is_empty() {
                prefix
            } else {
                prefix + &pprust::path_to_string(&use_tree.prefix) + "::"
            };
            for (sub_tree, _) in items {
                add_use_tree(tree, visibility, &prefix_, sub_tree);
            }
        }
    }
}
