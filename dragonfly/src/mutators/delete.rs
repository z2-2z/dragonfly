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
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let len = input.packets().len();

        if len <= self.min_length || len == 0 {
            return Ok(MutationResult::Skipped);
        }
        
        let rem_idx = state.rand_mut().below(len as u64) as usize;

        let selected_idx = if let Some(selected_idx) = state.metadata_map_mut().get_mut::<SelectedPacketMetadata>() {
            selected_idx.inner_mut().map(|x| {
                let tmp = *x;
                
                if rem_idx < tmp {
                    *x -= 1;
                }
                
                tmp
            })
        } else {
            None
        };
        
        if selected_idx.as_ref().map_or(false, |x| *x == rem_idx) {
            return Ok(MutationResult::Skipped);
        }
        
        input.packets_mut().remove(rem_idx);
        
        Ok(MutationResult::Mutated)
    }
}

impl Named for PacketDeleteMutator {
    fn name(&self) -> &str {
        "PacketDeleteMutator"
    }
}
