use crate::collator::Collator;
use crate::generated::stroke_map::STROKE_MAP;

pub(crate) fn stroke_count_for_char(character: char) -> Option<u16> {
    STROKE_MAP.get(&(character as u32)).copied()
}

/// Sorts Hanzi by total stroke count using the Unihan `kTotalStrokes` table
/// generated at build time. Characters not present in the table are treated
/// as unmapped and sort after every character with a known stroke count.
#[derive(Debug, Clone, Copy, Default)]
pub struct StrokesCollator;

impl Collator for StrokesCollator {
    type Data = u16;

    fn data_for(&self, character: char) -> Option<u16> {
        stroke_count_for_char(character)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collator::sort_strings_with;

    #[test]
    fn looks_up_representative_stroke_counts() {
        assert_eq!(stroke_count_for_char('一'), Some(1));
        assert_eq!(stroke_count_for_char('十'), Some(2));
        assert_eq!(stroke_count_for_char('重'), Some(9));
    }

    #[test]
    fn returns_none_for_unknown_non_hanzi_characters() {
        assert_eq!(stroke_count_for_char('a'), None);
        assert_eq!(stroke_count_for_char('1'), None);
    }

    #[test]
    fn collator_sorts_by_stroke_count_ascending() {
        let sorted = sort_strings_with(
            vec![
                "天".to_string(),
                "一".to_string(),
                "十".to_string(),
                "大".to_string(),
            ],
            &StrokesCollator,
        );
        assert_eq!(sorted, vec!["一", "十", "大", "天"]);
    }

    #[test]
    fn collator_places_unknown_characters_after_known_ones() {
        let sorted = sort_strings_with(
            vec!["abc".to_string(), "十".to_string(), "1".to_string()],
            &StrokesCollator,
        );
        assert_eq!(sorted, vec!["十", "1", "abc"]);
    }
}
