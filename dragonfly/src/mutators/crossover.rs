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

        if let Some(selected_packet) = state.metadata_map().get::<SelectedPacketMetadata>() {
            if let Some(dst_idx) = selected_packet.inner().copied() {
                let src_idx = state.rand_mut().below(packets_len as u64) as usize;
                let other = input.packets()[src_idx].clone();
                input.packets_mut()[dst_idx].crossover_insert(state, other);
                return Ok(MutationResult::Mutated);
            }
        } else if packets_len > 0 {
            /* Fall back to random selection of destination packet */
            let src_idx = state.rand_mut().below(packets_len as u64) as usize;
            let dst_idx = state.rand_mut().below(packets_len as u64) as usize;
            let other = input.packets()[src_idx].clone();
            input.packets_mut()[dst_idx].crossover_insert(state, other);
            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
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

        if let Some(selected_packet) = state.metadata_map().get::<SelectedPacketMetadata>() {
            if let Some(dst_idx) = selected_packet.inner().copied() {
                let src_idx = state.rand_mut().below(packets_len as u64) as usize;
                let other = input.packets()[src_idx].clone();
                input.packets_mut()[dst_idx].crossover_replace(state, other);
                return Ok(MutationResult::Mutated);
            }
        } else if packets_len > 0 {
            /* Fall back to random selection of destination packet */
            let src_idx = state.rand_mut().below(packets_len as u64) as usize;
            let dst_idx = state.rand_mut().below(packets_len as u64) as usize;
            let other = input.packets()[src_idx].clone();
            input.packets_mut()[dst_idx].crossover_replace(state, other);
            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
    }
}

impl Named for PacketCrossoverReplaceMutator {
    fn name(&self) -> &str {
        "PacketCrossoverReplaceMutator"
    }
}
