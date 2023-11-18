use crate::{
    input::HasPacketVector,
    mutators::packet::PacketMutatorTuple,
    mutators::selector::SelectedPacketMetadata,
};
use libafl::prelude::{
    Error,
    HasRand,
    MutationResult,
    Mutator,
    Named,
    Rand,
    HasMetadata,
};
use std::marker::PhantomData;

pub struct ScheduledPacketMutator<I, P, S, M>
where
    M: PacketMutatorTuple<P, S>,
    S: HasRand,
{
    mutators: M,
    max_stack_pow: u64,
    phantom: PhantomData<(I, P, S)>,
}

impl<I, P, S, M> ScheduledPacketMutator<I, P, S, M>
where
    M: PacketMutatorTuple<P, S>,
    S: HasRand,
{
    pub fn new(mutators: M) -> Self {
        assert!(!mutators.is_empty());

        Self {
            mutators,
            max_stack_pow: 7,
            phantom: PhantomData,
        }
    }

    pub fn with_max_stack_pow(mutators: M, max_stack_pow: u64) -> Self {
        assert!(!mutators.is_empty());

        Self {
            mutators,
            max_stack_pow,
            phantom: PhantomData,
        }
    }
}

impl<I, P, S, M> ScheduledPacketMutator<I, P, S, M>
where
    I: HasPacketVector<Packet = P>,
    M: PacketMutatorTuple<P, S>,
    S: HasRand + HasMetadata,
{
    fn schedule_packet(&self, state: &mut S, input: &I) -> usize {
        if let Some(selected_idx) = state.metadata_map().get::<SelectedPacketMetadata>().and_then(|x| x.inner()) {
            *selected_idx
        } else {
            state.rand_mut().below(input.packets().len() as u64) as usize
        }
    }
}

// Simulate ScheduledMutator
impl<I, P, S, M> ScheduledPacketMutator<I, P, S, M>
where
    M: PacketMutatorTuple<P, S>,
    S: HasRand,
{
    fn iterations(&self, state: &mut S) -> u64 {
        1 << (state.rand_mut().below(self.max_stack_pow) + 1)
    }

    fn schedule_mutation(&self, state: &mut S) -> usize {
        state.rand_mut().below(self.mutators.len() as u64) as usize
    }

    fn scheduled_mutate(&mut self, state: &mut S, packet: &mut P, stage_idx: i32) -> Result<MutationResult, Error> {
        #[cfg(test)]
        println!("--- NEW MUTATION RUN ---");

        let mut result = MutationResult::Skipped;
        let num = self.iterations(state);
        for _ in 0..num {
            let mutation = self.schedule_mutation(state);
            let outcome = self.mutators.get_and_mutate(mutation, state, packet, stage_idx)?;

            #[cfg(test)]
            {
                if outcome == MutationResult::Mutated {
                    println!("Ran mutation #{}", mutation);
                }
            }

            if outcome == MutationResult::Mutated {
                result = MutationResult::Mutated;
            }
        }
        Ok(result)
    }
}

impl<I, P, S, M> Mutator<I, S> for ScheduledPacketMutator<I, P, S, M>
where
    I: HasPacketVector<Packet = P>,
    M: PacketMutatorTuple<P, S>,
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut I, stage_idx: i32) -> Result<MutationResult, Error> {
        let idx = self.schedule_packet(state, input);
        let packet = &mut input.packets_mut()[idx];
        self.scheduled_mutate(state, packet, stage_idx)
    }
}

impl<I, P, S, M> Named for ScheduledPacketMutator<I, P, S, M>
where
    M: PacketMutatorTuple<P, S>,
    S: HasRand,
{
    fn name(&self) -> &str {
        "ScheduledPacketMutator"
    }
}
