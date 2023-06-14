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

pub struct PacketDuplicateMutator {
    max_length: usize,
}

impl PacketDuplicateMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
        }
    }
}

impl<I, S> Mutator<I, S> for PacketDuplicateMutator
where
    I: Input + HasPacketVector,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let len = input.packets().len();

        if len >= self.max_length {
            Ok(MutationResult::Skipped)
        } else {
            let from = state.rand_mut().below(len as u64) as usize;
            let to = state.rand_mut().below(len as u64 + 1) as usize;
            let copy = input.packets()[from].clone();
            input.packets_mut().insert(to, copy);
            Ok(MutationResult::Mutated)
        }
    }
}

impl Named for PacketDuplicateMutator {
    fn name(&self) -> &str {
        "PacketDuplicateMutator"
    }
}
