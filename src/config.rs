use std::path::PathBuf;

use crate::error::{PinyinSortError, Result};
use crate::format::FormatConfig;
use crate::r#override::PinyinOverride;

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
    pub format: FormatConfig,
    pub override_data: Option<PinyinOverride>,
}

impl RuntimeConfig {
    pub fn new(
        input: InputSource,
        format: FormatConfig,
        override_data: Option<PinyinOverride>,
    ) -> Result<Self> {
        if input.is_empty() {
            return Err(PinyinSortError::InvalidArgument(
                "at least one input file or text item is required".to_string(),
            ));
        }

        Ok(Self {
            input,
            format: format.validate()?,
            override_data,
        })
    }
}
