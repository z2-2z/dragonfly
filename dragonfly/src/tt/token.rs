
const WHITESPACE: [u8; 6] = [
    b' ',
    b'\t',
    b'\n',
    0x0b,
    0x0c,
    b'\r',
];

fn is_ascii(b: u8) -> bool {
    b <= 127
}

#[derive(Debug)]
pub enum TextToken {
    Constant(Vec<u8>),
    Number(Vec<u8>),
    Whitespace(Vec<u8>),
    Text(Vec<u8>),
    Blob(Vec<u8>),
}

#[derive(Default, Debug)]
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
