//! Printer for displaying module structure as a tree.

use petgraph::{
    algo::is_cyclic_directed,
    graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use ra_ap_hir::ModuleDef;
use ra_ap_ide::RootDatabase;
use yansi::Style;

use crate::{
    format::{cfg::FormattedCfgExpr, kind::FormattedKind, visibility::FormattedVisibility},
    graph::{Edge, Graph, Node, NodeKind},
    theme::styles,
};

#[derive(Debug)]
struct Twig {
    is_last: bool,
}

#[derive(Clone, Debug)]
pub struct Options {}

pub struct Printer<'a> {
    #[allow(dead_code)]
    options: Options,
    db: &'a RootDatabase,
}

impl<'a> Printer<'a> {
    pub fn new(options: Options, db: &'a RootDatabase) -> Self {
        Self { options, db }
    }

    pub fn print(&self, graph: &Graph, start_node_idx: NodeIndex) -> Result<(), anyhow::Error> {
        assert!(!is_cyclic_directed(graph));

        let mut twigs: Vec<Twig> = vec![Twig { is_last: true }];
        self.print_tree(graph, None, start_node_idx, &mut twigs)
    }

    fn print_tree(
        &self,
        graph: &Graph,
        edge_idx: Option<EdgeIndex>,
        node_idx: NodeIndex,
        twigs: &mut Vec<Twig>,
    ) -> Result<(), anyhow::Error> {
        let edge = edge_idx.map(|idx| &graph[idx]);
        let node = &graph[node_idx];

        self.print_branch(edge, &twigs[..]);
        self.print_node(node);
        println!();

        let mut children: Vec<_> = graph
            .edges_directed(node_idx, Direction::Outgoing)
            .filter_map(|edge_ref| {
                let edge_idx = edge_ref.id();
                let edge = &graph[edge_idx];

                // We're only interested in "has-a" relationships here:
                if edge != &Edge::HasA {
                    return None;
                }

                let node_idx = edge_ref.target();
                let node = &graph[node_idx];

                // if !self.options.with_types && !node.is_module() {
                //     return None;
                // }

                let key = node.name();
                Some((node_idx, edge_idx, key))
            })
            .collect();

        // Sort the children by name for easier visual scanning of output:
        children.sort_by(|lhs, rhs| {
            let (_lhs_node, _lhs_edge, lhs_key) = lhs;
            let (_rhs_node, _rhs_edge, rhs_key) = rhs;
            lhs_key.cmp(&rhs_key)
        });

        let count = children.len();
        for (pos, (node_idx, edge_idx, _)) in children.into_iter().enumerate() {
            let is_last = pos + 1 == count;
            twigs.push(Twig { is_last });
            self.print_tree(graph, Some(edge_idx), node_idx, twigs)?;
            twigs.pop();
        }

        Ok(())
    }

    /// Print a branch:
    fn print_node(&self, node: &Node) {
        match node.kind(self.db) {
            NodeKind::Crate => {
                self.print_crate_node(node);
            }
            NodeKind::Module => {
                self.print_module_node(node);
            }
            NodeKind::Type => {
                self.print_type_node(node);
            }
            NodeKind::Orphan => {
                self.print_orphan_node(node);
            }
        }
    }

    /// Print a crate branch:
    fn print_crate_node(&self, node: &Node) {
        assert!(node.kind(self.db) == NodeKind::Crate);

        let kind = FormattedKind::Crate;

        let kind_style = self.kind_style(&kind);
        let name_style = self.name_style(&kind);

        let kind = kind_style.paint(kind);
        let name = name_style.paint(node.name());

        print!("{} {}", kind, name);
    }

    /// Print a module-def branch:
    fn print_module_node(&self, node: &Node) {
        assert!(node.kind(self.db) == NodeKind::Module);

        self.print_module_or_crate_node(node)
    }

    fn print_type_node(&self, node: &Node) {
        assert!(node.kind(self.db) == NodeKind::Type);

        self.print_module_or_crate_node(node)
    }

    fn print_module_or_crate_node(&self, node: &Node) {
        let colon_style = self.colon_style();

        let module_def = match node.hir {
            Some(module_def) => module_def,
            None => unreachable!(),
        };

        let name = node.name();
        let kind = FormattedKind::new(module_def);
        let visibility = FormattedVisibility::new(module_def, self.db);

        let kind_style = self.kind_style(&kind);
        let name_style = self.name_style(&kind);
        let visibility_style = self.visibility_style(&visibility, &kind);

        {
            let kind = kind_style.paint(kind);
            let name = name_style.paint(name);
            let colon = colon_style.paint(":");
            let visibility = visibility_style.paint(format!("{}", visibility));

            print!("{} {}{} {}", kind, name, colon, visibility);
        }

        self.print_module_def_cfg(module_def);
    }

    /// Print a module-def's cfg suffix':
    fn print_module_def_cfg(&self, module_def: ModuleDef) {
        let cfg_chrome_style = self.cfg_chrome_style();
        let cfg_style = self.cfg_style();

        let cfgs: Vec<_> = match FormattedCfgExpr::from_hir(module_def, self.db) {
            Some(cfg) => cfg
                .top_level()
                .into_iter()
                .map(FormattedCfgExpr::new)
                .collect(),
            None => vec![],
        };

        for cfg in cfgs {
            let prefix = cfg_chrome_style.paint("#[cfg(");
            let cfg = cfg_style.paint(cfg);
            let suffix = cfg_chrome_style.paint(")]");

            print!(" {}{}{}", prefix, cfg, suffix);
        }
    }

    /// Print a orphan branch:
    fn print_orphan_node(&self, node: &Node) {
        assert!(node.kind(self.db) == NodeKind::Orphan);

        let kind = FormattedKind::Module;

        let kind_style = self.kind_style(&kind);
        let name_style = self.name_style(&kind);
        let orphan_style = self.orphan_style();

        let kind = kind_style.paint(kind);
        let name = name_style.paint(node.name());
        let orphan = orphan_style.paint("orphan");

        print!("{} {}: {}", kind, name, orphan);
    }

    fn print_branch(&self, _edge: Option<&Edge>, twigs: &[Twig]) {
        let prefix = self.branch_prefix(&twigs[..]);
        print!("{}", self.branch_style().paint(&prefix));
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

    fn cfg_chrome_style(&self) -> Style {
        Style::default().dimmed()
    }

    fn branch_style(&self) -> Style {
        Style::default().dimmed()
    }

    fn name_style(&self, kind: &FormattedKind) -> Style {
        let styles = styles();
        let base = styles.name;

        match kind {
            FormattedKind::Crate => base,
            FormattedKind::Module => base,
            _ => base.dimmed(),
        }
    }

    fn kind_style(&self, kind: &FormattedKind) -> Style {
        let styles = styles();
        let base = styles.kind;

        match kind {
            FormattedKind::Crate => base,
            FormattedKind::Module => base,
            _ => base.dimmed(),
        }
    }

    fn visibility_style(&self, visibility: &FormattedVisibility, kind: &FormattedKind) -> Style {
        let styles = styles().visibility;

        let base = match visibility {
            FormattedVisibility::Crate => styles.pub_crate,
            FormattedVisibility::Module(_) => styles.pub_module,
            FormattedVisibility::Private => styles.pub_private,
            FormattedVisibility::Public => styles.pub_global,
            FormattedVisibility::Super => styles.pub_super,
        };

        match kind {
            FormattedKind::Crate => base,
            FormattedKind::Module => base,
            _ => base.dimmed(),
        }
    }

    fn orphan_style(&self) -> Style {
        let styles = styles();
        styles.orphan
    }

    fn cfg_style(&self) -> Style {
        let styles = styles();
        styles.cfg
    }
}
