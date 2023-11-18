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
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len >= self.max_length {
            return Ok(MutationResult::Skipped);
        }
        
        let to = state.rand_mut().below(len as u64 + 1) as usize;

        let from = if let Some(selected_idx) = state.metadata_map_mut().get_mut::<SelectedPacketMetadata>() {
            let Some(selected_idx) = selected_idx.inner_mut() else {
                return Ok(MutationResult::Skipped);
            };
            
            let tmp = *selected_idx;
            
            if to <= tmp {
                *selected_idx += 1;
            }
            
            tmp
        } else {
            if len == 0 {
                return Ok(MutationResult::Skipped);
            }
            
            state.rand_mut().below(len as u64) as usize
        };
        
        let copy = input.packets()[from].clone();
        input.packets_mut().insert(to, copy);
        Ok(MutationResult::Mutated)
    }
}

impl Named for PacketDuplicateMutator {
    fn name(&self) -> &str {
        "PacketDuplicateMutator"
    }
}
