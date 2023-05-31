use std::marker::PhantomData;
use crate::{
    tt::token::{
        HasTokenStream,
    },
    mutators::PacketMutator,
};
use libafl::prelude::{MutationResult, Error, HasRand, Rand};

/// Copies a random token somewhere else into the tokenstream
pub struct TokenStreamCopyMutator<P, S>
where
    P: HasTokenStream,
{
    max_len: usize,
    phantom: PhantomData<(P,S)>,
}

impl<P, S> TokenStreamCopyMutator<P, S>
where
    P: HasTokenStream,
{
    #[allow(clippy::new_without_default)]
    pub fn new(max_len: usize) -> Self {
        Self {
            max_len,
            phantom: PhantomData,
        }
    }
}

impl<P, S> PacketMutator<P, S> for TokenStreamCopyMutator<P, S>
where
    P: HasTokenStream,
    S: HasRand,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            let len = token_stream.tokens().len();
            
            if len == 0 || len >= self.max_len {
                return Ok(MutationResult::Skipped);
            }
            
            let idx = state.rand_mut().below(len as u64) as usize;
            let new_token = token_stream.tokens()[idx].clone();
            
            let idx = state.rand_mut().below(len as u64 + 1) as usize;
            token_stream.tokens_mut().insert(idx, new_token);
            
            return Ok(MutationResult::Mutated);
        }
        
        Ok(MutationResult::Skipped)
    }
}
