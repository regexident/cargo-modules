extern crate arrayvec;
extern crate colored;
extern crate json;
extern crate petgraph;
extern crate structopt;
extern crate syntax;

mod builder;
mod dot_printer;
mod error;
mod manifest;
mod ng; // TODO: Remove this.
mod printer;
mod tree;

use std::path;
use std::process;

use syntax::ast::{Crate, NodeId};
use syntax::parse::{self, ParseSess};
use syntax::source_map;
use syntax::visit::Visitor;

use structopt::StructOpt;

use colored::*;

use builder::Builder;
use builder::Config as BuilderConfig;

use error::Error;

use manifest::{Edition, Manifest, Target};

use ng::analysis;
use ng::graph::Graph;
use ng::tree_printer;

use printer::Config as PrinterConfig;
use printer::Printer;

use dot_printer::Config as DotPrinterConfig;
use dot_printer::DotPrinter;

fn choose_target<'a>(args: &Arguments, manifest: &'a Manifest) -> Result<&'a Target, Error> {
    if args.lib {
        // If `--lib` is enabled use the first library target.
        manifest.lib()
    } else if let Some(ref name) = args.bin {
        // If a binary target is specified use that.
        manifest.bin(name)
    } else if manifest.targets().len() == 1 {
        // If neither `--lib` is enabled nor `--bin` target is specified but
        // there is only one target, use that.
        Ok(manifest.targets().first().unwrap())
    } else {
        // If there are multiple targets use the first library target.
        manifest
            .lib()
            .or_else(|_| Err(Error::NoTargetProvided(manifest.bin_names())))
    }
}

fn run_2018(args: &Arguments, manifest: &Manifest) -> Result<(), Error> {
    // TODO: Check to see if build scripts really need to be ignored.
    //       Seems like they are not mistaken as orphans anyway.
    let build_scripts: Vec<path::PathBuf> = manifest
        .custom_builds()
        .iter()
        .map(|t| t.src_path().clone())
        .collect();
    let target: &Target = choose_target(args, &manifest)?;
    let include_orphans: bool = args.orphans;
    let enable_color: bool = if cfg!(target_os = "windows") {
        false
    } else {
        !args.plain
    };
    colored::control::set_override(enable_color);

    eprintln!("{}", "Warning: Edition 2018 support is unstable.".red());

    match args.command {
        Command::Graph { .. } => {
            eprintln!(
                "\n{}\n{}",
                "graph is not implemented for Edition 2018 yet.".red(),
                "Try removing --enable-edition-2018".yellow()
            );
            Ok(())
        }
        Command::Tree => {
            let ignored_files = &build_scripts;
            let graph: Graph = analysis::build_graph(target, ignored_files)?;
            tree_printer::print(&graph, include_orphans)
        }
    }
}

fn run(args: &Arguments) -> Result<(), Error> {
    let manifest: Manifest = {
        let output = process::Command::new("cargo")
            .arg("metadata")
            .args(&["--no-deps", "--format-version", "1"])
            .output()
            .map_err(Error::CargoExecutionFailed)?;
        let stdout = output.stdout;
        if !output.status.success() {
            let error =
                String::from_utf8(output.stderr).expect("Failed reading cargo stderr output");
            return Err(Error::InvalidManifest(error));
        }
        let json_string = String::from_utf8(stdout).expect("Failed reading cargo output");
        Manifest::from_str(&json_string)?
    };

    // TODO: Check to see if build scripts really need to be ignored.
    //       Seems like they are not mistaken as orphans anyway.
    let build_scripts: Vec<path::PathBuf> = manifest
        .custom_builds()
        .iter()
        .map(|t| t.src_path().clone())
        .collect();

    let target: &Target = choose_target(args, &manifest)?;

    if args.enable_edition_2018 && target.edition == Edition::E2018 {
        return run_2018(args, &manifest);
    }

    let parse_session = ParseSess::new(source_map::FilePathMapping::empty());

    syntax::with_globals(|| {
        let krate: Crate =
            match parse::parse_crate_from_file(target.src_path().as_ref(), &parse_session) {
                Ok(_) if parse_session.span_diagnostic.has_errors() => Err(None),
                Ok(krate) => Ok(krate),
                Err(e) => Err(Some(e)),
            }
            .map_err(|e| Error::Syntax(format!("{:?}", e)))?;

        let builder_config = BuilderConfig {
            include_orphans: args.orphans,
            ignored_files: build_scripts,
        };
        let mut builder = Builder::new(
            builder_config,
            target.name().clone(),
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

    /// **Experimental** Enable support for edition 2018 of Rust (ignored)
    #[structopt(long = "enable-edition-2018")]
    enable_edition_2018: bool,

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
        println!("{} {}", "error:".red(), error);
    }
}
