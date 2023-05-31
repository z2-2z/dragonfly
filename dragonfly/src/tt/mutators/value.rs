use std::marker::PhantomData;
use crate::{
    tt::token::HasTokenStream,
    mutators::PacketMutator,
};
use libafl::prelude::{MutationResult, Error};

pub struct TokenStreamValueMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P,S)>,
}

impl<P, S> TokenStreamValueMutator<P, S>
where
    P: HasTokenStream,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<P, S> PacketMutator<P, S> for TokenStreamValueMutator<P, S>
where
    P: HasTokenStream,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            
            todo!()
        } else {
            Ok(MutationResult::Skipped)
        }
    }
}
