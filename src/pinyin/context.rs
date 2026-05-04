use super::key::{EncodedOverride, EncodedSortKey, EncodedSortToken, encode_primary_pinyin_unchecked};
use super::lookup::{all_pinyin_for_char, primary_pinyin_for_char};
use super::model::PinYinRecord;
use crate::error::Result;
use crate::r#override::PinyinOverride;

#[derive(Debug, Clone, Default)]
pub struct PinyinContext {
    override_data: Option<PinyinOverride>,
    encoded_override: Option<EncodedOverride>,
}

impl PinyinContext {
    /// Build a context with no override data.
    ///
    /// This constructor is infallible. Callers that want to apply an override
    /// table should use [`PinyinContext::with_override`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a context that applies the given override table.
    ///
    /// Override data must already pass [`PinyinOverride::validate`]. This
    /// constructor performs an additional defensive check: if any syllable
    /// cannot be encoded for fast comparisons (only possible if the caller
    /// skipped validation and passed non-ASCII or oversized syllables), the
    /// constructor returns [`PinyinSortError::InvalidOverride`].
    ///
    /// [`PinyinSortError::InvalidOverride`]: crate::PinyinSortError::InvalidOverride
    pub fn with_override(override_data: PinyinOverride) -> Result<Self> {
        let encoded_override = Some(EncodedOverride::try_from(&override_data)?);
        Ok(Self {
            override_data: Some(override_data),
            encoded_override,
        })
    }

    pub fn pinyin_of(&self, value: &str) -> Vec<PinYinRecord> {
        if let Some(override_data) = &self.override_data
            && let Some(pinyins) = override_data.phrase_override.get(value)
        {
            return value
                .chars()
                .zip(pinyins.iter())
                .map(|(character, pinyin)| PinYinRecord {
                    pinyin: vec![pinyin.clone()],
                    character,
                })
                .collect();
        }

        value
            .chars()
            .map(|character| self.lookup_char(character))
            .collect()
    }

    pub(crate) fn encoded_sort_key(&self, value: &str) -> EncodedSortKey {
        if let Some(encoded_override) = &self.encoded_override
            && let Some(pinyins) = encoded_override.phrase_override(value)
        {
            return value
                .chars()
                .zip(pinyins.iter().copied())
                .map(|(character, primary_pinyin)| EncodedSortToken {
                    character,
                    primary_pinyin: Some(primary_pinyin),
                })
                .collect();
        }

        value
            .chars()
            .map(|character| self.encoded_sort_token(character))
            .collect()
    }

    fn lookup_char(&self, character: char) -> PinYinRecord {
        if let Some(override_data) = &self.override_data
            && let Some(pinyin) = override_data.char_override.get(&character)
        {
            return PinYinRecord {
                pinyin: vec![pinyin.clone()],
                character,
            };
        }

        PinYinRecord {
            pinyin: all_pinyin_for_char(character),
            character,
        }
    }

    fn encoded_sort_token(&self, character: char) -> EncodedSortToken {
        if let Some(encoded_override) = &self.encoded_override
            && let Some(primary_pinyin) = encoded_override.char_override(character)
        {
            return EncodedSortToken {
                character,
                primary_pinyin: Some(primary_pinyin),
            };
        }

        EncodedSortToken {
            character,
            primary_pinyin: self
                .primary_pinyin_for_char(character)
                .map(encode_primary_pinyin_unchecked),
        }
    }

    fn primary_pinyin_for_char(&self, character: char) -> Option<&str> {
        if let Some(override_data) = &self.override_data
            && let Some(pinyin) = override_data.char_override.get(&character)
        {
            return Some(pinyin.as_str());
        }

        primary_pinyin_for_char(character)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#override::PinyinOverride;
    use std::collections::HashMap;

    #[test]
    fn pinyin_of_known_characters() {
        let context = PinyinContext::default();
        let expected = vec![
            PinYinRecord {
                pinyin: vec!["han4".to_string()],
                character: '汉',
            },
            PinYinRecord {
                pinyin: vec!["zi4".to_string()],
                character: '字',
            },
        ];
        assert_eq!(context.pinyin_of("汉字"), expected);
    }

    #[test]
    fn includes_generated_first_record() {
        let context = PinyinContext::default();
        let records = context.pinyin_of("〇");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].primary_pinyin(), Some("ling2"));
    }

    #[test]
    fn unknown_characters_are_preserved() {
        let context = PinyinContext::default();
        let records = context.pinyin_of("a1");
        assert_eq!(
            records,
            vec![
                PinYinRecord {
                    pinyin: vec![],
                    character: 'a',
                },
                PinYinRecord {
                    pinyin: vec![],
                    character: '1',
                },
            ]
        );
    }

    #[test]
    fn phrase_override_takes_precedence() {
        let context = PinyinContext::with_override(PinyinOverride {
            char_override: HashMap::from([('重', "zhong4".to_string())]),
            phrase_override: HashMap::from([(
                "重庆".to_string(),
                vec!["chong2".to_string(), "qing4".to_string()],
            )]),
        })
        .expect("valid override should construct");
        let records = context.pinyin_of("重庆");
        assert_eq!(records[0].primary_pinyin(), Some("chong2"));
    }

    #[test]
    fn char_override_applies_without_phrase_override() {
        let context = PinyinContext::with_override(PinyinOverride {
            char_override: HashMap::from([('重', "chong2".to_string())]),
            phrase_override: HashMap::new(),
        })
        .expect("valid override should construct");
        let records = context.pinyin_of("重要");
        assert_eq!(records[0].primary_pinyin(), Some("chong2"));
    }

    #[test]
    fn polyphonic_characters_expose_all_readings() {
        let context = PinyinContext::default();
        let records = context.pinyin_of("乐");
        assert_eq!(records.len(), 1);
        assert!(records[0].pinyin.len() > 1);
        assert!(records[0].pinyin.iter().any(|item| item == "le4"));
        assert!(records[0].pinyin.iter().any(|item| item == "yue4"));
    }

    #[test]
    fn encoded_sort_key_uses_phrase_override() {
        let context = PinyinContext::with_override(PinyinOverride {
            char_override: HashMap::new(),
            phrase_override: HashMap::from([(
                "重庆".to_string(),
                vec!["chong2".to_string(), "qing4".to_string()],
            )]),
        })
        .expect("valid override should construct");
        let encoded = context.encoded_sort_key("重庆");
        assert_eq!(encoded.len(), 2);
        assert_eq!(
            encoded[0].primary_pinyin,
            Some(encode_primary_pinyin_unchecked("chong2"))
        );
        assert_eq!(
            encoded[1].primary_pinyin,
            Some(encode_primary_pinyin_unchecked("qing4"))
        );
    }

    #[test]
    fn with_override_rejects_unencodable_syllable() {
        // Bypass PinyinOverride::validate (which would reject non-ASCII)
        // to make sure the encoding layer is also defensive.
        let bad = PinyinOverride {
            char_override: HashMap::from([('女', "nü3".to_string())]),
            phrase_override: HashMap::new(),
        };
        let err = PinyinContext::with_override(bad).expect_err("non-ASCII should fail");
        assert!(err.to_string().contains("ASCII"), "got: {err}");
    }

    #[test]
    fn new_is_infallible_for_no_override_case() {
        let _context = PinyinContext::new();
    }
}
