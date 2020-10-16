use criterion::{Benchmark};
use criterion::{criterion_group, criterion_main, Criterion};
use xwords::{Crossword, fill_crossword, ALL_WORDS};

pub fn criterion_benchmark(c: &mut Criterion) {

    let input = Crossword::new(String::from("         ")).unwrap();
    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_3x3_crossword",
        move |b| {
            b.iter(|| fill_crossword(&input, &ALL_WORDS));
        })
    );

    let input = Crossword::new(String::from("                ")).unwrap();

    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_4x4_crossword",
        move |b| {
            b.iter(|| fill_crossword(&input, &ALL_WORDS));
        })
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
