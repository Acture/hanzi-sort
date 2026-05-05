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
        validate_phrase_set(
            &self.char_override,
            &self.phrase_override,
            PINYIN_TONE_RANGE,
            "pinyin",
        )
    }
}

/// Override table for the Cantonese Jyutping collator. Schema mirrors
/// [`PinyinOverride`] but syllables must end in tone digit `1-6` (Cantonese
/// has six tones).
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[cfg(feature = "collator-jyutping")]
pub struct JyutpingOverride {
    #[serde(default)]
    pub char_override: HashMap<char, String>,
    #[serde(default)]
    pub phrase_override: HashMap<String, Vec<String>>,
}

#[cfg(feature = "collator-jyutping")]
impl JyutpingOverride {
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
        validate_phrase_set(
            &self.char_override,
            &self.phrase_override,
            JYUTPING_TONE_RANGE,
            "jyutping",
        )
    }
}

const PINYIN_TONE_RANGE: ToneRange = ToneRange { max: 5 };
#[cfg(feature = "collator-jyutping")]
const JYUTPING_TONE_RANGE: ToneRange = ToneRange { max: 6 };

#[derive(Copy, Clone)]
struct ToneRange {
    max: u8,
}

fn validate_phrase_set(
    char_override: &HashMap<char, String>,
    phrase_override: &HashMap<String, Vec<String>>,
    tone_range: ToneRange,
    syllable_kind: &'static str,
) -> Result<()> {
    for (character, syllable) in char_override {
        validate_syllable(syllable, tone_range).map_err(|reason| {
            HanziSortError::InvalidOverride(format!(
                "char_override entry '{character}' has invalid {syllable_kind} \
                 '{syllable}': {reason}"
            ))
        })?;
    }

    for (phrase, syllables) in phrase_override {
        if phrase.is_empty() {
            return Err(HanziSortError::InvalidOverride(
                "phrase_override key cannot be empty".to_string(),
            ));
        }
        let char_count = phrase.chars().count();
        if char_count != syllables.len() {
            return Err(HanziSortError::InvalidOverride(format!(
                "phrase_override entry '{}' has {} characters but {} {} values",
                phrase, char_count, syllables.len(), syllable_kind,
            )));
        }
        for syllable in syllables {
            validate_syllable(syllable, tone_range).map_err(|reason| {
                HanziSortError::InvalidOverride(format!(
                    "phrase_override entry '{phrase}' has invalid {syllable_kind} \
                     '{syllable}': {reason}"
                ))
            })?;
        }
    }

    Ok(())
}

/// Validate a single phonetic syllable in tone-numbered romanization.
///
/// `tone_range.max` is the highest valid tone digit (5 for Mandarin pinyin,
/// 6 for Cantonese Jyutping). Tone digits below `1` or above `tone_range.max`
/// are rejected. Toneless syllables are rejected because the `u128` sort-key
/// encoding right-pads with zero bytes — that would place toneless syllables
/// before every toned variant, breaking conventional Chinese dictionary
/// ordering. (For Mandarin, neutral tone is conventionally written `5`.)
///
/// The data pipeline normalizes `ü` to `v`, so non-ASCII input is rejected
/// here as well to keep the byte-wise encoding monotonic.
fn validate_syllable(
    syllable: &str,
    tone_range: ToneRange,
) -> std::result::Result<(), &'static str> {
    if syllable.is_empty() {
        return Err("syllable is empty");
    }
    if !syllable.is_ascii() {
        return Err("syllable must be ASCII (use 'v' instead of 'ü')");
    }

    let bytes = syllable.as_bytes();
    let last = match bytes.last() {
        Some(b) => *b,
        None => unreachable!("non-empty syllable always has a last byte"),
    };
    let letters = if last.is_ascii_digit() && (b'1'..=b'0' + tone_range.max).contains(&last) {
        &bytes[..bytes.len() - 1]
    } else if tone_range.max == 5 {
        return Err(
            "syllable must end with a tone digit 1-5 (use 5 for neutral / light tone)",
        );
    } else if tone_range.max == 6 {
        return Err("syllable must end with a tone digit 1-6");
    } else {
        return Err("syllable must end with a valid tone digit");
    };

    if letters.is_empty() {
        return Err("syllable must contain at least one letter before the tone digit");
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
            .expect_err("tone 9 should fail because 9 is not in the valid 1-5 range");
        assert!(
            error.to_string().contains("tone digit"),
            "unexpected: {error}"
        );
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
    fn test_override_rejects_toneless_syllable() {
        // Toneless syllables like "le" must be written with explicit tone 5
        // (e.g. "le5") so they sort after toned variants like "le4".
        let overrides: PinyinOverride = toml::from_str("[char_override]\n'了' = 'le'\n")
            .expect("override TOML should parse");
        let error = overrides
            .validate()
            .expect_err("toneless syllable should fail");
        assert!(
            error.to_string().contains("tone digit"),
            "unexpected: {error}"
        );
    }

    #[test]
    fn test_override_accepts_valid_syllables() {
        let toml_input = "[char_override]\n\
            '重' = 'chong2'\n\
            '行' = 'xing2'\n\
            '〇' = 'ling2'\n\
            '了' = 'le5'\n\
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
