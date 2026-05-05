use crate::generated::jyutping_map::JYUTPING_MAP;

pub(crate) fn all_jyutping_for_char(character: char) -> Vec<String> {
    JYUTPING_MAP
        .get(&(character as u32))
        .map(|(_, jyutping_vec)| {
            jyutping_vec
                .iter()
                .map(|item| (*item).to_string())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn primary_jyutping_for_char(character: char) -> Option<&'static str> {
    JYUTPING_MAP
        .get(&(character as u32))
        .and_then(|(_, jyutping_vec)| jyutping_vec.first().copied())
}
