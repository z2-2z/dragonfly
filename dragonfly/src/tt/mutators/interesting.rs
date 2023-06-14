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

const INTERESTING: [&[u8]; 34] = [
    b"0",
    b"-1",
    // 0x7F
    b"127",
    // 0x80
    b"-128",
    b"128",
    // 0xFF
    b"255",
    // 0x7FFF
    b"32767",
    // 0x8000
    b"-32768",
    b"32768",
    // 0xFFFF
    b"65535",
    // 0x7FFFFFFF
    b"2147483647",
    // 0x80000000
    b"2147483648",
    b"-2147483648",
    // 0xFFFFFFFF
    b"4294967295",
    // 0x7FFFFFFFFFFFFFFF
    b"9223372036854775807",
    // 0x8000000000000000
    b"9223372036854775808",
    b"-9223372036854775808",
    // 0xFFFFFFFFFFFFFFFF
    b"18446744073709551615",
    // powers of 2
    b"16",
    b"32",
    b"64",
    b"256",
    b"512",
    b"1024",
    b"4096",
    // random bullshit go
    b"100",
    b"-129",
    b"1000",
    b"32767",
    b"-100663046",
    b"-32769",
    b"32768",
    b"65536",
    b"100663045",
];

/// A mutator that replaces a single, random number token with an interesting value
pub struct TokenReplaceInterestingMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenReplaceInterestingMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenReplaceInterestingMutator<P, S>
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
                    if let TextToken::Number(data) = &mut token_stream.tokens_mut()[i] {
                        let idx = state.rand_mut().below(INTERESTING.len() as u64) as usize;
                        let new_value = INTERESTING[idx];
                        data.resize(new_value.len(), 0);
                        data[..].copy_from_slice(new_value);
                        return Ok(MutationResult::Mutated);
                    }
                }

                ceiling = start;
            }
        }

        Ok(MutationResult::Skipped)
    }
}

/// A mutator that insert a random number token with an interesting value
pub struct TokenStreamInsertInterestingMutator<P, S>
where
    P: HasTokenStream,
{
    max_len: usize,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenStreamInsertInterestingMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenStreamInsertInterestingMutator<P, S>
where
    P: HasTokenStream,
    S: HasRand,
{
    fn mutate_packet(&mut self, state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            let len = token_stream.tokens().len();

            if len >= self.max_len {
                return Ok(MutationResult::Skipped);
            }

            let idx = state.rand_mut().below(INTERESTING.len() as u64) as usize;
            let new_token = TextToken::Number(INTERESTING[idx].to_vec());

            let idx = state.rand_mut().below(len as u64 + 1) as usize;
            token_stream.tokens_mut().insert(idx, new_token);
            return Ok(MutationResult::Mutated);
        }

        Ok(MutationResult::Skipped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tt::TokenStream;

    #[test]
    fn test_interesting_valid() {
        for number in INTERESTING {
            let number = std::str::from_utf8(number).unwrap();
            TokenStream::builder().number(number).build();
        }
    }
}
