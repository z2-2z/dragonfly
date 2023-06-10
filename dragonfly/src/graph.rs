use libafl::prelude::{
    impl_serdeany,
    Error,
    HasMetadata,
    StdState,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    collections::HashSet,
    hash::Hasher,
};
use fnv::{FnvHasher, FnvHashSet};

use crate::observer::State;

pub type NodeId = u64;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Edge {
    from: NodeId,
    to: NodeId,
}

impl Edge {
    fn new(from: NodeId, to: NodeId) -> Self {
        Self {
            from,
            to,
        }
    }
    
    pub fn from(&self) -> NodeId {
        self.from
    }
    
    pub fn to(&self) -> NodeId {
        self.to
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateGraph {
    edges: FnvHashSet<Edge>,
}

impl StateGraph {
    pub const ENTRYPOINT: NodeId = 0;

    fn new() -> Self {
        Self {
            edges: HashSet::default(),
        }
    }

    pub fn add_node(&mut self, state: &State) -> NodeId {
        let mut hasher = FnvHasher::default();
        hasher.write(state);
        let id = hasher.finish();

        if id == Self::ENTRYPOINT {
            !Self::ENTRYPOINT
        } else {
            id
        }
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> bool {
        self.edges.insert(Edge::new(from, to))
    }

    pub fn edges(&self) -> &FnvHashSet<Edge> {
        &self.edges
    }
    
    fn dump_node<S: std::io::Write>(&self, id: NodeId, stream: &mut S) -> std::io::Result<()> {
        if id == Self::ENTRYPOINT {
            writeln!(stream, "  {} [shape=\"point\"];", id)?;
        } else {
            writeln!(stream, "  {0} [shape=\"oval\"; label=\"{0:016x}\"];", id)?;
        }
        
        Ok(())
    }
    
    pub fn dump_dot<S: std::io::Write>(&self, stream: &mut S) -> std::io::Result<()> {
        let mut seen = HashSet::new();
        
        writeln!(stream, "digraph ipsm {{")?;
        writeln!(stream, "  label = \"IPSM\";")?;
        writeln!(stream, "  center = true;")?;
        writeln!(stream)?;
        
        /* First print all the nodes */
        for edge in &self.edges {
            if !seen.contains(&edge.from) {
                self.dump_node(edge.from, stream)?;
                seen.insert(edge.from);
            }
            
            if !seen.contains(&edge.to) {
                self.dump_node(edge.to, stream)?;
                seen.insert(edge.to);
            }
        }
        
        writeln!(stream)?;
        
        /* Then print all the edges */
        for edge in &self.edges {
            writeln!(stream, "  {} -> {} [arrowhead=\"open\"];", edge.from, edge.to)?;
        }
        
        writeln!(stream, "}}")?;
        Ok(())
    }
}

impl_serdeany!(StateGraph);

pub trait HasStateGraph {
    fn get_stategraph(&self) -> Result<&StateGraph, Error>;
    fn get_stategraph_mut(&mut self) -> Result<&mut StateGraph, Error>;
    fn init_stategraph(&mut self);
}

impl<I, C, R, SC> HasStateGraph for StdState<I, C, R, SC> {
    fn get_stategraph(&self) -> Result<&StateGraph, Error> {
        self.metadata::<StateGraph>()
    }

    fn get_stategraph_mut(&mut self) -> Result<&mut StateGraph, Error> {
        self.metadata_mut::<StateGraph>()
    }

    fn init_stategraph(&mut self) {
        if !self.has_metadata::<StateGraph>() {
            self.add_metadata(StateGraph::new());
        }
    }
}
