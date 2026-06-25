//! Surname-aware name sorting (姓名模式).
//!
//! Many Chinese characters take a different reading when used as a surname
//! than in everyday text — 单 is `dān` ("single") but the surname is Shàn,
//! 解 is `jiě` but the surname is Xiè, 区 is `qū` but the surname is Ōu.
//! Sorting a list of names under plain pinyin therefore mis-files these
//! entries.
//!
//! [`NameCollator`] reuses the pinyin collator for ordinary characters but
//! applies a built-in surname table to the leading **surname position** of
//! each entry (longest-prefix, so compound 复姓 like 万俟 → Mòqí are matched
//! as a unit). Entries that do not start with a polyphonic surname sort
//! exactly as they would under [`crate::PinyinCollator`].

use crate::collator::Collator;
use crate::generated::surname_map::SURNAME_MAP;
use crate::pinyin::{PinyinCollator, encode_primary_pinyin_unchecked};

/// Sorts Hanzi names by Mandarin pinyin with surname-specific readings applied
/// to the leading surname character(s).
#[derive(Debug, Clone, Default)]
pub struct NameCollator {
    inner: PinyinCollator,
}

impl NameCollator {
    /// Build a surname-aware name collator.
    pub fn new() -> Self {
        Self {
            inner: PinyinCollator::new(),
        }
    }
}

impl Collator for NameCollator {
    type Data = u128;

    fn data_for(&self, character: char) -> Option<u128> {
        self.inner.data_for(character)
    }

    fn phrase_data(&self, name: &str) -> Option<Vec<u128>> {
        let (prefix_chars, readings) = match_surname_prefix(name)?;

        let mut out: Vec<u128> = Vec::with_capacity(name.chars().count());
        for &reading in readings {
            out.push(encode_primary_pinyin_unchecked(reading));
        }
        // The remainder of the name uses ordinary per-character pinyin. If any
        // trailing character is unmapped we cannot represent it in the all-data
        // phrase key, so fall back to the per-character path for the whole entry.
        for character in name.chars().skip(prefix_chars) {
            out.push(self.inner.data_for(character)?);
        }
        Some(out)
    }
}

/// Longest-prefix match of `name`'s leading character(s) against the built-in
/// surname table: a two-character compound surname (复姓) is preferred over a
/// single-character surname. Returns the matched prefix length in characters
/// and its surname readings.
fn match_surname_prefix(name: &str) -> Option<(usize, &'static [&'static str])> {
    let prefix2: String = name.chars().take(2).collect();
    if prefix2.chars().count() == 2
        && let Some(readings) = SURNAME_MAP.get(prefix2.as_str()).copied()
    {
        return Some((2, readings));
    }

    let prefix1: String = name.chars().take(1).collect();
    if !prefix1.is_empty()
        && let Some(readings) = SURNAME_MAP.get(prefix1.as_str()).copied()
    {
        return Some((1, readings));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collator::sort_strings_with;

    #[test]
    fn surname_polyphones_sort_by_surname_reading() {
        // 单 = Shàn, 区 = Ōu, 解 = Xiè  ->  ou1 < shan4 < xie4.
        // Under plain pinyin these would be dan1 < jie3 < qu1 (单 解 区).
        let sorted = sort_strings_with(
            vec!["单".to_string(), "区".to_string(), "解".to_string()],
            &NameCollator::new(),
        );
        assert_eq!(sorted, vec!["区", "单", "解"]);
    }

    #[test]
    fn compound_surname_matched_as_a_unit() {
        // 万俟 = Mòqí (mo4) sorts before 牛 = Niú (niu2); under plain pinyin
        // 万 = wàn would sort 万俟 after 牛 (niu < wan).
        let sorted = sort_strings_with(
            vec!["牛".to_string(), "万俟".to_string()],
            &NameCollator::new(),
        );
        assert_eq!(sorted, vec!["万俟", "牛"]);
    }

    #[test]
    fn surname_reading_applies_only_to_the_leading_character() {
        // A given-name 单 (second position) keeps its ordinary reading; only
        // a leading 单 becomes the surname Shàn. 陈单 vs 陈区: position-2
        // 单/区 use ordinary dan1/qu1, so 陈单 (dan) < 陈区 (qu).
        let sorted = sort_strings_with(
            vec!["陈区".to_string(), "陈单".to_string()],
            &NameCollator::new(),
        );
        assert_eq!(sorted, vec!["陈单", "陈区"]);
    }

    #[test]
    fn non_surname_entries_sort_like_pinyin() {
        // No leading polyphonic surname -> identical to PinyinCollator.
        let sorted = sort_strings_with(
            vec![
                "汉字".to_string(),
                "张三".to_string(),
                "赵四".to_string(),
            ],
            &NameCollator::new(),
        );
        assert_eq!(sorted, vec!["汉字", "张三", "赵四"]);
    }
}
