use crate::{
    input::HasPacketVector,
    mutators::selector::SelectedPacketMetadata,
};
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

pub trait HasCrossover<S> {
    fn crossover_insert(&mut self, state: &mut S, other: Self);
    fn crossover_replace(&mut self, state: &mut S, other: Self);
}

pub struct PacketCrossoverInsertMutator;

impl PacketCrossoverInsertMutator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

impl<I, S, P> Mutator<I, S> for PacketCrossoverInsertMutator
where
    I: Input + HasPacketVector<Packet = P>,
    S: HasRand + HasMetadata,
    P: HasCrossover<S> + Clone,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let packets_len = input.packets().len();

        let dst_idx = if let Some(selected_packet) = state.metadata_map().get::<SelectedPacketMetadata>() {
            let Some(dst_idx) = selected_packet.inner() else {
                return Ok(MutationResult::Skipped);
            };
            
            *dst_idx
        } else {
            if packets_len == 0 {
                return Ok(MutationResult::Skipped);
            }
            
            state.rand_mut().below(packets_len as u64) as usize
        };
        
        let src_idx = state.rand_mut().below(packets_len as u64) as usize;
        let other = input.packets()[src_idx].clone();
        
        input.packets_mut()[dst_idx].crossover_insert(state, other);

        Ok(MutationResult::Mutated)
    }
}

impl Named for PacketCrossoverInsertMutator {
    fn name(&self) -> &str {
        "PacketCrossoverInsertMutator"
    }
}

pub struct PacketCrossoverReplaceMutator;

impl PacketCrossoverReplaceMutator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

impl<I, S, P> Mutator<I, S> for PacketCrossoverReplaceMutator
where
    I: Input + HasPacketVector<Packet = P>,
    S: HasRand + HasMetadata,
    P: HasCrossover<S> + Clone,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, _stage_idx: i32) -> Result<MutationResult, Error> {
        let packets_len = input.packets().len();

        let dst_idx = if let Some(selected_packet) = state.metadata_map().get::<SelectedPacketMetadata>() {
            let Some(dst_idx) = selected_packet.inner() else {
                return Ok(MutationResult::Skipped);
            };
            
            *dst_idx
        } else {
            if packets_len == 0 {
                return Ok(MutationResult::Skipped);
            }
            
            state.rand_mut().below(packets_len as u64) as usize
        };
        
        let src_idx = state.rand_mut().below(packets_len as u64) as usize;
        let other = input.packets()[src_idx].clone();
        
        input.packets_mut()[dst_idx].crossover_replace(state, other);

        Ok(MutationResult::Mutated)
    }
}

impl Named for PacketCrossoverReplaceMutator {
    fn name(&self) -> &str {
        "PacketCrossoverReplaceMutator"
    }
}
