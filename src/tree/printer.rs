// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Printer for displaying module structure as a tree.

use std::fmt;

use ra_ap_ide::RootDatabase;
use yansi::Style;

use crate::{
    item::visibility::ItemVisibility,
    tree::{node::Node, options::SortBy, theme::styles, Tree},
};

#[derive(Debug)]
struct Twig {
    is_last: bool,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub sort_by: SortBy,
    pub sort_reversed: bool,
}

pub struct Printer<'a> {
    #[allow(dead_code)]
    options: Options,
    db: &'a RootDatabase,
}

impl<'a> Printer<'a> {
    pub fn new(options: Options, db: &'a RootDatabase) -> Self {
        Self { options, db }
    }

    pub fn fmt(&self, f: &mut dyn fmt::Write, tree: &Tree) -> Result<(), anyhow::Error> {
        let mut twigs: Vec<Twig> = vec![Twig { is_last: true }];
        self.fmt_tree(f, &tree.root_node, &mut twigs)
    }

    fn fmt_tree(
        &self,
        f: &mut dyn fmt::Write,
        root_node: &Node,
        twigs: &mut Vec<Twig>,
    ) -> Result<(), anyhow::Error> {
        self.fmt_branch(f, &twigs[..])?;
        self.fmt_node(f, root_node)?;
        writeln!(f)?;

        let mut subnodes = root_node.subnodes.clone();

        // Sort the children by name for easier visual scanning of output:
        subnodes.sort_by_cached_key(|node| node.item.display_name());

        // The default sorting functions in Rust are stable, so we can use it to re-sort,
        // resulting in a list that's sorted prioritizing whatever we re-sort by.

        // Re-sort the children by name, visibility or kind, for easier visual scanning of output:
        match self.options.sort_by {
            SortBy::Name => {
                subnodes.sort_by_cached_key(|node| node.item.display_name());
            }
            SortBy::Visibility => {
                subnodes.sort_by_cached_key(|node| node.item.visibility.clone());
            }
            SortBy::Kind => {
                subnodes.sort_by_cached_key(|node| node.item.kind.clone());
            }
        }

        if self.options.sort_reversed {
            subnodes.reverse();
        }

        let count = subnodes.len();
        for (pos, node) in subnodes.into_iter().enumerate() {
            let is_last = pos + 1 == count;
            twigs.push(Twig { is_last });
            self.fmt_tree(f, &node, twigs)?;
            twigs.pop();
        }

        Ok(())
    }

    fn fmt_node(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        self.fmt_node_kind(f, node)?;
        write!(f, " ")?;
        self.fmt_node_name(f, node)?;

        if node.item.is_crate(self.db) {
            return Ok(());
        }

        self.fmt_node_colon(f, node)?;
        write!(f, " ")?;
        self.fmt_node_visibility(f, node)?;

        if !node.item.attrs.is_empty() {
            write!(f, " ")?;
            self.fmt_node_attrs(f, node)?;
        }

        Ok(())
    }

    fn fmt_node_kind(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let kind_style = self.kind_style();

        let display_name = node.kind_display_name().unwrap_or_else(|| "mod".to_owned());
        let kind = kind_style.paint(display_name);

        write!(f, "{kind}")?;

        Ok(())
    }

    fn fmt_node_colon(&self, f: &mut dyn fmt::Write, _node: &Node) -> fmt::Result {
        let colon_style = self.colon_style();

        let colon = colon_style.paint(":");
        write!(f, "{colon}")?;

        Ok(())
    }

    fn fmt_node_visibility(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let (visibility, visibility_style) = match &node.item.visibility {
            Some(visibility) => {
                let visibility_style = self.visibility_style(visibility);
                (format!("{visibility}"), visibility_style)
            }
            None => {
                let orphan_style = self.orphan_style();
                ("orphan".to_owned(), orphan_style)
            }
        };

        let visibility = visibility_style.paint(visibility);
        write!(f, "{visibility}")?;

        Ok(())
    }

    fn fmt_node_name(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let name_style = self.name_style();

        let name = name_style.paint(node.display_name());
        write!(f, "{name}")?;

        Ok(())
    }

    fn fmt_node_attrs(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let attr_chrome_style = self.attr_chrome_style();
        let attr_style = self.attr_style();

        let mut is_first = true;

        if let Some(test_attr) = &node.item.attrs.test {
            let prefix = attr_chrome_style.paint("#[");
            let cfg = attr_style.paint(test_attr);
            let suffix = attr_chrome_style.paint("]");

            write!(f, "{prefix}{cfg}{suffix}")?;

            is_first = false;
        }

        let attr_chrome_style = self.attr_chrome_style();
        let attr_style = self.attr_style();

        for cfg in &node.item.attrs.cfgs[..] {
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
        let prefix = self.branch_prefix(twigs);
        write!(f, "{}", self.branch_style().paint(&prefix))
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

    fn colon_style(&self) -> Style {
        Style::default().dimmed()
    }

    fn attr_chrome_style(&self) -> Style {
        Style::default().dimmed()
    }

    fn branch_style(&self) -> Style {
        Style::default().dimmed()
    }

    fn name_style(&self) -> Style {
        let styles = styles();
        styles.name
    }

    fn kind_style(&self) -> Style {
        let styles = styles();
        styles.kind
    }

    fn visibility_style(&self, visibility: &ItemVisibility) -> Style {
        let styles = styles().visibility;

        match visibility {
            ItemVisibility::Crate => styles.pub_crate,
            ItemVisibility::Module(_) => styles.pub_module,
            ItemVisibility::Private => styles.pub_private,
            ItemVisibility::Public => styles.pub_global,
            ItemVisibility::Super => styles.pub_super,
        }
    }

    fn orphan_style(&self) -> Style {
        let styles = styles();
        styles.orphan
    }

    fn attr_style(&self) -> Style {
        let styles = styles();
        styles.attr
    }
}
