use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct PinyinOverride {
	pub char_override: HashMap<char, String>,
	pub phrase_override: HashMap<String, Vec<String>>,
}
impl PinyinOverride {
	pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
		let file_content = std::fs::read_to_string(path)?;
		let overrides: PinyinOverride = toml::from_str(&file_content)?;
		Ok(overrides)
	}
}

