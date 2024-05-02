use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::components::{DragonflyInput, Packet};

pub struct PacketRepeatMutator {
    max_length: usize,
}

impl PacketRepeatMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
        }
    }
}

impl Named for PacketRepeatMutator {
    fn name(&self) -> &str {
        "PacketRepeatMutator"
    }
}

impl<P, S> Mutator<DragonflyInput<P>, S> for PacketRepeatMutator
where
    P: Packet + Clone,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut DragonflyInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len == 0 || len >= self.max_length {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().below(len as u64) as usize;
        let n = 1 + state.rand_mut().below((self.max_length - len) as u64) as usize;
        let packet = input.packets()[idx].clone();
        input.packets_mut().splice(idx..idx, vec![packet; n]);
        
        Ok(MutationResult::Mutated)
    }
}
