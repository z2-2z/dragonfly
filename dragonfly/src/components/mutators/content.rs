use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::components::{DragonflyInput, Packet};
use std::marker::PhantomData;

pub trait PacketMutator<P, S>
where
    P: Packet,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P) -> Result<MutationResult, Error>;
}

pub struct PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
{
    mutator: M,
    phantom: PhantomData<(P, S)>,
}

impl<P, S, M> PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
{
    #[allow(clippy::new_without_default)]
    pub fn new(mutator: M) -> Self {
        Self {
            mutator,
            phantom: PhantomData,
        }
    }
}

impl<P, S, M> Named for PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
{
    fn name(&self) -> &str {
        "PacketContentMutator"
    }
}

impl<P, S, M> Mutator<DragonflyInput<P>, S> for PacketContentMutator<P, S, M>
where
    M: PacketMutator<P, S>,
    P: Packet,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut DragonflyInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        
        if len == 0 {
            return Ok(MutationResult::Skipped);
        }
        
        let idx = state.rand_mut().below(len as u64) as usize;
        let packet = &mut input.packets_mut()[idx];
        self.mutator.mutate_packet(state, packet)
    }
}
