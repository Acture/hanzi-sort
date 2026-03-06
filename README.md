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

Write the result to a file:

```bash
hanzi-sort -t 重庆 银行 -o ./sorted.txt
```

## Features

- Sort by `pinyin` or `strokes`
- Keep unknown characters in the comparison key instead of dropping them
- Break ties by original character so output stays deterministic
- Read repeated `--text` values or one non-blank record per line from `--file`
- Override single characters or full phrases with TOML
- Format output with configurable columns, alignment, padding, separators, and blank-line cadence
- Use the same core sorter from Rust via `PinyinContext` and `SortMode`

## Install and build

### From source

Prerequisites:

- Rust toolchain
- Python 3 and `pypinyin` if you need to regenerate `data/pinyin.csv`

Build:

```bash
cargo build --release
target/release/hanzi-sort -h
```

### With Nix

```bash
nix develop
just build
```

## CLI usage

Basic help:

```bash
hanzi-sort -h
```

Input rules:

- `--text` and `--file` are mutually exclusive
- `--file` reads one non-blank line per record
- directory inputs are rejected
- success exits with `0`; invalid args, bad override files, and I/O failures exit non-zero

### Sort modes

- `--sort-by pinyin`
  Default. Compares the primary tone3 pinyin for each mapped character, then falls back to the original character.
- `--sort-by strokes`
  Compares total stroke count per character, then falls back to the original character.

### CLI options

- `-f, --file <FILE>`: input file path, can be repeated
- `-t, --text <TEXT>...`: inline text input, can be repeated
- `-o, --output <PATH>`: write output to a file instead of stdout
- `-c, --config <PATH>`: TOML override file
- `--sort-by <MODE>`: `pinyin` or `strokes`
- `--columns <N>`: entries per row, must be greater than `0`
- `--blank-every <N>`: insert a blank line every `N` rows; use `0` to disable
- `--entry-width <N>`: target display width per entry, must be greater than `0`
- `--align <MODE>`: `left`, `center`, `right`, or `even`
- `--padding-char <CHAR>`: padding character, must have display width `1`
- `--separator <CHAR>`: separator between entries
- `--line-ending <CHAR>`: line ending character

## Override config

`--config` accepts TOML with either or both sections:

```toml
[char_override]
'重' = "chong2"
'行' = "xing2"

[phrase_override]
"重庆" = ["chong2", "qing4"]
"银行" = ["yin2", "hang2"]
```

Rules:

- `phrase_override` takes precedence over `char_override`
- each `phrase_override` entry must provide exactly one pinyin syllable per character
- omitted sections default to empty maps

## Library usage

The CLI is the primary product, but the sorter is available as a Rust library.

```rust
use hanzi_sort::{PinyinContext, SortMode, sort_strings_by};

let context = PinyinContext::default();
let sorted = sort_strings_by(
    vec!["一".into(), "十".into(), "天".into()],
    &context,
    SortMode::Strokes,
);
```

Key APIs:

- `PinyinContext::pinyin_of(&str) -> Vec<PinYinRecord>`
- `sort_strings(Vec<String>, &PinyinContext) -> Vec<String>`
- `sort_strings_by(Vec<String>, &PinyinContext, SortMode) -> Vec<String>`

## Data pipeline

- `scripts/convert_pinyin_to_csv.py` builds `data/pinyin.csv` from the vendored pinyin dataset
- `scripts/convert_strokes_to_csv.py` builds `data/strokes.csv` from Unicode `kTotalStrokes` data
- `build.rs` generates static lookup tables in `src/generated/`
- the build validates representative codepoints such as `〇`, `汉`, `重`, and `一`

## Development

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Nix helpers:

```bash
nix develop
just prep-data
just build
```

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
