use std::marker::PhantomData;
use crate::{
    tt::token::{HasTokenStream, TextToken, WHITESPACE, is_ascii},
    mutators::PacketMutator,
};
use libafl::prelude::{MutationResult, Error, HasRand, Rand};

const MAX_NUMBER_LEN: usize = 32;
const MAX_WHITESPACE_LEN: usize = 4;
const MAX_TEXT_LEN: usize = 16;
const MAX_BLOB_LEN: usize = 16;

fn random_number_value<R: Rand>(rand: &mut R, output: &mut Vec<u8>) {
    let mut text = [0u8; MAX_NUMBER_LEN];
    let mut i = MAX_NUMBER_LEN - 1;
    
    /* Convert a random number to string */
    let mut value = rand.next();
    let mut pool = rand.next();
    
    match pool % 4 {
        0 => value &= 0xFF,
        1 => value &= 0xFFFF,
        2 => value &= 0xFFFFFFFF,
        3 => value &= 0xFFFFFFFFFFFFFFFF,
        _ => unreachable!(),
    };
    pool >>= 4;
    
    while i < MAX_NUMBER_LEN {
        text[i] = (value % 10) as u8 + b'0';
        i = i.wrapping_sub(1);
        value /= 10;
        if value == 0 {
            break;
        }
    }
    
    /* Generate leading zeros */
    if i < MAX_NUMBER_LEN && (pool & 1) == 1 {
        let amount = rand.below(i as u64 + 2);
        
        for _ in 0..amount {
            text[i] = b'0';
            i = i.wrapping_sub(1);
        }
    }
    pool >>= 1;
    
    /* Generate a sign */
    if i < MAX_NUMBER_LEN {
        match pool % 4 {
            0 | 1 => {},
            2 => {
                text[i] = b'+';
                i = i.wrapping_sub(1);
            },
            3 => {
                text[i] = b'-';
                i = i.wrapping_sub(1);
            },
            _ => unreachable!()
        }
    }
    
    let new_len = MAX_NUMBER_LEN - (i + 1);
    output.resize(new_len, 0);
    output[..].copy_from_slice(&text[i + 1..MAX_NUMBER_LEN]);
}

fn random_whitespace_value<R: Rand>(rand: &mut R, output: &mut Vec<u8>) {
    let new_len = 1 + rand.below(MAX_WHITESPACE_LEN as u64) as usize;
    
    output.resize(new_len, 0);
    
    let num_bits = (WHITESPACE.len().wrapping_next_power_of_two() - 1).count_ones();
    debug_assert!(num_bits > 0);
    
    let mut pool = rand.next();
    let mut pool_size = 64;
    
    for byte in output.iter_mut() {
        if pool_size < num_bits {
            pool = rand.next();
            pool_size = 64;
        }
        
        let idx = pool as usize % WHITESPACE.len();
        *byte = WHITESPACE[idx];
        
        pool >>= num_bits;
        pool_size -= num_bits;
    }
}

fn random_text_value<R: Rand>(rand: &mut R, output: &mut Vec<u8>) {
    let new_len = 1 + rand.below(MAX_TEXT_LEN as u64) as usize;
    output.resize(new_len, 0);
    
    let mut pool = rand.next();
    let mut pool_size = 64;
    
    for byte in output.iter_mut() {
        if pool_size < 8 {
            pool = rand.next();
            pool_size = 64;
        }
        
        *byte = (pool as usize % 94) as u8 + 33;
        debug_assert!(is_ascii(*byte));
        
        pool >>= 8;
        pool_size -= 8;
    }
}

fn random_blob_value<R: Rand>(rand: &mut R, output: &mut Vec<u8>) {
    let new_len = 1 + rand.below(MAX_BLOB_LEN as u64) as usize;
    output.resize(new_len, 0);
    
    let mut pool = rand.next();
    let mut pool_size = 64;
    
    for byte in output.iter_mut() {
        if pool_size < 8 {
            pool = rand.next();
            pool_size = 64;
        }
        
        *byte = pool as u8;
        
        pool >>= 8;
        pool_size -= 8;
    }
}

/// Replaces one random TextToken in a TokenStream with a randomly
/// generated value.
pub struct RandomTokenValueMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P,S)>,
}

impl<P, S> RandomTokenValueMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for RandomTokenValueMutator<P, S>
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
                    random_number_value(state.rand_mut(), data);
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

#[cfg(test)]
mod tests {
    extern crate test;
    
    use super::*;
    use libafl::prelude::{RomuDuoJrRand, current_nanos};
    use test::{Bencher, black_box};
    
    #[test]
    fn random_number() {
        let mut rand = RomuDuoJrRand::with_seed(current_nanos());
        let mut result = Vec::with_capacity(MAX_NUMBER_LEN);
        
        for _ in 0..10 {
            random_number_value(&mut rand, &mut result);
            println!("{:?}", std::str::from_utf8(&result).unwrap());
        }
    }
    
    #[bench]
    fn bench_random_number(b: &mut Bencher) {
        let mut rand = RomuDuoJrRand::with_seed(current_nanos());
        let mut result = Vec::with_capacity(MAX_NUMBER_LEN);
        
        b.iter(|| random_number_value(
            black_box(&mut rand),
            black_box(&mut result)
        ));
    }
    
    #[test]
    fn random_whitespace() {
        let mut rand = RomuDuoJrRand::with_seed(current_nanos());
        let mut result = Vec::with_capacity(MAX_WHITESPACE_LEN);
        
        for _ in 0..10 {
            random_whitespace_value(&mut rand, &mut result);
            let _ = std::str::from_utf8(&result).unwrap();
            println!("{:?}", result);
        }
    }
    
    #[bench]
    fn bench_random_whitespace(b: &mut Bencher) {
        let mut rand = RomuDuoJrRand::with_seed(current_nanos());
        let mut result = Vec::with_capacity(MAX_WHITESPACE_LEN);
        
        b.iter(|| random_whitespace_value(
            black_box(&mut rand),
            black_box(&mut result)
        ));
    }
    
    #[test]
    fn random_text() {
        let mut rand = RomuDuoJrRand::with_seed(current_nanos());
        let mut result = Vec::with_capacity(MAX_TEXT_LEN);
        
        for _ in 0..10 {
            random_text_value(&mut rand, &mut result);
            println!("{:?}", std::str::from_utf8(&result).unwrap());
        }
    }
    
    #[test]
    fn random_blob() {
        let mut rand = RomuDuoJrRand::with_seed(current_nanos());
        let mut result = Vec::with_capacity(MAX_BLOB_LEN);
        
        for _ in 0..10 {
            random_blob_value(&mut rand, &mut result);
            println!("{:?}", result);
        }
    }
}
