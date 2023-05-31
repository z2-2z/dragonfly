use std::marker::PhantomData;
use crate::{
    tt::token::{
        HasTokenStream, TextToken,
    },
    mutators::PacketMutator,
};
use libafl::prelude::{MutationResult, Error, HasRand, Rand};

fn rotate_char<R: Rand>(rand: &mut R, data: &mut Vec<u8>) -> Result<MutationResult, Error> {
    if data.is_empty() {
        return Ok(MutationResult::Skipped);
    }
    
    let n = rand.below(data.len() as u64 / 2);
    let mut changed = false;
    
    for _ in 0..n {
        let idx = rand.below(data.len() as u64) as usize;
        let byte = data[idx];
        
        if (b'0'..=b'9').contains(&byte) {
            let amount = 1 + rand.below(9) as u8;
            data[idx] = (((byte - b'0') + amount) % 10) + b'0';
            changed = true;
        } else if (b'a'..=b'z').contains(&byte)  {
            let amount = 1 + rand.below(25) as u8;
            data[idx] = (((byte - b'a') + amount) % 26) + b'a';
            changed = true;
        } else if (b'A'..=b'Z').contains(&byte)  {
            let amount = 1 + rand.below(25) as u8;
            data[idx] = (((byte - b'A') + amount) % 26) + b'A';
            changed = true;
        }
    }
    
    if changed {
        Ok(MutationResult::Mutated)
    } else {
        Ok(MutationResult::Skipped)
    }
}

/// Rotates random chars of a random token in a Caesar-cipher like manner
pub struct TokenRotateCharMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P,S)>,
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
                TextToken::Number(data) |
                TextToken::Text(data) |
                TextToken::Blob(data) |
                TextToken::Whitespace(data) => return rotate_char(state.rand_mut(), data),
            }
        }
        
        Ok(MutationResult::Skipped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libafl::prelude::{RomuDuoJrRand, current_nanos};
    
    #[test]
    fn test_rotate_char() {
        let mut r = RomuDuoJrRand::with_seed(current_nanos());
        
        for _ in 0..10 {
            let mut d = b"Hello World!".to_vec();
            rotate_char(&mut r, &mut d).unwrap();
            println!("{}", std::str::from_utf8(&d).unwrap());
        }
    }
}
