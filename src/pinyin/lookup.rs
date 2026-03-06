use crate::generated::pinyin_map::PINYIN_MAP;

pub(crate) fn all_pinyin_for_char(character: char) -> Vec<String> {
    PINYIN_MAP
        .get(&(character as u32))
        .map(|(_, pinyin_vec)| pinyin_vec.iter().map(|item| (*item).to_string()).collect())
        .unwrap_or_default()
}

pub(crate) fn primary_pinyin_for_char(character: char) -> Option<&'static str> {
    PINYIN_MAP
        .get(&(character as u32))
        .and_then(|(_, pinyin_vec)| pinyin_vec.first().copied())
}
