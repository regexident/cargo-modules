// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Printer for displaying module structure as a tree.

use ra_ap_ide::RootDatabase;

use super::{options::Options, orphan::Orphan, theme::styles};

pub struct Printer<'a> {
    #[allow(dead_code)]
    options: &'a Options,
    #[allow(dead_code)]
    db: &'a RootDatabase,
}

impl<'a> Printer<'a> {
    pub fn new(options: &'a Options, db: &'a RootDatabase) -> Self {
        Self { options, db }
    }

    pub fn fmt(&self, f: &mut dyn std::io::Write, orphans: &[Orphan]) -> Result<(), anyhow::Error> {
        let styles = styles();

        if orphans.is_empty() {
            writeln!(f)?;
            writeln!(f, "{}", styles.success.paint("No orphans found."))?;

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

        writeln!(f)?;
        writeln!(f, "{count} orphans found:", count = orphans.len())?;
        writeln!(f)?;

        for orphan in orphans {
            let file_path = orphan
                .file_path
                .strip_prefix(&prefix_path)
                .expect("relative path")
                .to_string_lossy();

            let parent_module_path = &orphan.parent_module_path;
            let parent_file_path = orphan
                .parent_file_path
                .strip_prefix(&prefix_path)
                .expect("relative path")
                .to_string_lossy();

            let issue = if self.options.deny {
                styles.error.paint("error")
            } else {
                styles.warning.paint("warning")
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
                arrow = styles.chrome.paint("-->"),
            )?;

            let carets =
                String::from_utf8(vec![b'^'; parent_file_path.len()]).expect("valid string");
            writeln!(
                f,
                "   {pipe}  {carets} {message}",
                pipe = styles.chrome.paint("|"),
                carets = styles.deletion.paint(carets),
                message = styles.deletion.paint("orphan module not loaded from file")
            )?;

            writeln!(f, "   {pipe}", pipe = styles.chrome.paint("|"),)?;

            writeln!(
                f,
                " {help}: consider loading `{orphan_name}` from module `{parent_module_path}`",
                // eq = styles.chrome.paint("="),
                help = styles.help.paint("help"),
                orphan_name = orphan.name,
                parent_module_path = parent_module_path,
            )?;

            writeln!(f, "   {pipe}", pipe = styles.chrome.paint("|"),)?;

            writeln!(
                f,
                "   {pipe}  {insertion}",
                pipe = styles.chrome.paint("|"),
                insertion = styles.insertion.paint(format!("mod {};", orphan.name))
            )?;

            let plusses =
                String::from_utf8(vec![b'+'; orphan.name.len() + 5]).expect("valid string");
            writeln!(
                f,
                "   {pipe}  {plusses}",
                pipe = styles.chrome.paint("|"),
                plusses = styles.insertion.paint(plusses)
            )?;

            writeln!(f, "   {pipe}", pipe = styles.chrome.paint("|"),)?;

            writeln!(f)?;
        }

        Ok(())
    }
}
