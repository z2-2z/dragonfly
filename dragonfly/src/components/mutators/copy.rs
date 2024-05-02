use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::components::{DragonflyInput, Packet};

pub struct PacketCopyMutator {
    max_length: usize,
}

impl PacketCopyMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
        }
    }
}

impl Named for PacketCopyMutator {
    fn name(&self) -> &str {
        "PacketCopyMutator"
    }
}

impl<P, S> Mutator<DragonflyInput<P>, S> for PacketCopyMutator
where
    P: Packet + Clone,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut DragonflyInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len == 0 || len >= self.max_length {
            return Ok(MutationResult::Skipped);
        }
        
        let to = state.rand_mut().below(len as u64 + 1) as usize;
        let from = state.rand_mut().below(len as u64) as usize;
        
        let packet = input.packets()[from].clone();
        input.packets_mut().insert(to, packet);
        
        Ok(MutationResult::Mutated)
    }
}
