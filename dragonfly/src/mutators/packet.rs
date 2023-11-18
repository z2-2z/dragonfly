use libafl::prelude::{
    Error,
    MutationResult,
    HasConstLen,
};

pub trait PacketMutator<P, S> {
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, stage_idx: i32) -> Result<MutationResult, Error>;
}

pub trait PacketMutatorTuple<P, S>: HasConstLen {
    fn get_and_mutate(&mut self, index: usize, state: &mut S, packet: &mut P, stage_idx: i32) -> Result<MutationResult, Error>;
}

impl<P, S> PacketMutatorTuple<P, S> for () {
    fn get_and_mutate(&mut self, _index: usize, _state: &mut S, _packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        Ok(MutationResult::Skipped)
    }
}

impl<Head, Tail, P, S> PacketMutatorTuple<P, S> for (Head, Tail)
where
    Head: PacketMutator<P, S>,
    Tail: PacketMutatorTuple<P, S>,
{
    fn get_and_mutate(&mut self, index: usize, state: &mut S, packet: &mut P, stage_idx: i32) -> Result<MutationResult, Error> {
        if index == 0 {
            self.0.mutate_packet(state, packet, stage_idx)
        } else {
            self.1.get_and_mutate(index - 1, state, packet, stage_idx)
        }
    }
}
