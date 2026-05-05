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
- Use the same core sorter from Rust via `PinyinCollator`, `StrokesCollator`, and the generic `Collator` trait
- Opt in to additional collators (Cantonese Jyutping, Mandarin Zhuyin, Kangxi Radical) via cargo features

## Install and build

### From crates.io

```bash
# default: pinyin + strokes
cargo install hanzi-sort

# enable extra collators selectively
cargo install hanzi-sort --features collator-jyutping
cargo install hanzi-sort --features collator-zhuyin
cargo install hanzi-sort --features collator-radical

# everything on
cargo install hanzi-sort --all-features
```

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
- `--file` reads one non-blank line per record (`-f -` reads from stdin)
- when neither `--file` nor `--text` is given and stdin is piped, hanzi-sort reads stdin (`cat names.txt | hanzi-sort` works)
- directory inputs are rejected
- success exits with `0`; invalid args, bad override files, and I/O failures exit non-zero

### Sort modes

Always available:

- `--sort-by pinyin`
  Default. Compares the primary tone3 pinyin for each mapped character, then falls back to the original character.
- `--sort-by strokes`
  Compares total stroke count per character, then falls back to the original character.

Opt-in via cargo features (see Install above):

- `--sort-by jyutping` (`--features collator-jyutping`)
  Compares the primary Cantonese Jyutping reading per character (Unihan `kCantonese`).
- `--sort-by zhuyin` (`--features collator-zhuyin`)
  Compares the Mandarin Zhuyin / Bopomofo reading per character (derived from the bundled pinyin data).
- `--sort-by radical` (`--features collator-radical`)
  Compares the Kangxi radical index plus residual stroke count per character (Unihan `kRSUnicode`).

### CLI options

- `-f, --file <FILE>`: input file path, can be repeated; `-` reads stdin
- `-t, --text <TEXT>...`: inline text input, can be repeated
- `-o, --output <PATH>`: write output to a file instead of stdout
- `-c, --config <PATH>`: TOML override file (pinyin only)
- `-r, --reverse`: reverse the sorted output
- `-u, --unique`: remove adjacent duplicates after sorting (like `sort -u`)
- `--sort-by <MODE>`: see "Sort modes" above
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

Key APIs:

- `PinyinCollator::pinyin_of(&str) -> Vec<PinYinRecord>`
- `PinyinCollator::with_override(PinyinOverride) -> Result<PinyinCollator>`
- `StrokesCollator` (zero-sized, just construct it)
- `sort_strings_with<C: Collator>(Vec<String>, &C) -> Vec<String>`
- `AnyCollator::pinyin() / strokes() / pinyin_with_override(...)` and `AnyCollator::sort(...)`

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
