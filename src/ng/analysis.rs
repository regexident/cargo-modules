use error::Error;
use manifest::Target;
use ng::graph::{Graph, GraphBuilder, Visibility, SEP};
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::io::Error as IoError;
use std::path::PathBuf;
use syntax::ast::{Attribute, Crate, Item, ItemKind, Mac, Mod, NodeId, VisibilityKind};
use syntax::parse::{self, ParseSess};
use syntax::source_map::{FilePathMapping, Span};
use syntax::visit::{self, Visitor};

const DIR_SEP: &str = "/";

struct Builder<'a> {
    graph_builder: GraphBuilder,
    ignored_files: &'a Vec<PathBuf>,
    path: Vec<String>,
}

impl<'a> Builder<'a> {
    fn new(ignored_files: &'a Vec<PathBuf>) -> Self {
        Builder {
            graph_builder: GraphBuilder::new(),
            ignored_files,
            path: vec![],
        }
    }

    fn path_str(&self) -> String {
        self.path.join(SEP)
    }
}

pub fn build_graph<'a>(target: &Target, ignored_files: &'a Vec<PathBuf>) -> Result<Graph, Error> {
    syntax::with_globals(|| {
        let parse_session = ParseSess::new(FilePathMapping::empty());
        let crate_: Crate =
            match parse::parse_crate_from_file(&target.src_path(), &parse_session) {
                Ok(_) if parse_session.span_diagnostic.has_errors() => Err(None),
                Ok(krate) => Ok(krate),
                Err(e) => Err(Some(e)),
            }
            .map_err(|e| Error::Syntax(format!("{:?}", e)))?;
        let mut builder = Builder::new(ignored_files);
        builder.graph_builder.add_crate_root(target.name());
        builder.path.push(target.name().to_owned());

        // Starting point is Builder.visit_mod with the crate
        builder.visit_mod(
            &crate_.module,
            crate_.span,
            &crate_.attrs[..],
            NodeId::from(0 as u32),
        );

        builder
            .graph_builder
            .build()
            .map_err(|e| Error::Syntax(format!("{:?}", e)))
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
        .filter(|e| !ignored_files.contains(&e.path()))
        .filter_map(|e| file_name(&e))
        .collect::<Vec<_>>())
}

impl<'a> Visitor<'a> for Builder<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        match item.node {
            ItemKind::Mod(_) => {
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
                let path = self.path_str();
                let name = item.ident.to_string();
                let visibility = match item.vis.node {
                    VisibilityKind::Public => Visibility::Public,
                    _ => Visibility::Private,
                };
                self.graph_builder.add_mod(&path, &name, visibility);
                self.path.push(name);
                visit::walk_item(self, item);
                self.path.pop();
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

        // NOTE: This is useful for orphaned modules.
        // FIXME: I am not sure how it's supposed to work though.
        // let path = &"";
        // let name = &"";
        // let visibility = Visibility::Public;
        // self.add_mod(path, name, visibility);

        if let Ok(candidates) = find_orphan_candidates(&self.path, &self.ignored_files[..]) {
            println!("{:?}", candidates);
        }
    }
}
