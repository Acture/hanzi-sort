use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use hanzi_sort::{
    Align, FormatConfig, InputSource, PinyinOverride, Result, RuntimeConfig, SortMode,
};

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum CliAlign {
    Left,
    Center,
    Right,
    Even,
}

impl From<CliAlign> for Align {
    fn from(value: CliAlign) -> Self {
        match value {
            CliAlign::Left => Self::Left,
            CliAlign::Center => Self::Center,
            CliAlign::Right => Self::Right,
            CliAlign::Even => Self::Even,
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum CliSortMode {
    Pinyin,
    Strokes,
}

impl From<CliSortMode> for SortMode {
    fn from(value: CliSortMode) -> Self {
        match value {
            CliSortMode::Pinyin => Self::Pinyin,
            CliSortMode::Strokes => Self::Strokes,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    #[arg(
        short = 'f',
        long = "file",
        value_name = "FILE",
        help = "Input file path (can be multiple)",
        conflicts_with = "input_texts"
    )]
    pub input_files: Vec<PathBuf>,

    #[arg(
		short = 't',
		long = "text",
		help = "Directly input text data (can be multiple)",
		num_args = 1..,
		conflicts_with = "input_files"
	)]
    pub input_texts: Vec<String>,

    #[arg(
        short = 'o',
        long = "output",
        help = "Output file path, defaults to stdout"
    )]
    pub output_path: Option<PathBuf>,

    #[arg(
        short = 'c',
        long = "config",
        help = "Override pronunciation configuration file path"
    )]
    pub config_path: Option<PathBuf>,

    #[arg(long = "columns", help = "Number of entries per row")]
    pub columns_per_row: Option<usize>,

    #[arg(
        long = "blank-every",
        help = "Insert a blank line every N lines; use 0 to disable"
    )]
    pub blank_per: Option<usize>,

    #[arg(long = "entry-width", help = "Pad each entry to this display width")]
    pub entry_width: Option<usize>,

    #[arg(
        long = "align",
        value_enum,
        help = "Alignment strategy: left, center, right, even"
    )]
    pub align: Option<CliAlign>,

    #[arg(long = "padding-char", help = "Character used for padding")]
    pub padding_char: Option<char>,

    #[arg(long = "separator", help = "Character used to separate entries")]
    pub separator: Option<char>,

    #[arg(long = "line-ending", help = "Line ending character")]
    pub line_ending: Option<char>,

    #[arg(
        long = "sort-by",
        value_enum,
        help = "Sort strategy: pinyin or strokes"
    )]
    pub sort_by: Option<CliSortMode>,
}

impl CliArgs {
    pub fn has_input(&self) -> bool {
        !self.input_files.is_empty() || !self.input_texts.is_empty()
    }

    pub fn into_runtime_parts(self) -> Result<(RuntimeConfig, Option<PathBuf>)> {
        let input = if !self.input_files.is_empty() {
            InputSource::Files(self.input_files)
        } else {
            InputSource::Text(self.input_texts)
        };

        let mut format = FormatConfig::default();
        if let Some(value) = self.columns_per_row {
            format.columns_per_row = value;
        }
        if let Some(value) = self.blank_per {
            format.blank_per = (value > 0).then_some(value);
        }
        if let Some(value) = self.entry_width {
            format.entry_width = value;
        }
        if let Some(value) = self.align {
            format.align = value.into();
        }
        if let Some(value) = self.padding_char {
            format.padding_char = value;
        }
        if let Some(value) = self.separator {
            format.separator = value;
        }
        if let Some(value) = self.line_ending {
            format.line_ending = value;
        }

        let override_data = self
            .config_path
            .as_deref()
            .map(PinyinOverride::load_from_file)
            .transpose()?;

        let sort_mode = self.sort_by.map(Into::into).unwrap_or(SortMode::Pinyin);
        let config = RuntimeConfig::with_sort_mode(input, format, override_data, sort_mode)?;
        Ok((config, self.output_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
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
                "hanzi-sort-args-test-{}-{}",
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

    #[test]
    fn detects_whether_any_input_was_provided() {
        let args = CliArgs::parse_from(["hanzi-sort", "-t", "赵四"]);
        assert!(args.has_input());

        let args = CliArgs::parse_from(["hanzi-sort"]);
        assert!(!args.has_input());
    }

    #[test]
    fn converts_zero_blank_every_to_none() {
        let args = CliArgs::parse_from(["hanzi-sort", "-t", "赵四", "--blank-every", "0"]);
        let (config, output_path) = args
            .into_runtime_parts()
            .expect("runtime config should be created");

        assert_eq!(config.format.blank_per, None);
        assert_eq!(output_path, None);
    }

    #[test]
    fn applies_format_option_overrides() {
        let args = CliArgs::parse_from([
            "hanzi-sort",
            "-t",
            "赵四",
            "--columns",
            "3",
            "--blank-every",
            "2",
            "--entry-width",
            "6",
            "--align",
            "right",
            "--padding-char",
            ".",
            "--separator",
            ",",
            "--line-ending",
            ";",
            "-o",
            "sorted.txt",
        ]);
        let (config, output_path) = args
            .into_runtime_parts()
            .expect("runtime config should be created");

        assert_eq!(config.format.columns_per_row, 3);
        assert_eq!(config.format.blank_per, Some(2));
        assert_eq!(config.format.entry_width, 6);
        assert_eq!(config.format.align, Align::Right);
        assert_eq!(config.format.padding_char, '.');
        assert_eq!(config.format.separator, ',');
        assert_eq!(config.format.line_ending, ';');
        assert_eq!(config.sort_mode, SortMode::Pinyin);
        assert_eq!(output_path, Some(PathBuf::from("sorted.txt")));
    }

    #[test]
    fn supports_stroke_sort_mode() {
        let args = CliArgs::parse_from(["hanzi-sort", "-t", "十", "一", "--sort-by", "strokes"]);
        let (config, _) = args
            .into_runtime_parts()
            .expect("runtime config should be created");

        assert_eq!(config.sort_mode, SortMode::Strokes);
    }

    #[test]
    fn loads_override_data_when_config_path_is_provided() {
        let temp = TempWorkspace::new();
        let override_path = temp.path().join("override.toml");
        fs::write(&override_path, "[char_override]\n'重' = 'chong2'\n")
            .expect("override file should be written");

        let args = CliArgs::parse_from([
            "hanzi-sort",
            "-t",
            "重要",
            "--config",
            override_path.to_str().expect("path should be valid UTF-8"),
        ]);
        let (config, _) = args
            .into_runtime_parts()
            .expect("override data should load");

        assert_eq!(
            config
                .override_data
                .as_ref()
                .and_then(|data| data.char_override.get(&'重')),
            Some(&"chong2".to_string())
        );
    }

    #[test]
    fn returns_override_load_errors() {
        let temp = TempWorkspace::new();
        let missing_path = temp.path().join("missing.toml");

        let args = CliArgs::parse_from([
            "hanzi-sort",
            "-t",
            "重要",
            "--config",
            missing_path.to_str().expect("path should be valid UTF-8"),
        ]);
        let error = args
            .into_runtime_parts()
            .expect_err("missing override file should fail");

        assert!(error.to_string().contains("failed to read override config"));
    }
}
