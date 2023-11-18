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

pub struct PacketReorderMutator;

impl PacketReorderMutator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

impl<I, S> Mutator<I, S> for PacketReorderMutator
where
    I: Input + HasPacketVector,
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let num_packets = input.packets().len();

        if num_packets <= 1 {
            return Ok(MutationResult::Skipped);
        }
        
        let to = state.rand_mut().below(num_packets as u64) as usize;
        let from = if let Some(selected_idx) = state.metadata_map_mut().get_mut::<SelectedPacketMetadata>() {
            let Some(selected_idx) = selected_idx.inner_mut() else {
                return Ok(MutationResult::Skipped);
            };
            
            let tmp = *selected_idx;
            *selected_idx = to;
            tmp
        } else {
            state.rand_mut().below(num_packets as u64) as usize
        };

        if from == to {
            return Ok(MutationResult::Skipped);
        }

        input.packets_mut().swap(from, to);
        
        Ok(MutationResult::Mutated)
    }
}

impl Named for PacketReorderMutator {
    fn name(&self) -> &str {
        "PacketReorderMutator"
    }
}
