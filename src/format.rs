use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::error::{PinyinSortError, Result};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Align {
    Left,
    #[default]
    Center,
    Right,
    Even,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FormatConfig {
    pub columns_per_row: usize,
    pub blank_per: Option<usize>,
    pub entry_width: usize,
    pub align: Align,
    pub padding_char: char,
    pub separator: char,
    pub line_ending: char,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            columns_per_row: 6,
            blank_per: Some(7),
            entry_width: 4,
            align: Align::Center,
            padding_char: ' ',
            separator: '\t',
            line_ending: '\n',
        }
    }
}

impl FormatConfig {
    pub fn validate(self) -> Result<Self> {
        if self.columns_per_row == 0 {
            return Err(PinyinSortError::InvalidArgument(
                "--columns must be greater than 0".to_string(),
            ));
        }

        if self.entry_width == 0 {
            return Err(PinyinSortError::InvalidArgument(
                "--entry-width must be greater than 0".to_string(),
            ));
        }

        if UnicodeWidthChar::width(self.padding_char).unwrap_or(0) != 1 {
            return Err(PinyinSortError::InvalidArgument(
                "--padding-char must have display width 1".to_string(),
            ));
        }

        Ok(self)
    }
}

pub fn format_cell(item: &str, format_config: &FormatConfig) -> String {
    let entry_width = UnicodeWidthStr::width(item);
    if entry_width >= format_config.entry_width {
        return item.to_string();
    }

    let total_padding = format_config.entry_width - entry_width;

    match format_config.align {
        Align::Left => format!(
            "{item}{}",
            padding(total_padding, format_config.padding_char)
        ),
        Align::Right => format!(
            "{}{}",
            padding(total_padding, format_config.padding_char),
            item
        ),
        Align::Center => {
            let left = total_padding / 2;
            let right = total_padding - left;
            format!(
                "{}{}{}",
                padding(left, format_config.padding_char),
                item,
                padding(right, format_config.padding_char)
            )
        }
        Align::Even => format_even(item, total_padding, format_config.padding_char),
    }
}

pub fn format_items<T: AsRef<str>>(items: &[T], format_config: &FormatConfig) -> String {
    if items.is_empty() {
        return String::new();
    }

    items
        .iter()
        .map(|item| format_cell(item.as_ref(), format_config))
        .collect::<Vec<_>>()
        .chunks(format_config.columns_per_row)
        .enumerate()
        .flat_map(|(index, chunk)| {
            let mut row = vec![chunk.join(&format_config.separator.to_string())];
            if let Some(blank_per) = format_config.blank_per
                && blank_per > 0
                && (index + 1) % blank_per == 0
            {
                row.push(String::new());
            }
            row
        })
        .collect::<Vec<_>>()
        .join(&format_config.line_ending.to_string())
}

fn format_even(item: &str, total_padding: usize, padding_char: char) -> String {
    let chars: Vec<char> = item.chars().collect();
    if chars.len() < 2 {
        let left = total_padding / 2;
        let right = total_padding - left;
        return format!(
            "{}{}{}",
            padding(left, padding_char),
            item,
            padding(right, padding_char)
        );
    }

    let slot_count = chars.len() + 1;
    let base_padding = total_padding / slot_count;
    let mut slots = vec![base_padding; slot_count];
    let mut remainder = total_padding % slot_count;

    for slot in slots.iter_mut().take(chars.len()).skip(1) {
        if remainder == 0 {
            break;
        }
        *slot += 1;
        remainder -= 1;
    }

    for edge_index in [0, chars.len()] {
        if remainder == 0 {
            break;
        }
        slots[edge_index] += 1;
        remainder -= 1;
    }

    let mut output = String::new();
    output.push_str(&padding(slots[0], padding_char));
    for (index, ch) in chars.iter().enumerate() {
        output.push(*ch);
        if index + 1 < chars.len() {
            output.push_str(&padding(slots[index + 1], padding_char));
        }
    }
    output.push_str(&padding(slots[chars.len()], padding_char));
    output
}

fn padding(width: usize, padding_char: char) -> String {
    padding_char.to_string().repeat(width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cell_alignment() {
        let left = FormatConfig {
            entry_width: 4,
            align: Align::Left,
            ..Default::default()
        };
        assert_eq!(format_cell("甲", &left), "甲  ");

        let right = FormatConfig {
            entry_width: 4,
            align: Align::Right,
            ..Default::default()
        };
        assert_eq!(format_cell("甲", &right), "  甲");

        let center = FormatConfig {
            entry_width: 6,
            align: Align::Center,
            ..Default::default()
        };
        assert_eq!(format_cell("test", &center), " test ");
    }

    #[test]
    fn test_even_alignment_preserves_display_width() {
        let format_config = FormatConfig {
            entry_width: 10,
            align: Align::Even,
            ..Default::default()
        };
        let formatted = format_cell("测试", &format_config);
        assert_eq!(UnicodeWidthStr::width(formatted.as_str()), 10);
        assert_eq!(formatted, "  测  试  ");
    }

    #[test]
    fn test_format_items() {
        let items = vec!["甲", "乙", "丙", "丁"];
        let format_config = FormatConfig {
            columns_per_row: 2,
            entry_width: 2,
            align: Align::Left,
            blank_per: Some(1),
            ..Default::default()
        };
        let formatted = format_items(&items, &format_config);
        assert_eq!(formatted, "甲\t乙\n\n丙\t丁\n");
    }

    #[test]
    fn test_validate_rejects_zero_columns() {
        let result = FormatConfig {
            columns_per_row: 0,
            ..Default::default()
        }
        .validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rejects_zero_entry_width() {
        let result = FormatConfig {
            entry_width: 0,
            ..Default::default()
        }
        .validate();
        assert_eq!(
            result
                .expect_err("zero entry width should fail")
                .to_string(),
            "--entry-width must be greater than 0"
        );
    }

    #[test]
    fn test_validate_rejects_wide_padding_char() {
        let result = FormatConfig {
            padding_char: '汉',
            ..Default::default()
        }
        .validate();
        assert_eq!(
            result
                .expect_err("wide padding char should fail")
                .to_string(),
            "--padding-char must have display width 1"
        );
    }
}
