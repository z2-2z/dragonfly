#![no_main]

use libfuzzer_sys::fuzz_target;
use dragonfly::tokens::TokenStream;
use libafl_bolts::prelude::StdRand;
use ahash::RandomState;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut new_data = vec![0; data.len()];
        let stream = s.parse::<TokenStream>().unwrap();
        let new_data_len = stream.serialize_into_buffer(&mut new_data);
        assert_eq!(new_data_len, new_data.len());
        assert_eq!(&new_data, data);
    }
});
