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

pub struct PacketRepeatMutator {
    max_length: usize,
    max_repeats: usize,
}

impl PacketRepeatMutator {
    #[allow(clippy::new_without_default)]
    pub fn new(max_length: usize, max_repeats: usize) -> Self {
        Self {
            max_length,
            max_repeats,
        }
    }
}

impl<I, S> Mutator<I, S> for PacketRepeatMutator
where
    I: Input + HasPacketVector,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let len = input.packets().len();

        if len == 0 || len >= self.max_length {
            Ok(MutationResult::Skipped)
        } else {
            let limit = std::cmp::min(self.max_length - len, self.max_repeats);
            let n = state.rand_mut().below(limit as u64) + 1;
            let idx = state.rand_mut().below(len as u64) as usize;
            let packets = vec![input.packets()[idx].clone(); n as usize];
            input.packets_mut().splice(idx..idx, packets);
            Ok(MutationResult::Mutated)
        }
    }
}

impl Named for PacketRepeatMutator {
    fn name(&self) -> &str {
        "PacketRepeatMutator"
    }
}
