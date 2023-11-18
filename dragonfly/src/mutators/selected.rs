use crate::{
    input::HasPacketVector,
    mutators::{
        packet::PacketMutator,
        selector::SelectedPacketMetadata,
    },
};
use libafl_bolts::{
    Named,
};
use libafl::prelude::{
    Error,
    HasRand,
    MutationResult,
    Mutator,
    HasMetadata,
};
use std::marker::PhantomData;

pub struct SelectedPacketMutator<I, P, S, M>
where
    M: PacketMutator<P, S>,
    S: HasRand,
{
    mutator: M,
    phantom: PhantomData<(I, P, S)>,
}

impl<I, P, S, M> SelectedPacketMutator<I, P, S, M>
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
    
    pub fn inner(&self) -> &M {
        &self.mutator
    }
    
    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.mutator
    }
}

impl<I, P, S, M> Mutator<I, S> for SelectedPacketMutator<I, P, S, M>
where
    I: HasPacketVector<Packet = P>,
    M: PacketMutator<P, S>,
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, stage_idx: i32) -> Result<MutationResult, Error> {
        let scheduled_packet = state.metadata::<SelectedPacketMetadata>()?.inner().copied();
        
        if let Some(idx) = scheduled_packet {
            let packet = &mut input.packets_mut()[idx];
            self.mutator.mutate_packet(state, packet, stage_idx)
        } else {
            Ok(MutationResult::Skipped)
        }
    }
}

impl<I, P, S, M> Named for SelectedPacketMutator<I, P, S, M>
where
    M: PacketMutator<P, S>,
    S: HasRand,
{
    fn name(&self) -> &str {
        "SelectedPacketMutator"
    }
}
