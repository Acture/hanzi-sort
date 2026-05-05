# Contributing to hanzi-sort

## How to add a new collator (sort scheme)

`hanzi-sort` is structured so that adding a new sort strategy
(e.g. for a new language, romanization, or ordering rule) is a
self-contained, mostly mechanical change. The shared infrastructure
already pre-stages cargo features and `AnyCollator` placeholders for
the upcoming Phase 3.1 collators (Jyutping, Zhuyin, Radical), so
those streams only need to fill in their own files.

For an entirely new collator beyond the pre-staged set, follow this
recipe in order. Numbered sections that say "(pre-staged)" are
already in place for `collator-jyutping`, `collator-zhuyin`, and
`collator-radical`; only follow them if you are adding a fully new
collator name.

### 1. Pick a name and a feature flag

Convention: lowercase, hyphenated, prefixed with `collator-`
(matching the cargo features section). Examples: `collator-pinyin`,
`collator-jyutping`, `collator-radical`.

### 2. Add a cargo feature (pre-staged for jyutping/zhuyin/radical)

In `Cargo.toml`:

```toml
[features]
collator-<name> = []
```

Decide whether to include in `default = [...]`. The current default
includes `collator-pinyin` and `collator-strokes`; new collators
should generally NOT be in default (binary size, optional data
download, etc.) until they are stable.

### 3. Create the collator module (pre-staged for jyutping/zhuyin/radical)

```
src/<name>/
  mod.rs       # exports the collator struct and tests
  key.rs       # (optional) per-character encoding helpers
  lookup.rs    # (optional) wrappers around generated PHF maps
  model.rs     # (optional) public record types if exposed
```

`mod.rs` should define `pub struct <Name>Collator` and `impl Collator
for <Name>Collator { type Data = ...; fn data_for(...) ... }`.

### 4. Wire `lib.rs` (pre-staged for jyutping/zhuyin/radical)

```rust
#[cfg(feature = "collator-<name>")]
mod <name>;

#[cfg(feature = "collator-<name>")]
pub use <name>::<Name>Collator;
```

### 5. Add an `AnyCollator` variant (pre-staged for jyutping/zhuyin/radical)

In `src/collator.rs`:

```rust
pub enum AnyCollator {
    // ... existing variants ...
    #[cfg(feature = "collator-<name>")]
    <Name>(crate::<name>::<Name>Collator),
}

impl AnyCollator {
    #[cfg(feature = "collator-<name>")]
    pub fn <name>() -> Self {
        Self::<Name>(crate::<name>::<Name>Collator::new())
    }

    pub fn sort(&self, input: Vec<String>) -> Vec<String> {
        match self {
            // ... existing arms ...
            #[cfg(feature = "collator-<name>")]
            Self::<Name>(c) => sort_strings_with(input, c),
        }
    }
}
```

### 6. Add a CLI sort mode (pre-staged for jyutping/zhuyin/radical)

In `src/args.rs`:

```rust
pub enum CliSortMode {
    Pinyin,
    Strokes,
    #[cfg(feature = "collator-<name>")]
    <Name>,
}
```

And in `build_collator`:

```rust
#[cfg(feature = "collator-<name>")]
CliSortMode::<Name> => {
    if override_data.is_some() {
        return Err(reject_override("<name>"));
    }
    Ok(AnyCollator::<name>())
}
```

### 7. Provide lookup data

Pick the simplest viable option:

a. **Bundle a CSV**: drop `data/<name>.csv` (header row + data rows
   under 100 KB) into the repository; add a `convert_<name>_to_csv.py`
   script in `scripts/` that regenerates the CSV from upstream sources;
   teach `build.rs` to emit a PHF map at build time.
b. **Derive from existing data**: if your collator can compute the key
   from another collator's data (e.g. zhuyin can be derived from pinyin
   via a small mapping table), no new CSV is needed.
c. **No data**: collators based on raw codepoint properties (e.g. plain
   Unicode order) need no PHF table.

If you bundled a CSV, in `build.rs`:

```rust
#[cfg(feature = "collator-<name>")]
fn generate_<name>_map(data_csv: &Path, out_path: &Path) {
    // ... csv read + phf_codegen build ...
    // Validate at least one representative codepoint is present
    // (defensive: catches data corruption early).
}
```

Generated PHF goes to `src/generated/<name>_map.rs`. Add it to
`src/generated/mod.rs` with the same cfg gate.

### 8. Tests

Per collator, at minimum:

- A unit test asserting the collator's `data_for(ch)` returns the
  expected key for representative characters.
- A unit test using `sort_strings_with` to verify ordering on a small
  fixture input.
- An integration test in `tests/cli.rs` invoking
  `hanzi-sort --sort-by <name> -t ...` and asserting the output (gated
  with `#[cfg(feature = "collator-<name>")]` if the test cannot run
  without the feature).
- (Optional) A criterion benchmark in `benches/sort.rs` mirroring
  `bench_pinyin_sort`.

### 9. CHANGELOG entry

Append a line under `[Unreleased]` -> `Added`.

### 10. Verify

```bash
cargo test --features collator-<name>
cargo clippy --all-targets --features collator-<name> -- -D warnings
cargo bench --features collator-<name>   # optional, if you added benches
```

If everything is green, commit on a feature branch and open a PR
(or merge directly if you have rights).

---

## Worktree-parallel collator workstream (Phase 3.1)

For the three pre-staged collators (Jyutping, Zhuyin, Radical), the
recommended workflow uses three concurrent git worktrees so that the
streams do not block each other:

```bash
git worktree add ../hanzi-sort-jyutping -b feat/collator-jyutping
git worktree add ../hanzi-sort-zhuyin   -b feat/collator-zhuyin
git worktree add ../hanzi-sort-radical  -b feat/collator-radical
```

Each worktree only needs to:

1. Replace the placeholder `JyutpingCollator` / `ZhuyinCollator` /
   `RadicalCollator` in `src/<name>/mod.rs` with a real implementation.
2. Add data files under `data/` and conversion scripts under `scripts/`.
3. Add the `generate_<name>_map(...)` function to `build.rs`.
4. Add tests.
5. Add a CHANGELOG entry.

Each worktree should NOT modify:

- `Cargo.toml` features (already declared).
- `src/lib.rs` mod / pub use lines (already gated).
- `src/collator.rs` `AnyCollator` enum or its match arms (already
  gated; the constructor is already there).
- `src/args.rs` `CliSortMode` or `build_collator` (already gated).
- Other collators' files.

This keeps merge conflicts after Phase 3.1 limited to CHANGELOG
appends, which are trivial to resolve.

---

## Style and process notes

- Follow existing `.editorconfig` rules (tab indent, width 4).
- Run `cargo clippy --all-targets --all-features -- -D warnings`
  before committing — CI will reject any warnings.
- Commit messages: imperative mood, prefixed with conventional-commit
  type (`feat`, `fix`, `refactor`, `test`, `bench`, `docs`).
- Do not add `Co-authored-by` trailers (the maintainer has opted out
  for this repository).
