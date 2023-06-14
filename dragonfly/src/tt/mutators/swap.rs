use crate::{
    mutators::PacketMutator,
    tt::token::HasTokenStream,
};
use libafl::prelude::{
    Error,
    HasRand,
    MutationResult,
    Rand,
};
use std::marker::PhantomData;

/// Swaps two random tokens in a tokenstream
pub struct TokenStreamSwapMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenStreamSwapMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenStreamSwapMutator<P, S>
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

            let idx1 = state.rand_mut().below(len as u64) as usize;
            let idx2 = state.rand_mut().below(len as u64) as usize;
            token_stream.tokens_mut().swap(idx1, idx2);

            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
    }
}
