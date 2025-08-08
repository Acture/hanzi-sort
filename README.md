# pinyin-sort

A small Rust CLI that sorts Chinese strings by their Hanyu Pinyin (tone3) order, with sensible tie‑breaking by the original character and flexible output formatting. It can read input from files or directly from command‑line text arguments. A simple TOML override file lets you correct or customize pinyin for specific characters or phrases.

Note
- This repository generates a large static map from codepoint to pinyin at build time from the vendored pinyin-data source.
- Pinyin syllables are normalized to tone3 style (e.g., han4, zhao4).


## Features
- Sort a list of Chinese strings by pinyin
- Deterministic tie‑breaking by original character when pinyin matches
- Accept input via files or inline text
- Highly configurable output formatting (columns, alignment, padding, separators, blank line cadence)
- Optional pinyin override file (TOML) for characters and phrases
- Reproducible development environment with Nix and Just tasks


## Installation

### From source (Cargo)
Prerequisites:
- Rust toolchain (cargo + rustc)

Steps:
1) Prepare data (convert vendored pinyin list to CSV):
   - If you do NOT use Nix:
     - Ensure you have Python 3 and the pypinyin package installed: `pip install pypinyin`
     - Run: `python3 scripts/convert_pinyin_to_csv.py`
   - If you use Nix: see the Nix section below or simply run: `just prep-data`
2) Build:
   - `cargo build --release`
3) The binary will be at:
   - `target/release/pinyin-sort`

### With Nix
This repo includes a flake and a development shell.
- Enter the dev shell (provides rustup, cargo, python, pypinyin, just):
  - `nix develop`
- Prepare data:
  - `just prep-data`
- Build (using Nix):
  - `just build`
  - or `nix build`
- Binary location when building via Cargo inside the dev shell:
  - `target/release/pinyin-sort`


## Usage
Basic help:
- `pinyin-sort -h`

Inputs are provided either as files or inline text. If neither is provided, the tool prints its help and exits.

Examples:
- Sort two inline strings and print as a table with defaults:
  - `pinyin-sort -t 汉字 张三 赵四`
- Sort lines from a file:
  - `pinyin-sort -f ./data.txt`
- Sort multiple files and override output layout:
  - `pinyin-sort -f a.txt b.txt --columns 5 --entry-width 6 --align center --separator ","`

Behavior overview:
- The program converts each string to a vector of pinyin syllables (tone3). It compares the first pinyin of each character in order. If syllables at a position are equal, it falls back to comparing the original characters so that, for example, 赵 sorts after 照 when pronunciations match. If all compared syllables match, shorter strings come first.

Exit codes:
- 0 on success
- Non‑zero on I/O or configuration parsing errors (e.g., reading files, loading override TOML)


## CLI options
These options are defined in src/args.rs and parsed via clap.

Inputs
- -f, --file <FILE>        Input file path (can be passed multiple times)
- -t, --text <TEXT>...     Inline text data (can be passed multiple times)

Output destination
- Currently outputs to stdout.
  Note: The flag -o/--output is defined in CLI args but not yet wired; output redirection via shell is recommended for now.

Pinyin overrides
- -c, --config <PATH>      TOML file with override rules (see below)

Formatting
- --columns <N>            Number of entries per row (default: 6)
- --blank-every <N>        Insert a blank line every N rows (default: 7)
- --entry-width <N>        Pad each entry to this width (default: 4)
- --align <MODE>           One of: left, center, right, even (default: center)
- --padding-char <CHAR>    Character for padding (default: space)
- --separator <CHAR>       Entry separator (default: tab)
- --line-ending <CHAR>     Line ending (default: \n)

Note: When using shell characters like tab or newline on the command line, ensure they are quoted or escaped appropriately for your shell.


## Override configuration (TOML)
You can customize pinyin for specific characters or phrases. Provide a TOML file via `--config`.

Schema (see src/override.rs):
- char_override: map from single char to a single pinyin string
- phrase_override: map from full phrase (string) to an array of pinyin strings, one per character

Example override.toml:

[char_override]
'重' = "chong2"
'行' = "xing2"

[phrase_override]
"重庆" = ["chong2", "qing4"]
"银行" = ["yin2", "hang2"]

Usage:
- `pinyin-sort -t 重庆 -t 重庆市 --config ./override.toml`

Notes:
- phrase_override takes precedence when the full input matches a phrase key.
- For characters not listed in the overrides, built‑in data is used.


## Data and build process
Generated file:
- src/generated/pinyin_map.rs is generated at build time by build.rs from data/pinyin.csv using phf (perfect hash function) for fast lookups.

Data preparation:
- The project vendors OpenChinese convert data under vendor/pinyin-data/pinyin.txt.
- scripts/convert_pinyin_to_csv.py transforms the vendored pinyin data into data/pinyin.csv and normalizes to tone3 using pypinyin.

Build steps:
1) Ensure data/pinyin.csv exists (create it via the script above).
2) Run `cargo build` (or `cargo build --release`). The build script regenerates src/generated/pinyin_map.rs when data/pinyin.csv changes.


## Programmatic use
The library code includes simple helpers:
- pinyin::pinyin_of(&str) -> Vec<PinYinRecord>
- sort::sort_by_pinyin(Vec<T: ToString>) -> Vec<T>
- format::{format, format_cell, FormatConfig}

Caveats:
- The pinyin_of function relies on generated data and optional overrides. It returns per‑character pinyin (first reading or the override for the position).


## Development
- Tests: run `cargo test`
- Dev shell (Nix): `nix develop`
- Just recipes: `just prep-data`, `just build`


## License
AGPL-3.0-only. See Cargo.toml.


## Acknowledgements
- pinyin-data (https://github.com/mozillazg/pinyin-data) for the source data under vendor/pinyin-data.
- pypinyin for tone conversion in the preprocessing script.
- phf for fast compile‑time maps.
