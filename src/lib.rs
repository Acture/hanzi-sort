//! `hanzi-sort` sorts Chinese text by Hanyu Pinyin or stroke count.
//!
//! The CLI is the primary product, but the same core sorter is available as a
//! Rust library for pipelines, tests, and small embedding use cases.
//!
//! ```rust
//! use hanzi_sort::{sort_strings_by, PinyinContext, SortMode};
//!
//! let context = PinyinContext::default();
//! let sorted = sort_strings_by(
//!     vec!["一".into(), "十".into(), "天".into()],
//!     &context,
//!     SortMode::Strokes,
//! );
//!
//! assert_eq!(sorted, vec!["一", "十", "天"]);
//! ```
//!
//! `phrase_override` rules let you resolve polyphonic phrases like `重庆` or
//! `银行` without rewriting the core lookup tables.
//!
//! ```compile_fail
//! use hanzi_sort::SortKey;
//! ```
//!
//! ```compile_fail
//! use hanzi_sort::SortToken;
//! ```
//!
//! ```compile_fail
//! use hanzi_sort::compare_sort_key;
//! ```

pub mod app;

mod config;
mod error;
mod format;
mod generated;
mod input;
mod r#override;
mod pinyin;
mod sort;
mod stroke;

pub use config::{InputSource, RuntimeConfig};
pub use error::{PinyinSortError, Result};
pub use format::{Align, FormatConfig};
pub use r#override::PinyinOverride;
pub use pinyin::{PinYinRecord, PinyinContext};
pub use sort::{SortMode, sort_strings, sort_strings_by};
