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

fn rotate_char<R: Rand>(rand: &mut R, data: &mut Vec<u8>) -> Result<MutationResult, Error> {
    let idx = rand.below(data.len() as u64) as usize;

    if let Some(byte) = data.get_mut(idx) {
        if byte.is_ascii_digit() {
            let amount = 1 + rand.below(9) as u8;
            *byte = (((*byte - b'0') + amount) % 10) + b'0';
            return Ok(MutationResult::Mutated);
        } else if byte.is_ascii_lowercase() {
            let amount = 1 + rand.below(25) as u8;
            *byte = (((*byte - b'a') + amount) % 26) + b'a';
            return Ok(MutationResult::Mutated);
        } else if byte.is_ascii_uppercase() {
            let amount = 1 + rand.below(25) as u8;
            *byte = (((*byte - b'A') + amount) % 26) + b'A';
            return Ok(MutationResult::Mutated);
        }
    }

    Ok(MutationResult::Skipped)
}

/// Rotates random chars of a random token in a Caesar-cipher like manner
pub struct TokenRotateCharMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenRotateCharMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenRotateCharMutator<P, S>
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
                TextToken::Number(data) | TextToken::Text(data) | TextToken::Blob(data) | TextToken::Whitespace(data) => return rotate_char(state.rand_mut(), data),
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
    fn test_rotate_char() {
        let mut r = RomuDuoJrRand::with_seed(current_nanos());
        let mut d = b"Hello World!".to_vec();

        for _ in 0..10 {
            rotate_char(&mut r, &mut d).unwrap();
        }
        
        println!("{}", std::str::from_utf8(&d).unwrap());
    }
}
