## [Unreleased]

### Added

- expose the core sorter as a library with `PinyinContext`, `sort_strings`, and `format_items`
- add CI to run `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`
- add integration coverage for CLI behavior, file input, override validation, and output writing
- add stroke-count sorting alongside pinyin sorting

### Changed

- make `--file` read one non-blank line per record and reject directory inputs
- make `--file` and `--text` mutually exclusive
- wire `-o/--output` to write to a file instead of stdout
- switch formatting width calculations to terminal display width
- correct `left` and `right` alignment semantics

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
