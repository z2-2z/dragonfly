use libafl::prelude::{
    AsMutSlice,
    AsSlice,
    Error,
    ExitKind,
    HasCorpus,
    Named,
    Observer,
    OwnedMutSlice,
    ShMem,
    ShMemProvider,
    UsesInput,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::mem::size_of;

use crate::graph::{
    HasStateGraph,
    StateGraph,
};

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

        // Dropping the variable would deallocate it so we need to forget it
        std::mem::forget(state_channel);

        Ok(Self {
            state_channel: raw_mem,
            name: name.into(),
            new_transitions: false,
        })
    }

    pub fn get_total_states(&self) -> u64 {
        let slice = &self.state_channel.as_slice();
        debug_assert!(slice.len() >= 8);
        unsafe { *std::mem::transmute::<*const u8, *const u64>(slice.as_ptr()) }
    }

    fn set_total_states(&mut self, total_states: u64) {
        let slice = self.state_channel.as_mut_slice();
        debug_assert!(slice.len() >= 8);
        unsafe {
            *std::mem::transmute::<*mut u8, *mut u64>(slice.as_mut_ptr()) = total_states;
        }
    }

    pub fn get_state(&self, idx: usize) -> &State {
        assert!(idx < NUM_STATES);
        let offset = 8 + idx * size_of::<State>();
        let slice = self.state_channel.as_slice();
        unsafe { &*std::mem::transmute::<*const u8, *const State>(slice.as_ptr().add(offset)) }
    }

    pub fn get_all_states(&self) -> &[State] {
        let len = self.get_total_states() as usize;
        assert!(len < NUM_STATES);

        let start_offset = 8;
        let end_offset = start_offset + len * size_of::<State>();
        let states = &self.state_channel.as_slice()[start_offset..end_offset];
        unsafe {
            let states = std::mem::transmute::<*const u8, *const State>(states.as_ptr());
            std::slice::from_raw_parts(states, len)
        }
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
    S: UsesInput + HasStateGraph + HasCorpus,
{
    fn pre_exec(&mut self, _state: &mut S, _input: &S::Input) -> Result<(), Error> {
        self.set_total_states(0);
        self.new_transitions = false;
        Ok(())
    }

    fn post_exec(&mut self, state: &mut S, _input: &S::Input, _exit_kind: &ExitKind) -> Result<(), Error> {
        let state_graph = state.get_stategraph_mut()?;
        let total_states = self.get_total_states() as usize;
        let mut old_node = StateGraph::ENTRYPOINT;

        assert!(total_states <= NUM_STATES);

        for i in 0..total_states {
            let state = self.get_state(i);
            let new_node = state_graph.add_node(state);

            if new_node != old_node {
                self.new_transitions |= state_graph.add_edge(old_node, new_node);
            }

            old_node = new_node;
        }

        Ok(())
    }
}
