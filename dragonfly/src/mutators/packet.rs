use libafl::prelude::{
    Error,
    MutationResult,
};

pub trait PacketMutator<P, S> {
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, stage_idx: i32) -> Result<MutationResult, Error>;
}
