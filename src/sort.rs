use crate::pinyin::{EncodedSortKey, PinyinContext, compare_encoded_sort_key};

pub fn sort_strings(input: Vec<String>, context: &PinyinContext) -> Vec<String> {
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
}
