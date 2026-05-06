# Benchmarks

This crate ships a `criterion` benchmark suite. Run all of it with:

```bash
cargo bench --all-features
```

Or a single suite:

```bash
cargo bench --bench sort
cargo bench --bench format
cargo bench --bench icu_compare      # requires the icu dev-dep
```

For a 10× faster (slightly noisier) run during development:

```bash
cargo bench -- --quick
```

Results land in `target/criterion/` with HTML reports.

---

## hanzi-sort vs ICU vs naive `Vec::sort`

The `icu_compare` benchmark sorts the same Chinese inputs three ways:

1. **hanzi-sort** — `sort_strings_with(input, &PinyinCollator::new())`
2. **ICU** — `Vec::sort_by(|a, b| collator.compare(a, b))` with `icu_collator`
   2.x configured for the BCP47 locale `zh-u-co-pinyin` (the canonical
   Chinese pinyin tailoring). This is the API ICU recommends for sort use
   cases; the alternative sort-key API is documented as "many times more
   expensive than comparison."
3. **naive** — `Vec::sort()` on the raw `String`s. This is *Unicode
   codepoint order*, not pinyin order at all, but it's a useful lower
   bound on what comparison-based sort can achieve when each comparison
   is essentially free.

### Numbers

Apple Silicon (M-series), `cargo bench --bench icu_compare -- --quick`,
deterministic input from `benches/common.rs::pairs` / `triples_subset`:

| N | hanzi-sort | ICU `zh-u-co-pinyin` | naive `Vec::sort` | hanzi vs ICU |
|--:|--:|--:|--:|--:|
| 1,000 | 188 µs | 759 µs | 36 µs | **4.0×** faster |
| 10,000 | 2.51 ms | 11.99 ms | 532 µs | **4.8×** faster |
| 100,000 | 34.9 ms | 131.5 ms | 5.22 ms | **3.8×** faster |

Run `cargo bench --bench icu_compare` (no `--quick`) for the full criterion
report with confidence intervals.

### Why the gap

ICU's general-purpose collation is built on the Unicode Collation
Algorithm: per-character lookup into multi-level collation element tables,
weights packed into variable-length sort keys, and `memcmp`-based
comparison. It correctly handles every locale tailoring and every
character class.

`hanzi-sort` is narrower:

- Pinyin syllables draw from a tiny domain — ~1,300 distinct primary
  syllables. Every primary fits in a `u128` after the byte-packed
  encoding in `src/pinyin/key.rs`, and `u128::cmp` decides the
  per-character comparison in roughly two integer compares.
- The `Collator` trait dispatch is monomorphic per concrete collator,
  so the inner loop is just `(key, index, item)` tuple compare on
  `Vec`s.
- Build-time PHF lookup (`phf::Map`) gives O(1) per-character data
  fetch with cache-friendly layout.

In other words: `hanzi-sort` trades ICU's full-locale generality for
domain-specific compactness, and the tradeoff pays off ~4× on this
workload.

### What this does NOT prove

- Output identity. ICU's `zh-u-co-pinyin` and `hanzi-sort` tiebreak
  same-pinyin characters differently — `hanzi-sort` falls back to
  codepoint, ICU uses CLDR's pinyin tailoring secondary weights
  (likely stroke count). Both are valid Chinese sort orders for
  different audiences. The benchmark measures throughput, not
  agreement.
- Cold-start cost. ICU's `Collator` construction (compiled-data lookup)
  is amortized once outside the bench loop; for one-shot use cases the
  setup cost is also a real factor.
- Other locales. We only test pinyin sort. ICU is competitive or better
  for less-constrained scripts.

### Reproducing on other hardware

```bash
# capture a baseline once
cargo bench --bench icu_compare > baseline.txt

# after a change
cargo bench --bench icu_compare > after.txt

# diff
diff baseline.txt after.txt
```

The full criterion report (HTML + JSON) lives in
`target/criterion/pinyin_sort/` and contains regression detection.

---

## Internal benchmarks

`benches/sort.rs` covers per-collator throughput (pinyin / strokes /
plus radical / zhuyin / jyutping when their cargo features are enabled).
Baseline numbers are recorded in the commit message of `d95b4d1` (initial
benchmark harness).

`benches/format.rs` covers the formatting hot path (`format_items`) at
multiple input sizes and alignment modes. Useful when changing the
formatter's allocation pattern.
