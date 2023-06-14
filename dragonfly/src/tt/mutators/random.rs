use crate::{
    mutators::PacketMutator,
    tt::token::{
        has_valid_sign,
        random_blob_value,
        random_number_value,
        random_text_value,
        random_whitespace_value,
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

/// Replaces the value of one random TextToken with a purely random new one
pub struct TokenReplaceRandomMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenReplaceRandomMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenReplaceRandomMutator<P, S>
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

            match &mut token_stream.tokens_mut()[idx] {
                TextToken::Constant(_) => {},
                TextToken::Number(data) => {
                    random_number_value(state.rand_mut(), data, true);
                    return Ok(MutationResult::Mutated);
                },
                TextToken::Whitespace(data) => {
                    random_whitespace_value(state.rand_mut(), data);
                    return Ok(MutationResult::Mutated);
                },
                TextToken::Text(data) => {
                    random_text_value(state.rand_mut(), data);
                    return Ok(MutationResult::Mutated);
                },
                TextToken::Blob(data) => {
                    random_blob_value(state.rand_mut(), data);
                    return Ok(MutationResult::Mutated);
                },
            }
        }

        Ok(MutationResult::Skipped)
    }
}

/// Inserts a new TextToken with random value
pub struct TokenStreamInsertRandomMutator<P, S>
where
    P: HasTokenStream,
{
    max_len: usize,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenStreamInsertRandomMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenStreamInsertRandomMutator<P, S>
where
    P: HasTokenStream,
    S: HasRand,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            let len = token_stream.tokens().len();

            if len >= self.max_len {
                return Ok(MutationResult::Skipped);
            }

            let idx = state.rand_mut().below(len as u64 + 1) as usize;
            let mut data = Vec::new();

            let new_token = match state.rand_mut().below(4) {
                0 => {
                    random_number_value(state.rand_mut(), &mut data, true);
                    TextToken::Number(data)
                },
                1 => {
                    random_whitespace_value(state.rand_mut(), &mut data);
                    TextToken::Whitespace(data)
                },
                2 => {
                    random_text_value(state.rand_mut(), &mut data);
                    TextToken::Text(data)
                },
                3 => {
                    random_blob_value(state.rand_mut(), &mut data);
                    TextToken::Blob(data)
                },
                _ => unreachable!(),
            };

            token_stream.tokens_mut().insert(idx, new_token);
            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
    }
}

/// Inserts random data into a single, random token
pub struct TokenValueInsertRandomMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenValueInsertRandomMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenValueInsertRandomMutator<P, S>
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
            let mut new_data = Vec::new();

            match &mut token_stream.tokens_mut()[idx] {
                TextToken::Constant(_) => {},
                TextToken::Number(data) => {
                    let start = usize::from(has_valid_sign(&data));
                    let idx = start + state.rand_mut().below(data.len() as u64 + 1 - start as u64) as usize;
                    random_number_value(state.rand_mut(), &mut new_data, idx == 0);
                    data.splice(idx..idx, new_data);
                    return Ok(MutationResult::Mutated);
                },
                TextToken::Whitespace(data) => {
                    let idx = state.rand_mut().below(data.len() as u64 + 1) as usize;
                    random_whitespace_value(state.rand_mut(), &mut new_data);
                    data.splice(idx..idx, new_data);
                    return Ok(MutationResult::Mutated);
                },
                TextToken::Text(data) => {
                    let idx = state.rand_mut().below(data.len() as u64 + 1) as usize;
                    random_text_value(state.rand_mut(), &mut new_data);
                    data.splice(idx..idx, new_data);
                    return Ok(MutationResult::Mutated);
                },
                TextToken::Blob(data) => {
                    let idx = state.rand_mut().below(data.len() as u64 + 1) as usize;
                    random_blob_value(state.rand_mut(), &mut new_data);
                    data.splice(idx..idx, new_data);
                    return Ok(MutationResult::Mutated);
                },
            }
        }

        Ok(MutationResult::Skipped)
    }
}
