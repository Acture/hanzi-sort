use super::key::{EncodedOverride, EncodedSortKey, EncodedSortToken, encode_primary_pinyin};
use super::lookup::{all_pinyin_for_char, primary_pinyin_for_char};
use super::model::PinYinRecord;
use crate::r#override::PinyinOverride;

#[derive(Debug, Clone)]
pub struct PinyinContext {
    override_data: Option<PinyinOverride>,
    encoded_override: Option<EncodedOverride>,
}

impl Default for PinyinContext {
    fn default() -> Self {
        Self::new(None)
    }
}

impl PinyinContext {
    pub fn new(override_data: Option<PinyinOverride>) -> Self {
        let encoded_override = override_data.as_ref().map(EncodedOverride::from);
        Self {
            override_data,
            encoded_override,
        }
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
                .map(encode_primary_pinyin),
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
        let context = PinyinContext::new(Some(PinyinOverride {
            char_override: HashMap::from([('重', "zhong4".to_string())]),
            phrase_override: HashMap::from([(
                "重庆".to_string(),
                vec!["chong2".to_string(), "qing4".to_string()],
            )]),
        }));
        let records = context.pinyin_of("重庆");
        assert_eq!(records[0].primary_pinyin(), Some("chong2"));
    }

    #[test]
    fn char_override_applies_without_phrase_override() {
        let context = PinyinContext::new(Some(PinyinOverride {
            char_override: HashMap::from([('重', "chong2".to_string())]),
            phrase_override: HashMap::new(),
        }));
        let records = context.pinyin_of("重要");
        assert_eq!(records[0].primary_pinyin(), Some("chong2"));
    }

    #[test]
    fn encoded_sort_key_uses_phrase_override() {
        let context = PinyinContext::new(Some(PinyinOverride {
            char_override: HashMap::new(),
            phrase_override: HashMap::from([(
                "重庆".to_string(),
                vec!["chong2".to_string(), "qing4".to_string()],
            )]),
        }));
        let encoded = context.encoded_sort_key("重庆");
        assert_eq!(encoded.len(), 2);
        assert_eq!(
            encoded[0].primary_pinyin,
            Some(encode_primary_pinyin("chong2"))
        );
        assert_eq!(
            encoded[1].primary_pinyin,
            Some(encode_primary_pinyin("qing4"))
        );
    }
}
