use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dragonfly::{TokenStream, mutators::mutate_split};
use libafl_bolts::prelude::StdRand;

pub fn bench_mutate_split(c: &mut Criterion) {
    let mut rand = StdRand::with_seed(1234);
    let stream = "200  fuck  my  shit  up\r\nPORT  127,,,00,,,00,,,11,,,80,,,,80\r\n12  ++  12  ==  24".parse::<TokenStream>().unwrap();
    c.bench_function("mutate_split", |b| b.iter(|| {
        let mut stream = black_box(stream.clone());
        mutate_split(&mut rand, &mut stream);
    }));
}

criterion_group!(benches, bench_mutate_split);
criterion_main!(benches);
