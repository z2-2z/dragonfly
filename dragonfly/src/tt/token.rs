
const WHITESPACE: [u8; 6] = [
    b' ',
    b'\t',
    b'\n',
    0x0b,
    0x0c,
    b'\r',
];

#[derive(Debug)]
pub enum TextToken {
    Constant(String),
    Number(bool, u64),
    Whitespace(Vec<u8>),
    Text(String),
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
    
    pub fn tokens(&self) -> &[TextToken] {
        &self.tokens
    }
}

pub struct TokenStreamBuilder {
    tokens: Vec<TextToken>,
}

impl TokenStreamBuilder {
    pub fn constant<S: Into<String>>(mut self, c: S) -> Self {
        self.tokens.push(TextToken::Constant(c.into()));
        self
    }
    
    pub fn number<S: AsRef<str>>(mut self, s: S) -> Self {
        let mut s = s.as_ref();
        let mut negative = false;
        
        if s.starts_with('-') {
            negative = true;
            s = &s[1..];
        } else if s.starts_with('+') {
            s = &s[1..];
        }
        
        let digits = s.parse::<u64>().expect("Invalid decimal number");
        
        self.tokens.push(TextToken::Number(negative, digits));
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
    
    pub fn text<S: Into<String>>(mut self, s: S) -> Self {
        self.tokens.push(TextToken::Text(s.into()));
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
}
