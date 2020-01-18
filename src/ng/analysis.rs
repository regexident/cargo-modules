//! Graph generation AST traversal.

use std::ffi::OsStr;
use std::fs;
use std::io::Error as IoError;
use std::path::PathBuf;
use std::string::ToString;

use error::Error;
use manifest::Target;
use ng::graph::{Graph, GraphBuilder, Visibility, GLOB, SEP};

use syntax::ast::{
    Attribute, Crate, Item, ItemKind, Mac, Mod, NodeId, UseTree, UseTreeKind, VisibilityKind,
};
use syntax::print::pprust;
use syntax::sess::ParseSess;
// use syntax::source_map::{edition::Edition, FilePathMapping, SourceMap, Span, Symbol};
use syntax::visit::{self, Visitor};

use rustc_span::source_map::{edition::Edition, FilePathMapping, SourceMap, Span, Symbol};

use rustc_parse;

const SOURCE_DIR: &str = "./src/";
const DIR_SEP: &str = "/";

struct Builder<'a> {
    graph_builder: GraphBuilder,
    ignored_files: &'a [PathBuf],
    path: Vec<String>,
    source_map: &'a SourceMap,
}

impl<'a> Builder<'a> {
    fn new(edition: Edition, ignored_files: &'a [PathBuf], source_map: &'a SourceMap) -> Self {
        Builder {
            graph_builder: GraphBuilder::new(edition),
            ignored_files,
            path: vec![],
            source_map,
        }
    }

    fn path_str(&self) -> String {
        self.path.join(SEP)
    }

    fn add_use_tree(&mut self, prefix: &str, use_tree: &UseTree, visibility: Visibility) {
        let path_string = pprust::path_to_string(&use_tree.prefix);
        let new_prefix = if path_string != "self" {
            [prefix, &path_string].join(SEP)
        } else {
            prefix.to_string()
        };

        match use_tree.kind {
            UseTreeKind::Simple(..) => self.graph_builder.add_use(&self.path_str(), new_prefix),
            UseTreeKind::Nested(ref children) => {
                for (child, _) in children {
                    self.add_use_tree(&new_prefix, child, visibility);
                }
            }
            UseTreeKind::Glob => {
                self.graph_builder
                    .add_use(&self.path_str(), format!("{}{}{}", new_prefix, SEP, GLOB));
            }
        }
    }
}

pub fn build_graph<'a>(
    edition: Edition,
    target: &Target,
    ignored_files: &'a [PathBuf],
) -> Result<Graph, Error> {
    syntax::with_globals(edition, || {
        let parse_session = ParseSess::new(FilePathMapping::empty());
        let crate_: Crate =
            match rustc_parse::parse_crate_from_file(&target.src_path(), &parse_session) {
                Ok(_) if parse_session.span_diagnostic.has_errors() => Err(None),
                Ok(krate) => Ok(krate),
                Err(e) => Err(Some(e)),
            }
            .map_err(|e| Error::Syntax(format!("{:?}", e)))?;
        let mut builder = Builder::new(edition, ignored_files, parse_session.source_map());
        builder.graph_builder.add_crate_root(target.name());
        builder.path.push(target.name().to_owned());

        // Starting point is Builder.visit_mod with the crate
        builder.visit_mod(
            &crate_.module,
            crate_.span,
            &crate_.attrs[..],
            NodeId::from(0 as u32),
        );

        Ok(builder.graph_builder.build())
    })
}

// This function walks a single directory non-recursively and it is called
// every time a module is visited.  It would have been great not to
// repetitively call this but figure out orphan candidates at once.
// Unfortunately that solution, although elegant, becomes quite complex. And
// we would still be relying on an assumption that maps modules to directories
// as the original version relies on.  A real elegant solution would be to get
// the filename somehow from syntax module, in the span or something.
fn find_orphan_candidates(
    path: &[String],
    ignored_files: &[PathBuf],
) -> Result<Vec<String>, IoError> {
    let mut dir_path = SOURCE_DIR.to_string();
    for name in path.iter() {
        dir_path.push_str(name);
        dir_path.push_str(DIR_SEP);
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
        .filter(|e| !ignored_files.contains(&e.path()))
        .filter_map(|e| file_name(&e))
        .collect::<Vec<_>>())
}

impl<'a> Visitor<'a> for Builder<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        match item.kind {
            ItemKind::Mod(_) => {
                let path = self.path_str();
                let name = item.ident.to_string();
                let visibility = match item.vis.node {
                    VisibilityKind::Public => Visibility::Public,
                    _ => Visibility::Private,
                };
                let conditions: Option<String> = item
                    .attrs
                    .iter()
                    .find(|attr| attr.check_name(Symbol::intern("cfg")))
                    .map(|attr| {
                        self.source_map
                            .span_to_snippet(attr.span)
                            .map(|attr| attr)
                            .unwrap_or_else(|_| String::from(""))
                    });
                self.graph_builder.add_mod(
                    &path,
                    &name,
                    visibility,
                    conditions.as_ref().map(|x| &**x),
                );
                self.path.push(name);
                visit::walk_item(self, item);
                self.path.pop();
            }
            ItemKind::Use(ref use_tree) => {
                let visibility = if let VisibilityKind::Public = item.vis.node {
                    Visibility::Public
                } else {
                    Visibility::Private
                };
                self.add_use_tree("", use_tree, visibility);
                visit::walk_item(self, item);
            }
            _ => visit::walk_item(self, item),
        }
    }

    fn visit_mac(&mut self, macro_: &'a Mac) {
        visit::walk_mac(self, macro_);
    }

    fn visit_mod(&mut self, module: &'a Mod, _: Span, _: &[Attribute], _: NodeId) {
        // This is the default behavior:
        visit::walk_mod(self, module);

        // Add orphaned modules.
        if let Ok(candidates) = find_orphan_candidates(&self.path[1..], &self.ignored_files[..]) {
            for candidate in candidates {
                let path: String = [&self.path_str(), SEP, &candidate].concat();
                match self.graph_builder.find(&path) {
                    Some(_) => (), // Already visited, not an orphan.
                    None => {
                        self.graph_builder.add_orphan(&self.path_str(), &candidate);
                    }
                }
            }
        }
    }
}
