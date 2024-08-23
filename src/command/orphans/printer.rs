// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Printer for displaying module structure as a tree.

use ra_ap_ide::{self as ide};

use sugar_path::SugarPath as _;
use yansi::Paint as _;

use super::{options::Options, orphan::Orphan, theme::styles};

pub struct Printer<'a> {
    #[allow(dead_code)]
    options: &'a Options,
    #[allow(dead_code)]
    db: &'a ide::RootDatabase,
}

impl<'a> Printer<'a> {
    pub fn new(options: &'a Options, db: &'a ide::RootDatabase) -> Self {
        Self { options, db }
    }

    pub fn fmt(&self, f: &mut dyn std::io::Write, orphans: &[Orphan]) -> Result<(), anyhow::Error> {
        let styles = styles();

        if orphans.is_empty() {
            writeln!(f)?;
            writeln!(f, "{}", "No orphans found.".paint(styles.success))?;

            return Ok(());
        }

        let prefix_path = std::fs::canonicalize({
            let manifest_path = self.options.project.manifest_path.as_path();
            if manifest_path.is_file() {
                manifest_path.parent().expect("parent directory")
            } else {
                manifest_path
            }
        })
        .expect("canonical path");

        // The `canonicalize()` invoking can make sure the file path is meaningful.
        // But on Windows, this invoking will make the path be with verbatim path prefix.
        // So, we needs to make the path `simplified`, otherwise the `strip_prefix()` invoking will be failed.
        let prefix_path = dunce::simplified(&prefix_path).to_path_buf();

        writeln!(f)?;
        writeln!(f, "{count} orphans found:", count = orphans.len())?;
        writeln!(f)?;

        for orphan in orphans {
            let file_path = orphan
                .file_path
                .strip_prefix(&prefix_path)
                .expect("relative path")
                .to_slash_lossy(); // Change the slashes from `\` to `/` on Windows.

            let parent_module_path = &orphan.parent_module_path;
            let parent_file_path = orphan
                .parent_file_path
                .strip_prefix(&prefix_path)
                .expect("relative path")
                .to_slash_lossy(); // Change the slashes from `\` to `/` on Windows.

            let issue = if self.options.deny {
                "error".paint(styles.error)
            } else {
                "warning".paint(styles.warning)
            };

            writeln!(
                f,
                "{issue}: orphaned module `{name}` at {file_path}",
                name = orphan.name,
                file_path = file_path
            )?;

            writeln!(
                f,
                "  {arrow} {parent_file_path}",
                arrow = "-->".paint(styles.chrome),
            )?;

            let carets =
                String::from_utf8(vec![b'^'; parent_file_path.len()]).expect("valid string");
            writeln!(
                f,
                "   {pipe}  {carets} {message}",
                pipe = "|".paint(styles.chrome),
                carets = carets.paint(styles.deletion),
                message = "orphan module not loaded from file".paint(styles.deletion)
            )?;

            writeln!(f, "   {pipe}", pipe = "|".paint(styles.chrome),)?;

            writeln!(
                f,
                " {help}: consider loading `{orphan_name}` from module `{parent_module_path}`",
                // eq = styles.chrome.paint("="),
                help = "help".paint(styles.help),
                orphan_name = orphan.name,
                parent_module_path = parent_module_path,
            )?;

            writeln!(f, "   {pipe}", pipe = "|".paint(styles.chrome),)?;

            writeln!(
                f,
                "   {pipe}  {insertion}",
                pipe = "|".paint(styles.chrome),
                insertion = format!("mod {};", orphan.name).paint(styles.insertion)
            )?;

            let plusses =
                String::from_utf8(vec![b'+'; orphan.name.len() + 5]).expect("valid string");
            writeln!(
                f,
                "   {pipe}  {plusses}",
                pipe = "|".paint(styles.chrome),
                plusses = plusses.paint(styles.insertion)
            )?;

            writeln!(f, "   {pipe}", pipe = "|".paint(styles.chrome),)?;

            writeln!(f)?;
        }

        Ok(())
    }
}
