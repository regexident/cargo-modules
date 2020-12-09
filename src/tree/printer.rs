//! Printer for displaying module hierarchy as a tree.

// use colored::{ColoredString, Colorize};
use petgraph::{
    algo::is_cyclic_directed,
    graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use yansi::Style;

use crate::{
    graph::modules::{Edge, Graph, ModuleNode, Node, NodeKind, Visibility},
    theme::theme,
};

#[derive(Debug)]
struct BranchInfo {
    is_last: bool,
}

pub fn print<I>(graph: &Graph, roots: I)
where
    I: IntoIterator<Item = NodeIndex<usize>>,
{
    assert!(!is_cyclic_directed(graph));

    for root_idx in roots.into_iter() {
        let mut branches: Vec<BranchInfo> = vec![BranchInfo { is_last: true }];
        print_tree(graph, None, root_idx, &mut branches);
    }
}

fn print_tree(
    graph: &Graph,
    edge_idx: Option<EdgeIndex<usize>>,
    node_idx: NodeIndex<usize>,
    branches: &mut Vec<BranchInfo>,
) {
    let edge = edge_idx.map(|idx| &graph[idx]);
    let node = &graph[node_idx];

    print_branch(edge, &branches[..]);

    match &node.kind {
        NodeKind::Module(module_node) => {
            print_module_node(node, module_node);
        }
        NodeKind::Orphan => {
            print_orphan_node(node);
        }
    }

    println!();

    let mut children: Vec<_> = graph
        .edges_directed(node_idx, Direction::Outgoing)
        .map(|edge_ref| {
            let child_edge_idx = edge_ref.id();
            let child_node_idx = edge_ref.target();
            let child_node = &graph[child_node_idx];
            let child_key = child_node.name.as_str();
            (child_node_idx, child_edge_idx, child_key)
        })
        .collect();

    // Sort the children by name for easier visual scanning of output:
    children.sort_by_key(|(_, _, key)| *key);

    let count = children.len();
    for (pos, (node_idx, edge_idx, _)) in children.into_iter().enumerate() {
        let is_last = pos + 1 == count;
        branches.push(BranchInfo { is_last });
        print_tree(graph, Some(edge_idx), node_idx, branches);
        branches.pop();
    }
}

/// Print a module branch:
fn print_module_node(node: &Node, module_node: &ModuleNode) {
    let colored_name = name_style().paint(&node.name);

    let visibility = &module_node.visibility;
    let visibility_style = visibility_style(visibility);
    let colored_visibility = visibility_style.paint(format!("{:?}", visibility));

    print!("{}: {}", colored_name, colored_visibility);

    if let Some(cfgs) = node.non_empty_cfgs() {
        let cfg_strings: Vec<_> = cfgs
            .iter()
            .map(|cfg| format!("{}", cfg_style().paint(cfg)))
            .collect();

        let cfgs_prefix = dimmed_style().paint("#[cfg(");
        let cfgs_suffix = dimmed_style().paint(")]");
        let cfgs_separator = format!("{}", dimmed_style().paint(", "));
        let cfgs_string = cfg_strings.join(&cfgs_separator);
        print!(" {}{}{}", cfgs_prefix, cfgs_string, cfgs_suffix);
    }
}

/// Print a orphan branch:
fn print_orphan_node(node: &Node) {
    let colored_name = name_style().paint(&node.name);

    let orphan_style = orphan_style();
    let colored_orphan = orphan_style.paint("orphan");

    print!("{}: {}", colored_name, colored_orphan);
}

fn print_branch(_edge: Option<&Edge>, branches: &[BranchInfo]) {
    let prefix = branch_prefix(&branches[..]);
    print!("{}", branch_style().paint(&prefix));
}

/// Print a branch's prefix:
fn branch_prefix(branch_infos: &[BranchInfo]) -> String {
    fn trunk_str(_is_last: bool) -> &'static str {
        ""
    }

    fn branch_str(is_last: bool) -> &'static str {
        if is_last {
            "    "
        } else {
            " │  "
        }
    }

    fn leaf_str(is_last: bool) -> &'static str {
        if is_last {
            " └── "
        } else {
            " ├── "
        }
    }

    let mut string = String::new();

    // First level is crate level, we need to skip it when
    // printing. But we cannot easily drop the first value.
    match branch_infos {
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

fn dimmed_style() -> Style {
    Style::default().dimmed()
}

fn branch_style() -> Style {
    Style::default().dimmed()
}

fn name_style() -> Style {
    theme().name
}

fn visibility_style(visibility: &Visibility) -> Style {
    let theme = theme().visibility;

    match visibility {
        Visibility::Crate => theme.pub_crate,
        Visibility::Module(_) => theme.pub_module,
        Visibility::Private => theme.pub_private,
        Visibility::Public => theme.pub_public,
        Visibility::Super => theme.pub_super,
    }
}

fn orphan_style() -> Style {
    let theme = theme();
    theme.orphan
}

fn cfg_style() -> Style {
    let theme = theme();
    theme.cfg
}
