use std::marker::PhantomData;
use crate::{
    tt::token::{
        HasTokenStream, TextToken, is_number,
        is_whitespace, is_text,
    },
    mutators::PacketMutator,
};
use libafl::prelude::{MutationResult, Error, HasRand, Rand};

/// Changes the type of a random token to a random other type
pub struct TokenConvertMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P,S)>,
}

impl<P, S> TokenConvertMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenConvertMutator<P, S>
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
            let dst_type = state.rand_mut().below(4);
            
            let new_token = match &token_stream.tokens()[idx] {
                TextToken::Number(data) |
                TextToken::Whitespace(data) |
                TextToken::Text(data) |
                TextToken::Blob(data) |
                TextToken::Constant(data) => {
                    match dst_type {
                        0 => {
                            if !is_number(data) {
                                return Ok(MutationResult::Skipped);
                            }
                            
                            TextToken::Number(data.clone())
                        },
                        1 => {
                            if !is_whitespace(data) {
                                return Ok(MutationResult::Skipped);
                            }
                            
                            TextToken::Whitespace(data.clone())
                        },
                        2 => {
                            if !is_text(data) {
                                return Ok(MutationResult::Skipped);
                            }
                            
                            TextToken::Text(data.clone())
                        },
                        3 => TextToken::Blob(data.clone()),
                        _ => unreachable!()
                    }
                },
            };
            
            token_stream.tokens_mut()[idx] = new_token;
            return Ok(MutationResult::Mutated);
        }
        
        Ok(MutationResult::Skipped)
    }
}
