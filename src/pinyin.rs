use crate::generated::pinyin_map::PINYIN_MAP;
use crate::r#override::PinyinOverride;

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

#[derive(Debug, Clone, Default)]
pub struct PinyinContext {
    override_data: Option<PinyinOverride>,
}

impl PinyinContext {
    pub fn new(override_data: Option<PinyinOverride>) -> Self {
        Self { override_data }
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
        self.pinyin_of(value)
            .into_iter()
            .map(|record| SortToken {
                character: record.character,
                primary_pinyin: record.primary_pinyin().map(str::to_string),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
}
