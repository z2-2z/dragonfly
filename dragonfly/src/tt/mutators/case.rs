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

fn invert_case<R: Rand>(rand: &mut R, data: &mut [u8]) -> Result<MutationResult, Error> {
    let idx = rand.below(data.len() as u64) as usize;
    
    if let Some(byte) = data.get_mut(idx) {
        if byte.is_ascii_lowercase() {
            *byte -= 32;
            return Ok(MutationResult::Mutated);
        } else if byte.is_ascii_uppercase() {
            *byte += 32;
            return Ok(MutationResult::Mutated);
        }
    }

    Ok(MutationResult::Skipped)
}

/// Inverts the case of a random subset of chars inside a single, random token
pub struct TokenInvertCaseMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenInvertCaseMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenInvertCaseMutator<P, S>
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
                TextToken::Whitespace(_) | TextToken::Blob(_) | TextToken::Constant(_) | TextToken::Number(_) => {},
                TextToken::Text(data) => return invert_case(state.rand_mut(), data),
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
    fn test_invert_case() {
        for _ in 0..10 {
            let mut b = b"Hello World!".to_vec();
            let mut r = RomuDuoJrRand::with_seed(current_nanos());

            invert_case(&mut r, &mut b).unwrap();

            println!("{}", std::str::from_utf8(&b).unwrap());
        }
    }
}
