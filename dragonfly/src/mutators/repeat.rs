use crate::TokenStream;
use libafl_bolts::prelude::Rand;

pub fn mutate_repeat_token<R: Rand, const AMNT: usize>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let idx = rand.below(stream.len() as u64) as usize;
    let elem = stream.tokens()[idx].clone();
    
    if elem.is_empty() {
        return false;
    }
    
    let n = 1 + (AMNT / elem.len());
    
    stream.tokens_mut().splice(idx..idx, vec![elem; n]);
    
    true
}

pub fn mutate_repeat_char<R: Rand, const AMNT: usize>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let idx = rand.below(stream.len() as u64) as usize;
    let elem = &mut stream.tokens_mut()[idx];
    
    if elem.is_constant() {
        return false;
    }
    
    let elem_len = elem.len();
    
    if elem_len == 0 || elem_len >= AMNT {
        return false;
    }
    
    let n = AMNT - elem_len;
    let idx = rand.below(elem_len as u64) as usize;
    
    if idx == 0 && elem.is_number() {
        return false;
    }
    
    let c = elem.data()[idx];
    elem.data_mut().splice(idx..idx, vec![c; n]);
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_repeat_token() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_repeat_token::<_, 16>(&mut rand, &mut stream);
            let size = stream.serialize_into_buffer(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_repeat_char() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_repeat_char::<_, 16>(&mut rand, &mut stream);
            let size = stream.serialize_into_buffer(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
