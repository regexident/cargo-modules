use std::{fs, io, path};

use syntax::{ast, attr, visit, codemap};

use module::{Module, Kind as ModuleKind};

pub struct Config {
    pub target_name: String,
    pub include_tests: bool,
    pub include_orphans: bool,
    pub ignored_files: Vec<path::PathBuf>,
}

pub struct Visitor {
    root: Module,
    path: Vec<String>,
    config: Config,
}

impl Visitor {
    pub fn new(config: Config) -> Self {
        let root_kind = ModuleKind::Public;
        let root_module = Module::new(config.target_name.clone(), root_kind);
        Visitor {
            root: root_module,
            path: vec![],
            config: config,
        }
    }

    pub fn print_tree(&self) {
        self.root.print_tree(&mut vec![]);
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
}

impl<'v> visit::Visitor<'v> for Visitor {
    fn visit_item(&mut self, item: &ast::Item) {
        let is_test_cfg = item.attrs.iter().fold(false, |flag, attr| {
            match attr.node.value.node {
                ast::MetaItemKind::List(ref n, ref items) if (n == "cfg") => {
                    flag | attr::contains_name(items.as_slice(), "test")
                }
                _ => flag,
            }
        });
        if !self.config.include_tests && is_test_cfg {
            return;
        }
        let kind = if is_test_cfg {
            ModuleKind::Test
        } else if item.vis == ast::Visibility::Public {
            ModuleKind::Public
        } else {
            ModuleKind::Private
        };
        if let ast::ItemKind::Mod(_) = item.node {
            let name = item.ident.to_string();
            {
                let mut module = self.root.submodule_at_path(&self.path).unwrap();
                module.insert(name.clone(), kind);
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
        if let Ok(candidates) = Visitor::find_orphan_candidates(path, &self.config.ignored_files) {
            let mut module = self.root.submodule_at_path(path).unwrap();
            let names: Vec<_> = module.submodule_names();
            for candidate in candidates {
                if !names.contains(&candidate) {
                    module.insert(candidate.clone(), ModuleKind::Orphaned);
                }
            }
        }
    }

    fn visit_mac(&mut self, mac: &'v ast::Mac) {
        visit::walk_mac(self, mac);
    }
}
