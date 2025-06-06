use crate::pinyin::{pinyin_of, PinYinRecord};
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


pub fn compare_pinyin(a: Vec<PinYinRecord>, b: Vec<PinYinRecord>) -> std::cmp::Ordering {
	if a.is_empty() && b.is_empty() {
		return std::cmp::Ordering::Equal;
	}
	if a.is_empty() {
		return std::cmp::Ordering::Greater;
	}
	if b.is_empty() {
		return std::cmp::Ordering::Less;
	}

	unimplemented!()
}
