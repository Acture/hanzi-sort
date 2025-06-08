use clap::ValueEnum;

#[derive(Copy, Clone, Debug, ValueEnum, Default)]
pub enum Align {
	Left,
	#[default]
	Center,
	Right,
	Even,
}

#[derive(Copy, Clone, Debug)]
pub struct FormatConfig {
	pub columns_per_row: Option<usize>,
	pub blank_every: Option<usize>,
	pub entry_width: Option<usize>,
	pub align: Align,
	pub padding_char: char,
	pub separator: char,
	pub line_ending: char,
}

impl Default for FormatConfig {
	fn default() -> Self {
		Self {
			columns_per_row: Some(6),
			blank_every: Some(7),
			entry_width: Some(4),
			align: Align::Center,
			padding_char: ' ',
			separator: '\t',
			line_ending: '\n',
		}
	}
}

pub fn format_cell<T: ToString + Sized>(item: &T, format_config: &FormatConfig) -> String {
	let entry = item.to_string();
	let entry_len = entry.chars().count();
	let target_width = format_config.entry_width.unwrap_or(entry_len);

	if entry_len >= target_width {
		return entry;
	}

	let total_padding = target_width - entry_len;


	match format_config.align {
		Align::Left => format!("{}{}", format_config.padding_char.to_string().repeat(total_padding), entry),
		Align::Right => format!("{}{}", entry, format_config.padding_char.to_string().repeat(total_padding)),
		Align::Center => {
			let left = total_padding / 2;
			let right = total_padding - left;
			format!(
				"{}{}{}",
				format_config.padding_char.to_string().repeat(left),
				entry,
				format_config.padding_char.to_string().repeat(right)
			)
		}
		Align::Even => {
			let each_insert = total_padding / entry_len;
			if each_insert == 0 {
				// If the padding is less than the number of characters, fallback to center alignment
				let left = total_padding / 2;
				let right = total_padding - left;
				return format!(
					"{}{}{}",
					format_config.padding_char.to_string().repeat(left),
					entry,
					format_config.padding_char.to_string().repeat(right)
				);
			}

			entry.chars().map(|c| c.to_string()).collect::<Vec<String>>().join(
				&format_config.padding_char.to_string().repeat(each_insert)
			)
		}
	}
}

pub fn format<T: ToString + Sized>(items: Vec<T>, format_config: Option<FormatConfig>) -> String {
	let format_config = format_config.unwrap_or_default();

	let mut result = String::new();

	items
		.iter()
		.map(|item| format_cell(item, &format_config))
		.collect::<Vec<String>>()
		.chunks(format_config.columns_per_row.unwrap_or(items.len()))
		.enumerate()
		.flat_map(|(i, chunk)| {
			let mut row = vec![chunk.join(&format_config.separator.to_string())];
			if let Some(n) = format_config.blank_every {
				if n > 0 && (i + 1) % n == 0 {
					row.push(String::new()); // 插入空行
				}
			}
			row
		})
		.collect::<Vec<String>>()
		.join(&format_config.line_ending.to_string())
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_format_cell() {
		let mut format_config = FormatConfig {
			entry_width: Some(6),
			align: Align::Center,
			..Default::default()
		};

		assert_eq!(format_cell(&"test", &format_config), " test ");

		format_config.align = Align::Right;

		assert_eq!(format_cell(&"test", &format_config), "test  ");

		let format_config = FormatConfig {
			entry_width: Some(6),
			align: Align::Left,
			..Default::default()
		};
		assert_eq!(format_cell(&"test", &format_config), "  test");
		let format_config = FormatConfig {
			entry_width: Some(12),
			align: Align::Even,
			..Default::default()
		};
		assert_eq!(format_cell(&"test", &format_config), "t  e  s  t");
	}


	#[test]
	fn test_format() {
		let items = vec!["test1", "test2", "test3", "test4"];
		let format_config = FormatConfig {
			columns_per_row: Some(2),
			entry_width: Some(6),
			align: Align::Center,
			blank_every: Some(1),
			..Default::default()
		};
		let formatted = format(items, Some(format_config));
		println!("Formatted output:\n{}", formatted);
		assert_eq!(formatted, "test1 \ttest2 \n\ntest3 \ttest4 ");
	}
}