use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::error::{HanziSortError, Result};

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
            HanziSortError::io(
                format!("failed to read override config {}", path.display()),
                source,
            )
        })?;
        let overrides: Self =
            toml::from_str(&file_content).map_err(|source| HanziSortError::OverrideParse {
                path: path.to_path_buf(),
                source,
            })?;
        overrides.validate()?;
        Ok(overrides)
    }

    pub fn validate(&self) -> Result<()> {
        for (character, syllable) in &self.char_override {
            validate_syllable(syllable).map_err(|reason| {
                HanziSortError::InvalidOverride(format!(
                    "char_override entry '{character}' has invalid pinyin '{syllable}': {reason}"
                ))
            })?;
        }

        for (phrase, pinyins) in &self.phrase_override {
            if phrase.is_empty() {
                return Err(HanziSortError::InvalidOverride(
                    "phrase_override key cannot be empty".to_string(),
                ));
            }
            let char_count = phrase.chars().count();
            if char_count != pinyins.len() {
                return Err(HanziSortError::InvalidOverride(format!(
                    "phrase_override entry '{}' has {} characters but {} pinyin values",
                    phrase,
                    char_count,
                    pinyins.len()
                )));
            }
            for syllable in pinyins {
                validate_syllable(syllable).map_err(|reason| {
                    HanziSortError::InvalidOverride(format!(
                        "phrase_override entry '{phrase}' has invalid pinyin '{syllable}': {reason}"
                    ))
                })?;
            }
        }

        Ok(())
    }
}

/// Validate a single tone3 pinyin syllable.
///
/// Accepts lowercase ASCII letters optionally followed by a single tone digit
/// in `1..=5`. The data pipeline normalizes `ü` to `v`, so non-ASCII input is
/// rejected here as well to keep the byte-wise sort key encoding monotonic.
fn validate_syllable(syllable: &str) -> std::result::Result<(), &'static str> {
    if syllable.is_empty() {
        return Err("syllable is empty");
    }
    if !syllable.is_ascii() {
        return Err("syllable must be ASCII (use 'v' instead of 'ü')");
    }

    let bytes = syllable.as_bytes();
    let letters = match bytes.last() {
        Some(b'1'..=b'5') => &bytes[..bytes.len() - 1],
        _ => bytes,
    };

    if letters.is_empty() {
        return Err("syllable must contain at least one letter");
    }
    if !letters.iter().all(|b| b.is_ascii_lowercase()) {
        return Err("syllable letters must be lowercase ASCII");
    }
    Ok(())
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

    #[test]
    fn test_override_rejects_empty_syllable_in_phrase() {
        let overrides: PinyinOverride =
            toml::from_str("[phrase_override]\n\"重庆\" = [\"\", \"qing4\"]\n")
                .expect("override TOML should parse");
        let error = overrides
            .validate()
            .expect_err("empty syllable should fail");
        assert!(
            error.to_string().contains("syllable is empty"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_override_rejects_non_ascii_syllable() {
        let overrides: PinyinOverride = toml::from_str("[char_override]\n'女' = 'nü3'\n")
            .expect("override TOML should parse");
        let error = overrides
            .validate()
            .expect_err("non-ASCII syllable should fail");
        assert!(error.to_string().contains("ASCII"), "unexpected: {error}");
    }

    #[test]
    fn test_override_rejects_invalid_tone() {
        let overrides: PinyinOverride = toml::from_str("[char_override]\n'重' = 'chong9'\n")
            .expect("override TOML should parse");
        let error = overrides
            .validate()
            .expect_err("tone 9 should fail because '9' is not stripped and 'chong9' has digit in letters position");
        assert!(error.to_string().contains("lowercase ASCII"));
    }

    #[test]
    fn test_override_rejects_uppercase_syllable() {
        let overrides: PinyinOverride = toml::from_str("[char_override]\n'重' = 'Chong2'\n")
            .expect("override TOML should parse");
        let error = overrides
            .validate()
            .expect_err("uppercase letter should fail");
        assert!(error.to_string().contains("lowercase ASCII"));
    }

    #[test]
    fn test_override_rejects_empty_phrase_key() {
        // TOML allows empty string keys; build the value programmatically.
        let mut overrides = PinyinOverride::default();
        overrides
            .phrase_override
            .insert(String::new(), vec!["a1".to_string()]);
        let error = overrides
            .validate()
            .expect_err("empty phrase key should fail");
        assert!(error.to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_override_accepts_valid_syllables() {
        let toml_input = "[char_override]\n\
            '重' = 'chong2'\n\
            '行' = 'xing2'\n\
            '〇' = 'ling2'\n\
            'a' = 'a'\n\
            \n\
            [phrase_override]\n\
            \"重庆\" = [\"chong2\", \"qing4\"]\n";
        let overrides: PinyinOverride =
            toml::from_str(toml_input).expect("override TOML should parse");
        overrides
            .validate()
            .expect("all syllables should be accepted");
    }
}
