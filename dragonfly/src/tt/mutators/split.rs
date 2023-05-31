use std::marker::PhantomData;
use crate::{
    tt::token::{
        HasTokenStream, TextToken,
    },
    mutators::PacketMutator,
};
use libafl::prelude::{MutationResult, Error, HasRand, Rand};

/// A mutator that splits a token in two at a random point
pub struct TokenSplitMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P,S)>,
}

impl<P, S> TokenSplitMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenSplitMutator<P, S>
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
            
            if let TextToken::Constant(_) = token {
                return Ok(MutationResult::Skipped);
            }
            
            let len = token.len();
            
            if len <= 1 {
                return Ok(MutationResult::Skipped);
            }
            
            let split_point = 1 + state.rand_mut().below(len as u64 - 1) as usize;
            
            let new_token = match token {
                TextToken::Constant(_) => unreachable!(),
                TextToken::Number(data) => TextToken::Number(data.split_off(split_point)),
                TextToken::Whitespace(data) => TextToken::Whitespace(data.split_off(split_point)),
                TextToken::Text(data) => TextToken::Text(data.split_off(split_point)),
                TextToken::Blob(data) => TextToken::Blob(data.split_off(split_point)),
            };
            
            token_stream.tokens_mut().insert(idx + 1, new_token);
            return Ok(MutationResult::Mutated);
        }
        
        Ok(MutationResult::Skipped)
    }
}
