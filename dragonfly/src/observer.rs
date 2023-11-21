#![allow(unreachable_code)]
#![allow(clippy::diverging_sub_expression)]

use libafl_bolts::{
    AsMutSlice,
    Named,
    shmem::{ShMem, ShMemProvider},
};
use libafl::prelude::{
    Error,
    ExitKind,
    HasCorpus,
    Observer,
    UsesInput,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::mem::size_of;

use crate::graph::{
    HasStateGraph,
};

const STATES_SHM_ID: &str = "__DRAGONFLY_STATES_SHM_ID";
const STATES_SHM_SIZE: &str = "__DRAGONFLY_STATES_SHM_SIZE";

pub type State = [u8; 16];
const NUM_STATES: usize = 1024;

fn non_unique_error() -> ! {
    panic!("Please use EventConfig::AlwaysUnique, other configurations are not supported by the StateObserver")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateObserver<'a> {
    #[serde(skip, default = "non_unique_error")]
    total_states: *mut u64,
    
    #[serde(skip, default = "non_unique_error")]
    states_slice: &'a mut [State],
    
    name: String,
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

        let raw_pointer = state_channel.as_mut_slice().as_mut_ptr();
        
        let total_states = raw_pointer as *mut u64;
        let states_slice = unsafe {
            std::slice::from_raw_parts_mut(
                raw_pointer.add(8) as *mut State,
                NUM_STATES,
            )
        };

        // Dropping the variable would deallocate it so we need to forget it
        std::mem::forget(state_channel);

        Ok(Self {
            total_states,
            states_slice,
            name: name.into(),
        })
    }

    #[inline]
    pub fn get_total_states(&self) -> u64 {
        unsafe { *self.total_states }
    }

    #[inline]
    fn set_total_states(&mut self, total_states: u64) {
        unsafe { *self.total_states = total_states; }
    }

    #[inline]
    pub fn get_state(&self, idx: usize) -> &State {
        &self.states_slice[idx]
    }

    #[inline]
    pub fn get_all_states(&self) -> &[State] {
        let len = self.get_total_states() as usize;
        &self.states_slice[..len]
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
        Ok(())
    }

    fn post_exec(&mut self, _state: &mut S, _input: &S::Input, _exit_kind: &ExitKind) -> Result<(), Error> {
        Ok(())
    }
}
