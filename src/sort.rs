use crate::pinyin::{pinyin_of, PinYinRecord};
use std::cmp::Ordering;
pub fn sort_by_pinyin<T: ToString>(input: Vec<T>) -> Vec<T> {
	let mut with_keys: Vec<(Vec<PinYinRecord>, T)> = input
		.into_iter()
		.map(|item| {
			let key = pinyin_of(&item.to_string());
			(key, item)
		})
		.collect();

	with_keys.sort_by(|a, b| compare_pinyin(a.0.clone(), b.0.clone()));

	with_keys.into_iter().map(|(_, item)| item).collect()
}


pub fn compare_pinyin(a: Vec<PinYinRecord>, b: Vec<PinYinRecord>) -> Ordering {
	let len = a.len().min(b.len());

	for i in 0..len {
		let temp_a = String::new();
		let temp_b = String::new();
		let pa = a[i].pinyin.first().unwrap_or(&"");
		let pb = b[i].pinyin.first().unwrap_or(&"");

		match pa.cmp(pb) {
			Ordering::Equal => {
				// 拼音相同，比较原始字符，确保 "赵" 在 "照" 之前
				match a[i].character.cmp(&b[i].character) {
					Ordering::Equal => continue,
					ord => return ord,
				}
			}
			ord => return ord,
		}
	}

	// 若前缀都相等，短的排前面
	a.len().cmp(&b.len())
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sort_by_pinyin() {
		let input = vec!["汉字", "照相", "赵云", "赵四", "张三"];
		let sorted = sort_by_pinyin(input);
		assert_eq!(sorted, vec!["汉字", "张三", "照相", "赵四", "赵云"]);
	}

	#[test]
	fn test_compare_pinyin() {
		let a = pinyin_of("张三");
		let b = pinyin_of("赵四");
		assert_eq!(compare_pinyin(a, b), Ordering::Less);
	}
}