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

pub trait NewRandom<S>{
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
    S: HasRand,
    P: NewRandom<S>,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let idx = state.rand_mut().below(input.packets().len() as u64) as usize;
        let new_packet = P::new_random(state);
        input.packets_mut().insert(idx, new_packet);
        Ok(MutationResult::Mutated)
    }
}

impl Named for InsertRandomPacketMutator {
    fn name(&self) -> &str {
        "InsertRandomPacketMutator"
    }
}
