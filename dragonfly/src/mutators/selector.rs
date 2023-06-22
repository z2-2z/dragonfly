use crate::{
    input::HasPacketVector,
};
use libafl::prelude::{
    Error,
    HasRand,
    MutationResult,
    Mutator,
    Named,
    Rand,
    impl_serdeany,
    HasMetadata,
    CorpusId,
};
use std::marker::PhantomData;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectedPacketMetadata(Option<usize>);

impl_serdeany!(SelectedPacketMetadata);

impl SelectedPacketMetadata {
    pub fn inner(&self) -> Option<&usize> {
        self.0.as_ref()
    }
    
    pub fn inner_mut(&mut self) -> Option<&mut usize> {
        self.0.as_mut()
    }
}

pub struct PacketSelectorMutator<I, P, S, M>
where
    M: Mutator<I, S>,
    S: HasRand,
{
    mutator: M,
    phantom: PhantomData<(I, P, S)>,
}

impl<I, P, S, M> PacketSelectorMutator<I, P, S, M>
where
    M: Mutator<I, S>,
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

impl<I, P, S, M> PacketSelectorMutator<I, P, S, M>
where
    I: HasPacketVector<Packet = P>,
    M: Mutator<I, S>,
    S: HasRand,
{
    fn schedule_packet(&self, state: &mut S, input: &I) -> Option<usize> {
        let packets = input.packets();
        
        if packets.is_empty() {
            None
        } else {
            Some(state.rand_mut().below(packets.len() as u64) as usize)
        }
    }
}

impl<I, P, S, M> Mutator<I, S> for PacketSelectorMutator<I, P, S, M>
where
    I: HasPacketVector<Packet = P>,
    M: Mutator<I, S>,
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, stage_idx: i32) -> Result<MutationResult, Error> {
        let packet = self.schedule_packet(state, input);
        
        #[cfg(test)]
        println!("Selecting packet {:?}", packet);
        
        state.add_metadata(SelectedPacketMetadata(packet));
        self.mutator.mutate(state, input, stage_idx)
    }
    
    fn post_exec(&mut self, state: &mut S, stage_idx: i32, corpus_idx: Option<CorpusId>) -> Result<(), Error> {
        self.mutator.post_exec(state, stage_idx, corpus_idx)
    }
}

impl<I, P, S, M> Named for PacketSelectorMutator<I, P, S, M>
where
    M: Mutator<I, S>,
    S: HasRand,
{
    fn name(&self) -> &str {
        "PacketSelectorMutator"
    }
}
