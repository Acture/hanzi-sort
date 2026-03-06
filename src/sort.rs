use std::cmp::Ordering;

use smallvec::SmallVec;

use crate::pinyin::{EncodedSortKey, PinyinContext, compare_encoded_sort_key};
use crate::stroke::stroke_count_for_char;

const INLINE_STROKE_KEY_LEN: usize = 8;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SortMode {
    #[default]
    Pinyin,
    Strokes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StrokeSortToken {
    character: char,
    strokes: Option<u16>,
}

type StrokeSortKey = SmallVec<[StrokeSortToken; INLINE_STROKE_KEY_LEN]>;

pub fn sort_strings(input: Vec<String>, context: &PinyinContext) -> Vec<String> {
    sort_strings_by(input, context, SortMode::Pinyin)
}

pub fn sort_strings_by(input: Vec<String>, context: &PinyinContext, mode: SortMode) -> Vec<String> {
    match mode {
        SortMode::Pinyin => sort_by_pinyin(input, context),
        SortMode::Strokes => sort_by_strokes(input),
    }
}

fn sort_by_pinyin(input: Vec<String>, context: &PinyinContext) -> Vec<String> {
    let mut with_keys: Vec<(EncodedSortKey, String)> = input
        .into_iter()
        .map(|item| {
            let key = context.encoded_sort_key(&item);
            (key, item)
        })
        .collect();

    with_keys.sort_unstable_by(|a, b| compare_encoded_sort_key(&a.0, &b.0));

    with_keys.into_iter().map(|(_, item)| item).collect()
}

fn sort_by_strokes(input: Vec<String>) -> Vec<String> {
    let mut with_keys: Vec<(StrokeSortKey, String)> = input
        .into_iter()
        .map(|item| {
            let key = stroke_sort_key(&item);
            (key, item)
        })
        .collect();

    with_keys.sort_unstable_by(|a, b| compare_stroke_sort_key(&a.0, &b.0));

    with_keys.into_iter().map(|(_, item)| item).collect()
}

fn stroke_sort_key(value: &str) -> StrokeSortKey {
    value
        .chars()
        .map(|character| StrokeSortToken {
            character,
            strokes: stroke_count_for_char(character),
        })
        .collect()
}

fn compare_stroke_sort_key(a: &StrokeSortKey, b: &StrokeSortKey) -> Ordering {
    let len = a.len().min(b.len());

    for index in 0..len {
        let ordering = compare_stroke_token(&a[index], &b[index]);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    a.len().cmp(&b.len())
}

fn compare_stroke_token(a: &StrokeSortToken, b: &StrokeSortToken) -> Ordering {
    match (a.strokes, b.strokes) {
        (Some(sa), Some(sb)) => sa.cmp(&sb).then_with(|| a.character.cmp(&b.character)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.character.cmp(&b.character),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pinyin::PinyinContext;

    #[test]
    fn test_sort_strings() {
        let input = vec![
            "汉字".to_string(),
            "照相".to_string(),
            "赵云".to_string(),
            "赵四".to_string(),
            "张三".to_string(),
        ];
        let sorted = sort_strings(input, &PinyinContext::default());
        assert_eq!(sorted, vec!["汉字", "张三", "照相", "赵四", "赵云"]);
    }

    #[test]
    fn test_unknown_characters_sort_after_mapped_characters() {
        let context = PinyinContext::default();
        let sorted = sort_strings(
            vec!["abc".to_string(), "张三".to_string(), "123".to_string()],
            &context,
        );
        assert_eq!(sorted, vec!["张三", "123", "abc"]);
    }

    #[test]
    fn encoded_sort_key_preserves_ordering_for_known_strings() {
        let context = PinyinContext::default();
        let encoded_a = context.encoded_sort_key("张三");
        let encoded_b = context.encoded_sort_key("赵四");
        assert_eq!(
            compare_encoded_sort_key(&encoded_a, &encoded_b),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_same_pinyin_uses_original_character_tiebreak() {
        let context = PinyinContext::default();
        let sorted = sort_strings(vec!["赵".to_string(), "照".to_string()], &context);
        assert_eq!(sorted, vec!["照", "赵"]);
    }

    #[test]
    fn test_shorter_prefix_sorts_first_when_keys_match() {
        let context = PinyinContext::default();
        let sorted = sort_strings(vec!["张三丰".to_string(), "张三".to_string()], &context);
        assert_eq!(sorted, vec!["张三", "张三丰"]);
    }

    #[test]
    fn sorts_by_stroke_count_when_requested() {
        let sorted = sort_strings_by(
            vec![
                "天".to_string(),
                "一".to_string(),
                "十".to_string(),
                "大".to_string(),
            ],
            &PinyinContext::default(),
            SortMode::Strokes,
        );
        assert_eq!(sorted, vec!["一", "十", "大", "天"]);
    }

    #[test]
    fn stroke_sort_uses_original_character_as_tiebreak() {
        let sorted = sort_strings_by(
            vec!["七".to_string(), "十".to_string()],
            &PinyinContext::default(),
            SortMode::Strokes,
        );
        assert_eq!(sorted, vec!["七", "十"]);
    }

    #[test]
    fn stroke_sort_places_unknown_characters_after_known_ones() {
        let sorted = sort_strings_by(
            vec!["abc".to_string(), "十".to_string(), "1".to_string()],
            &PinyinContext::default(),
            SortMode::Strokes,
        );
        assert_eq!(sorted, vec!["十", "1", "abc"]);
    }
}
