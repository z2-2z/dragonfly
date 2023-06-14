use crate::{
    mutators::PacketMutator,
    tt::token::{
        has_valid_sign,
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

/// A mutator that duplicates a single, random token
pub struct TokenStreamDuplicateMutator<P, S>
where
    P: HasTokenStream,
{
    max_len: usize,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenStreamDuplicateMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenStreamDuplicateMutator<P, S>
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
            token_stream.tokens_mut().insert(idx + 1, new_token);
            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
    }
}

fn duplicate_subslice<R: Rand>(rand: &mut R, data: &mut Vec<u8>, data_start: usize, data_len: usize) -> Result<MutationResult, Error> {
    if data_len == 0 {
        return Ok(MutationResult::Skipped);
    }

    let start = data_start + rand.below(data_len as u64) as usize;
    let len = rand.below((data_len - start) as u64) as usize;

    if len == 0 {
        return Ok(MutationResult::Skipped);
    }

    let new_data = data[start..start + len].to_vec();
    let idx = data_start + rand.below(data_len as u64) as usize;
    data.splice(idx..idx, new_data);

    Ok(MutationResult::Mutated)
}

/// A mutator that duplicates a part of the value of a single, random token
pub struct TokenValueDuplicateMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenValueDuplicateMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenValueDuplicateMutator<P, S>
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
                    let mut start = 0;
                    let mut len = data.len();

                    if has_valid_sign(&data) {
                        start += 1;
                        len -= 1;
                    }

                    return duplicate_subslice(state.rand_mut(), data, start, len);
                },
                TextToken::Whitespace(data) | TextToken::Text(data) | TextToken::Blob(data) => return duplicate_subslice(state.rand_mut(), data, 0, data.len()),
            }
        }

        Ok(MutationResult::Skipped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libafl::prelude::{
        current_nanos,
        RomuDuoJrRand,
    };

    #[test]
    fn test_duplicate() {
        for _ in 0..10 {
            let mut b = b"Hello World!".to_vec();
            let l = b.len();
            let mut r = RomuDuoJrRand::with_seed(current_nanos());

            duplicate_subslice(&mut r, &mut b, 0, l).unwrap();

            println!("{}", std::str::from_utf8(&b).unwrap());
        }
    }
}
