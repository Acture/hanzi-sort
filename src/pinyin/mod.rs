mod collator;
mod key;
mod lookup;
mod model;

pub use collator::PinyinCollator;
pub use model::PinYinRecord;

/// Encoder for the surname-aware [`crate::NameCollator`], which reuses the
/// pinyin `u128` packing for its built-in surname readings.
#[cfg(feature = "collator-names")]
pub(crate) use key::encode_primary_pinyin_unchecked;
