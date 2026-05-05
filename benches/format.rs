use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hanzi_sort::{Align, FormatConfig};

#[path = "common.rs"]
mod common;

// `format_items` is private to the crate; we benchmark by going through the
// public `app::render` path which formats internally. To isolate the format
// cost from sort+lookup, we pre-sort identical inputs and use a no-op pinyin
// collator (any AnyCollator will do; we just need *some* sorted strings).
//
// In practice the format_items hot path dominates only at large entry counts
// with wide formatting; at small N the sort dominates.

fn bench_format_items(c: &mut Criterion) {
    use hanzi_sort::AnyCollator;

    let mut group = c.benchmark_group("format_items");

    // Pre-sorted inputs so the benchmark is dominated by formatting work,
    // not sort. We use the fully public render path here — there is no
    // public format_items function to call directly.
    let collator = AnyCollator::pinyin();

    for &n in &[100usize, 1_000, 10_000] {
        let data = common::pairs(n);
        let sorted = collator.sort(data);
        group.throughput(Throughput::Elements(n as u64));

        // Default format config (6 columns, 4 entry width, center align).
        let default_format = FormatConfig::default();
        group.bench_function(format!("default/n={n}"), |b| {
            b.iter(|| render_via_app(&sorted, &default_format));
        });

        // Even alignment with wider entries (most expensive align mode).
        let even_format = FormatConfig {
            columns_per_row: 4,
            entry_width: 8,
            align: Align::Even,
            ..Default::default()
        };
        group.bench_function(format!("even_align/n={n}"), |b| {
            b.iter(|| render_via_app(&sorted, &even_format));
        });
    }
    group.finish();
}

fn render_via_app(sorted: &[String], format: &FormatConfig) -> String {
    use hanzi_sort::{AnyCollator, InputSource, RuntimeConfig, app};
    // We can't reuse a sorted Vec via app::render without re-sorting; instead
    // we exercise the formatter through render with already-sorted inputs.
    // The collator will re-sort, but for already-sorted inputs the work is
    // bounded by O(N) comparisons (effectively zero swaps).
    let config = RuntimeConfig::new(
        InputSource::Text(sorted.to_vec()),
        *format,
        AnyCollator::pinyin(),
    )
    .expect("benchmark runtime config should construct");
    app::render(config).expect("benchmark render should succeed")
}

criterion_group!(benches, bench_format_items);
criterion_main!(benches);
