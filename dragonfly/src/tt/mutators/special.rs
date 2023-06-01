use std::marker::PhantomData;
use crate::{
    tt::token::{
        HasTokenStream, TextToken, 
    },
    mutators::PacketMutator,
};
use libafl::prelude::{MutationResult, Error, HasRand, Rand};

const TEXT_SPECIAL: [u8; 33] = [
    0,
    b'!',
    b'"',
    b'#',
    b'$',
    b'%',
    b'&',
    b'\'',
    b'(',
    b')',
    b'*',
    b'+',
    b',',
    b'-',
    b'.',
    b'/',
    b':',
    b';',
    b'<',
    b'=',
    b'>',
    b'?',
    b'@',
    b'\\',
    b'[',
    b']',
    b'^',
    b'_',
    b'`',
    b'{',
    b'|',
    b'}',
    127,
];
const NUMBER_SPECIAL: [u8; 2] = [
    b'+',
    b'-',
];

fn replace_special<R: Rand>(rand: &mut R, data: &mut [u8], data_len: usize, charset: &[u8]) {
    let n = std::cmp::min(
        rand.below(data_len as u64 / 2),
        1
    );
    
    for _ in 0..n {
        let idx = rand.below(charset.len() as u64) as usize;
        let byte = charset[idx];
        
        let idx = rand.below(data_len as u64) as usize;
        data[idx] = byte;
    }
}

fn insert_special<R: Rand>(rand: &mut R, data: &mut Vec<u8>, data_len: usize, charset: &[u8]) {
    let n = std::cmp::min(
        rand.below(data_len as u64 / 2),
        1
    );
    
    for _ in 0..n {
        let idx = rand.below(charset.len() as u64) as usize;
        let byte = charset[idx];
        
        let idx = rand.below(data_len as u64 + 1) as usize;
        data.insert(idx, byte);
    }
}

/// Replaces chars in a single, random text token with special chars
pub struct TokenReplaceSpecialCharMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P,S)>,
}

impl<P, S> TokenReplaceSpecialCharMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenReplaceSpecialCharMutator<P, S>
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
                TextToken::Whitespace(_) |
                TextToken::Blob(_) |
                TextToken::Constant(_) => {},
                TextToken::Number(data) => {
                    if !data.is_empty() && !matches!(data.first(), Some(b'+') | Some(b'-')) {
                        replace_special(state.rand_mut(), data, 1, &NUMBER_SPECIAL);
                        return Ok(MutationResult::Mutated);
                    }
                },
                TextToken::Text(data) => {
                    let len = data.len();
                    if len > 0 {
                        replace_special(state.rand_mut(), data, len, &TEXT_SPECIAL);
                        return Ok(MutationResult::Mutated);
                    }
                },
            }
        }
        
        Ok(MutationResult::Skipped)
    }
}

/// Inserts special chars into a single, random text token
pub struct TokenInsertSpecialCharMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P,S)>,
}

impl<P, S> TokenInsertSpecialCharMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenInsertSpecialCharMutator<P, S>
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
                TextToken::Whitespace(_) |
                TextToken::Constant(_) => {},
                TextToken::Number(data) => {
                    if !data.is_empty() && !matches!(data.first(), Some(b'+') | Some(b'-')) {
                        insert_special(state.rand_mut(), data, 1, &NUMBER_SPECIAL);
                        return Ok(MutationResult::Mutated);
                    }
                },
                TextToken::Blob(data) |
                TextToken::Text(data) => {
                    let len = data.len();
                    insert_special(state.rand_mut(), data, len, &TEXT_SPECIAL);
                    return Ok(MutationResult::Mutated);
                },
            }
        }
        
        Ok(MutationResult::Skipped)
    }
}
