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

pub trait NewGenerated<S>{
    fn new_generated(state: &mut S) -> Self;
}

pub struct InsertGeneratedPacketMutator;

impl InsertGeneratedPacketMutator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

impl<I, S, P> Mutator<I, S> for InsertGeneratedPacketMutator
where
    I: Input + HasPacketVector<Packet = P>,
    S: HasRand,
    P: NewGenerated<S>,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let n = std::cmp::min(
            state.rand_mut().below(input.packets().len() as u64 / 8),
            1
        );
        
        for _ in 0..n {
            let idx = state.rand_mut().below(input.packets().len() as u64) as usize;
            let new_packet = P::new_generated(state);
            input.packets_mut().insert(idx, new_packet);
        }
        
        Ok(MutationResult::Mutated)
    }
}

impl Named for InsertGeneratedPacketMutator {
    fn name(&self) -> &str {
        "PacketGeneratorMutator"
    }
}
