use crate::{TokenStream, TextToken};
use libafl_bolts::prelude::Rand;

pub fn mutate_split<R: Rand>(rand: &mut R, stream: &mut TokenStream) -> bool {
    if stream.is_empty() {
        return false;
    }
    
    let idx = rand.below(stream.len() as u64) as usize;
    let token = &mut stream.tokens_mut()[idx];
    
    if token.len() <= 1 || token.is_constant() {
        return false;
    }
    
    let pos = 1 + rand.below(token.len() as u64 - 1) as usize;
    
    if token.is_number() && pos == 1 {
        return false;
    }
    
    let mut split_elem = token.clone_nodata();
    *split_elem.data_mut() = token.data_mut().split_off(pos);
    
    let new_elem = match rand.next() % 4 {
        0 => TextToken::random_number::<_, 16>(rand),
        1 => TextToken::random_whitespace::<_, 1, 16>(rand),
        2 ..= 3 => TextToken::random_text::<_, 1, 16>(rand),
        _ => unreachable!(),
    };
    
    stream.tokens_mut().splice(idx + 1..idx + 1, [new_elem, split_elem]);
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use libafl_bolts::prelude::{StdRand, current_nanos};
    
    #[test]
    fn test_split() {
        let mut buffer = [0; 1024];
        let mut rand = StdRand::with_seed(current_nanos());
        let stream = "200 fuck my shit up".parse::<TokenStream>().unwrap();
        let mut count = 0;
        
        for _ in 0..10 {
            let mut stream = stream.clone();
            
            if mutate_split(&mut rand, &mut stream) {
                let size = stream.serialize_into_buffer(&mut buffer);
                let s = std::str::from_utf8(&buffer[0..size]).unwrap();
                println!("{}", s);
                count += 1;
            }
        }
        
        println!();
        println!("Mutated {}/10", count);
    }
}
