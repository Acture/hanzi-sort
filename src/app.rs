use crate::config::RuntimeConfig;
use crate::error::Result;
use crate::format::format_items;
use crate::input::{Input, read_input_lines};

pub fn render(config: RuntimeConfig) -> Result<String> {
    let Input { headers, body } = read_input_lines(&config.input, &config.header)?;
    let mut sorted = config.collator.sort(body);
    if config.unique {
        sorted.dedup();
    }
    if config.reverse {
        sorted.reverse();
    }
    let formatted = format_items(&sorted, &config.format);

    if headers.is_empty() {
        return Ok(formatted);
    }

    let line_ending = config.format.line_ending.to_string();
    let header_block = headers.join(&line_ending);
    if formatted.is_empty() {
        Ok(header_block)
    } else {
        Ok(format!("{header_block}{line_ending}{formatted}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collator::AnyCollator;
    use crate::config::{HeaderSpec, InputSource, RuntimeConfig};
    use crate::format::{Align, FormatConfig};
    use crate::r#override::PinyinOverride;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempWorkspace {
        path: PathBuf,
    }

    impl TempWorkspace {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after UNIX_EPOCH")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hanzi-sort-app-test-{}-{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&path).expect("temporary directory should be created");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn single_column_format() -> FormatConfig {
        FormatConfig {
            columns_per_row: 1,
            blank_per: None,
            entry_width: 2,
            align: Align::Left,
            ..Default::default()
        }
    }

    #[test]
    fn renders_sorted_text_input() {
        let config = RuntimeConfig::new(
            InputSource::Text(vec![
                "赵四".to_string(),
                "张三".to_string(),
                "汉字".to_string(),
            ]),
            single_column_format(),
            AnyCollator::pinyin(),
        )
        .expect("runtime config should be created");

        let rendered = render(config).expect("render should succeed");
        assert_eq!(rendered, "汉字\n张三\n赵四");
    }

    #[test]
    fn renders_file_input_and_ignores_blank_lines() {
        let temp = TempWorkspace::new();
        let input_path = temp.path().join("names.txt");
        fs::write(&input_path, "赵四\n\n张三\n汉字\n").expect("input file should be written");

        let config = RuntimeConfig::new(
            InputSource::Files(vec![input_path]),
            single_column_format(),
            AnyCollator::pinyin(),
        )
        .expect("runtime config should be created");

        let rendered = render(config).expect("render should succeed");
        assert_eq!(rendered, "汉字\n张三\n赵四");
    }

    #[test]
    fn renders_with_overrides() {
        let collator = AnyCollator::pinyin_with_override(PinyinOverride {
            char_override: HashMap::new(),
            phrase_override: HashMap::from([(
                "重庆".to_string(),
                vec!["chong2".to_string(), "qing4".to_string()],
            )]),
        })
        .expect("override collator should construct");
        let config = RuntimeConfig::new(
            InputSource::Text(vec!["银行".to_string(), "重庆".to_string()]),
            single_column_format(),
            collator,
        )
        .expect("runtime config should be created");

        let rendered = render(config).expect("render should succeed");
        assert_eq!(rendered, "重庆\n银行");
    }

    #[test]
    fn applies_formatting_options_during_render() {
        let format = FormatConfig {
            columns_per_row: 2,
            blank_per: Some(1),
            entry_width: 4,
            align: Align::Right,
            padding_char: '.',
            separator: ',',
            line_ending: ';',
        };
        let config = RuntimeConfig::new(
            InputSource::Text(vec!["乙".to_string(), "甲".to_string(), "丙".to_string()]),
            format,
            AnyCollator::pinyin(),
        )
        .expect("runtime config should be created");

        let rendered = render(config).expect("render should succeed");
        assert_eq!(rendered, "..丙,..甲;;..乙;");
    }

    #[test]
    fn renders_stroke_sorted_output() {
        let config = RuntimeConfig::new(
            InputSource::Text(vec!["天".to_string(), "一".to_string(), "十".to_string()]),
            single_column_format(),
            AnyCollator::strokes(),
        )
        .expect("runtime config should be created");

        let rendered = render(config).expect("render should succeed");
        assert_eq!(rendered, "一\n十\n天");
    }

    #[test]
    fn skip_header_drops_leading_line_before_sorting() {
        let temp = TempWorkspace::new();
        let input_path = temp.path().join("names.csv");
        fs::write(&input_path, "name\n赵四\n汉字\n").expect("input file should be written");

        let config = RuntimeConfig::new(
            InputSource::Files(vec![input_path]),
            single_column_format(),
            AnyCollator::pinyin(),
        )
        .expect("runtime config should be created")
        .with_header(HeaderSpec { lines: 1, keep: false });

        let rendered = render(config).expect("render should succeed");
        assert_eq!(rendered, "汉字\n赵四");
    }

    #[test]
    fn keep_header_pins_leading_line_above_sorted_body() {
        let temp = TempWorkspace::new();
        let input_path = temp.path().join("names.csv");
        fs::write(&input_path, "name\n赵四\n汉字\n").expect("input file should be written");

        let config = RuntimeConfig::new(
            InputSource::Files(vec![input_path]),
            single_column_format(),
            AnyCollator::pinyin(),
        )
        .expect("runtime config should be created")
        .with_header(HeaderSpec { lines: 1, keep: true });

        let rendered = render(config).expect("render should succeed");
        assert_eq!(rendered, "name\n汉字\n赵四");
    }

    #[test]
    fn skip_header_applies_per_file_when_merging() {
        let temp = TempWorkspace::new();
        let first = temp.path().join("a.csv");
        let second = temp.path().join("b.csv");
        fs::write(&first, "hdr1\n赵四\n汉字\n").expect("first file should be written");
        fs::write(&second, "hdr2\n张三\n").expect("second file should be written");

        let config = RuntimeConfig::new(
            InputSource::Files(vec![first, second]),
            single_column_format(),
            AnyCollator::pinyin(),
        )
        .expect("runtime config should be created")
        .with_header(HeaderSpec { lines: 1, keep: false });

        let rendered = render(config).expect("render should succeed");
        assert_eq!(rendered, "汉字\n张三\n赵四");
    }
}
