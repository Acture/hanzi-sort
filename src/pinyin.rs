use crate::generated::pinyin_map::PINYIN_MAP;
use derive_builder::Builder;

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct PinYinRecord {
	pub pinyin: Vec<&'static str>,
	pub character: char,
}

pub fn pinyin_of<T: ToString + ?Sized>(s: &T) -> Vec<PinYinRecord> {
	// This function should convert a string to its Pinyin representation.
	// The implementation is not provided here, as it depends on external libraries or custom logic.
	// You can use libraries like `pinyin` or `pinyin-rs` for this purpose.
	s.to_string()
		.chars()
		.filter_map(|c| {
			PINYIN_MAP.get(&(c as u32)).map(|(_char, pinyin_vec)| {
				PinYinRecord {
					pinyin: pinyin_vec.to_vec(),
					character: c,
				}
			})
		})
		.collect()
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_pinyin() {
		let input = "汉字";

		let expected = vec![
			PinYinRecordBuilder::default()
				.pinyin(vec!["han4"])
				.character('汉')
				.build()
				.expect("Failed to build PinYinRecord for 汉"),
			PinYinRecordBuilder::default()
				.pinyin(vec!["zi4"])
				.character('字')
				.build()
				.expect("Failed to build PinYinRecord for 字"),
		];
		let result = pinyin_of(input);
		assert_eq!(result, expected);
	}

	#[test]
	fn test_empty_string() {
		let input = "";
		let expected = vec![];
		let result = pinyin_of(input);
		assert_eq!(result, expected);
	}
}