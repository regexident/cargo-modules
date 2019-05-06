use petgraph::graph::Graph as PetGraph;
use petgraph::Directed;

pub struct GraphBuilder;
pub struct Mod;
pub struct Edge;

impl GraphBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self) -> PetGraph<Mod, Edge, Directed> {
        PetGraph::new()
    }

    // TODO: add_crate_root();
    // TODO: add_mod();
    // TODO: add_dep();
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::Graph as PetGraph;
    use petgraph::Directed;

    #[test]
    fn new_builder_produces_an_empty_directed_graph() {
        let builder = GraphBuilder::new();
        let graph: PetGraph<Mod, Edge, Directed> = builder.build();
        assert_eq!(0, graph.node_count());
        assert_eq!(0, graph.edge_count());
    }
}
