use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::error::{PinyinSortError, Result};

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct PinyinOverride {
    #[serde(default)]
    pub char_override: HashMap<char, String>,
    #[serde(default)]
    pub phrase_override: HashMap<String, Vec<String>>,
}

impl PinyinOverride {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let file_content = std::fs::read_to_string(path).map_err(|source| {
            PinyinSortError::io(
                format!("failed to read override config {}", path.display()),
                source,
            )
        })?;
        let overrides: Self =
            toml::from_str(&file_content).map_err(|source| PinyinSortError::OverrideParse {
                path: path.to_path_buf(),
                source,
            })?;
        overrides.validate()?;
        Ok(overrides)
    }

    pub fn validate(&self) -> Result<()> {
        for (phrase, pinyins) in &self.phrase_override {
            let char_count = phrase.chars().count();
            if char_count != pinyins.len() {
                return Err(PinyinSortError::InvalidOverride(format!(
                    "phrase_override entry '{}' has {} characters but {} pinyin values",
                    phrase,
                    char_count,
                    pinyins.len()
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_override_defaults_missing_sections() {
        let overrides: PinyinOverride = toml::from_str("[char_override]\n'重' = 'chong2'\n")
            .expect("override TOML should parse");
        assert_eq!(
            overrides.char_override.get(&'重'),
            Some(&"chong2".to_string())
        );
        assert!(overrides.phrase_override.is_empty());
    }

    #[test]
    fn test_override_rejects_phrase_length_mismatch() {
        let overrides: PinyinOverride =
            toml::from_str("[phrase_override]\n\"重庆\" = [\"chong2\"]\n")
                .expect("override TOML should parse");
        let error = overrides
            .validate()
            .expect_err("length mismatch should fail");
        assert_eq!(
            error.to_string(),
            "phrase_override entry '重庆' has 2 characters but 1 pinyin values"
        );
    }
}
