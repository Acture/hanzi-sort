//! Cantonese Jyutping (粤拼) collator. Phase 3.1 Stream A.
//!
//! Sorts Hanzi by their Cantonese Jyutping reading using Unicode Unihan
//! `kCantonese` data bundled in `data/jyutping.csv`.

mod key;
mod lookup;
mod model;

pub use model::JyutpingRecord;

use key::encode_primary_jyutping_unchecked;
use lookup::{all_jyutping_for_char, primary_jyutping_for_char};

use crate::collator::Collator;

/// Sorts Hanzi by Cantonese Jyutping (lowercase syllable plus tone digit).
#[derive(Debug, Clone, Copy, Default)]
pub struct JyutpingCollator;

impl JyutpingCollator {
    pub fn new() -> Self {
        Self
    }

    /// Inspect the Jyutping readings the collator knows for `value`.
    pub fn jyutping_of(&self, value: &str) -> Vec<JyutpingRecord> {
        value
            .chars()
            .map(|character| JyutpingRecord {
                jyutping: all_jyutping_for_char(character),
                character,
            })
            .collect()
    }
}

impl Collator for JyutpingCollator {
    type Data = u128;

    fn data_for(&self, character: char) -> Option<u128> {
        primary_jyutping_for_char(character).map(encode_primary_jyutping_unchecked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collator::sort_strings_with;

    #[test]
    fn data_for_returns_known_jyutping() {
        let collator = JyutpingCollator::new();
        assert_eq!(
            collator.data_for('中'),
            Some(encode_primary_jyutping_unchecked("zung1"))
        );
        assert_eq!(
            collator.data_for('汉'),
            Some(encode_primary_jyutping_unchecked("hon3"))
        );
        assert_eq!(
            collator.data_for('香'),
            Some(encode_primary_jyutping_unchecked("hoeng1"))
        );
    }

    #[test]
    fn sort_orders_by_jyutping_then_tone() {
        let collator = JyutpingCollator::new();
        let sorted = sort_strings_with(
            vec![
                "是".to_string(),
                "試".to_string(),
                "時".to_string(),
                "詩".to_string(),
                "史".to_string(),
                "市".to_string(),
            ],
            &collator,
        );
        assert_eq!(sorted, vec!["詩", "史", "試", "時", "市", "是"]);
    }

    #[test]
    fn jyutping_of_exposes_all_readings() {
        let collator = JyutpingCollator::new();
        let records = collator.jyutping_of("中");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].primary_jyutping(), Some("zung1"));
        assert_eq!(records[0].jyutping, vec!["zung1"]);
    }
}
