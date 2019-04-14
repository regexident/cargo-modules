extern crate colored;
extern crate json;
extern crate structopt;
extern crate syntax;

mod builder;
mod dot_printer;
mod error;
mod manifest;
mod printer;
mod tree;

use std::path;
use std::process;

use syntax::ast::NodeId;
use syntax::parse::{self, ParseSess};
use syntax::source_map;
use syntax::visit::Visitor;

use structopt::StructOpt;

use colored::*;

use builder::Builder;
use builder::Config as BuilderConfig;

use error::Error;

use manifest::Manifest;

use printer::Config as PrinterConfig;
use printer::Printer;

use dot_printer::Config as DotPrinterConfig;
use dot_printer::DotPrinter;

fn get_manifest() -> Result<json::JsonValue, Error> {
    let output = process::Command::new("cargo").arg("read-manifest").output();
    let stdout = try!(output.map_err(Error::CargoExecutionFailed)).stdout;
    let json_string = String::from_utf8(stdout).expect("Failed reading cargo output");
    json::parse(&json_string).map_err(Error::InvalidManifestJson)
}

fn get_target_config<'a>(
    target_cfgs: &'a [json::JsonValue],
    args: &Arguments,
) -> Result<&'a json::JsonValue, Error> {
    fn is_lib(cfg: &json::JsonValue) -> bool {
        let is_lib = cfg["kind"].contains("lib");
        let is_rlib = cfg["kind"].contains("rlib");
        let is_staticlib = cfg["kind"].contains("staticlib");
        is_lib || is_rlib || is_staticlib
    }
    if args.lib {
        target_cfgs
            .iter()
            .find(|cfg| is_lib(cfg))
            .ok_or(Error::NoLibraryTargetFound)
    } else if let Some(ref name) = args.bin {
        target_cfgs
            .iter()
            .find(|cfg| cfg["kind"].contains("bin") && cfg["name"] == name.as_ref())
            .ok_or(Error::NoMatchingBinaryTargetFound)
    } else if target_cfgs.len() == 1 {
        Ok(&target_cfgs[0])
    } else {
        target_cfgs
            .iter()
            .find(|cfg| is_lib(cfg))
            .ok_or(Error::NoTargetProvided)
    }
}

fn run(args: &Arguments) -> Result<(), Error> {
    let manifest: Manifest = {
        let output = process::Command::new("cargo").arg("read-manifest").output();
        let stdout = try!(output.map_err(Error::CargoExecutionFailed)).stdout;
        let json_string = String::from_utf8(stdout).expect("Failed reading cargo output");
        Manifest::from_str(&json_string)?
    };

    let json = try!(get_manifest());
    let target_cfgs: Vec<_> = json["targets"].members().cloned().collect();
    // TODO: Check to see if build scripts really need to be ignored.
    //       Seems like they are not mistaken as orphans anyway.
    let build_scripts: Vec<path::PathBuf> = manifest
        .custom_builds()
        .iter()
        .map(|t| path::Path::new(t.src_path()).to_path_buf())
        .collect();
    let target_config = try!(get_target_config(&target_cfgs, args));
    let target_name = target_config["name"]
        .as_str()
        .expect("Expected `name` property.");
    let src_path = target_config["src_path"]
        .as_str()
        .expect("Expected `src_path` property.");
    let parse_session = ParseSess::new(source_map::FilePathMapping::empty());

    syntax::with_globals(|| {
        let krate = try!(
            match parse::parse_crate_from_file(src_path.as_ref(), &parse_session) {
                Ok(_) if parse_session.span_diagnostic.has_errors() => Err(None),
                Ok(krate) => Ok(krate),
                Err(e) => Err(Some(e)),
            }
            .map_err(|e| Error::Syntax(format!("{:?}", e)))
        );

        let builder_config = BuilderConfig {
            include_orphans: args.orphans,
            ignored_files: build_scripts,
        };
        let mut builder = Builder::new(
            builder_config,
            target_name.to_string(),
            parse_session.source_map(),
        );
        builder.visit_mod(
            &krate.module,
            krate.span,
            &krate.attrs[..],
            NodeId::from(0_usize),
        );

        match args.command {
            Command::Graph {
                conditional,
                external,
                types,
            } => {
                let printer_config = DotPrinterConfig {
                    colored: !args.plain,
                    show_conditional: conditional,
                    show_external: external,
                    show_types: types,
                };
                println!("digraph something {{");
                let tree = builder.tree();
                let printer = DotPrinter::new(printer_config, tree);
                tree.accept(&mut vec![], &mut vec![], &printer);
                println!("}}");
            }
            Command::Tree => {
                let printer_config = PrinterConfig {
                    colored: !args.plain,
                };
                let printer = Printer::new(printer_config);
                println!();
                let tree = builder.tree();
                tree.accept(&mut vec![], &mut vec![], &printer);
                println!();
            }
        }

        Ok(())
    })
}

#[derive(StructOpt)]
#[structopt(
    name = "cargo-modules",
    about = "Print a crate's module tree or graph.",
    author = "",
    after_help = "If neither `--bin` nor `--example` are given,\n\
                  then if the project only has one bin target it will be run.\n\
                  Otherwise `--bin` specifies the bin target to run.\n\
                  At most one `--bin` can be provided.\n\
                  \n(On 'Windows' systems coloring is disabled. Sorry.)\n"
)]
struct Arguments {
    /// Include orphaned modules (i.e. unused files in /src).
    #[structopt(short = "o", long = "orphans")]
    orphans: bool,

    /// List modules of this package's library (overrides '--bin')
    #[structopt(short = "l", long = "lib")]
    lib: bool,

    /// Plain uncolored output.
    #[structopt(short = "p", long = "plain")]
    plain: bool,

    /// List modules of the specified binary
    #[structopt(short = "b", long = "bin")]
    bin: Option<String>,

    /// [Experimental] Enable support for edition 2018 of Rust (ignored)
    #[structopt(long = "enable-edition-2018")]
    _enable_edition_2018: bool,

    /// Sets an explicit crate path (ignored)
    #[structopt(name = "CRATE_DIR")]
    _dir: Option<String>,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "tree", about = "Print a crate's module tree.", author = "")]
    Tree,
    #[structopt(
        name = "graph",
        about = "Print a crate's module graph.",
        author = "",
        after_help = "If you have xdot installed on your system, you can run this using:\n\
                      `cargo modules graph | xdot -`"
    )]
    Graph {
        /// Show external types.
        #[structopt(short = "e", long = "external")]
        external: bool,
        /// Show conditional modules.
        #[structopt(short = "c", long = "conditional")]
        conditional: bool,
        /// Plain uncolored output.
        #[structopt(short = "t", long = "types")]
        types: bool,
    },
}

fn main() {
    let arguments = Arguments::from_args();

    if let Err(error) = run(&arguments) {
        let error_string = match error {
            Error::CargoExecutionFailed(error) => {
                format!("Error: Failed to run `cargo` command.\n{:?}", error)
            }
            Error::InvalidManifestJson(error) => {
                format!("Error: Failed to parse JSON response.\n{:?}", error)
            }
            Error::NoLibraryTargetFound => "Error: No library target found.".to_string(),
            Error::NoMatchingBinaryTargetFound => {
                "Error: No matching binary target found.".to_string()
            }
            Error::NoTargetProvided => "Error: Please specify a target to process.".to_string(),
            Error::Syntax(error) => format!("Error: Failed to parse: {}", error),
        };
        println!("{}", error_string.red());
    }
}
