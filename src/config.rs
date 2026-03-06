use std::path::PathBuf;

use crate::error::{PinyinSortError, Result};
use crate::format::FormatConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputSource {
    Files(Vec<PathBuf>),
    Text(Vec<String>),
}

impl InputSource {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Files(paths) => paths.is_empty(),
            Self::Text(items) => items.is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub input: InputSource,
    pub output_path: Option<PathBuf>,
    pub override_path: Option<PathBuf>,
    pub format: FormatConfig,
}

impl RuntimeConfig {
    pub fn new(
        input: InputSource,
        output_path: Option<PathBuf>,
        override_path: Option<PathBuf>,
        format: FormatConfig,
    ) -> Result<Self> {
        if input.is_empty() {
            return Err(PinyinSortError::InvalidArgument(
                "at least one input file or text item is required".to_string(),
            ));
        }

        Ok(Self {
            input,
            output_path,
            override_path,
            format: format.validate()?,
        })
    }
}
