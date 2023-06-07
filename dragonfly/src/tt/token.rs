use serde::{Serialize, Deserialize};
use crate::{
    input::SerializeIntoBuffer,
    mutators::{NewRandom, HasCrossover},
};
use libafl::prelude::{Rand, HasRand, HasMetadata, Tokens};
use std::fmt::Display;

pub(crate) const MAX_NUMBER_LEN: usize = 32;
pub(crate) const MAX_WHITESPACE_LEN: usize = 4;
pub(crate) const MAX_TEXT_LEN: usize = 16;
pub(crate) const MAX_BLOB_LEN: usize = 16;

pub(crate) fn random_number_value<R: Rand>(rand: &mut R, output: &mut Vec<u8>, gen_sign: bool) {
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
    if gen_sign && i < MAX_NUMBER_LEN {
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
    
    let start_text = i.wrapping_add(1);
    let new_len = MAX_NUMBER_LEN - start_text;
    output.resize(new_len, 0);
    output[..].copy_from_slice(&text[start_text..MAX_NUMBER_LEN]);
    debug_assert!(is_number(output));
}

pub(crate) fn random_whitespace_value<R: Rand>(rand: &mut R, output: &mut Vec<u8>) {
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
    
    debug_assert!(is_whitespace(output));
}

pub(crate) fn random_text_value<R: Rand>(rand: &mut R, output: &mut Vec<u8>) {
    let new_len = 1 + rand.below(MAX_TEXT_LEN as u64) as usize;
    output.resize(new_len, 0);
    
    let mut pool = rand.next();
    let mut pool_size = 64;
    
    for byte in output.iter_mut() {
        if pool_size < 8 {
            pool = rand.next();
            pool_size = 64;
        }
        
        *byte = (pool as usize % 128) as u8;
        debug_assert!(is_ascii(*byte));
        
        pool >>= 8;
        pool_size -= 8;
    }
    
    debug_assert!(is_text(output));
}

pub(crate) fn random_blob_value<R: Rand>(rand: &mut R, output: &mut Vec<u8>) {
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

pub(crate) fn has_valid_sign<S: AsRef<[u8]>>(s: S) -> bool {
    let s = s.as_ref();
    matches!(s.first(), Some(b'+') | Some(b'-'))
}

pub(crate) fn is_decimal<S: AsRef<[u8]>>(s: S) -> bool {
    let s = s.as_ref();
    
    for byte in s {
        if !(b'0'..=b'9').contains(byte) {
            return false;
        }
    }
    
    !s.is_empty()
}

pub(crate) fn is_number<S: AsRef<[u8]>>(s: S) -> bool { 
    let s = s.as_ref();
    let mut i = 0;
    
    if has_valid_sign(s) {
        i = 1;
    }
    
    is_decimal(&s[i..])
}

pub(crate) fn is_ascii(b: u8) -> bool {
    b <= 127
}

pub(crate) fn is_text<S: AsRef<[u8]>>(s: S) -> bool {
    let s = s.as_ref();
    let mut result = true;
    
    for byte in s {
        result &= is_ascii(*byte);
    }
    
    result
}

const WHITESPACE: [u8; 6] = [
    b' ',
    b'\t',
    b'\n',
    0x0b,
    0x0c,
    b'\r',
];

pub(crate) fn is_whitespace<S: AsRef<[u8]>>(s: S) -> bool {
    let s = s.as_ref();
    let mut result = true;
    
    for byte in s {
        result &= WHITESPACE.contains(byte);
    }
    
    result
}

#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub enum TextToken {
    Constant(Vec<u8>),
    Number(Vec<u8>),
    Whitespace(Vec<u8>),
    Text(Vec<u8>),
    Blob(Vec<u8>),
}

impl TextToken {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    pub fn len(&self) -> usize {
        match self {
            TextToken::Constant(data) |
            TextToken::Number(data) |
            TextToken::Whitespace(data) |
            TextToken::Text(data) |
            TextToken::Blob(data) => data.len(),
        }
    }
}

impl Display for TextToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextToken::Constant(data) => {
                if is_text(data) {
                    write!(f, "Constant(\"{}\")", std::str::from_utf8(data).unwrap())
                } else {
                    write!(f, "Constant({:?})", data)
                }
            },
            TextToken::Number(data) => write!(f, "Number(\"{}\")", std::str::from_utf8(data).unwrap()),
            TextToken::Whitespace(data) => write!(f, "Whitespace({:?})", data),
            TextToken::Text(data) => write!(f, "Text(\"{:?}\")", std::str::from_utf8(data).unwrap()),
            TextToken::Blob(data) => write!(f, "Blob({:?})", data),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct TokenStream {
    tokens: Vec<TextToken>,
}

impl TokenStream {
    pub fn builder() -> TokenStreamBuilder {
        TokenStreamBuilder {
            tokens: Vec::new(),
        }
    }
    
    pub fn generate_text(&self, buf: &mut [u8]) -> usize {
        let mut written = 0;
        
        for token in &self.tokens {
            if written >= buf.len() {
                break;
            }
            
            match token {
                TextToken::Number(data) |
                TextToken::Constant(data) |
                TextToken::Whitespace(data) |
                TextToken::Text(data) |
                TextToken::Blob(data) => {
                    let rem_buf = &mut buf[written..];
                    let size = std::cmp::min(rem_buf.len(), data.len());
                    rem_buf[..size].copy_from_slice(&data[..size]);
                    written += size;
                },
            }
        }
        
        written
    }
    
    pub fn tokens(&self) -> &[TextToken] {
        &self.tokens
    }
    
    pub fn tokens_mut(&mut self) -> &mut Vec<TextToken> {
        &mut self.tokens
    }
}

pub struct TokenStreamBuilder {
    tokens: Vec<TextToken>,
}

impl TokenStreamBuilder {
    pub fn constant<S: AsRef<[u8]>>(mut self, s: S) -> Self {
        self.tokens.push(TextToken::Constant(s.as_ref().to_vec()));
        self
    }
    
    pub fn number<S: AsRef<[u8]>>(mut self, s: S) -> Self {
        let s = s.as_ref();
        assert!(is_number(s));
        self.tokens.push(TextToken::Number(s.to_vec()));
        self
    }
    
    pub fn whitespace<S: AsRef<[u8]>>(mut self, s: S) -> Self {
        let s = s.as_ref();
        assert!(is_whitespace(s));
        self.tokens.push(TextToken::Whitespace(s.to_vec()));
        self
    }
    
    pub fn text<S: AsRef<[u8]>>(mut self, s: S) -> Self {
        let s = s.as_ref();
        assert!(is_text(s));
        self.tokens.push(TextToken::Text(s.to_vec()));
        self
    }
    
    pub fn blob<B: AsRef<[u8]>>(mut self, b: B) -> Self {
        self.tokens.push(TextToken::Blob(b.as_ref().to_vec()));
        self
    }
    
    pub fn build(self) -> TokenStream {
        TokenStream {
            tokens: self.tokens,
        }
    }
}

pub trait HasTokenStream {
    fn get_tokenstream(&mut self) -> Option<&mut TokenStream>;
}

impl HasTokenStream for TokenStream {
    fn get_tokenstream(&mut self) -> Option<&mut TokenStream> {
        Some(self)
    }
}

impl SerializeIntoBuffer for TokenStream {
    fn serialize_into_buffer(&self, buffer: &mut [u8]) -> Option<usize> {
        Some(self.generate_text(buffer))
    }

    fn get_connection(&self) -> usize {
        0
    }
}

impl<S> NewRandom<S> for TokenStream
where
    S: HasRand + HasMetadata,
{
    fn new_random(state: &mut S) -> Self {
        let mut builder = TokenStream::builder();
        let n = state.rand_mut().below(16);
        let (dict_size, choices) = if let Some(dict) = state.metadata_map().get::<Tokens>() {
            (dict.len(), 2)
        } else {
            (0, 1)
        };
        
        for _ in 0..n {
            let use_dict = state.rand_mut().below(choices);
            
            if use_dict == 1 {
                let idx = state.rand_mut().below(dict_size as u64) as usize;
                let dict = state.metadata_map().get::<Tokens>().unwrap();
                builder = builder.constant(&dict.tokens()[idx]);
            } else {
                let mut new_data = Vec::new();
                let typ = state.rand_mut().below(4);
                
                match typ {
                    0 => {
                        random_number_value(state.rand_mut(), &mut new_data, true);
                        builder = builder.number(new_data);
                    },
                    1 => {
                        random_whitespace_value(state.rand_mut(), &mut new_data);
                        builder = builder.whitespace(new_data);
                    },
                    2 => {
                        random_text_value(state.rand_mut(), &mut new_data);
                        builder = builder.text(new_data);
                    },
                    3 => {
                        random_blob_value(state.rand_mut(), &mut new_data);
                        builder = builder.blob(new_data);
                    },
                    _ => unreachable!()
                }
            }
        }
        
        builder.build()
    }
}

impl Display for TokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.tokens.len();
        
        if len > 0 {
            for i in 0..len - 1 {
                write!(f, "{}, ", self.tokens[i])?;
            }
            
            write!(f, "{}", self.tokens[len - 1])?;
        }
        
        Ok(())
    }
}

impl<S> HasCrossover<S> for TokenStream
where
    S: HasRand,
{
    fn crossover_insert(&mut self, state: &mut S, mut other: Self) {
        if other.tokens().is_empty() {
            return;
        }
        
        let other_start = state.rand_mut().below(other.tokens().len() as u64) as usize;
        let other_len = 1 + state.rand_mut().below(other.tokens().len() as u64 - other_start as u64) as usize;
        
        let self_start = state.rand_mut().below(self.tokens.len() as u64 + 1) as usize;
        
        self.tokens.splice(self_start..self_start, other.tokens.drain(other_start..other_start + other_len));
    }

    fn crossover_replace(&mut self, state: &mut S, mut other: Self) {
        if other.tokens.is_empty() {
            return;
        }
        
        let other_start = state.rand_mut().below(other.tokens().len() as u64) as usize;
        let other_len = 1 + state.rand_mut().below(other.tokens().len() as u64 - other_start as u64) as usize;
        
        let self_start = state.rand_mut().below(self.tokens.len() as u64) as usize;
        let self_len  = state.rand_mut().below(self.tokens.len() as u64 - self_start as u64 + 1) as usize;
        
        self.tokens.splice(self_start..self_start + self_len, other.tokens.drain(other_start..other_start + other_len));
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    
    use super::*;
    use libafl::prelude::{RomuDuoJrRand, current_nanos};
    use test::{Bencher, black_box};
    
    #[test]
    fn api_test() {
        let stream = TokenStream::builder()
            .constant("asdf")
            .number("-1234")
            .whitespace(" ")
            .text("hello world")
            .blob(b"binaryyy")
            .build();
        
        println!("{:?}", stream);
    }
    
    #[test]
    #[should_panic]
    fn api_test_non_ascii() {
        TokenStream::builder().constant("ö");
    }
    
    #[test]
    fn generate_text() {
        let stream = TokenStream::builder()
            .text("hello")
            .whitespace(" ")
            .text("world")
            .build();
        
        let mut buf = vec![0; 256];
        
        assert_eq!(
            stream.generate_text(&mut buf),
            11
        );
        assert_eq!(
            std::str::from_utf8(&buf[..11]),
            Ok("hello world")
        );
        
        assert_eq!(
            stream.generate_text(&mut buf[..6]),
            6
        );
    }
    
    #[test]
    fn random_number() {
        let mut rand = RomuDuoJrRand::with_seed(current_nanos());
        let mut result = Vec::with_capacity(MAX_NUMBER_LEN);
        
        for _ in 0..10 {
            random_number_value(&mut rand, &mut result, true);
            println!("{:?}", std::str::from_utf8(&result).unwrap());
        }
    }
    
    #[bench]
    fn bench_random_number(b: &mut Bencher) {
        let mut rand = RomuDuoJrRand::with_seed(current_nanos());
        let mut result = Vec::with_capacity(MAX_NUMBER_LEN);
        
        b.iter(|| random_number_value(
            black_box(&mut rand),
            black_box(&mut result),
            true
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
    
    struct TestState {
        rand: RomuDuoJrRand,
    }
    
    impl TestState {
        fn new() -> Self {
            Self {
                rand: RomuDuoJrRand::with_seed(1234),
            }
        }
    }
    
    impl HasRand for TestState {
        type Rand = RomuDuoJrRand;

        fn rand(&self) -> &Self::Rand {
            &self.rand
        }

        fn rand_mut(&mut self) -> &mut Self::Rand {
            &mut self.rand
        }
    }
    
    #[test]
    fn test_crossover() {
        let mut state = TestState::new();
        let mut ts = TokenStream::builder().blob("").text("").build();
        
        loop {
            ts.crossover_replace(
                &mut state, 
                ts.clone()
            );
        }
    }
}
