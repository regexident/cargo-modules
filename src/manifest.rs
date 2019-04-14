use error::Error;
use json;
use std::default::Default;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Manifest {
    pub edition: Edition,
    pub targets: Vec<Target>,
}

impl Manifest {
    pub fn from_str(src: &str) -> Result<Self, Error> {
        let mut j = json::parse(src).map_err(Error::InvalidManifestJson)?;

        let edition: Edition = match j["edition"].as_str() {
            Some("2015") => Edition::E2015,
            Some("2018") => Edition::E2018,
            Some(unknown) => panic!("Unrecognized value for edition \"{}\"", unknown),
            None => Edition::default(),
        };

        let targets: Vec<Target> = j["targets"].members_mut().map(Target::from_json).collect();

        Result::Ok(Manifest { edition, targets })
    }

    pub fn custom_builds(&self) -> Vec<&Target> {
        self.targets
            .iter()
            .filter(|t| t.is_custom_build())
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edition {
    E2015,
    E2018,
}

impl Default for Edition {
    fn default() -> Self {
        Edition::E2015
    }
}

#[derive(Debug, PartialEq)]
pub struct Target {
    kind: Vec<String>,
    crate_types: Vec<String>,
    name: String,
    src_path: PathBuf,
    edition: Option<String>,
}

impl Target {
    const LIB_KINDS: [&'static str; 4] = ["lib", "rlib", "dylib", "staticlib"];

    pub fn src_path(&self) -> &PathBuf {
        &self.src_path
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
        let edition: Option<String> = j["edition"].take_string();
        Target {
            kind,
            crate_types,
            name,
            src_path,
            edition,
        }
    }

    fn is_bin(&self) -> bool {
        self.kind.contains(&String::from("bin"))
    }

    fn is_custom_build(&self) -> bool {
        self.kind.contains(&String::from("custom-build"))
    }

    fn is_lib(&self) -> bool {
        self.kind.iter().any(|k| Self::LIB_KINDS.contains(&&k[..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn read_manifest(filename: &str) -> Manifest {
        let manifest_str: String =
            fs::read_to_string(filename).expect("manifest file cannot be read");
        Manifest::from_str(&manifest_str).expect("manifest cannot be read")
    }

    #[test]
    fn manifest_with_edition2018_can_be_parsed() {
        let manifest = read_manifest("test-resources/example-edition-2018.json");
        assert_eq!(Edition::E2018, manifest.edition);
    }

    #[test]
    fn manifest_with_edition2015_can_be_parsed() {
        let manifest = read_manifest("test-resources/example-edition-2015.json");
        assert_eq!(Edition::E2015, manifest.edition);
    }

    #[test]
    fn manifest_without_edition_can_be_parsed() {
        let manifest = read_manifest("test-resources/example-no-edition.json");
        assert_eq!(Edition::E2015, manifest.edition);
    }

    #[test]
    fn manifest_for_simple_lib() {
        let manifest = read_manifest("test-resources/example-lib.json");
        assert_eq!(
            Target {
                kind: vec!(String::from("lib")),
                crate_types: vec!(String::from("lib")),
                name: String::from("example-lib"),
                src_path: Path::new("/home/muhuk/Documents/code/example-lib/src/lib.rs")
                    .to_path_buf(),
                edition: Some(String::from("2018"))
            },
            manifest.targets[0]
        );
        assert!(manifest.targets[0].is_lib());
        assert!(!manifest.targets[0].is_bin());
    }

    #[test]
    fn manifest_for_simple_bin() {
        let manifest = read_manifest("test-resources/example-bin.json");
        assert_eq!(
            Target {
                kind: vec!(String::from("bin")),
                crate_types: vec!(String::from("bin")),
                name: String::from("example-bin"),
                src_path: Path::new("/home/muhuk/Documents/code/example-bin/src/main.rs")
                    .to_path_buf(),
                edition: Some(String::from("2018"))
            },
            manifest.targets[0]
        );
        assert!(manifest.targets[0].is_bin());
        assert!(!manifest.targets[0].is_lib());
    }

    #[test]
    fn manifest_with_custom_build() {
        let manifest = read_manifest("test-resources/example-custom-build.json");
        assert_eq!(
            vec![
                Target {
                    kind: vec!(String::from("lib")),
                    crate_types: vec!(String::from("lib")),
                    name: String::from("example-custom-build"),
                    src_path: Path::new(
                        "/home/muhuk/Documents/code/example-custom-build/src/lib.rs"
                    )
                    .to_path_buf(),
                    edition: Some(String::from("2018"))
                },
                Target {
                    kind: vec!(String::from("custom-build")),
                    crate_types: vec!(String::from("bin")),
                    name: String::from("build-script-build"),
                    src_path: Path::new("/home/muhuk/Documents/code/example-custom-build/build.rs")
                        .to_path_buf(),
                    edition: Some(String::from("2018"))
                }
            ],
            manifest.targets
        );
        assert!(manifest.targets[1].is_custom_build());
    }
}
