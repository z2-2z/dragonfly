use crate::{TokenStream, TextToken};
use libafl_bolts::prelude::Rand;
use libafl::prelude::Tokens;

pub fn mutate_dict_insert<R: Rand>(rand: &mut R, stream: &mut TokenStream, dict: &Tokens) -> bool {
    if dict.is_empty() {
        return false;
    }
    
    let idx = rand.below(stream.len() as u64 + 1) as usize;
    let item = rand.below(dict.len() as u64) as usize;
    let new_elem = dict.tokens()[item].to_owned();
    
    if (rand.next() & 1) == 1 {
        stream.tokens_mut().insert(idx, TextToken::Constant(new_elem));
    } else {
        let new_elems = [
            TextToken::random_whitespace::<_, 1, 1>(rand),
            TextToken::Constant(new_elem),
            TextToken::random_whitespace::<_, 1, 1>(rand),
        ];
        stream.tokens_mut().splice(idx..idx, new_elems);
    }
    
    true
}

pub fn mutate_dict_replace<R: Rand>(rand: &mut R, stream: &mut TokenStream, dict: &Tokens) -> bool {
    if dict.is_empty() || stream.is_empty() {
        return false;
    }
    
    let start = rand.below(stream.len() as u64) as usize;
    let item = rand.below(dict.len() as u64) as usize;
    let new_elem = dict.tokens()[item].to_owned();
    
    for token in &mut stream.tokens_mut()[start..] {
        if !token.is_whitespace() {
            *token = TextToken::Constant(new_elem);
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_dict_insert() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"X".to_vec(),
            &b"Y".to_vec(),
            &b"Z".to_vec(),
        ]);
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_dict_insert(&mut rand, &mut stream, &dict);
            let size = stream.serialize_into_buffer(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_dict_replace() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"X".to_vec(),
            &b"Y".to_vec(),
            &b"Z".to_vec(),
        ]);
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_dict_replace(&mut rand, &mut stream, &dict);
            let size = stream.serialize_into_buffer(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
