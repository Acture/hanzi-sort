//! Compare hanzi-sort's PinyinCollator against ICU (`icu_collator` crate)
//! and a naive Unicode-codepoint baseline (`Vec::sort`) on the same data.
//!
//! ## What this measures
//!
//! Three sort throughput backends, each given the same pre-cloned input:
//!
//!  - **hanzi-sort**: `sort_strings_with(input, &PinyinCollator::new())`
//!  - **ICU compare-based**: `Vec::sort_by(|a, b| icu.compare(a, b))`. This is
//!    the API ICU's docs recommend for sort use cases, since the alternative
//!    sort-key API is "many times more expensive than comparison" per ICU.
//!  - **Naive**: `Vec::sort()` (Unicode codepoint order; not pinyin order at
//!    all, but a useful lower bound on what comparison-based sort can do).
//!
//! ## What this does NOT measure
//!
//! Output identity. ICU and hanzi-sort tiebreak same-pinyin characters
//! differently (hanzi-sort by codepoint, ICU by CLDR's pinyin tailoring,
//! likely stroke count). Both are valid, just different. The benchmark is
//! about *how fast* each backend sorts, not whether they agree.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hanzi_sort::{PinyinCollator, sort_strings_with};
use icu::collator::*;
use icu::collator::options::*;
use icu::locale::locale;

#[path = "common.rs"]
mod common;

fn icu_pinyin_collator() -> CollatorBorrowed<'static> {
    let locale = locale!("zh-u-co-pinyin");
    let opts = CollatorOptions::default();
    CollatorBorrowed::try_new(locale.into(), opts)
        .expect("zh-u-co-pinyin collator should construct under compiled_data feature")
}

fn bench_pinyin_sort_hanzi(c: &mut Criterion) {
    let collator = PinyinCollator::new();
    let mut group = c.benchmark_group("pinyin_sort/hanzi-sort");

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

fn bench_pinyin_sort_icu(c: &mut Criterion) {
    let collator = icu_pinyin_collator();
    let mut group = c.benchmark_group("pinyin_sort/icu");

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
                |mut input| {
                    input.sort_by(|a, b| collator.compare(a, b));
                    input
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_pinyin_sort_naive(c: &mut Criterion) {
    // Lower bound: pure codepoint order. Not pinyin at all, but tells us
    // what `Vec::sort` can do without any per-element work.
    let mut group = c.benchmark_group("pinyin_sort/naive_codepoint");

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
                |mut input| {
                    input.sort();
                    input
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_pinyin_sort_hanzi,
    bench_pinyin_sort_icu,
    bench_pinyin_sort_naive
);
criterion_main!(benches);
