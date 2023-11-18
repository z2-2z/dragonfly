use crate::{
    mutators::PacketMutator,
    tt::token::{
        HasTokenStream,
        TextToken,
    },
};
use libafl::prelude::{
    Error,
    HasRand,
    MutationResult,
    Rand,
};
use std::marker::PhantomData;

/// Deletes a random slice of tokens
pub struct TokenStreamDeleteMutator<P, S>
where
    P: HasTokenStream,
{
    min_length: usize,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenStreamDeleteMutator<P, S>
where
    P: HasTokenStream,
{
    #[allow(clippy::new_without_default)]
    pub fn new(min_length: usize) -> Self {
        Self {
            min_length,
            phantom: PhantomData,
        }
    }
}

impl<P, S> PacketMutator<P, S> for TokenStreamDeleteMutator<P, S>
where
    P: HasTokenStream,
    S: HasRand,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            let len = token_stream.tokens().len();

            if len <= self.min_length {
                return Ok(MutationResult::Skipped);
            }

            let idx = state.rand_mut().below(len as u64) as usize;
            let mut slice_len = 1 + state.rand_mut().below((len - idx) as u64) as usize;
            
            let new_len = len - slice_len;
            if new_len < self.min_length {
                slice_len -= self.min_length - new_len;
            }
            
            token_stream.tokens_mut().drain(idx..idx + slice_len);
            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
    }
}

/// Deletes a random substring of a single token
pub struct TokenValueDeleteMutator<P, S>
where
    P: HasTokenStream,
{
    min_length: usize,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenValueDeleteMutator<P, S>
where
    P: HasTokenStream,
{
    #[allow(clippy::new_without_default)]
    pub fn new(min_length: usize) -> Self {
        Self {
            phantom: PhantomData,
            min_length,
        }
    }
}

impl<P, S> PacketMutator<P, S> for TokenValueDeleteMutator<P, S>
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
            let token = &mut token_stream.tokens_mut()[idx];
            let token_len = token.len();

            if token_len == 0 {
                return Ok(MutationResult::Skipped);
            }

            let start = state.rand_mut().below(token_len as u64) as usize;
            let len = 1 + state.rand_mut().below(token_len as u64 - start as u64) as usize;

            let min_length = match token {
                TextToken::Number(_) => std::cmp::max(self.min_length, 2),
                _ => self.min_length,
            };

            if token_len - len < min_length {
                return Ok(MutationResult::Skipped);
            }

            match token {
                TextToken::Constant(_) => {},
                TextToken::Number(data) | TextToken::Whitespace(data) | TextToken::Text(data) | TextToken::Blob(data) => {
                    data.drain(start..start + len);
                    return Ok(MutationResult::Mutated);
                },
            }
        }

        Ok(MutationResult::Skipped)
    }
}
