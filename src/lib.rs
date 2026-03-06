//! ```compile_fail
//! use pinyin_sort::SortKey;
//! ```
//!
//! ```compile_fail
//! use pinyin_sort::SortToken;
//! ```
//!
//! ```compile_fail
//! use pinyin_sort::compare_sort_key;
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
