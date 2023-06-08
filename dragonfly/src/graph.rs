use ahash::{
    AHasher,
    RandomState,
};
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

use crate::observer::State;

type NodeId = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct StateGraph {
    edges: HashSet<(NodeId, NodeId), RandomState>,
}

impl StateGraph {
    pub const ENTRYPOINT: NodeId = 0;

    fn new() -> Self {
        Self {
            edges: HashSet::default(),
        }
    }

    pub fn add_node(&mut self, state: &State) -> NodeId {
        let mut hasher = AHasher::default();
        hasher.write(state);
        let id = hasher.finish();

        if id == Self::ENTRYPOINT {
            !Self::ENTRYPOINT
        } else {
            id
        }
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> bool {
        self.edges.insert((from, to))
    }

    pub fn edges(&self) -> &HashSet<(NodeId, NodeId), RandomState> {
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
        for (from, to) in &self.edges {
            if !seen.contains(from) {
                self.dump_node(*from, stream)?;
                seen.insert(*from);
            }
            
            if !seen.contains(to) {
                self.dump_node(*to, stream)?;
                seen.insert(*to);
            }
        }
        
        writeln!(stream)?;
        
        /* Then print all the edges */
        for (from, to) in &self.edges {
            writeln!(stream, "  {} -> {} [arrowhead=\"open\"];", from, to)?;
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
