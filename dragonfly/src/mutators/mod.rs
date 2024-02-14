mod split;
mod crossover;
mod delete;
mod common;
mod copy;
mod swap;
mod repeat;
mod random;
mod interesting;
mod special;
mod dict;
mod flip;
mod truncate;

pub use split::*;
pub use crossover::*;
 pub use delete::*;
pub use copy::*;
pub use swap::*;
pub use repeat::*;
pub use random::*;
pub use interesting::*;
pub use special::*;
pub use dict::*;
pub use flip::*;
pub use truncate::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TokenStream;
    use libafl::prelude::Tokens;
    use libafl_bolts::prelude::{StdRand, current_nanos, Rand};
    
    #[test]
    fn fuzz_mutators() {
        let stream = "200 fuck my shit up\r\nPORT 127,0,0,1,80,80\r\n12 + 12 = 24".parse::<TokenStream>().unwrap();
        let mut dict = Tokens::new();
        dict.add_tokens([
            &b"X".to_vec(),
            &b"Y".to_vec(),
            &b"Z".to_vec(),
        ]);
        let mut rand = StdRand::with_seed(current_nanos());
        let mut count = 0;
        const MAX_LEN: usize = 16;
        
        loop {
            count += 1;
            println!("iter {}", count);
            
            let mut stream = stream.clone();
            
            for _ in 0..1000 {
                let mutation = rand.below(19);
                
                let mutated = match mutation {
                    0 => mutate_copy(&mut rand, &mut stream, MAX_LEN),
                    1 => {
                        let other = stream.clone();
                        mutate_crossover_insert(&mut rand, &mut stream, &other, MAX_LEN)
                    },
                    2 => {
                        let other = stream.clone();
                        mutate_crossover_replace(&mut rand, &mut stream, &other)
                    },
                    3 => mutate_delete(&mut rand, &mut stream),
                    4 => mutate_dict_insert(&mut rand, &mut stream, &dict),
                    5 => mutate_dict_replace(&mut rand, &mut stream, &dict),
                    6 => mutate_flip(&mut rand, &mut stream),
                    7 => mutate_interesting(&mut rand, &mut stream),
                    8 => mutate_random_insert(&mut rand, &mut stream),
                    9 => mutate_random_replace(&mut rand, &mut stream),
                    10 => mutate_repeat_char::<_, 8>(&mut rand, &mut stream),
                    11 => mutate_repeat_token::<_, 8>(&mut rand, &mut stream),
                    12 => mutate_special_insert(&mut rand, &mut stream),
                    13 => mutate_special_replace(&mut rand, &mut stream),
                    14 => mutate_split(&mut rand, &mut stream),
                    15 => mutate_swap_constants(&mut rand, &mut stream, &dict),
                    16 => mutate_swap_tokens(&mut rand, &mut stream),
                    17 => mutate_swap_words(&mut rand, &mut stream),
                    18 => mutate_truncate(&mut rand, &mut stream),
                    _ => unreachable!(),
                };
                
                if mutated {
                    for token in stream.tokens() {
                        if !token.verify() {
                            panic!("Mutation #{} produced invalid token: {:?} (full stream: {:?})", mutation, token, stream);
                        }
                    }
                }
            }
        }
    }
}
