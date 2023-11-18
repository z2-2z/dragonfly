use crate::input::HasPacketVector;
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

pub trait NewRandom<S> {
    fn new_random(state: &mut S) -> Self;
}

pub struct InsertRandomPacketMutator;

impl InsertRandomPacketMutator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

impl<I, S, P> Mutator<I, S> for InsertRandomPacketMutator
where
    I: Input + HasPacketVector<Packet = P>,
    S: HasRand + HasMetadata,
    P: NewRandom<S>,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let idx = state.rand_mut().below(input.packets().len() as u64 + 1) as usize;
        let new_packet = P::new_random(state);
        input.packets_mut().insert(idx, new_packet);
        
        /* Adjust selected packet after modifying array */
        if let Some(selected_idx) = state.metadata_map_mut().get_mut::<SelectedPacketMetadata>() {
            if let Some(selected_idx) = selected_idx.inner_mut() {
                if idx <= *selected_idx {
                    *selected_idx += 1;
                    debug_assert!(*selected_idx < input.packets().len());
                }
            }
        }
        
        Ok(MutationResult::Mutated)
    }
}

impl Named for InsertRandomPacketMutator {
    fn name(&self) -> &str {
        "InsertRandomPacketMutator"
    }
}
