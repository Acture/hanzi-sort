use std::cmp::Ordering;
use std::collections::HashMap;

use smallvec::SmallVec;

use crate::r#override::PinyinOverride;

const INLINE_SORT_KEY_LEN: usize = 8;
const MAX_ENCODED_PINYIN_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EncodedSortToken {
    pub(crate) character: char,
    pub(crate) primary_pinyin: Option<u128>,
}

pub(crate) type EncodedSortKey = SmallVec<[EncodedSortToken; INLINE_SORT_KEY_LEN]>;

#[derive(Debug, Clone, Default)]
pub(crate) struct EncodedOverride {
    char_override: HashMap<char, u128>,
    phrase_override: HashMap<String, SmallVec<[u128; INLINE_SORT_KEY_LEN]>>,
}

impl EncodedOverride {
    pub(crate) fn phrase_override(&self, phrase: &str) -> Option<&[u128]> {
        self.phrase_override.get(phrase).map(SmallVec::as_slice)
    }

    pub(crate) fn char_override(&self, character: char) -> Option<u128> {
        self.char_override.get(&character).copied()
    }
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

pub(crate) fn compare_encoded_sort_key(a: &EncodedSortKey, b: &EncodedSortKey) -> Ordering {
    let len = a.len().min(b.len());

    for index in 0..len {
        let ordering = compare_encoded_token(&a[index], &b[index]);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    a.len().cmp(&b.len())
}

pub(crate) fn encode_primary_pinyin(pinyin: &str) -> u128 {
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

fn compare_encoded_token(a: &EncodedSortToken, b: &EncodedSortToken) -> Ordering {
    match (a.primary_pinyin, b.primary_pinyin) {
        (Some(pa), Some(pb)) => pa.cmp(&pb).then_with(|| a.character.cmp(&b.character)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.character.cmp(&b.character),
    }
}

#[cfg(test)]
mod tests {
    use super::encode_primary_pinyin;

    #[test]
    fn encode_primary_pinyin_preserves_lexicographic_order() {
        assert!(encode_primary_pinyin("a") < encode_primary_pinyin("aa"));
        assert!(encode_primary_pinyin("chong2") < encode_primary_pinyin("qing4"));
        assert!(encode_primary_pinyin("zhong4") < encode_primary_pinyin("zhong5"));
    }
}
