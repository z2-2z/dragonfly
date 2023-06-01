use std::marker::PhantomData;
use crate::{
    tt::token::{
        HasTokenStream, TextToken, is_ascii,
    },
    mutators::PacketMutator,
};
use libafl::prelude::{MutationResult, Error, HasRand, Rand};

/// Changes the type of a random constant text token to either text or blob
pub struct TokenTransformConstantMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P,S)>,
}

impl<P, S> TokenTransformConstantMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenTransformConstantMutator<P, S>
where
    P: HasTokenStream,
    S: HasRand,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            let mut ceiling = token_stream.tokens().len();
            
            while ceiling > 0 {
                let start = state.rand_mut().below(ceiling as u64) as usize;
                
                for i in start..ceiling {
                    let new_token = if let TextToken::Constant(data) = &token_stream.tokens()[i] {
                        let mut ascii = true;
                        
                        for byte in data {
                            if !is_ascii(*byte) {
                                ascii = false;
                                break;
                            }
                        }
                        
                        if ascii {
                            Some(TextToken::Text(data.clone()))
                        } else {
                            Some(TextToken::Blob(data.clone()))
                        }
                    } else {
                        None
                    };
                    
                    if let Some(new_token) = new_token {
                        token_stream.tokens_mut()[i] = new_token;
                        return Ok(MutationResult::Mutated);
                    }
                }
                
                ceiling = start;
            }
        }
        
        Ok(MutationResult::Skipped)
    }
}
