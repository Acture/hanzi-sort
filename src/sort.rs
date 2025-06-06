use crate::pinyin::pinyin;
pub fn sort_by_pinyin<T: ToString>(input: Vec<T>) -> Vec<T> {
	let mut with_keys: Vec<(Vec<String>, T)> = input
		.into_iter()
		.map(|item| {
			let key = pinyin(&item.to_string());
			(key, item)
		})
		.collect();

	with_keys.sort_by(|a, b| a.0.cmp(&b.0));

	with_keys.into_iter().map(|(_, item)| item).collect()
}
