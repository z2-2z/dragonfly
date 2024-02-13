use crate::{TokenStream, TextToken, mutators::common::copy_vec};
use libafl_bolts::prelude::Rand;
use libafl::prelude::Tokens;

pub fn mutate_swap<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let from = rand.below(stream.len() as u64) as usize;
    let to = rand.below(stream.len() as u64) as usize;
    
    if from == to {
        return false;
    }
    
    stream.tokens_mut().swap(from, to);
    
    true
}

pub fn mutate_swap_constants<R: Rand>(rand: &mut R, stream: &mut TokenStream, dict: &Tokens) -> bool {
    if stream.is_empty() || dict.is_empty() {
        return false;
    }
    
    let start = rand.below(stream.len() as u64) as usize;
    
    for token in &mut stream.tokens_mut()[start..] {
        if let TextToken::Constant(data) = token {
            let idx = rand.below(dict.len() as u64) as usize;
            copy_vec(data, &dict.tokens()[idx]);
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
    fn test_swap() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "PORT 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_swap(&mut rand, &mut stream);
            let size = stream.serialize_into_buffer(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
    
    #[test]
    fn test_swap_constants() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let mut stream = " 127,0,0,1,80,80\r\n".parse::<TokenStream>().unwrap();
        stream.tokens_mut().insert(0, TextToken::Constant(b"PORT".to_vec()));
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"X".to_vec(),
            &b"Y".to_vec(),
            &b"Z".to_vec(),
        ]);
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            mutate_swap_constants(&mut rand, &mut stream, &dict);
            let size = stream.serialize_into_buffer(&mut buffer);
            let s = std::str::from_utf8(&buffer[0..size]).unwrap();
            println!("{}", s);
        }
    }
}
