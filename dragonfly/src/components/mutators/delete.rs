use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::components::{DragonflyInput, Packet};

pub struct PacketDeleteMutator {
    min_length: usize,
}

impl PacketDeleteMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(min_length: usize) -> Self {
        Self {
            min_length,
        }
    }
}

impl Named for PacketDeleteMutator {
    fn name(&self) -> &str {
        "PacketDeleteMutator"
    }
}

impl<P, S> Mutator<DragonflyInput<P>, S> for PacketDeleteMutator
where
    P: Packet,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut DragonflyInput<P>, _stage_idx: i32) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len <= self.min_length || len == 0 {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().below(len as u64) as usize;
        input.packets_mut().remove(idx);
        
        Ok(MutationResult::Mutated)
    }
}
