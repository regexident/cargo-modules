// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{fs::ReadDir, path::PathBuf};

pub struct PossibleOrphan {
    pub name: String,
    pub path: PathBuf,
}

pub struct PossibleOrphansIterator {
    read_dir: ReadDir,
}

impl PossibleOrphansIterator {
    pub fn new(read_dir: ReadDir) -> Self {
        Self { read_dir }
    }

    fn is_possible_identifier(&self, name: &str) -> bool {
        name.chars().all(|c| c.is_alphanumeric())
    }
}

impl Iterator for PossibleOrphansIterator {
    type Item = PossibleOrphan;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(dir_entry) = self.read_dir.next() {
            let entry_path = match dir_entry {
                Ok(dir_entry) => dir_entry.path(),
                Err(_) => {
                    // If we can't retrieve the entry, then skip over it:
                    continue;
                }
            };
            let file_stem = match entry_path.file_stem().and_then(|os_str| os_str.to_str()) {
                Some(name) => name,
                None => {
                    // A rust file should have a file-stem (aka file-name)
                    // if the file doesn't, then skip over it:
                    continue;
                }
            };
            let extension = entry_path.extension().and_then(|os_str| os_str.to_str());

            // Skip any files with names that could not be valid identifiers:
            if !self.is_possible_identifier(file_stem) {
                continue;
            }

            let ignored_names = ["lib", "main", "mod"];
            let is_ignored_name = ignored_names.contains(&file_stem);

            let path = if entry_path.is_dir() {
                // If it's a directory, then there might be a 'mod.rs' file within:
                entry_path.join("mod.rs")
            } else if extension == Some("rs") && !is_ignored_name {
                // If it's a '.rs' file then we already know the path:
                entry_path.clone()
            } else {
                // The directory can contain other arbitrary files
                // as such we simply skip over them:
                continue;
            };

            // Check if our file guesses actually exist on the file-system
            // if they don't, simply skip over them:
            if !path.exists() {
                continue;
            }

            let name = file_stem.to_owned();
            return Some(PossibleOrphan { name, path });
        }

        None
    }
}
