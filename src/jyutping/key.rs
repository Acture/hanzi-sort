use std::collections::HashMap;

use smallvec::SmallVec;

use crate::error::{HanziSortError, Result};
use crate::r#override::JyutpingOverride;

const MAX_ENCODED_JYUTPING_LEN: usize = 16;
const INLINE_OVERRIDE_LEN: usize = 8;

/// Override table whose syllables have been pre-encoded into `u128` for the
/// fast comparison path. Built from a [`JyutpingOverride`] via [`TryFrom`].
#[derive(Debug, Clone, Default)]
pub(crate) struct EncodedJyutpingOverride {
    char_override: HashMap<char, u128>,
    phrase_override: HashMap<String, SmallVec<[u128; INLINE_OVERRIDE_LEN]>>,
}

impl EncodedJyutpingOverride {
    pub(crate) fn phrase_override(&self, phrase: &str) -> Option<&[u128]> {
        self.phrase_override.get(phrase).map(SmallVec::as_slice)
    }

    pub(crate) fn char_override(&self, character: char) -> Option<u128> {
        self.char_override.get(&character).copied()
    }
}

impl TryFrom<&JyutpingOverride> for EncodedJyutpingOverride {
    type Error = HanziSortError;

    fn try_from(value: &JyutpingOverride) -> Result<Self> {
        let mut char_override = HashMap::with_capacity(value.char_override.len());
        for (character, syllable) in &value.char_override {
            let encoded = encode_primary_jyutping(syllable).map_err(|reason| {
                HanziSortError::InvalidOverride(format!(
                    "char_override entry '{character}' has unencodable jyutping \
                     '{syllable}': {reason}"
                ))
            })?;
            char_override.insert(*character, encoded);
        }

        let mut phrase_override = HashMap::with_capacity(value.phrase_override.len());
        for (phrase, syllables) in &value.phrase_override {
            let mut encoded: SmallVec<[u128; INLINE_OVERRIDE_LEN]> =
                SmallVec::with_capacity(syllables.len());
            for syllable in syllables {
                let value = encode_primary_jyutping(syllable).map_err(|reason| {
                    HanziSortError::InvalidOverride(format!(
                        "phrase_override entry '{phrase}' has unencodable jyutping \
                         '{syllable}': {reason}"
                    ))
                })?;
                encoded.push(value);
            }
            phrase_override.insert(phrase.clone(), encoded);
        }

        Ok(Self {
            char_override,
            phrase_override,
        })
    }
}

/// Pack a Jyutping syllable into a `u128` such that `u128::cmp` agrees with
/// the lexicographic order on the original byte sequence.
pub(crate) fn encode_primary_jyutping(jyutping: &str) -> std::result::Result<u128, &'static str> {
    if jyutping.is_empty() {
        return Err("jyutping syllable is empty");
    }
    if jyutping.len() > MAX_ENCODED_JYUTPING_LEN {
        return Err("jyutping syllable exceeds 16 bytes");
    }
    if !jyutping.is_ascii() {
        return Err("jyutping syllable must be ASCII");
    }
    match jyutping.as_bytes().last() {
        Some(b'1'..=b'6') => {}
        _ => return Err("jyutping syllable must end with a tone digit 1-6"),
    }

    Ok(encode_primary_jyutping_unchecked(jyutping))
}

/// Encode a syllable that the caller already knows is well-formed.
///
/// Used on the hot path inside [`crate::jyutping::JyutpingCollator`] where the
/// syllable comes from the generated PHF table. Callers must guarantee
/// `jyutping` is non-empty, ASCII, at most 16 bytes, and ends in tone 1-6.
pub(crate) fn encode_primary_jyutping_unchecked(jyutping: &str) -> u128 {
    debug_assert!(
        !jyutping.is_empty()
            && jyutping.is_ascii()
            && jyutping.len() <= MAX_ENCODED_JYUTPING_LEN
            && matches!(jyutping.as_bytes().last(), Some(b'1'..=b'6')),
        "encode_primary_jyutping_unchecked invariants violated for {jyutping:?}"
    );
    let mut encoded = 0_u128;
    for byte in jyutping.bytes() {
        encoded = (encoded << 8) | byte as u128;
    }
    encoded << ((MAX_ENCODED_JYUTPING_LEN - jyutping.len()) * 8)
}

#[cfg(test)]
mod tests {
    use super::{encode_primary_jyutping, encode_primary_jyutping_unchecked};

    #[test]
    fn encode_primary_jyutping_preserves_lexicographic_order() {
        assert!(
            encode_primary_jyutping("haa1").unwrap() < encode_primary_jyutping("haai1").unwrap()
        );
        assert!(encode_primary_jyutping("si1").unwrap() < encode_primary_jyutping("si6").unwrap());
        assert!(
            encode_primary_jyutping("hon3").unwrap() < encode_primary_jyutping("zung1").unwrap()
        );
    }

    #[test]
    fn encode_primary_jyutping_rejects_invalid_tone() {
        let err = encode_primary_jyutping("si7").expect_err("tone 7 should fail");
        assert!(err.contains("1-6"));
    }

    #[test]
    fn encode_primary_jyutping_rejects_toneless_input() {
        let err = encode_primary_jyutping("si").expect_err("missing tone should fail");
        assert!(err.contains("tone digit"));
    }

    #[test]
    fn encode_primary_jyutping_rejects_non_ascii() {
        let err = encode_primary_jyutping("siː1").expect_err("non-ASCII should fail");
        assert!(err.contains("ASCII"));
    }

    #[test]
    fn encode_primary_jyutping_unchecked_matches_checked_for_valid_input() {
        for syllable in ["hoeng1", "hon3", "si6", "zung1"] {
            let checked = encode_primary_jyutping(syllable).expect("valid input");
            let unchecked = encode_primary_jyutping_unchecked(syllable);
            assert_eq!(checked, unchecked, "mismatch for {syllable}");
        }
    }
}
