use std::marker::PhantomData;
use crate::{
    tt::token::{
        HasTokenStream
    },
    mutators::PacketMutator,
};
use libafl::prelude::{MutationResult, Error, HasRand, Rand};

/// A mutator that duplicates a single, random token
pub struct TokenStreamDuplicateMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P,S)>,
}

impl<P, S> TokenStreamDuplicateMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenStreamDuplicateMutator<P, S>
where
    P: HasTokenStream,
    S: HasRand,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            let len = token_stream.tokens().len();
            
            if len == 0 {
                return Ok(MutationResult::Skipped);
            }
            
            let idx = state.rand_mut().below(len as u64) as usize;
            let new_token = token_stream.tokens()[idx].clone();
            token_stream.tokens_mut().insert(idx + 1, new_token);
            return Ok(MutationResult::Mutated);
        }
        
        Ok(MutationResult::Skipped)
    }
}
