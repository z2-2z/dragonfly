use std::marker::PhantomData;
use libafl::prelude::{HasRand, Mutator, Rand, Named};
use crate::{
    mutators::packet::PacketMutator,
    input::HasPacketVector,
};

pub struct ScheduledPacketMutator<I, P, S, M>
where
    M: PacketMutator<P, S>,
    S: HasRand,
{
    mutator: M,
    phantom: PhantomData<(I, P, S)>,
}

impl<I, P, S, M> ScheduledPacketMutator<I, P, S, M>
where
    M: PacketMutator<P, S>,
    S: HasRand,
{
    pub fn new(mutator: M) -> Self {
        Self {
            mutator,
            phantom: PhantomData,
        }
    }
}

impl<I, P, S, M> Mutator<I, S> for ScheduledPacketMutator<I, P, S, M>
where
    I: HasPacketVector<Packet = P>,
    M: PacketMutator<P, S>,
    S: HasRand,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
        stage_idx: i32,
    ) -> Result<libafl::prelude::MutationResult, libafl::Error> {
        let idx = state.rand_mut().below(input.packets().len() as u64) as usize;
        let packet = &mut input.packets_mut()[idx];
        self.mutator.mutate_packet(state, packet, stage_idx)
    }
}

impl<I, P, S, M> Named for ScheduledPacketMutator<I, P, S, M>
where
    M: PacketMutator<P, S>,
    S: HasRand,
{
    fn name(&self) -> &str {
        "ScheduledPacketMutator"
    }
}
