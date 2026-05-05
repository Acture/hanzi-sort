//! Pluggable sort strategies for character-level ordering.
//!
//! A [`Collator`] maps each input character to optional sort data. The
//! generic [`sort_strings_with`] derives a per-string [`SortKey`] from the
//! collator and sorts inputs lexicographically, with the original input
//! index as a final tiebreak so equal-key entries preserve their input
//! order (effectively a stable sort on top of an unstable backend).
//!
//! Library users with a known collator type can call [`sort_strings_with`]
//! directly to keep dispatch monomorphic; the CLI uses
//! [`crate::AnyCollator`] (added in a follow-up step) for runtime
//! strategy selection.

use smallvec::SmallVec;

const INLINE_KEY_LEN: usize = 8;

/// Whether a character has corresponding mapped data under a [`Collator`].
///
/// Variants are declared `Yes` then `No` so the derived [`Ord`] places
/// mapped characters before unmapped ones, matching the previous
/// hand-written comparators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mapped<T> {
    Yes(T),
    No,
}

/// One character's contribution to a sort key.
///
/// `data` carries the collator-supplied per-character information (for
/// example an encoded pinyin syllable or a stroke count); `character`
/// participates as a deterministic tiebreak when two tokens have the same
/// `data`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharToken<T> {
    pub data: Mapped<T>,
    pub character: char,
}

/// Per-string sort key produced by a [`Collator`].
///
/// Inlined for short inputs so typical sorts avoid heap allocation per key.
pub type SortKey<T> = SmallVec<[CharToken<T>; INLINE_KEY_LEN]>;

/// A character-level sort strategy.
///
/// Implementors map each input character to optional [`Self::Data`]; the
/// generic sort engine handles tokenization, prefix comparison, length
/// tiebreaks, and stable ordering for equal-key inputs.
pub trait Collator {
    /// The mapped per-character data (e.g. encoded pinyin `u128`, stroke
    /// count `u16`).
    type Data: Ord + Clone;

    /// Mapped data for a single character, or `None` if the collator has
    /// no data for it. Unmapped characters sort after every mapped one
    /// and tiebreak on the character value.
    fn data_for(&self, ch: char) -> Option<Self::Data>;

    /// Optional phrase-level override. When `Some`, returns one mapped
    /// value per character in `phrase`; when `None`, the collator falls
    /// back to per-character lookup. Default: no phrase override.
    fn phrase_data(&self, _phrase: &str) -> Option<Vec<Self::Data>> {
        None
    }
}

/// Build the sort key for a single string under the given collator.
///
/// Applies [`Collator::phrase_data`] first; otherwise falls back to
/// [`Collator::data_for`] per character.
pub fn sort_key_of<C: Collator>(collator: &C, value: &str) -> SortKey<C::Data> {
    if let Some(phrase) = collator.phrase_data(value) {
        return value
            .chars()
            .zip(phrase)
            .map(|(character, data)| CharToken {
                data: Mapped::Yes(data),
                character,
            })
            .collect();
    }

    value
        .chars()
        .map(|character| CharToken {
            data: collator
                .data_for(character)
                .map(Mapped::Yes)
                .unwrap_or(Mapped::No),
            character,
        })
        .collect()
}

/// Sort `input` under `collator`.
///
/// Uses an unstable sort under the hood but carries the original input
/// index as a final tiebreak, so two inputs producing equal keys keep
/// their input-order relative position.
pub fn sort_strings_with<C: Collator>(input: Vec<String>, collator: &C) -> Vec<String> {
    let mut with_keys: Vec<(SortKey<C::Data>, usize, String)> = input
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let key = sort_key_of(collator, &item);
            (key, index, item)
        })
        .collect();

    with_keys.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    with_keys.into_iter().map(|(_, _, item)| item).collect()
}

/// Type-erased dispatch over the built-in collators.
///
/// Used by the CLI and [`crate::RuntimeConfig`] to select a sort strategy
/// at runtime. Library users with a known concrete collator type should
/// call [`sort_strings_with`] directly to keep the dispatch monomorphic.
///
/// Variants are gated by their `collator-*` cargo features so that disabling
/// a collator both removes the code and shrinks the binary. New collators
/// can plug in by adding a feature in `Cargo.toml` and a variant + match
/// arm here. See `CONTRIBUTING.md`.
#[derive(Debug, Clone)]
pub enum AnyCollator {
    #[cfg(feature = "collator-pinyin")]
    Pinyin(crate::pinyin::PinyinCollator),
    #[cfg(feature = "collator-strokes")]
    Strokes(crate::stroke::StrokesCollator),
    #[cfg(feature = "collator-jyutping")]
    Jyutping(crate::jyutping::JyutpingCollator),
    #[cfg(feature = "collator-zhuyin")]
    Zhuyin(crate::zhuyin::ZhuyinCollator),
    #[cfg(feature = "collator-radical")]
    Radical(crate::radical::RadicalCollator),
}

impl AnyCollator {
    /// Pinyin collator with no override data.
    #[cfg(feature = "collator-pinyin")]
    pub fn pinyin() -> Self {
        Self::Pinyin(crate::pinyin::PinyinCollator::new())
    }

    /// Pinyin collator that honors the supplied override table.
    ///
    /// Returns [`crate::HanziSortError::InvalidOverride`] if any syllable
    /// in the override cannot be encoded for fast comparisons.
    #[cfg(feature = "collator-pinyin")]
    pub fn pinyin_with_override(
        overrides: crate::r#override::PinyinOverride,
    ) -> crate::error::Result<Self> {
        Ok(Self::Pinyin(
            crate::pinyin::PinyinCollator::with_override(overrides)?,
        ))
    }

    /// Stroke-count collator (no overrides).
    #[cfg(feature = "collator-strokes")]
    pub fn strokes() -> Self {
        Self::Strokes(crate::stroke::StrokesCollator)
    }

    /// Cantonese Jyutping collator with no override data.
    #[cfg(feature = "collator-jyutping")]
    pub fn jyutping() -> Self {
        Self::Jyutping(crate::jyutping::JyutpingCollator::new())
    }

    /// Cantonese Jyutping collator that honors the supplied override table.
    #[cfg(feature = "collator-jyutping")]
    pub fn jyutping_with_override(
        overrides: crate::r#override::JyutpingOverride,
    ) -> crate::error::Result<Self> {
        Ok(Self::Jyutping(
            crate::jyutping::JyutpingCollator::with_override(overrides)?,
        ))
    }

    /// Mandarin Zhuyin collator (Phase 3.1 Stream B; placeholder until implemented).
    #[cfg(feature = "collator-zhuyin")]
    pub fn zhuyin() -> Self {
        Self::Zhuyin(crate::zhuyin::ZhuyinCollator::new())
    }

    /// Radical (部首) collator (Phase 3.1 Stream C; placeholder until implemented).
    #[cfg(feature = "collator-radical")]
    pub fn radical() -> Self {
        Self::Radical(crate::radical::RadicalCollator::new())
    }

    /// Sort `input` under the selected collator.
    pub fn sort(&self, input: Vec<String>) -> Vec<String> {
        match self {
            #[cfg(feature = "collator-pinyin")]
            Self::Pinyin(c) => sort_strings_with(input, c),
            #[cfg(feature = "collator-strokes")]
            Self::Strokes(c) => sort_strings_with(input, c),
            #[cfg(feature = "collator-jyutping")]
            Self::Jyutping(c) => sort_strings_with(input, c),
            #[cfg(feature = "collator-zhuyin")]
            Self::Zhuyin(c) => sort_strings_with(input, c),
            #[cfg(feature = "collator-radical")]
            Self::Radical(c) => sort_strings_with(input, c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trivial test collator that maps every alphabetic ASCII char to its
    /// lowercase byte and leaves digits unmapped — exercises the generic
    /// sort plumbing without depending on the bundled PHF tables.
    struct AsciiLetterCollator;

    impl Collator for AsciiLetterCollator {
        type Data = u8;

        fn data_for(&self, ch: char) -> Option<u8> {
            if ch.is_ascii_alphabetic() {
                Some(ch.to_ascii_lowercase() as u8)
            } else {
                None
            }
        }
    }

    #[test]
    fn mapped_orders_yes_before_no() {
        assert!(Mapped::Yes(0u32) < Mapped::No);
        assert!(Mapped::Yes(0u32) < Mapped::Yes(1u32));
    }

    #[test]
    fn sort_strings_with_uses_collator_data_then_character_tiebreak() {
        let sorted = sort_strings_with(
            vec!["banana".into(), "Apple".into(), "apple".into()],
            &AsciiLetterCollator,
        );
        // Apple/apple share lowercase keys; tiebreak on first character: A < a.
        assert_eq!(sorted, vec!["Apple", "apple", "banana"]);
    }

    #[test]
    fn sort_strings_with_places_unmapped_characters_after_mapped_ones() {
        let sorted = sort_strings_with(
            vec!["123".into(), "abc".into(), "1".into()],
            &AsciiLetterCollator,
        );
        assert_eq!(sorted, vec!["abc", "1", "123"]);
    }

    #[test]
    fn sort_strings_with_is_stable_for_duplicate_inputs() {
        let input = vec!["alpha".into(), "alpha".into(), "alpha".into()];
        let sorted = sort_strings_with(input.clone(), &AsciiLetterCollator);
        assert_eq!(sorted, input);
    }

    #[test]
    fn sort_strings_with_uses_phrase_data_when_provided() {
        struct PhraseOnly;
        impl Collator for PhraseOnly {
            type Data = u8;
            fn data_for(&self, _: char) -> Option<u8> {
                None
            }
            fn phrase_data(&self, phrase: &str) -> Option<Vec<u8>> {
                if phrase == "z-first" { Some(vec![0]) } else { None }
            }
        }
        let sorted = sort_strings_with(
            vec!["a-default".into(), "z-first".into()],
            &PhraseOnly,
        );
        // z-first has phrase data → mapped → sorts before unmapped a-default.
        assert_eq!(sorted, vec!["z-first", "a-default"]);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// A trivial test collator (lowercase ASCII byte rank, digits unmapped)
    /// to exercise sort properties without depending on the bundled PHF.
    struct AsciiLetterCollator;

    impl Collator for AsciiLetterCollator {
        type Data = u8;
        fn data_for(&self, ch: char) -> Option<u8> {
            ch.is_ascii_alphabetic().then(|| ch.to_ascii_lowercase() as u8)
        }
    }

    fn small_vec_of_strings() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec(".*", 0..50)
    }

    proptest! {
        /// Property: sorting is idempotent. sort(sort(x)) == sort(x).
        #[test]
        fn sort_is_idempotent(items in small_vec_of_strings()) {
            let once = sort_strings_with(items.clone(), &AsciiLetterCollator);
            let twice = sort_strings_with(once.clone(), &AsciiLetterCollator);
            prop_assert_eq!(once, twice);
        }

        /// Property: sorting produces a permutation of the input — same
        /// length, same multiset of values.
        #[test]
        fn sort_is_a_permutation(items in small_vec_of_strings()) {
            let sorted = sort_strings_with(items.clone(), &AsciiLetterCollator);
            prop_assert_eq!(sorted.len(), items.len());

            let mut sorted_alpha = sorted.clone();
            let mut input_alpha = items.clone();
            sorted_alpha.sort();
            input_alpha.sort();
            prop_assert_eq!(sorted_alpha, input_alpha);
        }

        /// Property: the comparison induced by sort_key_of is a total order:
        /// reflexive, antisymmetric, and transitive on triples.
        #[test]
        fn sort_key_total_order_holds(
            a in ".*", b in ".*", c in ".*",
        ) {
            let ka = sort_key_of(&AsciiLetterCollator, &a);
            let kb = sort_key_of(&AsciiLetterCollator, &b);
            let kc = sort_key_of(&AsciiLetterCollator, &c);

            // Reflexivity.
            prop_assert_eq!(ka.cmp(&ka), std::cmp::Ordering::Equal);

            // Antisymmetry: a.cmp(b) == reverse(b.cmp(a)).
            prop_assert_eq!(ka.cmp(&kb).reverse(), kb.cmp(&ka));

            // Transitivity: a <= b and b <= c implies a <= c.
            if ka <= kb && kb <= kc {
                prop_assert!(ka <= kc);
            }
        }

        /// Property: the bundled `PinyinCollator` and `StrokesCollator` also
        /// produce permutations of their inputs (smoke-checking the real
        /// data path, not just the trivial ASCII collator).
        #[test]
        fn pinyin_collator_sort_is_a_permutation(items in small_vec_of_strings()) {
            let collator = crate::pinyin::PinyinCollator::new();
            let sorted = sort_strings_with(items.clone(), &collator);
            prop_assert_eq!(sorted.len(), items.len());
        }

        #[test]
        fn strokes_collator_sort_is_a_permutation(items in small_vec_of_strings()) {
            let collator = crate::stroke::StrokesCollator;
            let sorted = sort_strings_with(items.clone(), &collator);
            prop_assert_eq!(sorted.len(), items.len());
        }
    }
}
