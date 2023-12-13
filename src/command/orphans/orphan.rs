// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Orphan {
    pub name: String,
    pub file_path: PathBuf,
    pub parent_module_path: String,
    pub parent_file_path: PathBuf,
}
