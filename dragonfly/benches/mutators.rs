use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dragonfly::{TokenStream, mutators::{
    mutate_split, mutate_swap_words,
    }
};
use libafl_bolts::prelude::StdRand;

pub fn bench_mutate_split(c: &mut Criterion) {
    let mut rand = StdRand::with_seed(1234);
    let stream = "200  fuck  my  shit  up\r\nPORT  127,,,00,,,00,,,11,,,80,,,,80\r\n12  ++  12  ==  24".parse::<TokenStream>().unwrap();
    c.bench_function("mutate_split", |b| b.iter(|| {
        let mut stream = black_box(stream.clone());
        mutate_split(&mut rand, &mut stream, 16);
    }));
}

pub fn bench_mutate_swap_words(c: &mut Criterion) {
    let mut rand = StdRand::with_seed(1234);
    let stream = "a couple of words\r\nseparated by whitespaces\r\n1 2 3 4 5 6 7 8 9 8 7 6 5 4 3 2 1 2 3 4 5 6 7 8 9\r\n".parse::<TokenStream>().unwrap();
    c.bench_function("mutate_swap_words", |b| b.iter(|| {
        let mut stream = black_box(stream.clone());
        mutate_swap_words(&mut rand, &mut stream);
    }));
}

criterion_group!(benches, bench_mutate_split, bench_mutate_swap_words);
criterion_main!(benches);
