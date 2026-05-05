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

/// Owns everything needed to drive [`crate::app::render`]: where to read
/// input, how to format the output, and which collator to sort with.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub input: InputSource,
    pub format: FormatConfig,
    pub collator: AnyCollator,
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
        })
    }
}
