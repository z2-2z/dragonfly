use criterion::{criterion_group, criterion_main, Criterion};
use dragonfly::tokens::TextToken;
use libafl_bolts::prelude::StdRand;

pub fn bench_random_whitespace(c: &mut Criterion) {
    let mut rand = StdRand::with_seed(1234);
    c.bench_function("random_whitespace", |b| b.iter(|| TextToken::random_whitespace::<_, 4096, 4096>(&mut rand)));
}

pub fn bench_random_number(c: &mut Criterion) {
    let mut rand = StdRand::with_seed(1234);
    c.bench_function("random_number", |b| b.iter(|| TextToken::random_number::<_, 4096>(&mut rand)));
}

pub fn bench_random_text(c: &mut Criterion) {
    let mut rand = StdRand::with_seed(1234);
    c.bench_function("random_text", |b| b.iter(|| TextToken::random_text::<_, 4096, 4096>(&mut rand)));
}


criterion_group!(benches, bench_random_whitespace, bench_random_number, bench_random_text);
criterion_main!(benches);
