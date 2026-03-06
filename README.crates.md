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

- Sort by `pinyin` or `strokes`
- Keep unknown characters in the comparison key instead of dropping them
- Break ties by original character so output stays deterministic
- Override single characters or full phrases with TOML
- Format output with configurable columns, alignment, padding, separators, and blank-line cadence
- Reuse the same core sorter from Rust via `PinyinContext` and `SortMode`

## Library usage

```rust
use hanzi_sort::{sort_strings_by, PinyinContext, SortMode};

let context = PinyinContext::default();
let sorted = sort_strings_by(
    vec!["一".into(), "十".into(), "天".into()],
    &context,
    SortMode::Strokes,
);

assert_eq!(sorted, vec!["一", "十", "天"]);
```

## Learn more

- Repository: <https://github.com/Acture/hanzi-sort>
- Documentation: <https://docs.rs/hanzi-sort>
- License: <https://github.com/Acture/hanzi-sort/blob/master/LICENSE>
