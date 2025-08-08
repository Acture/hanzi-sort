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
