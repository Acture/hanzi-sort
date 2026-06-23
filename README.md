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

---

# Usage

## Install

### From a prebuilt binary (recommended)

Every [GitHub release](https://github.com/Acture/hanzi-sort/releases) ships a
self-contained binary for Linux, macOS, and Windows (x86-64 and arm64). The
binary embeds all character data and **all five sort schemes** (pinyin,
strokes, jyutping, zhuyin, radical) — no runtime files and no extra install
step.

```bash
# example: Apple Silicon macOS
curl -fsSL https://github.com/Acture/hanzi-sort/releases/latest/download/hanzi-sort-aarch64-apple-darwin.tar.gz | tar xz
./hanzi-sort -t 汉字 张三 赵四
```

Pick the asset matching your platform:

| OS | x86-64 | arm64 |
|----|--------|-------|
| Linux | `…-x86_64-unknown-linux-gnu.tar.gz` | `…-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | `…-x86_64-apple-darwin.tar.gz` | `…-aarch64-apple-darwin.tar.gz` |
| Windows | `…-x86_64-pc-windows-msvc.zip` | `…-aarch64-pc-windows-msvc.zip` |

Each archive has a matching `.sha256` for integrity verification.

### From crates.io

A source install defaults to pinyin + strokes; add the opt-in collators
explicitly (the prebuilt binaries above already bundle all of them).

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

## Quick start

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

Read a file, sort by strokes into a grid, and write the result:

```bash
hanzi-sort -f names.txt -o sorted.txt --sort-by strokes --columns 3 --entry-width 6 --align left
```

Sort piped input (works the way Unix users expect):

```bash
cat names.txt | hanzi-sort
```

Run `hanzi-sort --help` for the full option list and a built-in examples block.

## Features

- Sort by `pinyin` or `strokes`
- Keep unknown characters in the comparison key instead of dropping them
- Break ties by original character so output stays deterministic
- Read repeated `--text` values or one non-blank record per line from `--file`
- Override single characters or full phrases with TOML
- Format output with configurable columns, alignment, padding, separators, and blank-line cadence
- Use the same core sorter from Rust via `PinyinCollator`, `StrokesCollator`, and the generic `Collator` trait
- Opt in to additional collators (Cantonese Jyutping, Mandarin Zhuyin, Kangxi Radical) via cargo features (already bundled in the prebuilt binaries)

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

Bundled in the prebuilt binaries, opt-in for source installs via cargo features (see Install above):

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
- `-c, --config <PATH>`: TOML override file (pinyin or jyutping)
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
- each `phrase_override` entry must provide exactly one syllable per character
- omitted sections default to empty maps
- override files apply to `--sort-by pinyin` (tone digits `1-5`) and `--sort-by jyutping` (tone digits `1-6`)

## Use as a library

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
- `sort_indices_with<C: Collator>(&[String], &C) -> Vec<usize>` for the sort permutation
- `AnyCollator::pinyin() / strokes() / pinyin_with_override(...)` and `AnyCollator::sort(...)`

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
[`BENCHMARKS.md`](BENCHMARKS.md) for methodology, caveats (output identity
is not preserved across the two collators), and reproduction steps.

---

# Development

## Build from source

Prerequisites:

- Rust toolchain
- Python 3 and `pypinyin` if you need to regenerate `data/pinyin.csv`

Build:

```bash
cargo build --release
target/release/hanzi-sort -h
```

## With Nix

```bash
nix develop
just build
```

## Tests and lints

```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Nix helpers:

```bash
nix develop
just prep-data
just build
```

## Data pipeline

- `scripts/convert_pinyin_to_csv.py` builds `data/pinyin.csv` from the vendored pinyin dataset
- `scripts/convert_strokes_to_csv.py` builds `data/strokes.csv` from Unicode `kTotalStrokes` data
- `build.rs` generates static lookup tables in `src/generated/`
- the build validates representative codepoints such as `〇`, `汉`, `重`, and `一`

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
