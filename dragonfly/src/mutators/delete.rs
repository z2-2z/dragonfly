use libafl::prelude::{
    Error,
    HasRand,
    Input,
    MutationResult,
    Mutator,
    Named,
    Rand,
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

        if len >= self.min_length && len > 0 {
            let selected_idx = if let Some(selected_packet) = state.metadata_map().get::<SelectedPacketMetadata>() {
                selected_packet.inner().copied() 
            } else {
                None
            };
            
            let rem_idx = state.rand_mut().below(len as u64) as usize;
            
            if Some(rem_idx) != selected_idx {
                input.packets_mut().remove(rem_idx);
                
                /* Adjust the selected packet index after modifying the array */
                if let Some(selected_idx) = selected_idx {
                    if rem_idx < selected_idx {
                        *state.metadata_map_mut().get_mut::<SelectedPacketMetadata>().unwrap().inner_mut().unwrap() = selected_idx - 1;
                        assert!(selected_idx - 1 < input.packets().len());
                    }
                }
                
                return Ok(MutationResult::Mutated);
            }
        }
        
        Ok(MutationResult::Skipped)
    }
}

impl Named for PacketDeleteMutator {
    fn name(&self) -> &str {
        "PacketDeleteMutator"
    }
}
