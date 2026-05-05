use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hanzi_sort::{PinyinCollator, StrokesCollator, sort_strings_with};
#[cfg(feature = "collator-radical")]
use hanzi_sort::RadicalCollator;
#[cfg(feature = "collator-zhuyin")]
use hanzi_sort::ZhuyinCollator;

#[path = "common.rs"]
mod common;

fn bench_pinyin_sort(c: &mut Criterion) {
    let collator = PinyinCollator::new();
    let mut group = c.benchmark_group("pinyin_sort");

    for &n in &[1_000usize, 10_000, 100_000] {
        let data = if n <= 10_000 {
            common::pairs(n)
        } else {
            common::triples_subset(n)
        };
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(format!("n={n}"), |b| {
            b.iter_batched(
                || data.clone(),
                |input| sort_strings_with(input, &collator),
                criterion::BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_strokes_sort(c: &mut Criterion) {
    let collator = StrokesCollator;
    let mut group = c.benchmark_group("strokes_sort");

    for &n in &[1_000usize, 10_000, 100_000] {
        let data = if n <= 10_000 {
            common::pairs(n)
        } else {
            common::triples_subset(n)
        };
        group.throughput(Throughput::Elements(n as u64));
        group.bench_function(format!("n={n}"), |b| {
            b.iter_batched(
                || data.clone(),
                |input| sort_strings_with(input, &collator),
                criterion::BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_radical_sort(c: &mut Criterion) {
    #[cfg(feature = "collator-radical")]
    {
        let collator = RadicalCollator;
        let mut group = c.benchmark_group("radical_sort");

        for &n in &[1_000usize, 10_000, 100_000] {
            let data = if n <= 10_000 {
                common::pairs(n)
            } else {
                common::triples_subset(n)
            };
            group.throughput(Throughput::Elements(n as u64));
            group.bench_function(format!("n={n}"), |b| {
                b.iter_batched(
                    || data.clone(),
                    |input| sort_strings_with(input, &collator),
                    criterion::BatchSize::LargeInput,
                );
            });
        }
        group.finish();
    }
    #[cfg(not(feature = "collator-radical"))]
    let _ = c;
}

fn bench_zhuyin_sort(c: &mut Criterion) {
    #[cfg(feature = "collator-zhuyin")]
    {
        let collator = ZhuyinCollator::new();
        let mut group = c.benchmark_group("zhuyin_sort");

        for &n in &[1_000usize, 10_000, 100_000] {
            let data = if n <= 10_000 {
                common::pairs(n)
            } else {
                common::triples_subset(n)
            };
            group.throughput(Throughput::Elements(n as u64));
            group.bench_function(format!("n={n}"), |b| {
                b.iter_batched(
                    || data.clone(),
                    |input| sort_strings_with(input, &collator),
                    criterion::BatchSize::LargeInput,
                );
            });
        }
        group.finish();
    }
    #[cfg(not(feature = "collator-zhuyin"))]
    let _ = c;
}

fn bench_pinyin_of_lookup(c: &mut Criterion) {
    let collator = PinyinCollator::new();
    let inputs = common::pairs(1_000);
    c.bench_function("pinyin_of_per_string_pair", |b| {
        b.iter(|| {
            for s in &inputs {
                let _ = collator.pinyin_of(s);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_pinyin_sort,
    bench_strokes_sort,
    bench_radical_sort,
    bench_zhuyin_sort,
    bench_pinyin_of_lookup
);
criterion_main!(benches);
