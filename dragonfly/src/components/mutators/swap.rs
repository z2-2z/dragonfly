use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::components::{DragonflyInput, Packet};

pub struct PacketSwapMutator;

impl PacketSwapMutator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

impl Named for PacketSwapMutator {
    fn name(&self) -> &str {
        "PacketSwapMutator"
    }
}

impl<P, S> Mutator<DragonflyInput<P>, S> for PacketSwapMutator
where
    P: Packet,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut DragonflyInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len <= 1 {
            return Ok(MutationResult::Skipped);
        }
        
        let to = state.rand_mut().below(len as u64) as usize;
        let from = state.rand_mut().below(len as u64) as usize;
        
        if to == from {
            return Ok(MutationResult::Skipped);
        }
        
        input.packets_mut().swap(to, from);
        
        Ok(MutationResult::Mutated)
    }
}
