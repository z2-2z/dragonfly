use libafl::prelude::{
    OwnedMutSlice, AsMutSlice, AsSlice,
    ShMemProvider, ShMem,
    Observer, UsesInput,
    Named, Error, ExitKind,
};
use serde::{Serialize, Deserialize};
use std::mem::size_of;

use crate::graph::{StateGraph, HasStateGraph};

const STATES_SHM_ID: &str = "__DRAGONFLY_STATES_SHM_ID";
const STATES_SHM_SIZE: &str = "__DRAGONFLY_STATES_SHM_SIZE";

pub type State = [u8; 16];
const NUM_STATES: usize = 1024;

#[derive(Debug, Serialize, Deserialize)]
pub struct StateObserver<'a> {
    state_channel: OwnedMutSlice<'a, u8>,
    name: String,
    new_transitions: bool,
}

impl<'a> StateObserver<'a> {
    pub fn new<S, P>(shmem_provider: &mut P, name: S) -> Result<Self, Error>
    where
        S: Into<String>,
        P: ShMemProvider,
    {
        let state_channel_size = 8 + size_of::<State>() * NUM_STATES;
        let mut state_channel = shmem_provider.new_shmem(state_channel_size)?;
        state_channel.write_to_env(STATES_SHM_ID)?;
        std::env::set_var(STATES_SHM_SIZE, format!("{}", state_channel_size));
        
        let raw_mem = unsafe {
            let slice = state_channel.as_mut_slice();
            OwnedMutSlice::from_raw_parts_mut(slice.as_mut_ptr(), slice.len())
        };
        
        std::mem::forget(state_channel);
        
        Ok(Self {
            state_channel: raw_mem,
            name: name.into(),
            new_transitions: false,
        })
    }
    
    fn get_total_states(&self) -> u64 {
        u64::from_ne_bytes(self.state_channel.as_slice()[0..8].try_into().unwrap())
    }
    
    fn set_total_states(&mut self, total_states: u64) {
        self.state_channel.as_mut_slice()[0..8].copy_from_slice(&total_states.to_ne_bytes());
    }
    
    pub fn had_new_transitions(&self) -> bool {
        self.new_transitions
    }
}

impl<'a> Named for StateObserver<'a> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a, S> Observer<S> for StateObserver<'a>
where
    S: UsesInput + HasStateGraph,
{
    fn pre_exec(&mut self, _state: &mut S, _input: &S::Input) -> Result<(), Error> {
        self.set_total_states(0);
        self.new_transitions = false;
        Ok(())
    }
    
    fn post_exec(&mut self, state: &mut S, _input: &S::Input, _exit_kind: &ExitKind) -> Result<(), Error> {
        let state_graph = state.get_stategraph_mut()?;
        let total_states = self.get_total_states() as usize;
        
        assert!(total_states <= NUM_STATES);
        
        let mut cursor = 8;
        let mut old_node = StateGraph::ENTRYPOINT;
        
        for _ in 0..total_states {
            let state = &self.state_channel.as_slice()[cursor..cursor + size_of::<State>()];
            let new_node = state_graph.add_node(state.try_into().unwrap());
            self.new_transitions |= state_graph.add_edge(old_node, new_node);
            old_node = new_node;
            cursor += size_of::<State>();
        }
        
        Ok(())
    }
}
