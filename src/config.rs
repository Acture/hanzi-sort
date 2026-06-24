use std::path::PathBuf;

use crate::collator::AnyCollator;
use crate::error::{HanziSortError, Result};
use crate::format::FormatConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputSource {
    Files(Vec<PathBuf>),
    Text(Vec<String>),
    Stdin,
}

impl InputSource {
    /// Returns `true` if the source is statically known to carry no input
    /// (empty `--file` or `--text` list). [`InputSource::Stdin`] is always
    /// considered non-empty at construction time; emptiness is determined
    /// by [`crate::input::read_input_lines`] when reading.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Files(paths) => paths.is_empty(),
            Self::Text(items) => items.is_empty(),
            Self::Stdin => false,
        }
    }
}

/// How to treat the leading lines of each file / stdin source before sorting.
///
/// `lines` is the per-source header height (`0` disables header handling).
/// When `keep` is `true` the header lines are pinned, unsorted, to the top of
/// the output; when `false` they are dropped entirely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeaderSpec {
    pub lines: usize,
    pub keep: bool,
}

/// Owns everything needed to drive [`crate::app::render`]: where to read
/// input, how to format the output, and which collator to sort with.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub input: InputSource,
    pub format: FormatConfig,
    pub collator: AnyCollator,
    /// When `true`, remove adjacent duplicates after sorting (like `sort -u`).
    /// Because the sort is stable and total, adjacent dedup is equivalent to
    /// full dedup.
    pub unique: bool,
    /// When `true`, reverse the sorted (and possibly de-duplicated) output.
    pub reverse: bool,
    /// Per-source header handling applied before sorting.
    pub header: HeaderSpec,
}

impl RuntimeConfig {
    pub fn new(input: InputSource, format: FormatConfig, collator: AnyCollator) -> Result<Self> {
        if input.is_empty() {
            return Err(HanziSortError::InvalidArgument(
                "at least one input file or text item is required".to_string(),
            ));
        }

        Ok(Self {
            input,
            format: format.validate()?,
            collator,
            unique: false,
            reverse: false,
            header: HeaderSpec::default(),
        })
    }

    /// Builder-style setter for the `unique` flag.
    pub fn with_unique(mut self, unique: bool) -> Self {
        self.unique = unique;
        self
    }

    /// Builder-style setter for the `reverse` flag.
    pub fn with_reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    /// Builder-style setter for the header-handling spec.
    pub fn with_header(mut self, header: HeaderSpec) -> Self {
        self.header = header;
        self
    }
}
