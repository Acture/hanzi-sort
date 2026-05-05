## [Unreleased]

### Added

- expose the core sorter as a library with `PinyinContext`, `sort_strings`, and `format_items`
- `Collator` trait, `Mapped<T>`, `CharToken<T>`, `SortKey<T>`, `sort_key_of`, and `sort_strings_with` for pluggable per-character sort strategy abstraction
- `AnyCollator` enum for runtime collator selection (Pinyin / Strokes)
- `PinyinCollator` and `StrokesCollator` types implementing `Collator`
- CLI rejects `--config` together with `--sort-by strokes` (the override is pinyin-specific)
- add CI to run `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`
- add integration coverage for CLI behavior, file input, override validation, and output writing
- add stroke-count sorting alongside pinyin sorting
- `PinyinContext::with_override` constructor for fallible override loading
- `PinyinOverride::validate` now rejects empty phrase keys, empty syllables, non-ASCII syllables, and tone3 shapes outside `^[a-z]+[1-5]?$`
- `build.rs` validates that the primary pinyin syllable for every codepoint is ASCII and ≤16 bytes, so the encoded sort key path is safe by construction
- `build.rs` enforces exact column counts and verifies that the `char` column matches its codepoint
- preserve original input order for duplicate or equal-key entries via an index tiebreak in both pinyin and stroke sort
- read input from stdin when neither `--file` nor `--text` is provided and stdin is not a TTY (`cat names.txt | hanzi-sort` now works)
- accept `-f -` as an alias for reading stdin in addition to or interleaved with file inputs
- `InputSource::Stdin` variant on the public `InputSource` enum
- `criterion` benchmark suite (`cargo bench`) covering `sort_strings_with`, stroke sort, `pinyin_of`, and `format_items` at multiple input sizes
- `-r/--reverse` flag to reverse the sorted output
- `-u/--unique` flag to remove adjacent duplicates from the sorted output (like `sort -u`); ordering is `unique` then `reverse`
- `RuntimeConfig::with_unique` and `RuntimeConfig::with_reverse` builder-style setters

### Changed

- rename the project from `pinyin-sort` to `hanzi-sort`; this is a hard rename with no compatibility binary alias
- renamed `PinyinContext` to `PinyinCollator` (public API break)
- `RuntimeConfig` constructor signature changed: now takes `(input, format, collator: AnyCollator)` instead of `(input, format, override_data, sort_mode)` — the old `RuntimeConfig::new` and `RuntimeConfig::with_sort_mode` are gone
- `app::render` dispatches via `AnyCollator::sort` instead of `sort_strings_by`
- make `--file` read one non-blank line per record and reject directory inputs
- make `--file` and `--text` mutually exclusive
- wire `-o/--output` to write to a file instead of stdout
- switch formatting width calculations to terminal display width
- correct `left` and `right` alignment semantics
- `PinyinContext::new()` is now infallible and creates an empty context; pass overrides via `PinyinContext::with_override`
- `encode_primary_pinyin` returns a `Result` instead of panicking on invalid input
- `build.rs` reports row/codepoint context on failure and re-runs when the build script itself changes

### Removed

- public `SortMode` enum (replaced by `AnyCollator` variants)
- public `sort_strings` / `sort_strings_by` (use `sort_strings_with` or `AnyCollator::sort`)
- internal `EncodedSortToken` / `EncodedSortKey` / `compare_encoded_sort_key` (subsumed by the trait-based key infrastructure)

### Fixed

- preserve unknown characters in sort keys instead of dropping them
- return non-zero exits for invalid input, invalid override config, and write failures
- include the first CSV record in the generated pinyin map so `〇` resolves correctly

## [0.1.1] - 2025-08-08

### 🚀 Features

- *(project)* Initialize pinyin-sort crate
- *(sorting)* Add pinyin-based sorting functionality
- *(build)* Integrate Nix flake and sparse checkout for dependencies
- *(scripts)* Add script to convert Pinyin data to CSV
- *(build)* Generate static Pinyin map with phf
- *(build)* Replace generated module with static Pinyin map
- *(data)* Add static Pinyin CSV file to `data` directory
- *(build)* Add build command to `justfile` and update dependencies
- *(pinyin)* Refactor Pinyin handling with `derive_builder`
- *(pinyin)* Add debug output for pinyin_of function results
- *(sort)* Enhance pinyin comparison and add unit tests
- *(args)* Add command-line argument parsing with Clap
- *(args)* Enhance argument parsing with alignment options and additional parameters
- *(format)* Add formatting utilities with alignment options and tests
- *(format, args)* Integrate dynamic formatting overrides and enhance argument structure
- *(main, args)* Enhance input handling and formatting flow
- *(pinyin)* Add override support and serialization for Pinyin handling
- *(ci)* Add GitHub Actions workflow for release automation
- *(dependencies)* Update dependencies and add project metadata
- *(metadata)* Update project metadata and introduce `README.md`
- *(metadata)* Update project metadata and introduce `README.md`

### 🐛 Bug Fixes

- *(gitignore)* Remove unused data directory from ignored files
- *(gitignore)* Add `/result` directory to ignored files

### 🚜 Refactor

- *(flake.nix)* Clean up formatting and remove commented code
- *(pinyin, sort)* Clean up unused variables and update dependencies

### ⚙️ Miscellaneous Tasks

- *(release)* Bump version to 0.1.1
- *(ci)* Update release workflow binary name to `pinyin-sort`
