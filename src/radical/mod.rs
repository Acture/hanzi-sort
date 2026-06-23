//! Kangxi radical (部首) collator, backed by Unihan `kRSUnicode`.
//!
//! Sorts Hanzi by the Unicode Unihan `kRSUnicode` Kangxi radical/stroke
//! index generated at build time. The sort key packs `radical * 1000 +
//! residual_strokes` so ordinary integer ordering gives radical-first,
//! residual-stroke-second ordering.

use crate::collator::Collator;

/// Sorts Hanzi by Kangxi radical and residual stroke count.
#[derive(Debug, Clone, Copy, Default)]
pub struct RadicalCollator;

impl RadicalCollator {
	pub fn new() -> Self {
		Self
	}
}

impl Collator for RadicalCollator {
	type Data = u32;

	fn data_for(&self, character: char) -> Option<u32> {
		crate::generated::radical_map::RADICAL_MAP
			.get(&(character as u32))
			.copied()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::collator::sort_strings_with;

	fn packed(radical: u32, residual: u32) -> u32 {
		radical * 1000 + residual
	}

	#[test]
	fn data_for_returns_known_radicals() {
		let collator = RadicalCollator;

		assert_eq!(collator.data_for('一'), Some(packed(1, 0)));
		assert_eq!(collator.data_for('中'), Some(packed(2, 3)));
		assert_eq!(collator.data_for('口'), Some(packed(30, 0)));
		assert_eq!(collator.data_for('水'), Some(packed(85, 0)));
		assert_eq!(collator.data_for('汉'), Some(packed(85, 2)));
	}

	#[test]
	fn unknown_chars_return_none() {
		let collator = RadicalCollator;

		assert_eq!(collator.data_for('a'), None);
		assert_eq!(collator.data_for('1'), None);
	}

	#[test]
	fn sort_orders_by_radical_then_residual_strokes() {
		let sorted = sort_strings_with(
			vec![
				"汉".to_string(),
				"水".to_string(),
				"口".to_string(),
				"中".to_string(),
				"一".to_string(),
			],
			&RadicalCollator,
		);

		assert_eq!(sorted, vec!["一", "中", "口", "水", "汉"]);
	}
}
