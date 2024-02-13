use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dragonfly::TextToken;
use libafl_bolts::prelude::StdRand;

pub fn bench_random_whitespace(c: &mut Criterion) {
    let mut rand = StdRand::with_seed(1234);
    c.bench_function("random_whitespace", |b| b.iter(|| TextToken::random_whitespace(&mut rand, black_box(4096), black_box(4096))));
}

criterion_group!(benches, bench_random_whitespace);
criterion_main!(benches);
