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
    pub fn random_whitespace<R: Rand, const MIN: u64, const MAX: u64>(rand: &mut R) -> Self {
        debug_assert!(MIN <= MAX);
        
        const WHITESPACE: [u8; 6] = [b' ', b'\t', b'\n', 0x0b, 0x0c, b'\r'];
        let random_len = rand.between(MIN, MAX) as usize;
        let mut data = vec![0; random_len];
        
        for byte in &mut data {
            *byte = rand.choose(WHITESPACE);
        }
        
        TextToken::Whitespace(data)
    }
    
    #[doc(hidden)]
    pub fn random_number<R: Rand, const MAX: u64>(rand: &mut R) -> Self {
        debug_assert!(MAX >= 2);
        
        const DIGITS: [u8; 10] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
        let random_len = rand.between(2, MAX) as usize;
        let mut data = vec![0; random_len];
        
        for byte in &mut data {
            *byte = rand.choose(DIGITS);
        }
        
        match rand.below(4) {
            0 => data[0] = b'-',
            1 => data[0] = b'+',
            _ => {},
        }
        
        TextToken::Number(data)
    }
    
    #[doc(hidden)]
    pub fn random_text<R: Rand, const MIN: u64, const MAX: u64>(rand: &mut R) -> Self {
        const ALLOW_MAP: [bool; 256] = [true, true, true, true, true, true, true, true, true, false, false, false, false, false, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, false, true, true, true, true, true, true, true, true, true, true, false, true, false, true, true, false, false, false, false, false, false, false, false, false, false, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false];
        let random_len = rand.between(MIN, MAX) as usize;
        let mut data = vec![0; random_len];
        
        let num_qwords = random_len / 8;
        let ptr = unsafe { std::mem::transmute::<*mut u8, *mut u64>(data.as_mut_ptr()) };
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, num_qwords) };
        
        for qword in slice {
            *qword = rand.next() & 0x7F7F7F7F7F7F7F7Fu64;
        }
        
        for byte in &mut data[num_qwords * 8..] {
            *byte = (rand.next() as u8) & 0x7Fu8;
        }
        
        for byte in &mut data {
            if ! unsafe { *ALLOW_MAP.get_unchecked(*byte as usize) } {
                *byte = rand.between(58, 126) as u8;
            }
        }
        
        TextToken::Text(data)
    }
    
    pub(crate) fn clone_nodata(&self) -> Self {
        match self {
            TextToken::Constant(_) => TextToken::Constant(Vec::new()),
            TextToken::Number(_) => TextToken::Number(Vec::new()),
            TextToken::Whitespace(_) => TextToken::Whitespace(Vec::new()),
            TextToken::Text(_) => TextToken::Text(Vec::new()),
        }
    }
    
    #[cfg(test)]
    pub(crate) fn verify(&self) -> bool {
        match self {
            TextToken::Constant(_) => true,
            TextToken::Number(data) => {
                for (i, byte) in data.iter().enumerate() {
                    match *byte {
                        b'-' | b'+' => {
                            if i != 0 {
                                return false;
                            }
                        },
                        b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => {},
                        _ => return false,
                    }
                }
                true
            },
            TextToken::Whitespace(data) => {
                for byte in data {
                    match *byte {
                        b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r' => {},
                        _ => return false,
                    }
                }
                true
            },
            TextToken::Text(data) => {
                const BLACKLIST: [u8; 16] = [
                    // Whitespace
                    b' ', b'\t', b'\n', 0x0b, 0x0c, b'\r',
                    
                    // Number
                    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
                ];
                for byte in data {
                    if *byte >= 0x80 || BLACKLIST.contains(byte) {
                        return false;
                    }
                }
                true
            },
        }
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
    pub(crate) fn data_mut(&mut self) -> &mut Vec<u8> {
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
    
    #[inline]
    pub(crate) fn tokens_mut(&mut self) -> &mut Vec<TextToken> {
        &mut self.0
    }
    
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
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
    
    #[test]
    fn random_number() {
        let mut rand = StdRand::with_seed(current_nanos());
        
        for _ in 0..10 {
            let token = TextToken::random_number::<_, 16>(&mut rand);
            let data = std::str::from_utf8(token.data()).unwrap();
            println!("{}", data);
        }
    }
    
    #[test]
    fn random_text() {
        let mut rand = StdRand::with_seed(current_nanos());
        
        for _ in 0..10 {
            let token = TextToken::random_text::<_, 1, 16>(&mut rand);
            let data = std::str::from_utf8(token.data()).unwrap();
            println!("{}", data);
        }
    }
}
