# pinyin-sort

A Rust CLI and library for sorting Chinese strings by Hanyu Pinyin (tone3), with deterministic tie-breaking by the original character and configurable table-style output.

## Features

- Sort Chinese strings by pinyin while preserving unknown characters in the comparison key
- Break ties by original character so same-pronunciation entries still sort deterministically
- Read input from repeated `--text` arguments or from one or more files, one non-blank line per record
- Apply character-level or phrase-level override rules from a TOML file
- Format output with configurable columns, alignment, separators, padding, and blank-line cadence
- Use the core functionality from Rust via an explicit `PinyinContext`

## Installation

### From source

Prerequisites:

- Rust toolchain (`cargo`, `rustc`)
- Python 3 and `pypinyin` if you need to regenerate `data/pinyin.csv`

Build steps:

1. Regenerate CSV data if needed:
   - `python3 scripts/convert_pinyin_to_csv.py`
2. Build the binary:
   - `cargo build --release`
3. Run it from:
   - `target/release/pinyin-sort`

### With Nix

- Enter the development shell:
  - `nix develop`
- Regenerate CSV data:
  - `just prep-data`
- Build:
  - `just build`
  - or `nix build`

## CLI usage

Basic help:

- `pinyin-sort -h`

Inputs:

- `--text` and `--file` are mutually exclusive
- `--file` reads one record per non-blank line
- directory inputs are rejected

Examples:

- Sort inline strings:
  - `pinyin-sort -t 汉字 张三 赵四`
- Sort lines from a file and print one item per line:
  - `pinyin-sort -f ./names.txt --columns 1 --entry-width 2 --blank-every 0`
- Write the result to a file:
  - `pinyin-sort -t 重庆 银行 -o ./sorted.txt`

Exit behavior:

- `0` on success
- non-zero on invalid arguments, file I/O failures, output write failures, or invalid override TOML

## CLI options

- `-f, --file <FILE>`: input file path, can be repeated
- `-t, --text <TEXT>...`: inline text input, can be repeated
- `-o, --output <PATH>`: write output to a file instead of stdout
- `-c, --config <PATH>`: TOML override file
- `--columns <N>`: number of entries per row, must be greater than `0`
- `--blank-every <N>`: insert a blank line every `N` rows; use `0` to disable
- `--entry-width <N>`: target display width for each entry, must be greater than `0`
- `--align <MODE>`: `left`, `center`, `right`, or `even`
- `--padding-char <CHAR>`: padding character; must have display width `1`
- `--separator <CHAR>`: separator between entries
- `--line-ending <CHAR>`: line ending character

## Override configuration

`--config` accepts TOML with either or both sections below.

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
- `phrase_override` must provide exactly one pinyin syllable per character
- missing sections default to empty maps

## Library usage

Core types are exposed from `src/lib.rs`.

```rust
use pinyin_sort::{format_items, sort_strings, FormatConfig, PinyinContext};

let context = PinyinContext::default();
let sorted = sort_strings(vec!["张三".into(), "赵四".into()], &context);
let output = format_items(&sorted, &FormatConfig::default());
```

Key APIs:

- `PinyinContext::pinyin_of(&str) -> Vec<PinYinRecord>`
- `PinyinContext::sort_key(&str) -> SortKey`
- `sort_strings(Vec<String>, &PinyinContext) -> Vec<String>`
- `format_items(&[impl AsRef<str>], &FormatConfig) -> String`
- `read_input_lines(&InputSource) -> Result<Vec<String>>`

## Data and build process

- `scripts/convert_pinyin_to_csv.py` converts vendored `vendor/pinyin-data/pinyin.txt` into `data/pinyin.csv`
- `build.rs` regenerates `src/generated/pinyin_map.rs` from `data/pinyin.csv`
- the build now validates that representative codepoints such as `〇`, `汉`, and `重` exist in the generated data

## Development

- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `nix develop`
- `just prep-data`
- `just build`

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
