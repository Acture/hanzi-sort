#[cfg(feature = "collator-jyutping")]
pub mod jyutping_map;
pub mod pinyin_map;
#[cfg(feature = "collator-radical")]
pub mod radical_map;
pub mod stroke_map;
#[cfg(feature = "collator-zhuyin")]
pub mod zhuyin_map;
