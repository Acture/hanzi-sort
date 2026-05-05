## [Unreleased]

## [0.2.1] - 2026-05-05

A small follow-up to 0.2.0 that finishes the deferred Jyutping API
parity work, hardens CI, and exposes a sort entry point that lets
library users verify the crate's stable-sort guarantee directly.

### Added

- `JyutpingOverride` struct (gated by `collator-jyutping`) with the same
  TOML schema as `PinyinOverride` but tone digits in `1..=6`. Loadable
  via `JyutpingOverride::load_from_file`.
- `JyutpingCollator::with_override(JyutpingOverride) -> Result<Self>`
  for building an override-aware collator.
- `JyutpingCollator::jyutping_of(&str)` returns readings honoring any
  configured phrase or per-character overrides.
- `AnyCollator::jyutping_with_override` constructor.
- CLI accepts `--config <path>` together with `--sort-by jyutping`; the
  file is parsed as a `JyutpingOverride` (tone 1-6) when sort_by is
  Jyutping and as a `PinyinOverride` (tone 1-5) for the default mode.
- `sort_indices_with<C>(&[String], &C) -> Vec<usize>` exposes the sort
  permutation, which makes the index-tiebreak stability guarantee
  directly verifiable from outside the crate.
- Cargo.toml `rust-version = "1.85"` (MSRV declared, matching edition
  2024 requirement).
- `.gitattributes` enforcing LF line endings on text files (CSVs,
  Python scripts, generated PHF files), so Windows clones with default
  autocrlf don't corrupt build-time inputs.

### Changed

- CI now runs on a `ubuntu-latest`/`macos-latest`/`windows-latest`
  matrix with `fail-fast: false`. Previously only Linux was tested,
  so Windows path / line-ending / `IsTerminal` behavior had no
  coverage.
- The error message for `--config` paired with an unsupported
  `--sort-by` is now `"--config is not supported with --sort-by
  <scheme>"` (was previously phrased as "only supported with
  --sort-by pinyin", which is now incorrect since jyutping also
  accepts overrides).
- Internal `validate_syllable` in `src/override.rs` is parameterized
  over the valid tone range so `PinyinOverride` and `JyutpingOverride`
  share the syllable-shape check.

### Fixed

- Stability of equal-key inputs is now provable. Previous tests on
  duplicate strings could not actually verify that the unstable
  backend was promoted to stable behavior — the rubber-duck Phase 1
  review flagged this as a smoke-level test. With `sort_indices_with`
  exposed, proptest verifies the stronger property:
  *for any input, equal-sort-key items preserve their input-order
  relative position*.

## [0.2.0] - 2026-05-05

A major release. Project rebrand from `pinyin-sort` → `hanzi-sort`, the public
library API was reshaped around a pluggable `Collator` trait, three new
opt-in collators (Cantonese Jyutping, Mandarin Zhuyin, Kangxi Radical) joined
Pinyin and Strokes, and the CLI grew Unix-friendly stdin / `-r` / `-u`
behavior plus shell completions. Tone3 data normalization now treats
neutral tone as `5`, fixing a long-standing dictionary-order bug for
characters like `了`. See breakdown below.

### Highlights

- **5 sort schemes**: Pinyin, Strokes, Jyutping, Zhuyin, Radical (all but
  Pinyin/Strokes are opt-in `cargo features`).
- **Stable sort guarantee**: equal-key inputs preserve input order in every
  collator.
- **Stdin-friendly CLI**: `cat names.txt | hanzi-sort` works the way Unix
  users expect.
- **Pluggable collator API**: the trait is exposed; downstream Rust code can
  add its own collators without forking the crate.

### Added

#### Library

- `Collator` trait, `Mapped<T>`, `CharToken<T>`, `SortKey<T>`, `sort_key_of`,
  and `sort_strings_with` — a pluggable per-character sort strategy
  abstraction.
- `AnyCollator` enum for runtime collator selection across all enabled
  schemes.
- `PinyinCollator` (renamed from `PinyinContext`), `StrokesCollator`,
  `JyutpingCollator`, `ZhuyinCollator`, `RadicalCollator`.
- `PinyinCollator::with_override` (and the analogous fallible builder for
  any future override-aware collators) for explicit override loading.
- `RuntimeConfig::with_unique` and `RuntimeConfig::with_reverse`
  builder-style setters.
- `InputSource::Stdin` variant on the public `InputSource` enum.

#### CLI

- `-r/--reverse` flag.
- `-u/--unique` flag (adjacent dedup; `unique` is applied before `reverse`).
- `hanzi-sort completions <shell>` subcommand emitting a completion script
  for bash / zsh / fish / powershell / elvish (via `clap_complete`).
- `--help` examples block (after the options list).
- `--help` displays default values for every printable option.
- Stdin fallback when neither `--file` nor `--text` is provided and stdin
  is non-TTY; `-f -` accepted as a stdin alias.
- CLI rejects `--config` together with any non-pinyin `--sort-by` (override
  is pinyin-specific).

#### New collators (opt-in)

- `--sort-by jyutping` / `--features collator-jyutping` — Cantonese Jyutping
  from Unihan `kCantonese` (~30k characters covered).
- `--sort-by zhuyin` / `--features collator-zhuyin` — Mandarin Zhuyin /
  Bopomofo derived from the bundled pinyin data.
- `--sort-by radical` / `--features collator-radical` — Kangxi radical
  index + residual stroke count from Unihan `kRSUnicode`.

#### Build / testing / docs

- `criterion` benchmark suite (`cargo bench`) covering every collator's
  sort path plus `pinyin_of` and `format_items` at 1k / 10k / 100k inputs.
- `proptest`-based property tests verifying: encoded sort key preserves
  byte-wise lex order, unchecked vs checked encoders agree on valid input,
  sort is idempotent, sort is a permutation, sort key induces a total order.
- `build.rs` validates exact column counts, codepoint↔char correspondence,
  primary syllable ASCII / `≤16` bytes / mandatory tone digit, on every
  generated map.
- `CONTRIBUTING.md` with a step-by-step recipe for adding a new collator
  and a worktree-parallel workflow note.
- CI now runs `cargo test --all-features` in addition to default features
  and verifies all benchmarks compile under all features.
- `PinyinOverride::validate` rejects empty phrase keys, empty syllables,
  non-ASCII syllables, and tone3 shapes outside `^[a-z]+[1-5]$`.
- Integration coverage for stdin behavior, `-r/-u` composition, completions
  output, override correctness, and per-collator CLI invocation.

### Changed

#### Breaking (library)

- Renamed crate / binary from `pinyin-sort` to `hanzi-sort`. No
  compatibility alias.
- Renamed `PinyinContext` to `PinyinCollator`.
- Renamed `PinyinSortError` to `HanziSortError`.
- `RuntimeConfig::new` signature is now `(input, format, collator: AnyCollator)`
  instead of `(input, format, override_data, sort_mode)`. The old
  `RuntimeConfig::new` and `RuntimeConfig::with_sort_mode` are gone.
- `app::render` dispatches via `AnyCollator::sort` instead of `sort_strings_by`.
- `PinyinCollator::new()` is now infallible (no override); use
  `PinyinCollator::with_override(PinyinOverride)` for override-aware
  construction.
- `encode_primary_pinyin` returns a `Result` instead of panicking on
  invalid input.

#### Breaking (data semantics)

- Toneless primary syllables in `data/pinyin.csv` are now normalized to
  neutral tone `5` (e.g. `了 → le5` instead of `了 → le`). Override
  validation and build-time checks both enforce that every syllable ends
  in a tone digit `1-5`, so neutral-tone characters now sort *after*
  tone-4 variants instead of *before* tone-1 variants — matching
  conventional Chinese dictionary ordering. Override files using toneless
  syllables (e.g. `'了' = 'le'`) must update to `'le5'`.

#### Behavior (non-breaking)

- `--file` / `--text` are mutually exclusive at parse time; `--file` reads
  one non-blank line per record and rejects directory inputs.
- `-o/--output` writes to a file instead of stdout (instead of being
  silently ignored as in `0.1.1`).
- Formatting width calculations use terminal display width
  (`unicode-width`), and `left` / `right` alignment semantics are
  corrected.
- `build.rs` reports row / codepoint context on failure and re-runs when
  the build script itself changes.

### Removed

- public `SortMode` enum (replaced by `AnyCollator` variants).
- public `sort_strings` / `sort_strings_by` (use `sort_strings_with` or
  `AnyCollator::sort`).
- internal `EncodedSortToken` / `EncodedSortKey` / `compare_encoded_sort_key`
  (subsumed by the trait-based key infrastructure).

### Fixed

- preserve unknown characters in sort keys instead of dropping them.
- return non-zero exits for invalid input, invalid override config, and
  write failures.
- include the first CSV record in the generated pinyin map so `〇`
  resolves correctly.
- preserve original input order for duplicate or equal-key entries via an
  index tiebreak in both pinyin and stroke sort.

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
