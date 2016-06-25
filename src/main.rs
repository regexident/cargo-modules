extern crate syntex_syntax as syntax;
extern crate colored;
#[macro_use]
extern crate clap;
extern crate json;

mod module;
mod visit;

use std::{env, io, ffi, path};
use std::process::Command;

use syntax::parse::{self, ParseSess};
use syntax::visit::Visitor;

use clap::{App, Arg};

use colored::*;

use visit::{Visitor as ModulesVisitor, Config as VisitorConfig};

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

fn has_colors() -> bool {
    fn is_good(var: Option<ffi::OsString>) -> bool {
        var.is_none() || var != Some("0".into())
    }
    (env::var_os("CLICOLOR_FORCE").or_else(|| Some("0".into())) != Some("0".into())) ||
    is_good(env::var_os("CLICOLOR"))
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
    let visitor_config = VisitorConfig {
        target_name: target_name.to_string(),
        include_tests: args.is_present("tests"),
        include_orphans: args.is_present("orphans"),
        ignored_files: build_scripts,
    };
    let mut visitor = ModulesVisitor::new(visitor_config);
    visitor.visit_mod(&krate.module, krate.span, 0);
    if has_colors() {
        println!("");
        if args.is_present("tests") {
            println!("{}", "Test modules".cyan().bold());
        }
        println!("{}", "Public modules".green().bold());
        println!("{}", "Private modules".yellow().bold());
        if args.is_present("orphans") {
            println!("{}", "Orphaned modules".red().bold());
        }
    }
    println!("");
    visitor.print_tree();
    println!("");
    Ok(())
}

fn main() {
    let tests_arg = Arg::with_name("tests")
        .short("t")
        .long("tests")
        .help("Include modules with `#[cfg(test)]` attribute.");
    let orphans_arg = Arg::with_name("orphans")
        .short("o")
        .long("orphans")
        .help("Include orphaned modules (unused files in /src).");
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
    let dir_arg = Arg::with_name("path")
        .value_name("CRATE_DIR")
        .help("Sets an explicit crate path (optional)")
        .takes_value(true);
    let arguments = App::new("cargo-modules")
        .about("Print a crate's module tree.\n\
        \n\
        (Set environment variable `CLICOLOR=0` to disable colors.)")
        .arg(tests_arg)
        .arg(orphans_arg)
        .arg(lib_arg)
        .arg(bin_arg)
        .arg(dir_arg)
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
