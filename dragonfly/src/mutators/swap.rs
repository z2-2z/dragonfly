use crate::{TokenStream, mutators::common::random_range};
use libafl_bolts::prelude::Rand;

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
}
