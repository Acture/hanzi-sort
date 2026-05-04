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
    // Carry the original index as a final tiebreak so that two equal inputs
    // (whether equal as strings or merely equal under the sort key) keep their
    // relative order — turning the unstable underlying sort into a stable one.
    let mut with_keys: Vec<(EncodedSortKey, usize, String)> = input
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let key = context.encoded_sort_key(&item);
            (key, index, item)
        })
        .collect();

    with_keys.sort_unstable_by(|a, b| {
        compare_encoded_sort_key(&a.0, &b.0).then_with(|| a.1.cmp(&b.1))
    });

    with_keys.into_iter().map(|(_, _, item)| item).collect()
}

fn sort_by_strokes(input: Vec<String>) -> Vec<String> {
    let mut with_keys: Vec<(StrokeSortKey, usize, String)> = input
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let key = stroke_sort_key(&item);
            (key, index, item)
        })
        .collect();

    with_keys.sort_unstable_by(|a, b| {
        compare_stroke_sort_key(&a.0, &b.0).then_with(|| a.1.cmp(&b.1))
    });

    with_keys.into_iter().map(|(_, _, item)| item).collect()
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

    #[test]
    fn pinyin_sort_is_stable_for_duplicate_inputs() {
        // For this comparator, equal sort keys imply equal strings, so
        // swapping duplicates is observationally invisible. This test is a
        // smoke check that the index tiebreak in `sort_by_pinyin` does not
        // change observable behavior for repeated inputs (count + values
        // preserved). True stability is guaranteed by construction via the
        // `(key, original_index, string)` tuple sort.
        let input = vec![
            "张三".to_string(),
            "张三".to_string(),
            "张三".to_string(),
        ];
        let cloned = input.clone();
        let sorted = sort_strings(input, &PinyinContext::default());
        assert_eq!(sorted, cloned);
    }

    #[test]
    fn pinyin_sort_preserves_relative_order_of_equal_keys() {
        // 赵A/赵B/赵C/赵D have *different* sort keys (they differ in the
        // second character), so this test verifies that prefix ordering plus
        // character-level tie-break behaves as documented; it is not a
        // stability test per se.
        let input = vec![
            "赵A".to_string(),
            "赵B".to_string(),
            "赵C".to_string(),
            "赵D".to_string(),
        ];
        let cloned = input.clone();
        let sorted = sort_strings(input, &PinyinContext::default());
        assert_eq!(sorted, cloned);
    }

    #[test]
    fn stroke_sort_is_stable_for_duplicate_inputs() {
        // See note in `pinyin_sort_is_stable_for_duplicate_inputs`.
        let input = vec![
            "天".to_string(),
            "天".to_string(),
            "天".to_string(),
        ];
        let cloned = input.clone();
        let sorted = sort_strings_by(input, &PinyinContext::default(), SortMode::Strokes);
        assert_eq!(sorted, cloned);
    }
}
