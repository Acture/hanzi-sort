//! `hanzi-sort` sorts Chinese text by Hanyu Pinyin or stroke count.
//!
//! The CLI is the primary product, but the same core sorter is available as a
//! Rust library for pipelines, tests, and small embedding use cases.
//!
//! ```rust
//! use hanzi_sort::{AnyCollator, sort_strings_with, StrokesCollator};
//!
//! let collator = StrokesCollator;
//! let sorted = sort_strings_with(
//!     vec!["一".into(), "十".into(), "天".into()],
//!     &collator,
//! );
//! assert_eq!(sorted, vec!["一", "十", "天"]);
//!
//! // Or dispatch through `AnyCollator` when the strategy is chosen at runtime.
//! let sorted = AnyCollator::pinyin().sort(vec!["赵四".into(), "汉字".into()]);
//! assert_eq!(sorted, vec!["汉字", "赵四"]);
//! ```
//!
//! Phrase-level overrides for polyphonic phrases like `重庆` or `银行` are
//! supplied through [`PinyinOverride`] and [`AnyCollator::pinyin_with_override`].
//!
//! ```compile_fail
//! use hanzi_sort::PinyinContext;
//! ```
//!
//! ```compile_fail
//! use hanzi_sort::SortMode;
//! ```
//!
//! ```compile_fail
//! use hanzi_sort::sort_strings_by;
//! ```

pub mod app;

mod collator;
mod config;
mod error;
mod format;
mod generated;
mod input;
mod r#override;
#[cfg(feature = "collator-pinyin")]
mod pinyin;
#[cfg(feature = "collator-strokes")]
mod stroke;
#[cfg(feature = "collator-jyutping")]
mod jyutping;
#[cfg(feature = "collator-zhuyin")]
mod zhuyin;
#[cfg(feature = "collator-radical")]
mod radical;

pub use collator::{AnyCollator, CharToken, Collator, Mapped, SortKey, sort_key_of, sort_strings_with};
pub use config::{InputSource, RuntimeConfig};
pub use error::{HanziSortError, Result};
pub use format::{Align, FormatConfig};
pub use r#override::PinyinOverride;
#[cfg(feature = "collator-jyutping")]
pub use r#override::JyutpingOverride;
#[cfg(feature = "collator-pinyin")]
pub use pinyin::{PinYinRecord, PinyinCollator};
#[cfg(feature = "collator-strokes")]
pub use stroke::StrokesCollator;
#[cfg(feature = "collator-jyutping")]
pub use jyutping::JyutpingCollator;
#[cfg(feature = "collator-zhuyin")]
pub use zhuyin::ZhuyinCollator;
#[cfg(feature = "collator-radical")]
pub use radical::RadicalCollator;
