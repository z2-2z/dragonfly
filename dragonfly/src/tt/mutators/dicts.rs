use crate::{
    mutators::PacketMutator,
    tt::token::{
        HasTokenStream,
        TextToken,
    },
};
use libafl_bolts::{
    rands::Rand,
};
use libafl::prelude::{
    Error,
    HasMetadata,
    HasRand,
    MutationResult,
    Tokens,
};
use std::marker::PhantomData;

/// Insert dictionary items as constant tokens into the token stream
pub struct TokenStreamDictInsertMutator<P, S>
where
    P: HasTokenStream,
{
    max_len: usize,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenStreamDictInsertMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenStreamDictInsertMutator<P, S>
where
    P: HasTokenStream,
    S: HasRand + HasMetadata,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            let len = token_stream.tokens().len();

            if len >= self.max_len {
                return Ok(MutationResult::Skipped);
            }

            let Some(dict) = state.metadata_map().get::<Tokens>() else {
                return Ok(MutationResult::Skipped);
            };
            let dict_size = dict.len();

            if dict_size == 0 {
                return Ok(MutationResult::Skipped);
            }

            let idx = state.rand_mut().below(dict_size as u64) as usize;

            let dict = state.metadata::<Tokens>()?;
            let new_data = dict.tokens()[idx].clone();
            let new_token = TextToken::Constant(new_data);

            let idx = state.rand_mut().below(len as u64 + 1) as usize;
            token_stream.tokens_mut().insert(idx, new_token);

            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
    }
}

/// Replace a random token in a token stream with a constant item from a dictionary
pub struct TokenReplaceDictMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenReplaceDictMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenReplaceDictMutator<P, S>
where
    P: HasTokenStream,
    S: HasRand + HasMetadata,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            let len = token_stream.tokens().len();

            if len == 0 {
                return Ok(MutationResult::Skipped);
            }

            let Some(dict) = state.metadata_map().get::<Tokens>() else {
                return Ok(MutationResult::Skipped);
            };
            let dict_size = dict.len();

            if dict_size == 0 {
                return Ok(MutationResult::Skipped);
            }

            let idx = state.rand_mut().below(dict_size as u64) as usize;

            let dict = state.metadata::<Tokens>()?;
            let new_data = dict.tokens()[idx].clone();
            let new_token = TextToken::Constant(new_data);

            let idx = state.rand_mut().below(len as u64) as usize;
            token_stream.tokens_mut()[idx] = new_token;

            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
    }
}
