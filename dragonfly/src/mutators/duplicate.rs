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
        let to = state.rand_mut().below(len as u64 + 1) as usize;

        if len < self.max_length {
            if let Some(selected_idx) = state.metadata_map_mut().get_mut::<SelectedPacketMetadata>() {
                if let Some(from) = selected_idx.inner_mut() {
                    let copy = input.packets()[*from].clone();
                    input.packets_mut().insert(to, copy);
                    
                    /* Adjust the selected packet index after modifying the array */
                    if to <= *from {
                        *from += 1;
                        debug_assert!(*from < input.packets().len());
                    }
                    
                    return Ok(MutationResult::Mutated);
                }
            } else if len > 0 {
                /* Fall back to random selection of target packet */
                let from = state.rand_mut().below(len as u64) as usize;
                let copy = input.packets()[from].clone();
                input.packets_mut().insert(to, copy);
                return Ok(MutationResult::Mutated);
            }    
        }
        
        Ok(MutationResult::Skipped)
    }
}

impl Named for PacketDuplicateMutator {
    fn name(&self) -> &str {
        "PacketDuplicateMutator"
    }
}
