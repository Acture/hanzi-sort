//! Cantonese Jyutping (粤拼) collator. Phase 3.1 Stream A.
//!
//! Sorts Hanzi by their Cantonese Jyutping reading using Unicode Unihan
//! `kCantonese` data bundled in `data/jyutping.csv`. Optional
//! [`JyutpingOverride`] data can supply phrase-level and per-character
//! pronunciations for polyphonic phrases.

mod key;
mod lookup;
mod model;

pub use model::JyutpingRecord;

use key::{EncodedJyutpingOverride, encode_primary_jyutping_unchecked};
use lookup::{all_jyutping_for_char, primary_jyutping_for_char};

use crate::collator::Collator;
use crate::error::Result;
use crate::r#override::JyutpingOverride;

/// Sorts Hanzi by Cantonese Jyutping (lowercase syllable plus tone digit `1-6`).
#[derive(Debug, Clone, Default)]
pub struct JyutpingCollator {
    override_data: Option<JyutpingOverride>,
    encoded_override: Option<EncodedJyutpingOverride>,
}

impl JyutpingCollator {
    /// Build a collator with no override data.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a collator that honors the supplied override table.
    ///
    /// Override data must already pass [`JyutpingOverride::validate`]. As a
    /// defensive second check, this constructor returns
    /// [`crate::HanziSortError::InvalidOverride`] if any syllable cannot be
    /// encoded for the fast comparison path.
    pub fn with_override(override_data: JyutpingOverride) -> Result<Self> {
        let encoded_override = Some(EncodedJyutpingOverride::try_from(&override_data)?);
        Ok(Self {
            override_data: Some(override_data),
            encoded_override,
        })
    }

    /// Inspect the Jyutping readings the collator knows for `value`.
    ///
    /// Phrase override has the highest precedence, then per-character
    /// override, then the bundled lookup table.
    pub fn jyutping_of(&self, value: &str) -> Vec<JyutpingRecord> {
        if let Some(override_data) = &self.override_data
            && let Some(syllables) = override_data.phrase_override.get(value)
        {
            return value
                .chars()
                .zip(syllables.iter())
                .map(|(character, syllable)| JyutpingRecord {
                    jyutping: vec![syllable.clone()],
                    character,
                })
                .collect();
        }

        value
            .chars()
            .map(|character| self.lookup_char(character))
            .collect()
    }

    fn lookup_char(&self, character: char) -> JyutpingRecord {
        if let Some(override_data) = &self.override_data
            && let Some(syllable) = override_data.char_override.get(&character)
        {
            return JyutpingRecord {
                jyutping: vec![syllable.clone()],
                character,
            };
        }

        JyutpingRecord {
            jyutping: all_jyutping_for_char(character),
            character,
        }
    }
}

impl Collator for JyutpingCollator {
    type Data = u128;

    fn data_for(&self, character: char) -> Option<u128> {
        if let Some(encoded_override) = &self.encoded_override
            && let Some(encoded) = encoded_override.char_override(character)
        {
            return Some(encoded);
        }
        primary_jyutping_for_char(character).map(encode_primary_jyutping_unchecked)
    }

    fn phrase_data(&self, phrase: &str) -> Option<Vec<u128>> {
        self.encoded_override
            .as_ref()?
            .phrase_override(phrase)
            .map(<[u128]>::to_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collator::sort_strings_with;
    use std::collections::HashMap;

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

    #[test]
    fn char_override_changes_data_for() {
        let collator = JyutpingCollator::with_override(JyutpingOverride {
            char_override: HashMap::from([('中', "zung3".to_string())]),
            phrase_override: HashMap::new(),
        })
        .expect("valid override should construct");
        assert_eq!(
            collator.data_for('中'),
            Some(encode_primary_jyutping_unchecked("zung3"))
        );
        // Other characters fall through to the table.
        assert_eq!(
            collator.data_for('汉'),
            Some(encode_primary_jyutping_unchecked("hon3"))
        );
    }

    #[test]
    fn phrase_override_takes_precedence_in_sort() {
        // Without override, 中銀 sorts in zung1 + ngan4 order; with the override
        // we force zung3 + ngan4, which still sorts under z* but the records
        // returned by jyutping_of reflect the override.
        let collator = JyutpingCollator::with_override(JyutpingOverride {
            char_override: HashMap::new(),
            phrase_override: HashMap::from([(
                "中銀".to_string(),
                vec!["zung3".to_string(), "ngan4".to_string()],
            )]),
        })
        .expect("valid override should construct");
        let records = collator.jyutping_of("中銀");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].primary_jyutping(), Some("zung3"));
        assert_eq!(records[1].primary_jyutping(), Some("ngan4"));
    }

    #[test]
    fn with_override_rejects_unencodable_syllable() {
        // Bypass JyutpingOverride::validate (which would reject the bad input)
        // to verify the encoding layer is also defensive.
        let bad = JyutpingOverride {
            char_override: HashMap::from([('女', "nœy5".to_string())]),
            phrase_override: HashMap::new(),
        };
        let err = JyutpingCollator::with_override(bad).expect_err("non-ASCII should fail");
        assert!(err.to_string().contains("ASCII"), "got: {err}");
    }

    #[test]
    fn new_is_infallible_for_no_override_case() {
        let _collator = JyutpingCollator::new();
    }
}
