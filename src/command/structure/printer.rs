// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Printer for displaying module structure as a tree.

use std::fmt;

use ra_ap_ide::RootDatabase;

use crate::{analyzer, item::visibility::ItemVisibility, tree::Tree};

use super::{
    options::{Options, SortBy},
    theme::styles,
};

#[derive(Debug)]
struct Twig {
    is_last: bool,
}

pub struct Printer<'a> {
    #[allow(dead_code)]
    options: &'a Options,
    db: &'a RootDatabase,
}

impl<'a> Printer<'a> {
    pub fn new(options: &'a Options, db: &'a RootDatabase) -> Self {
        Self { options, db }
    }

    pub fn fmt(&self, f: &mut dyn fmt::Write, tree: &Tree) -> Result<(), anyhow::Error> {
        let mut twigs: Vec<Twig> = vec![Twig { is_last: true }];
        self.fmt_tree(f, tree, &mut twigs)
    }

    fn fmt_tree(
        &self,
        f: &mut dyn fmt::Write,
        tree: &Tree,
        twigs: &mut Vec<Twig>,
    ) -> Result<(), anyhow::Error> {
        self.fmt_branch(f, &twigs[..])?;
        self.fmt_subtree(f, tree)?;
        writeln!(f)?;

        let mut subtrees = tree.subtrees.clone();

        // Sort the children by name for easier visual scanning of output:
        subtrees.sort_by_cached_key(|tree: &Tree| tree.item.display_name(self.db));

        // The default sorting functions in Rust are stable, so we can use it to re-sort,
        // resulting in a list that's sorted prioritizing whatever we re-sort by.

        // Re-sort the children by name, visibility or kind, for easier visual scanning of output:
        match self.options.sort_by {
            SortBy::Name => {
                subtrees.sort_by_cached_key(|tree| tree.item.display_name(self.db));
            }
            SortBy::Visibility => {
                subtrees.sort_by_cached_key(|tree| tree.item.visibility(self.db).clone());
            }
            SortBy::Kind => {
                subtrees.sort_by_cached_key(|tree| tree.item.kind(self.db).clone());
            }
        }

        if self.options.sort_reversed {
            subtrees.reverse();
        }

        let count = subtrees.len();
        for (pos, tree) in subtrees.into_iter().enumerate() {
            let is_last = pos + 1 == count;
            twigs.push(Twig { is_last });
            self.fmt_tree(f, &tree, twigs)?;
            twigs.pop();
        }

        Ok(())
    }

    fn fmt_subtree(&self, f: &mut dyn fmt::Write, tree: &Tree) -> fmt::Result {
        self.fmt_tree_kind(f, tree)?;
        write!(f, " ")?;
        self.fmt_tree_name(f, tree)?;

        if analyzer::moduledef_is_crate(tree.item.hir, self.db) {
            return Ok(());
        }

        self.fmt_tree_colon(f, tree)?;
        write!(f, " ")?;
        self.fmt_tree_visibility(f, tree)?;

        if !tree.item.attrs(self.db).is_empty() {
            write!(f, " ")?;
            self.fmt_tree_attrs(f, tree)?;
        }

        Ok(())
    }

    fn fmt_tree_kind(&self, f: &mut dyn fmt::Write, tree: &Tree) -> fmt::Result {
        let styles = styles();
        let kind_style = styles.kind;

        let display_name = tree.item.kind_display_name(self.db);
        let kind = kind_style.paint(display_name);

        write!(f, "{kind}")?;

        Ok(())
    }

    fn fmt_tree_colon(&self, f: &mut dyn fmt::Write, _tree: &Tree) -> fmt::Result {
        let styles = styles();
        let colon_style = styles.colon;

        let colon = colon_style.paint(":");
        write!(f, "{colon}")?;

        Ok(())
    }

    fn fmt_tree_visibility(&self, f: &mut dyn fmt::Write, tree: &Tree) -> fmt::Result {
        let styles = styles();

        let visibility_styles = styles.visibility;
        let visibility_style = match &tree.item.visibility(self.db) {
            ItemVisibility::Crate => visibility_styles.pub_crate,
            ItemVisibility::Module(_) => visibility_styles.pub_module,
            ItemVisibility::Private => visibility_styles.pub_private,
            ItemVisibility::Public => visibility_styles.pub_global,
            ItemVisibility::Super => visibility_styles.pub_super,
        };

        write!(
            f,
            "{}",
            visibility_style.paint(&tree.item.visibility(self.db))
        )?;

        Ok(())
    }

    fn fmt_tree_name(&self, f: &mut dyn fmt::Write, tree: &Tree) -> fmt::Result {
        let styles = styles();

        let name_style = styles.name;

        write!(f, "{}", name_style.paint(tree.item.display_name(self.db)))?;

        Ok(())
    }

    fn fmt_tree_attrs(&self, f: &mut dyn fmt::Write, tree: &Tree) -> fmt::Result {
        let styles = styles();
        let attr_chrome_style = styles.attr_chrome;
        let attr_style = styles.attr;

        let mut is_first = true;

        if let Some(test_attr) = &tree.item.attrs(self.db).test {
            let prefix = attr_chrome_style.paint("#[");
            let cfg = attr_style.paint(test_attr);
            let suffix = attr_chrome_style.paint("]");

            write!(f, "{prefix}{cfg}{suffix}")?;

            is_first = false;
        }

        for cfg in &tree.item.attrs(self.db).cfgs[..] {
            if !is_first {
                write!(f, ", ")?;
            }

            let prefix = attr_chrome_style.paint("#[cfg(");
            let cfg = attr_style.paint(cfg);
            let suffix = attr_chrome_style.paint(")]");

            write!(f, "{prefix}{cfg}{suffix}")?;

            is_first = false;
        }

        Ok(())
    }

    fn fmt_branch(&self, f: &mut dyn fmt::Write, twigs: &[Twig]) -> fmt::Result {
        let styles = styles();
        let branch_style = styles.branch;

        let prefix = self.branch_prefix(twigs);
        write!(f, "{}", branch_style.paint(&prefix))
    }

    /// Print a branch's prefix:
    fn branch_prefix(&self, twigs: &[Twig]) -> String {
        fn trunk_str(_is_last: bool) -> &'static str {
            ""
        }

        fn branch_str(is_last: bool) -> &'static str {
            if is_last {
                "    "
            } else {
                "│   "
            }
        }

        fn leaf_str(is_last: bool) -> &'static str {
            if is_last {
                "└── "
            } else {
                "├── "
            }
        }

        let mut string = String::new();

        // First level is crate level, we need to skip it when
        // printing. But we cannot easily drop the first value.
        match twigs {
            [trunk, branches @ .., leaf] => {
                string.push_str(trunk_str(trunk.is_last));
                for branch in branches {
                    string.push_str(branch_str(branch.is_last));
                }
                string.push_str(leaf_str(leaf.is_last));
            }
            [trunk] => {
                string.push_str(trunk_str(trunk.is_last));
            }
            [] => {}
        }

        string
    }
}
