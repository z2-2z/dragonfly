use libafl_bolts::{
    Named,
    rands::Rand,
};
use libafl::prelude::{
    Error,
    HasRand,
    Input,
    MutationResult,
    Mutator,
    HasMetadata,
};
use crate::mutators::selector::SelectedPacketMetadata;
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
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let len = input.packets().len();

        if len == 0 || len >= self.max_length {
            return Ok(MutationResult::Skipped);
        }
        
        let limit = std::cmp::min(self.max_length - len, self.max_repeats);
        let n = state.rand_mut().below(limit as u64) as usize + 1;
        let idx = if let Some(selected_idx) = state.metadata_map_mut().get_mut::<SelectedPacketMetadata>() {
            let Some(selected_idx) = selected_idx.inner_mut() else {
                return Ok(MutationResult::Skipped);
            };
            
            let tmp = *selected_idx;
            *selected_idx += n;
            tmp
        } else {
            state.rand_mut().below(len as u64) as usize
        };
        let src = input.packets()[idx].clone();
        input.packets_mut().splice(idx..idx, vec![src; n]);
        
        Ok(MutationResult::Mutated)
    }
}

impl Named for PacketRepeatMutator {
    fn name(&self) -> &str {
        "PacketRepeatMutator"
    }
}
