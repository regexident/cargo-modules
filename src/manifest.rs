use error::Error;
use json;
use std::default::Default;

#[derive(Debug, Default)]
pub struct Manifest {
    pub edition: Edition,
    pub targets: Vec<Target>,
}

impl Manifest {
    fn from_str(src: &str) -> Result<Self, Error> {
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
pub enum Target {
    Lib {
        kind: Vec<String>,
        crate_types: Vec<String>,
        name: String,
        src_path: String,
        edition: Option<String>,
    },
}

impl Target {
    fn from_json(j: &mut json::JsonValue) -> Target {
        Target::Lib {
            kind: {
                assert!(j["kind"].is_array());
                j["kind"]
                    .members_mut()
                    .map(|k| k.take_string().unwrap())
                    .collect()
            },
            crate_types: {
                assert!(j["crate_types"].is_array());
                j["crate_types"]
                    .members_mut()
                    .map(|k| k.take_string().unwrap())
                    .collect()
            },
            name: j["name"].take_string().expect("name is missing"),
            src_path: j["src_path"].take_string().expect("src_path is missing"),
            edition: j["edition"].take_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn manifest_with_edition2018_can_be_parsed() {
        let manifest_filename = "test-resources/example-edition-2018.json";
        let manifest_str: String =
            fs::read_to_string(manifest_filename).expect("manifest file cannot be read");
        let manifest = Manifest::from_str(&manifest_str).expect("manifest cannot be read");
        assert_eq!(Edition::E2018, manifest.edition);
    }

    #[test]
    fn manifest_with_edition2015_can_be_parsed() {
        let manifest_filename = "test-resources/example-edition-2015.json";
        let manifest_str: String =
            fs::read_to_string(manifest_filename).expect("manifest file cannot be read");
        let manifest = Manifest::from_str(&manifest_str).expect("manifest cannot be read");
        assert_eq!(Edition::E2015, manifest.edition);
    }

    #[test]
    fn manifest_without_edition_can_be_parsed() {
        let manifest_filename = "test-resources/example-no-edition.json";
        let manifest_str: String =
            fs::read_to_string(manifest_filename).expect("manifest file cannot be read");
        let manifest = Manifest::from_str(&manifest_str).expect("manifest cannot be read");
        assert_eq!(Edition::E2015, manifest.edition);
    }

    #[test]
    fn manifest_for_simple_lib() {
        let manifest_filename = "test-resources/example-lib.json";
        let manifest_str: String =
            fs::read_to_string(manifest_filename).expect("manifest file cannot be read");
        let manifest = Manifest::from_str(&manifest_str).expect("manifest cannot be read");
        assert_eq!(
            Target::Lib {
                kind: vec!(String::from("lib")),
                crate_types: vec!(String::from("lib")),
                name: String::from("example-lib"),
                src_path: String::from("/home/muhuk/Documents/code/example-lib/src/lib.rs"),
                edition: Some(String::from("2018"))
            },
            manifest.targets[0]
        );
    }
}
