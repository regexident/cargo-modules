extern crate syntex_syntax as syntax;
extern crate colored;
#[macro_use]
extern crate clap;
extern crate json;

mod builder;
mod printer;
mod tree;

use std::{io, path};
use std::process::Command;

use syntax::parse::{self, ParseSess};
use syntax::visit::Visitor;

use clap::{App, Arg};

use colored::*;

use builder::Builder;
use builder::Config as BuilderConfig;

use printer::Printer;
use printer::Config as PrinterConfig;

pub enum Error {
    CargoExecutionFailed(io::Error),
    InvalidManifestJson(json::JsonError),
    NoLibraryTargetFound,
    NoMatchingBinaryTargetFound,
    NoTargetProvided,
    Syntax(String),
}

fn get_manifest() -> Result<json::JsonValue, Error> {
    let output = Command::new("cargo").arg("read-manifest").output();
    let stdout = try!(output.map_err(Error::CargoExecutionFailed)).stdout;
    let json_string = String::from_utf8(stdout).expect("Failed reading cargo output");
    json::parse(&json_string).map_err(Error::InvalidManifestJson)
}

pub fn get_target_config<'a>(target_cfgs: &'a [json::JsonValue],
                             args: &clap::ArgMatches)
                             -> Result<&'a json::JsonValue, Error> {
    fn is_lib(cfg: &json::JsonValue) -> bool {
        let is_lib = cfg["kind"].contains("lib");
        let is_rlib = cfg["kind"].contains("rlib");
        let is_staticlib = cfg["kind"].contains("staticlib");
        is_lib || is_rlib || is_staticlib
    }
    if args.is_present("lib") {
        target_cfgs.into_iter()
            .find(|cfg| is_lib(cfg))
            .ok_or(Error::NoLibraryTargetFound)
    } else if let Some(name) = args.value_of("bin") {
        target_cfgs.into_iter()
            .find(|cfg| cfg["kind"].contains("bin") && cfg["name"] == name)
            .ok_or(Error::NoMatchingBinaryTargetFound)
    } else if target_cfgs.len() == 1 {
        Ok(&target_cfgs[0])
    } else {
        target_cfgs.into_iter()
            .find(|cfg| is_lib(cfg))
            .ok_or(Error::NoTargetProvided)
    }
}

fn get_build_scripts(target_cfgs: &[json::JsonValue]) -> Vec<path::PathBuf> {
    target_cfgs.into_iter()
        .filter_map(|cfg| {
            if cfg["kind"].contains("custom-build") {
                cfg["src_path"].as_str().map(|s| path::Path::new("./").join(s))
            } else {
                None
            }
        })
        .collect()
}

fn run(args: &clap::ArgMatches) -> Result<(), Error> {
    let json = try!(get_manifest());
    let target_cfgs: Vec<_> = json["targets"].members().cloned().collect();
    let build_scripts = get_build_scripts(&target_cfgs);
    let target_config = try!(get_target_config(&target_cfgs, args));
    let target_name = target_config["name"].as_str().expect("Expected `name` property.");
    let src_path = target_config["src_path"].as_str().expect("Expected `src_path` property.");
    let parse_session = ParseSess::new();
    let cfgs = vec![];
    let krate = try!(match parse::parse_crate_from_file(src_path.as_ref(), cfgs, &parse_session) {
            Ok(_) if parse_session.span_diagnostic.has_errors() => Err(None),
            Ok(krate) => Ok(krate),
            Err(e) => Err(Some(e)),
        }
        .map_err(|e| Error::Syntax(format!("{:?}", e))));
    let builder_config = BuilderConfig {
        include_orphans: args.is_present("orphans"),
        ignored_files: build_scripts,
    };
    let mut builder = Builder::new(builder_config,
                                   target_name.to_string(),
                                   parse_session.codemap());
    builder.visit_mod(&krate.module, krate.span, 0);
    let printer_config = PrinterConfig { colored: !args.is_present("plain") };
    let printer = Printer::new(printer_config);
    println!("");
    let tree = builder.tree();
    tree.accept(&mut vec![], &printer);
    println!("");
    Ok(())
}

fn main() {
    let orphans_arg = Arg::with_name("orphans")
        .short("o")
        .long("orphans")
        .help("Include orphaned modules (i.e. unused files in /src).");
    let lib_arg = Arg::with_name("lib")
        .short("l")
        .long("lib")
        .help("List modules of this package's library (overrides '--bin')");
    let bin_arg = Arg::with_name("bin")
        .short("b")
        .long("bin")
        .value_name("NAME")
        .help("List modules of the specified binary")
        .takes_value(true);
    let plain_arg = Arg::with_name("plain")
        .short("p")
        .long("plain")
        .help("Plain uncolored output.");
    let arguments = App::new("cargo-modules")
        .about("Print a crate's module tree.\n\
        \n\
        (On 'Windows' systems coloring is disabled. Sorry. PRs welcome.)")
        .arg(orphans_arg)
        .arg(lib_arg)
        .arg(bin_arg)
        .arg(plain_arg)
        .get_matches();
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
