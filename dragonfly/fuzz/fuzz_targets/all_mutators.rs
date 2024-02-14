#![no_main]

use libfuzzer_sys::fuzz_target;
use dragonfly::{TokenStream, mutators::*};
use libafl_bolts::prelude::{StdRand, Rand};
use libafl::prelude::Tokens;
use ahash::RandomState;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(mut stream) = s.parse::<TokenStream>() {
            let mut dict = Tokens::new();
            dict.add_tokens([
                &b"X".to_vec(),
                &b"Y".to_vec(),
                &b"Z".to_vec(),
            ]);
            let seed = RandomState::new().hash_one(data);
            let mut rand = StdRand::with_seed(seed);
            let mut count = 0;
            
            while count < 100 {
                let mutated = match rand.below(18) {
                    0 => mutate_copy(&mut rand, &mut stream),
                    1 => {
                        let other = stream.clone();
                        mutate_crossover_insert(&mut rand, &mut stream, &other)
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
                    10 => mutate_repeat_char::<_, 4096>(&mut rand, &mut stream),
                    11 => mutate_repeat_token::<_, 4096>(&mut rand, &mut stream),
                    12 => mutate_special_insert(&mut rand, &mut stream),
                    13 => mutate_special_replace(&mut rand, &mut stream),
                    14 => mutate_split(&mut rand, &mut stream),
                    15 => mutate_swap_constants(&mut rand, &mut stream, &dict),
                    16 => mutate_swap_tokens(&mut rand, &mut stream),
                    17 => mutate_swap_words(&mut rand, &mut stream),
                    _ => unreachable!(),
                };
                
                if mutated {
                    count += 1;
                }
            }
        }
    }
});
