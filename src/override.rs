use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct PinyinOverride {
	pub char_override: HashMap<char, Vec<String>>,
	pub phrase_override: HashMap<String, Vec<String>>,
}