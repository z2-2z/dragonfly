use serde::{Serialize, Deserialize};
use std::str::FromStr;
use libafl_bolts::prelude::Rand;

#[derive(Clone, Serialize, Deserialize, Debug, Hash)]
pub enum TextToken {
    Constant(Vec<u8>),
    Number(Vec<u8>),
    Whitespace(Vec<u8>),
    Text(Vec<u8>),
}

impl TextToken {
    fn try_parse_whitespace(data: &[u8]) -> Option<Self> {
        let mut len = 0;
        
        for byte in data {
            if matches!(*byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r') {
                len += 1;
            } else {
                break;
            }
        }
        
        if len == 0 {
            None
        } else {
            Some(TextToken::Whitespace(data[0..len].to_vec()))
        }
    }
    
    fn try_parse_number(data: &[u8]) -> Option<Self> {
        let mut sign = 0;
        let mut len = 0;
        
        if matches!(data.first(), Some(b'+') | Some(b'-')) {
            sign = 1;
        }
        
        for byte in &data[sign..] {
            if byte.is_ascii_digit() {
                len += 1;
            } else {
                break;
            }
        }
        
        if len == 0 {
            None
        } else {
            Some(TextToken::Number(data[0..sign + len].to_vec()))
        }
    }
    
    fn try_parse_text(data: &[u8]) -> Option<Self> {
        const BLACKLIST: [u8; 18] = [
            // Whitespace
            b' ', b'\t', b'\n', 0x0b, 0x0c, b'\r',
            
            // Number
            b'+', b'-', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
        ];
        let mut len = 0;
        
        for byte in data {
            if *byte >= 0x80 || (BLACKLIST.contains(byte) && len > 0) {
                break;
            } else {
                len += 1;
            }
        }
        
        if len == 0 {
            None
        } else {
            Some(TextToken::Text(data[0..len].to_vec()))
        }
    }
    
    #[doc(hidden)]
    pub fn random_whitespace<R: Rand>(rand: &mut R, min_len: usize, max_len: usize) -> Self {
        const WHITESPACE: [u8; 6] = [b' ', b'\t', b'\n', 0x0b, 0x0c, b'\r',];
        debug_assert!(min_len <= max_len);
        let random_len = rand.between(min_len as u64, max_len as u64) as usize;
        let mut data = vec![0; random_len];
        
        for byte in &mut data {
            let idx = rand.next() as usize;
            *byte = WHITESPACE[idx % 6];
        }
        
        TextToken::Whitespace(data)
    }
}

impl TextToken {
    #[inline]
    pub fn data(&self) -> &[u8] {
        match self {
            TextToken::Constant(data) |
            TextToken::Number(data) |
            TextToken::Whitespace(data) |
            TextToken::Text(data) => data,
        }
    }
    
    #[inline]
    pub fn is_constant(&self) -> bool {
        matches!(self, TextToken::Constant(_))
    }
    
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, TextToken::Number(_))
    }
    
    #[inline]
    pub fn is_whitespace(&self) -> bool {
        matches!(self, TextToken::Whitespace(_))
    }
    
    #[inline]
    pub fn is_text(&self) -> bool {
        matches!(self, TextToken::Text(_))
    }
    
    pub fn len(&self) -> usize {
        self.data().len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Hash)]
pub struct TokenStream(Vec<TextToken>);

impl FromStr for TokenStream {
    type Err = u8;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.as_bytes();
        let mut stream = Vec::new();
        let mut cursor = 0;
        
        while cursor < s.len() {
            if let Some(token) = TextToken::try_parse_whitespace(&s[cursor..]) {
                cursor += token.len();
                stream.push(token);
            } else if let Some(token) = TextToken::try_parse_number(&s[cursor..]) {
                cursor += token.len();
                stream.push(token);
            } else if let Some(token) = TextToken::try_parse_text(&s[cursor..]) {
                cursor += token.len();
                stream.push(token);
            } else {
                return Err(s[cursor]);
            }
        }
        
        Ok(TokenStream(stream))
    }
}

impl TokenStream {
    #[inline]
    pub fn tokens(&self) -> &[TextToken] {
        &self.0
    }
    
    pub fn serialize_into_buffer(&self, buffer: &mut [u8]) -> usize {
        let mut cursor = 0;
        
        for token in &self.0 {
            let data = token.data();
            let rem_len = std::cmp::min(buffer.len() - cursor, data.len());
            buffer[cursor..cursor + rem_len].copy_from_slice(&data[..rem_len]);
            cursor += rem_len;
        }
        
        cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_roundtrip(s: &str) {
        let mut data = [0u8; 32];
        let stream = s.parse::<TokenStream>().unwrap();
        println!("{:?}", stream);
        let len = stream.serialize_into_buffer(&mut data);
        let s1 = std::str::from_utf8(&data[0..len]).unwrap();
        assert_eq!(s, s1);
    }
    
    #[test]
    fn test() {
        test_roundtrip("");
        test_roundtrip("200 fuck my shit up\r\n");
        test_roundtrip("PORT 127,0,0,1,80,80\r\n");
        test_roundtrip("12 + 12 = 24");
    }
}
