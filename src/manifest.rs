use std::default::Default;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rustc_span::source_map::edition::Edition;

use error::Error;
use json;

#[derive(Debug)]
pub struct Package {
    edition: Edition,
    targets: Vec<Target>,
}

#[derive(Debug, Default)]
pub struct Manifest {
    packages: Vec<Package>,
}

impl Manifest {
    pub fn from_str(src: &str) -> Result<Self, Error> {
        let mut j = json::parse(src).map_err(Error::InvalidManifestJson)?;

        let packages = j["packages"]
            .members_mut()
            .map(|package| {
                let edition: Edition = Edition::from_str(package["edition"].as_str().unwrap()).unwrap();
                let targets: Vec<Target> = package["targets"]
                    .members_mut()
                    .map(Target::from_json)
                    .collect();

                Package { edition, targets }
            })
            .collect();

        Result::Ok(Manifest { packages })
    }

    fn all_targets(&self) -> impl Iterator<Item = &Target> {
        self.packages
            .iter()
            .flat_map(|package| package.targets.iter())
    }

    pub fn custom_builds(&self) -> Vec<&Target> {
        self.all_targets().filter(|t| t.is_custom_build()).collect()
    }

    /// All valid targets that can be used to display modules
    pub fn targets(&self) -> Vec<&Target> {
        self.all_targets()
            .filter(|t| t.is_bin() || t.is_lib() || t.is_proc_macro())
            .collect()
    }

    pub fn lib(&self) -> Result<&Target, Error> {
        self.all_targets()
            .find(|t| t.is_lib())
            .ok_or(Error::NoLibraryTargetFound)
    }

    pub fn bin(&self, name: &str) -> Result<&Target, Error> {
        self.all_targets()
            .find(|t| t.is_bin() && t.name() == name)
            .ok_or_else(|| {
                let names = self.bin_names();
                Error::NoMatchingBinaryTargetFound(names)
            })
    }

    pub fn bin_names(&self) -> Vec<String> {
        self.all_targets()
            .filter(|t| t.is_bin())
            .map(|t| t.name().to_string())
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub struct Target {
    kind: Vec<String>,
    crate_types: Vec<String>,
    name: String,
    src_path: PathBuf,
    pub edition: Edition,
}

impl Target {
    const LIB_KINDS: [&'static str; 4] = ["lib", "rlib", "dylib", "staticlib"];

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn src_path(&self) -> &PathBuf {
        &self.src_path
    }

    pub fn is_bin(&self) -> bool {
        self.kind.contains(&String::from("bin"))
    }

    pub fn is_lib(&self) -> bool {
        self.kind.iter().any(|k| Self::LIB_KINDS.contains(&&k[..]))
    }

    pub fn is_proc_macro(&self) -> bool {
        self.kind.contains(&String::from("proc-macro"))
    }

    fn from_json(j: &mut json::JsonValue) -> Target {
        let kind: Vec<String> = {
            assert!(j["kind"].is_array());
            j["kind"]
                .members_mut()
                .map(|k| k.take_string().unwrap())
                .collect()
        };
        let crate_types: Vec<String> = {
            assert!(j["crate_types"].is_array());
            j["crate_types"]
                .members_mut()
                .map(|k| k.take_string().unwrap())
                .collect()
        };
        let name: String = j["name"].take_string().expect("name is missing");
        let src_path: PathBuf =
            Path::new(&j["src_path"].take_string().expect("src_path is missing")).to_path_buf();
        let edition: Edition = Edition::from_str(j["edition"].as_str().unwrap()).unwrap();
        Target {
            kind,
            crate_types,
            name,
            src_path,
            edition,
        }
    }

    fn is_custom_build(&self) -> bool {
        self.kind.contains(&String::from("custom-build"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path;

    fn read_manifest(filename: &path::Path) -> Manifest {
        let manifest_str: String = fs::read_to_string(filename.with_extension("json"))
            .expect("manifest file cannot be read");
        Manifest::from_str(&manifest_str).expect("manifest cannot be read")
    }

    #[test]
    fn manifest_with_edition2018_can_be_parsed() {
        let manifest = read_manifest(path::Path::new("test-resources/example-lib-edition-2018"));
        assert_eq!(Edition::Edition2018, manifest.targets()[0].edition);
    }

    #[test]
    fn manifest_with_edition2015_can_be_parsed() {
        let manifest = read_manifest(path::Path::new("test-resources/example-lib-edition-2015"));
        assert_eq!(Edition::Edition2015, manifest.targets()[0].edition);
    }

    #[test]
    fn manifest_without_edition_can_be_parsed() {
        let manifest = read_manifest(path::Path::new("test-resources/example-lib-no-edition"));
        assert_eq!(Edition::Edition2015, manifest.targets()[0].edition);
    }

    #[test]
    fn manifest_for_simple_lib() {
        let resource_path = path::Path::new("test-resources/example-lib-edition-2018");
        let manifest = read_manifest(resource_path);
        assert_eq!(
            &Target {
                kind: vec!(String::from("lib")),
                crate_types: vec!(String::from("lib")),
                name: String::from("example-lib-edition-2018"),
                src_path: resource_path.join("src/lib.rs"),
                edition: Edition::Edition2018
            },
            manifest.targets()[0]
        );
        assert!(manifest.targets()[0].is_lib());
        assert!(!manifest.targets()[0].is_bin());
    }

    #[test]
    fn manifest_for_simple_bin() {
        let resource_path = path::Path::new("test-resources/example-bin");
        let manifest = read_manifest(resource_path);
        assert_eq!(
            vec![
                &Target {
                    kind: vec!(String::from("bin")),
                    crate_types: vec!(String::from("bin")),
                    name: String::from("example2"),
                    src_path: resource_path.join("src/bin/example2.rs"),
                    edition: Edition::Edition2018
                },
                &Target {
                    kind: vec!(String::from("bin")),
                    crate_types: vec!(String::from("bin")),
                    name: String::from("example"),
                    src_path: resource_path.join("src/bin/example.rs"),
                    edition: Edition::Edition2018
                }
            ],
            manifest.targets()
        );
        assert!(manifest.targets()[0].is_bin());
        assert!(!manifest.targets()[0].is_lib());
    }

    #[test]
    fn manifest_with_custom_build() {
        let resource_path = path::Path::new("test-resources/example-lib-edition-2018");
        let manifest = read_manifest(resource_path);
        assert_eq!(
            vec![
                &Target {
                    kind: vec!(String::from("lib")),
                    crate_types: vec!(String::from("lib")),
                    name: String::from("example-lib-edition-2018"),
                    src_path: resource_path.join("src/lib.rs"),
                    edition: Edition::Edition2018
                },
                &Target {
                    kind: vec!(String::from("custom-build")),
                    crate_types: vec!(String::from("bin")),
                    name: String::from("build-script-build"),
                    src_path: resource_path.join("build.rs"),
                    edition: Edition::Edition2018
                }
            ],
            manifest.all_targets().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![&Target {
                kind: vec!(String::from("lib")),
                crate_types: vec!(String::from("lib")),
                name: String::from("example-lib-edition-2018"),
                src_path: resource_path.join("src/lib.rs"),
                edition: Edition::Edition2018
            },],
            manifest.targets()
        );
        assert_eq!(
            vec![&Target {
                kind: vec!(String::from("custom-build")),
                crate_types: vec!(String::from("bin")),
                name: String::from("build-script-build"),
                src_path: resource_path.join("build.rs"),
                edition: Edition::Edition2018
            }],
            manifest.custom_builds()
        );
    }

    #[test]
    fn manifest_for_plugin() {
        let resource_path = path::Path::new("test-resources/example-plugin");
        let manifest = read_manifest(resource_path);
        assert_eq!(
            &Target {
                kind: vec!(String::from("dylib")),
                crate_types: vec!(String::from("dylib")),
                name: String::from("example-plugin"),
                src_path: resource_path.join("src/lib.rs"),
                edition: Edition::Edition2018
            },
            manifest.targets()[0]
        );
        assert!(manifest.targets()[0].is_lib());
        assert!(!manifest.targets()[0].is_bin());
    }

    #[test]
    fn manifest_for_proc_macro() {
        let resource_path = path::Path::new("test-resources/example-proc-macro");
        let manifest = read_manifest(resource_path);
        assert_eq!(
            &Target {
                kind: vec!(String::from("proc-macro")),
                crate_types: vec!(String::from("proc-macro")),
                name: String::from("example-proc-macro"),
                src_path: resource_path.join("src/lib.rs"),
                edition: Edition::Edition2018
            },
            manifest.targets()[0]
        );
        assert!(!manifest.targets()[0].is_lib());
        assert!(!manifest.targets()[0].is_bin());
        assert!(manifest.targets()[0].is_proc_macro());
    }

    #[test]
    fn manifest_for_bin_and_lib() {
        let resource_path = path::Path::new("test-resources/example-bin-and-lib");
        let manifest = read_manifest(resource_path);
        assert_eq!(
            vec![
                &Target {
                    kind: vec!(String::from("lib")),
                    crate_types: vec!(String::from("lib")),
                    name: String::from("example-bin-and-lib"),
                    src_path: resource_path.join("src/lib.rs"),
                    edition: Edition::Edition2018
                },
                &Target {
                    kind: vec!(String::from("bin")),
                    crate_types: vec!(String::from("bin")),
                    name: String::from("example-bin-and-lib"),
                    src_path: resource_path.join("src/main.rs"),
                    edition: Edition::Edition2018
                }
            ],
            manifest.targets()
        );
        assert!(manifest.targets()[0].is_lib());
        assert!(!manifest.targets()[0].is_bin());
        assert!(!manifest.targets()[0].is_proc_macro());
        assert!(!manifest.targets()[1].is_lib());
        assert!(manifest.targets()[1].is_bin());
        assert!(!manifest.targets()[1].is_proc_macro());
    }
}
