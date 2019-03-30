use error::Error;
use json;
use std::default::Default;

#[derive(Debug, Default)]
pub struct Manifest {
    pub edition: Edition,
}

impl Manifest {
    fn from_str(src: &str) -> Result<Self, Error> {
        let j = json::parse(src).map_err(Error::InvalidManifestJson)?;

        println!("{:?}", j);

        let edition: Edition = match j["edition"].as_str() {
            Some("2015") => Edition::E2015,
            Some("2018") => Edition::E2018,
            Some(unknown) => panic!("Unrecognized value for edition \"{}\"", unknown),
            None => Edition::default(),
        };

        Result::Ok(Manifest { edition })
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
}
