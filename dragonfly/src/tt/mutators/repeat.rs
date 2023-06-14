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

// Be large to detect buffer overflows
const MAX_REPEAT_AMOUNT: u64 = 4096 * 2;

fn repeat_byte<R: Rand>(rand: &mut R, data: &mut Vec<u8>, data_start: usize, data_len: usize) -> Result<MutationResult, Error> {
    if data_len == 0 {
        return Ok(MutationResult::Skipped);
    }

    let idx = data_start + rand.below(data_len as u64) as usize;
    let byte = data[idx];
    let amount = 1 + rand.below(MAX_REPEAT_AMOUNT) as usize;

    let new_data = vec![byte; amount];

    data.splice(idx..idx, new_data);

    Ok(MutationResult::Mutated)
}

/// Repeat a single char in a random token
pub struct TokenRepeatCharMutator<P, S>
where
    P: HasTokenStream,
{
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenRepeatCharMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenRepeatCharMutator<P, S>
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
                    let offset = usize::from(has_valid_sign(&data));
                    return repeat_byte(state.rand_mut(), data, offset, data.len() - offset);
                },
                TextToken::Text(data) | TextToken::Blob(data) | TextToken::Whitespace(data) => return repeat_byte(state.rand_mut(), data, 0, data.len()),
            }
        }

        Ok(MutationResult::Skipped)
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use libafl::prelude::{
        current_nanos,
        RomuDuoJrRand,
    };
    use test::{
        black_box,
        Bencher,
    };

    #[test]
    fn test_repeat_char() {
        let mut r = RomuDuoJrRand::with_seed(current_nanos());

        for _ in 0..10 {
            let mut d = b"Hello World!".to_vec();
            let l = d.len();
            repeat_byte(&mut r, &mut d, 0, l).unwrap();
            println!("{}", std::str::from_utf8(&d).unwrap());
        }
    }

    #[bench]
    fn bench_repeat_char(b: &mut Bencher) {
        let mut rand = RomuDuoJrRand::with_seed(1234);

        b.iter(|| {
            let mut b = b"asdf".to_vec();

            repeat_byte(black_box(&mut rand), black_box(&mut b), black_box(0), black_box(4)).unwrap();
        });
    }
}
