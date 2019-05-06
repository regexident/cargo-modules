use petgraph::graphmap::DiGraphMap;

pub struct GraphBuilder;
pub struct Edge;

impl GraphBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self) -> DiGraphMap<Mod, Edge> {
        DiGraphMap::new()
    }

    // TODO: add_crate_root();
    // TODO: add_mod();
    // TODO: add_dep();
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mod;

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graphmap::DiGraphMap;

    #[test]
    fn new_builder_produces_an_empty_directed_graph() {
        let builder = GraphBuilder::new();
        let graph: DiGraphMap<Mod, Edge> = builder.build();
        assert_eq!(0, graph.node_count());
        assert_eq!(0, graph.edge_count());
    }
}
