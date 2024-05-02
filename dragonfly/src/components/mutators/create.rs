use libafl_bolts::prelude::{Named, Rand};
use libafl::prelude::{Mutator, MutationResult, Error, HasRand};
use crate::components::{DragonflyInput, Packet};
use std::marker::PhantomData;

pub trait PacketCreator<S>
where
    Self: Sized,
{
    fn create_packets(state: &mut S) -> Vec<Self>;
}

pub struct PacketInsertionMutator<P, S>
where
    P: Packet + PacketCreator<S>,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> PacketInsertionMutator<P, S>
where
    P: Packet + PacketCreator<S>,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<P, S> Named for PacketInsertionMutator<P, S>
where
    P: Packet + PacketCreator<S>,
{
    fn name(&self) -> &str {
        "PacketInsertionMutator"
    }
}

impl<P, S> Mutator<DragonflyInput<P>, S> for PacketInsertionMutator<P, S>
where
    P: Packet + PacketCreator<S>,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut DragonflyInput<P>) -> Result<MutationResult, Error> {
        let len = input.packets().len();
        let idx = state.rand_mut().below(len as u64) as usize;
        let new_packets = P::create_packets(state);
        
        if new_packets.is_empty() {
            Ok(MutationResult::Skipped)
        } else {
            input.packets_mut().splice(idx..idx, new_packets);
            Ok(MutationResult::Mutated)
        }
    }
}
