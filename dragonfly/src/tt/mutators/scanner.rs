use crate::{
    mutators::PacketMutator,
    tt::token::{
        has_valid_sign,
        is_decimal,
        is_whitespace,
        HasTokenStream,
        TextToken,
    },
};
use libafl::prelude::{
    Error,
    HasRand,
    MutationResult,
};
use std::marker::PhantomData;

#[derive(Debug)]
struct SplitInfo {
    pos: usize,
    len: usize,
    new_typ: fn(Vec<u8>) -> TextToken,
}

fn scan_tokens(data: &[u8]) -> Option<SplitInfo> {
    for i in 0..data.len() {
        /* check if its a whitespace */
        if is_whitespace(&data[i..i + 1]) {
            let mut j = i;

            while j < data.len() && is_whitespace(&data[j..j + 1]) {
                j += 1;
            }

            return Some(SplitInfo {
                pos: i,
                len: j - i,
                new_typ: TextToken::Whitespace,
            });
        }
        /* check if its a number */
        else if is_decimal(&data[i..i + 1]) {
            let mut pos = i;

            if i > 0 && has_valid_sign(&data[i - 1..i]) {
                pos -= 1;
            }

            let mut j = i;

            while j < data.len() && is_decimal(&data[j..j + 1]) {
                j += 1;
            }

            return Some(SplitInfo {
                pos,
                len: j - pos,
                new_typ: TextToken::Number,
            });
        }
    }

    None
}

/// Analyzes the contents of text and blob tokens to recognize numbers and whitespaces
/// and make them into their own tokens with the more correct semantic types
pub struct TokenStreamScannerMutator<P, S>
where
    P: HasTokenStream,
{
    max_len: usize,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> TokenStreamScannerMutator<P, S>
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

impl<P, S> PacketMutator<P, S> for TokenStreamScannerMutator<P, S>
where
    P: HasTokenStream,
    S: HasRand,
{
    fn mutate_packet(&mut self, _state: &mut S, packet: &mut P, _stage_idx: i32) -> Result<MutationResult, Error> {
        if let Some(token_stream) = packet.get_tokenstream() {
            let mut cursor = 0;
            let mut changed = false;

            while cursor < token_stream.tokens().len() && token_stream.tokens().len() < self.max_len {
                match &mut token_stream.tokens_mut()[cursor] {
                    TextToken::Constant(_) | TextToken::Number(_) | TextToken::Whitespace(_) => {},
                    TextToken::Text(data) => {
                        if let Some(info) = scan_tokens(data) {
                            let mut mid_data = data.split_off(info.pos);
                            let end_data = mid_data.split_off(info.len);
                            token_stream.tokens_mut().insert(cursor + 1, (info.new_typ)(mid_data));
                            token_stream.tokens_mut().insert(cursor + 2, TextToken::Text(end_data));

                            cursor += 1; // not += 2
                            changed = true;
                        }
                    },
                    TextToken::Blob(data) => {
                        if let Some(info) = scan_tokens(data) {
                            let mut mid_data = data.split_off(info.pos);
                            let end_data = mid_data.split_off(info.len);
                            token_stream.tokens_mut().insert(cursor + 1, (info.new_typ)(mid_data));
                            token_stream.tokens_mut().insert(cursor + 2, TextToken::Blob(end_data));

                            cursor += 1; // not += 2
                            changed = true;
                        }
                    },
                }

                cursor += 1;
            }

            if changed {
                return Ok(MutationResult::Mutated);
            }
        }

        Ok(MutationResult::Skipped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_tokens() {
        println!("{:?}", scan_tokens(b"AAA0A"));
        println!("{:?}", scan_tokens(b"xyz\n"));
        println!("{:?}", scan_tokens(b"asdf"));
    }
}
