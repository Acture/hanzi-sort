use std::collections::HashMap;

use smallvec::SmallVec;

use crate::generated::pinyin_map::PINYIN_MAP;
use crate::r#override::PinyinOverride;

const INLINE_SORT_KEY_LEN: usize = 8;
const MAX_ENCODED_PINYIN_LEN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinYinRecord {
    pub pinyin: Vec<String>,
    pub character: char,
}

impl PinYinRecord {
    pub fn primary_pinyin(&self) -> Option<&str> {
        self.pinyin.first().map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortToken {
    pub character: char,
    pub primary_pinyin: Option<String>,
}

pub type SortKey = Vec<SortToken>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EncodedSortToken {
    pub character: char,
    pub primary_pinyin: Option<u128>,
}

pub(crate) type EncodedSortKey = SmallVec<[EncodedSortToken; INLINE_SORT_KEY_LEN]>;

#[derive(Debug, Clone, Default)]
struct EncodedOverride {
    char_override: HashMap<char, u128>,
    phrase_override: HashMap<String, SmallVec<[u128; INLINE_SORT_KEY_LEN]>>,
}

impl From<&PinyinOverride> for EncodedOverride {
    fn from(value: &PinyinOverride) -> Self {
        let char_override = value
            .char_override
            .iter()
            .map(|(character, pinyin)| (*character, encode_primary_pinyin(pinyin)))
            .collect();

        let phrase_override = value
            .phrase_override
            .iter()
            .map(|(phrase, pinyins)| {
                let encoded = pinyins
                    .iter()
                    .map(|pinyin| encode_primary_pinyin(pinyin))
                    .collect();
                (phrase.clone(), encoded)
            })
            .collect();

        Self {
            char_override,
            phrase_override,
        }
    }
}

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

    pub fn sort_key(&self, value: &str) -> SortKey {
        if let Some(override_data) = &self.override_data
            && let Some(pinyins) = override_data.phrase_override.get(value)
        {
            return value
                .chars()
                .zip(pinyins.iter())
                .map(|(character, pinyin)| SortToken {
                    character,
                    primary_pinyin: Some(pinyin.clone()),
                })
                .collect();
        }

        value
            .chars()
            .map(|character| SortToken {
                character,
                primary_pinyin: self.primary_pinyin_for_char(character).map(str::to_string),
            })
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

        let pinyin = PINYIN_MAP
            .get(&(character as u32))
            .map(|(_, pinyin_vec)| pinyin_vec.iter().map(|item| (*item).to_string()).collect())
            .unwrap_or_default();

        PinYinRecord { pinyin, character }
    }

    pub(crate) fn encoded_sort_key(&self, value: &str) -> EncodedSortKey {
        if let Some(encoded_override) = &self.encoded_override
            && let Some(pinyins) = encoded_override.phrase_override.get(value)
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

    fn encoded_sort_token(&self, character: char) -> EncodedSortToken {
        if let Some(encoded_override) = &self.encoded_override
            && let Some(primary_pinyin) = encoded_override.char_override.get(&character)
        {
            return EncodedSortToken {
                character,
                primary_pinyin: Some(*primary_pinyin),
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

        PINYIN_MAP
            .get(&(character as u32))
            .and_then(|(_, pinyin_vec)| pinyin_vec.first().copied())
    }
}

fn encode_primary_pinyin(pinyin: &str) -> u128 {
    assert!(
        pinyin.len() <= MAX_ENCODED_PINYIN_LEN,
        "pinyin syllable exceeds {MAX_ENCODED_PINYIN_LEN} bytes: {pinyin}"
    );

    let mut encoded = 0_u128;
    for byte in pinyin.bytes() {
        encoded = (encoded << 8) | byte as u128;
    }

    encoded << ((MAX_ENCODED_PINYIN_LEN - pinyin.len()) * 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pinyin_of_known_characters() {
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
    fn test_includes_generated_first_record() {
        let context = PinyinContext::default();
        let records = context.pinyin_of("〇");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].primary_pinyin(), Some("ling2"));
    }

    #[test]
    fn test_unknown_characters_are_preserved() {
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
    fn test_phrase_override_takes_precedence() {
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
    fn test_char_override_applies_without_phrase_override() {
        let context = PinyinContext::new(Some(PinyinOverride {
            char_override: HashMap::from([('重', "chong2".to_string())]),
            phrase_override: HashMap::new(),
        }));
        let records = context.pinyin_of("重要");
        assert_eq!(records[0].primary_pinyin(), Some("chong2"));
    }

    #[test]
    fn test_encode_primary_pinyin_preserves_lexicographic_order() {
        assert!(encode_primary_pinyin("a") < encode_primary_pinyin("aa"));
        assert!(encode_primary_pinyin("chong2") < encode_primary_pinyin("qing4"));
        assert!(encode_primary_pinyin("zhong4") < encode_primary_pinyin("zhong5"));
    }

    #[test]
    fn test_encoded_sort_key_uses_phrase_override_without_allocating_strings() {
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
