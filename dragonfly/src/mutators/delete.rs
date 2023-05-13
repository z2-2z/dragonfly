use libafl::prelude::{
    Error,
    HasRand,
    Input,
    MutationResult,
    Mutator,
    Named,
    Rand,
};

use crate::input::HasPacketVector;

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

impl<I, S> Mutator<I, S> for PacketDeleteMutator
where
    I: Input + HasPacketVector,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let len = input.packets().len();

        if len <= self.min_length {
            Ok(MutationResult::Skipped)
        } else {
            let idx = state.rand_mut().below(len as u64) as usize;
            input.packets_mut().remove(idx);
            Ok(MutationResult::Mutated)
        }
    }
}

impl Named for PacketDeleteMutator {
    fn name(&self) -> &str {
        "PacketDeleteMutator"
    }
}
