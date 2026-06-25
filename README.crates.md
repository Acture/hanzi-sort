# hanzi-sort

Sort Chinese text the way Chinese readers expect.

`hanzi-sort` is a Rust CLI and library for sorting Hanzi by Hanyu Pinyin or by stroke count, with deterministic tie-breaking, phrase-level override rules for polyphonic characters, and terminal-friendly tabular output.

> Migration note  
> `pinyin-sort` has been renamed to `hanzi-sort`. This is a hard rename: there is no compatibility binary alias.

## Why it exists

Unicode codepoint order is not Chinese sort order.

If you want useful output for names, glossaries, study lists, or publishing workflows, you usually need more than plain lexical comparison:

- pinyin order for alphabetic-style indexes
- stroke order for dictionary-style or teaching workflows
- phrase-level override rules for polyphonic characters like `重庆` or `银行`
- stable tie-breaking so the same dataset always produces the same order

## Quick examples

Install the CLI:

```bash
cargo install hanzi-sort
```

Sort by pinyin:

```bash
hanzi-sort -t 汉字 张三 赵四
```

Sort by stroke count:

```bash
hanzi-sort -t 天 一 十 --sort-by strokes --columns 1 --entry-width 2 --blank-every 0
```

Resolve a polyphonic phrase with an override file:

```bash
hanzi-sort -t 重庆 银行 --config ./override.toml
```

## Features

- Sort by `pinyin`, `strokes`, or `names` (default), or opt in to `jyutping`, `zhuyin`, `radical` via cargo features
- `--sort-by names`: surname-aware sorting for name lists (单→Shàn, 解→Xiè, compound 复姓 too)
- Read from stdin when no input flags are given (`cat names.txt | hanzi-sort`)
- Keep unknown characters in the comparison key instead of dropping them
- Break ties by original character so output stays deterministic
- Override single characters or full phrases with TOML
- Format output with configurable columns, alignment, padding, separators, and blank-line cadence
- Reuse the same core sorter from Rust via `PinyinCollator`, `StrokesCollator`, and the `Collator` trait

## Performance

`hanzi-sort` is **3.8×–4.8× faster than `icu_collator` 2.x with `zh-u-co-pinyin`**
on Chinese pinyin sort workloads (Apple Silicon, deterministic input):

| N | hanzi-sort | ICU `zh-u-co-pinyin` | speedup |
|--:|--:|--:|--:|
| 1,000 | 188 µs | 759 µs | 4.0× |
| 10,000 | 2.51 ms | 11.99 ms | 4.8× |
| 100,000 | 34.9 ms | 131.5 ms | 3.8× |

The win comes from `hanzi-sort` trading ICU's full-locale generality for a
domain-specific compact representation: every primary pinyin syllable fits
in a `u128` after byte-packed encoding, so per-character comparison is two
integer compares instead of multi-level CE table lookup. See
[`BENCHMARKS.md`](https://github.com/Acture/hanzi-sort/blob/master/BENCHMARKS.md)
for methodology, caveats, and reproduction.

## Opt-in collators

A source install defaults to pinyin + strokes + names. Add the others
explicitly, or grab a [prebuilt release binary](https://github.com/Acture/hanzi-sort/releases)
— those bundle every collator with no extra flags.

```bash
# pinyin + strokes only (default)
cargo install hanzi-sort

# add Cantonese Jyutping (Unihan kCantonese)
cargo install hanzi-sort --features collator-jyutping

# add Mandarin Zhuyin / Bopomofo
cargo install hanzi-sort --features collator-zhuyin

# add Kangxi radical sort (Unihan kRSUnicode)
cargo install hanzi-sort --features collator-radical

# everything
cargo install hanzi-sort --all-features
```

## Library usage

```rust
use hanzi_sort::{StrokesCollator, sort_strings_with};

let sorted = sort_strings_with(
    vec!["一".into(), "十".into(), "天".into()],
    &StrokesCollator,
);
```

```rust
use hanzi_sort::AnyCollator;

let sorted = AnyCollator::pinyin().sort(vec!["赵四".into(), "汉字".into()]);
```

## Learn more

- Repository: <https://github.com/Acture/hanzi-sort>
- Documentation: <https://docs.rs/hanzi-sort>
- License: <https://github.com/Acture/hanzi-sort/blob/master/LICENSE>
