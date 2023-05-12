use libafl::prelude::{
    StdState, HasMetadata,
    impl_serdeany,
    Error,
};
use serde::{Serialize, Deserialize};
use ahash::{AHasher, RandomState};
use std::hash::Hasher;
use std::collections::HashSet;

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
