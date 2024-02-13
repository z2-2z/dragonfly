#![no_main]

use libfuzzer_sys::fuzz_target;
use dragonfly::{TokenStream, mutators::mutate_swap_words};
use libafl_bolts::prelude::StdRand;
use ahash::RandomState;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(mut stream) = s.parse::<TokenStream>() {
            let mut rand = StdRand::with_seed(RandomState::new().hash_one(data));
            mutate_swap_words(&mut rand, &mut stream);
        }
    }
});
