use std::{fs, io, path};

use syntax::{ast, visit, codemap};

use tree::{Tree, Visibility};

pub struct Config {
    pub include_orphans: bool,
    pub ignored_files: Vec<path::PathBuf>,
}

pub struct Builder<'a> {
    tree: Tree,
    path: Vec<String>,
    config: Config,
    codemap: &'a codemap::CodeMap,
}

impl<'a> Builder<'a> {
    pub fn new(config: Config, crate_name: String, codemap: &'a codemap::CodeMap) -> Self {
        let tree = Tree::new_crate(crate_name);
        Builder {
            tree: tree,
            path: vec![],
            config: config,
            codemap: codemap,
        }
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    fn find_orphan_candidates(path: &[String],
                              ignored_paths: &[path::PathBuf])
                              -> Result<Vec<String>, io::Error> {
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
            entry.path().file_stem().map_or(None, |s| s.to_str().map(|s| s.to_string()))
        }
        let mut orphans = vec![];
        if try!(fs::metadata(&dir_path)).is_dir() {
            for entry in try!(fs::read_dir(&dir_path)).filter_map(|e| e.ok()).filter(is_mod) {
                if !ignored_paths.contains(&entry.path()) {
                    if let Some(name) = file_name(&entry) {
                        orphans.push(name);
                    }
                }
            }
        }
        Ok(orphans)
    }

    fn sanitize_condition(condition: String) -> String {
        // this basically strips newlines and redundant whitespace:
        let mut sanitized = String::with_capacity(condition.len());
        let mut last_char_was_whitespace = false;
        for c in condition.chars() {
            let is_whitespace = (c == ' ') | (c == '\t');
            let is_newline = (c == '\n') | (c == '\r');
            match (last_char_was_whitespace, is_whitespace, is_newline) {
                (true, _, true) => {
                    last_char_was_whitespace = true;
                }
                (_, _, true) => {
                    last_char_was_whitespace = true;
                    sanitized.push(' ');
                }
                (true, true, _) => {
                    last_char_was_whitespace = true;
                }
                _ => {
                    last_char_was_whitespace = is_whitespace;
                    sanitized.push(c);
                }
            }
        }
        sanitized
    }
}

impl<'v, 'a> visit::Visitor<'v> for Builder<'a> {
    fn visit_item(&mut self, item: &ast::Item) {
        let condition = item.attrs
            .iter()
            .find(|attr| {
                match attr.node.value.node {
                    ast::MetaItemKind::List(ref n, _) if (n == "cfg") => true,
                    _ => false,
                }
            })
            .map(|attr| {
                self.codemap
                    .span_to_snippet(attr.span)
                    .map(|string| Builder::sanitize_condition(string))
                    .unwrap_or("".to_string())
            });
        if let ast::ItemKind::Mod(_) = item.node {
            let name = item.ident.to_string();
            {
                let mut tree = self.tree.subtree_at_path(&self.path).unwrap();
                let visibility = if item.vis == ast::Visibility::Public {
                    Visibility::Public
                } else {
                    Visibility::Private
                };
                tree.insert(Tree::new_module(name.clone(), visibility, condition));
            }
            self.path.push(name);
            visit::walk_item(self, item);
            let _ = self.path.pop();
        } else {
            visit::walk_item(self, item);
        }
    }

    fn visit_mod(&mut self, m: &'v ast::Mod, _s: codemap::Span, _n: ast::NodeId) {
        visit::walk_mod(self, m);
        if !self.config.include_orphans {
            return;
        }
        let path = &self.path;
        if let Ok(candidates) = Builder::find_orphan_candidates(path, &self.config.ignored_files) {
            let mut tree = self.tree.subtree_at_path(path).unwrap();
            let names: Vec<_> = tree.subtree_names();
            for candidate in candidates {
                if !names.contains(&candidate) {
                    tree.insert(Tree::new_orphan(candidate.clone()));
                }
            }
        }
    }

    fn visit_mac(&mut self, mac: &'v ast::Mac) {
        visit::walk_mac(self, mac);
    }
}
