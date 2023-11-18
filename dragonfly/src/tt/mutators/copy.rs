use crate::{
    mutators::PacketMutator,
    tt::token::{HasTokenStream, TextToken, has_valid_sign},
};
use libafl_bolts::{
    rands::Rand,
};
use libafl::prelude::{
    Error,
    HasRand,
    MutationResult,
};
use std::marker::PhantomData;

/// Copies a random slice of tokens somewhere else into the tokenstream
pub struct TokenStreamCopyMutator<P, S>
where
    P: HasTokenStream,
{
    max_len: usize,
    phantom: PhantomData<(P, S)>,
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
            
            let rem_len = len - idx;
            let slice_len = 1 + state.rand_mut().below(rem_len as u64) as usize;
            let slice_len = std::cmp::min(slice_len, self.max_len - len);
            
            let new_tokens = token_stream.tokens()[idx..idx + slice_len].to_vec();

            let idx = state.rand_mut().below(len as u64 + 1) as usize;
            token_stream.tokens_mut().splice(idx..idx, new_tokens);

            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
    }
}

fn duplicate_subslice<R: Rand>(rand: &mut R, data: &mut Vec<u8>, offset: usize) -> Result<MutationResult, Error> {
    let data_len = data.len() - offset;
    
    if data_len == 0 {
        return Ok(MutationResult::Skipped);
    }

    let slice_start = offset + rand.below(data_len as u64) as usize;
    let slice_len = 1 + rand.below((data_len - slice_start) as u64) as usize;

    let new_data = data[slice_start..slice_start + slice_len].to_vec();
    let idx = offset + rand.below(data_len as u64 + 1) as usize;
    data.splice(idx..idx, new_data);

    Ok(MutationResult::Mutated)
}

/// A mutator that duplicates a part of the value of a single, random token
pub struct TokenValueCopyMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenValueCopyMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenValueCopyMutator<P, S>
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
                    let offset = has_valid_sign(&data) as usize;
                    return duplicate_subslice(state.rand_mut(), data, offset);
                },
                TextToken::Whitespace(data) | TextToken::Text(data) | TextToken::Blob(data) => return duplicate_subslice(state.rand_mut(), data, 0),
            }
        }

        Ok(MutationResult::Skipped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libafl_bolts::{
        rands::RomuDuoJrRand,
        current_nanos
    };

    #[test]
    fn test_duplicate() {
        for _ in 0..10 {
            let mut b = b"Hello World!".to_vec();
            let mut r = RomuDuoJrRand::with_seed(current_nanos());

            duplicate_subslice(&mut r, &mut b, 0).unwrap();

            println!("{}", std::str::from_utf8(&b).unwrap());
        }
    }
}
