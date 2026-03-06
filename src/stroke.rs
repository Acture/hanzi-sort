use crate::generated::stroke_map::STROKE_MAP;

pub(crate) fn stroke_count_for_char(character: char) -> Option<u16> {
    STROKE_MAP.get(&(character as u32)).copied()
}

#[cfg(test)]
mod tests {
    use super::stroke_count_for_char;

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
}
