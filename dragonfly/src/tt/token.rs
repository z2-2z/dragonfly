use serde::{Serialize, Deserialize};
use crate::input::SerializeIntoBuffer;

pub(crate) const MAX_NUMBER_LEN: usize = 32;
pub(crate) const MAX_WHITESPACE_LEN: usize = 4;
pub(crate) const MAX_TEXT_LEN: usize = 16;
pub(crate) const MAX_BLOB_LEN: usize = 16;

pub(crate) const WHITESPACE: [u8; 6] = [
    b' ',
    b'\t',
    b'\n',
    0x0b,
    0x0c,
    b'\r',
];

pub(crate) fn is_ascii(b: u8) -> bool {
    b <= 127
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
    pub fn constant<S: AsRef<str>>(mut self, s: S) -> Self {
        let s = s.as_ref().as_bytes();
        
        for byte in s {
            if !is_ascii(*byte) {
                panic!("Not a pure ASCII constant");
            }
        }
        
        self.tokens.push(TextToken::Constant(s.to_vec()));
        self
    }
    
    pub fn number<S: AsRef<str>>(mut self, s: S) -> Self {
        let mut s = s.as_ref();
        
        if s.starts_with('-') || s.starts_with('+') {
            s = &s[1..];
        }
        
        let _ = s.parse::<u64>().expect("Invalid decimal number");
        
        self.tokens.push(TextToken::Number(s.as_bytes().to_vec()));
        self
    }
    
    pub fn whitespace<S: AsRef<str>>(mut self, s: S) -> Self {
        let s = s.as_ref().as_bytes();
        
        for byte in s {
            if !WHITESPACE.contains(byte) {
                panic!("Invalid whitespace character: {}", byte);
            }
        }
        
        self.tokens.push(TextToken::Whitespace(s.to_vec()));
        self
    }
    
    pub fn text<S: AsRef<str>>(mut self, s: S) -> Self {
        let s = s.as_ref().as_bytes();
        
        for byte in s {
            if !is_ascii(*byte) {
                panic!("Not a pure ASCII text");
            }
        }
        
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

#[cfg(test)]
mod tests {
    use super::*;
    
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
}
