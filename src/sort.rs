use std::cmp::Ordering;

use crate::pinyin::{PinyinContext, SortKey, SortToken};

pub fn sort_strings(input: Vec<String>, context: &PinyinContext) -> Vec<String> {
    let mut with_keys: Vec<(SortKey, String)> = input
        .into_iter()
        .map(|item| {
            let key = context.sort_key(&item);
            (key, item)
        })
        .collect();

    with_keys.sort_by(|a, b| compare_sort_key(&a.0, &b.0));

    with_keys.into_iter().map(|(_, item)| item).collect()
}

pub fn compare_sort_key(a: &SortKey, b: &SortKey) -> Ordering {
    let len = a.len().min(b.len());

    for index in 0..len {
        let ordering = compare_token(&a[index], &b[index]);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    a.len().cmp(&b.len())
}

fn compare_token(a: &SortToken, b: &SortToken) -> Ordering {
    match (&a.primary_pinyin, &b.primary_pinyin) {
        (Some(pa), Some(pb)) => pa.cmp(pb).then_with(|| a.character.cmp(&b.character)),
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
    fn test_compare_sort_key() {
        let context = PinyinContext::default();
        let a = context.sort_key("张三");
        let b = context.sort_key("赵四");
        assert_eq!(compare_sort_key(&a, &b), Ordering::Less);
    }
}
