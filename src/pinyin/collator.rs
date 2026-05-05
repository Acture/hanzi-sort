use super::key::{EncodedOverride, encode_primary_pinyin_unchecked};
use super::lookup::{all_pinyin_for_char, primary_pinyin_for_char};
use super::model::PinYinRecord;
use crate::collator::Collator;
use crate::error::Result;
use crate::r#override::PinyinOverride;

/// Sorts Hanzi by Mandarin pinyin (tone3 lowercase) using a static lookup
/// table generated at build time from the upstream pinyin dataset.
///
/// Optional [`PinyinOverride`] data can supply phrase-level and per-character
/// pronunciations to handle polyphonic phrases like `重庆` or `银行`.
#[derive(Debug, Clone, Default)]
pub struct PinyinCollator {
    override_data: Option<PinyinOverride>,
    encoded_override: Option<EncodedOverride>,
}

impl PinyinCollator {
    /// Build a collator with no override data.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a collator that honors the supplied override table.
    ///
    /// Override data must already pass [`PinyinOverride::validate`]. As a
    /// defensive second check, this constructor returns
    /// [`crate::HanziSortError::InvalidOverride`] if any syllable cannot be
    /// encoded for the fast comparison path (only possible if the caller
    /// skipped validation and passed non-ASCII or oversized syllables).
    pub fn with_override(override_data: PinyinOverride) -> Result<Self> {
        let encoded_override = Some(EncodedOverride::try_from(&override_data)?);
        Ok(Self {
            override_data: Some(override_data),
            encoded_override,
        })
    }

    /// Inspect the pinyin readings the collator would use for `value`.
    ///
    /// Phrase override has the highest precedence, then per-character
    /// override, then the bundled lookup table. Useful for diagnostics or
    /// for building dictionary-style displays.
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
}

impl Collator for PinyinCollator {
    type Data = u128;

    fn data_for(&self, character: char) -> Option<u128> {
        if let Some(encoded_override) = &self.encoded_override
            && let Some(primary_pinyin) = encoded_override.char_override(character)
        {
            return Some(primary_pinyin);
        }
        primary_pinyin_for_char(character).map(encode_primary_pinyin_unchecked)
    }

    fn phrase_data(&self, phrase: &str) -> Option<Vec<u128>> {
        self.encoded_override
            .as_ref()?
            .phrase_override(phrase)
            .map(<[u128]>::to_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collator::sort_strings_with;
    use crate::r#override::PinyinOverride;
    use std::collections::HashMap;

    #[test]
    fn pinyin_of_known_characters() {
        let collator = PinyinCollator::default();
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
        assert_eq!(collator.pinyin_of("汉字"), expected);
    }

    #[test]
    fn includes_generated_first_record() {
        let collator = PinyinCollator::default();
        let records = collator.pinyin_of("〇");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].primary_pinyin(), Some("ling2"));
    }

    #[test]
    fn unknown_characters_are_preserved() {
        let collator = PinyinCollator::default();
        let records = collator.pinyin_of("a1");
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
        let collator = PinyinCollator::with_override(PinyinOverride {
            char_override: HashMap::from([('重', "zhong4".to_string())]),
            phrase_override: HashMap::from([(
                "重庆".to_string(),
                vec!["chong2".to_string(), "qing4".to_string()],
            )]),
        })
        .expect("valid override should construct");
        let records = collator.pinyin_of("重庆");
        assert_eq!(records[0].primary_pinyin(), Some("chong2"));
    }

    #[test]
    fn char_override_applies_without_phrase_override() {
        let collator = PinyinCollator::with_override(PinyinOverride {
            char_override: HashMap::from([('重', "chong2".to_string())]),
            phrase_override: HashMap::new(),
        })
        .expect("valid override should construct");
        let records = collator.pinyin_of("重要");
        assert_eq!(records[0].primary_pinyin(), Some("chong2"));
    }

    #[test]
    fn polyphonic_characters_expose_all_readings() {
        let collator = PinyinCollator::default();
        let records = collator.pinyin_of("乐");
        assert_eq!(records.len(), 1);
        assert!(records[0].pinyin.len() > 1);
        assert!(records[0].pinyin.iter().any(|item| item == "le4"));
        assert!(records[0].pinyin.iter().any(|item| item == "yue4"));
    }

    #[test]
    fn collator_returns_phrase_data_for_known_phrase() {
        let collator = PinyinCollator::with_override(PinyinOverride {
            char_override: HashMap::new(),
            phrase_override: HashMap::from([(
                "重庆".to_string(),
                vec!["chong2".to_string(), "qing4".to_string()],
            )]),
        })
        .expect("valid override should construct");
        let phrase = collator.phrase_data("重庆").expect("phrase override should hit");
        assert_eq!(
            phrase,
            vec![
                encode_primary_pinyin_unchecked("chong2"),
                encode_primary_pinyin_unchecked("qing4"),
            ]
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
        let err = PinyinCollator::with_override(bad).expect_err("non-ASCII should fail");
        assert!(err.to_string().contains("ASCII"), "got: {err}");
    }

    #[test]
    fn new_is_infallible_for_no_override_case() {
        let _collator = PinyinCollator::new();
    }

    #[test]
    fn library_users_can_call_sort_strings_with_directly() {
        // Smoke test: a downstream library user can use PinyinCollator
        // through the generic sort entry point without going through
        // AnyCollator dispatch.
        let collator = PinyinCollator::new();
        let sorted = sort_strings_with(
            vec![
                "汉字".to_string(),
                "照相".to_string(),
                "赵云".to_string(),
                "赵四".to_string(),
                "张三".to_string(),
            ],
            &collator,
        );
        assert_eq!(sorted, vec!["汉字", "张三", "照相", "赵四", "赵云"]);
    }

    #[test]
    fn neutral_tone_sorts_after_marked_tones() {
        // After the toneless-to-5 normalization, 吗 (ma, neutral) has
        // primary "ma5" and must sort after 麻 (ma2) and 马 (ma3) but
        // before 骂 (ma4)... wait, no — under tone3 lexicographic order,
        // ma5 sorts after ma4. So expected ascending order is:
        //   麻 (ma2) < 马 (ma3) < 骂 (ma4) < 吗 (ma5)
        let collator = PinyinCollator::default();
        let sorted = sort_strings_with(
            vec![
                "吗".to_string(),
                "马".to_string(),
                "麻".to_string(),
                "骂".to_string(),
            ],
            &collator,
        );
        assert_eq!(sorted, vec!["麻", "马", "骂", "吗"]);
    }
}
