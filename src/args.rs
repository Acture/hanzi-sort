use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

use hanzi_sort::{
    Align, AnyCollator, FormatConfig, HanziSortError, InputSource, PinyinOverride, Result,
    RuntimeConfig,
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

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq, Default)]
pub enum CliSortMode {
    /// Hanyu Pinyin: primary tone3 reading per character (default).
    #[default]
    Pinyin,
    /// Total stroke count per character.
    Strokes,
    /// Cantonese Jyutping from Unihan kCantonese.
    #[cfg(feature = "collator-jyutping")]
    Jyutping,
    /// Mandarin Zhuyin / Bopomofo, derived from the bundled pinyin data.
    #[cfg(feature = "collator-zhuyin")]
    Zhuyin,
    /// Kangxi radical index plus residual stroke count (Unihan kRSUnicode).
    #[cfg(feature = "collator-radical")]
    Radical,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Generate a shell completion script for the given shell to stdout.
    Completions {
        /// The target shell (bash, zsh, fish, powershell, elvish).
        #[arg(value_enum)]
        shell: Shell,
    },
}

const AFTER_HELP: &str = "EXAMPLES:
  Sort by pinyin (default):
    hanzi-sort -t 汉字 张三 赵四

  Sort piped input:
    cat names.txt | hanzi-sort

  Sort by stroke count, single column:
    hanzi-sort -t 天 一 十 --sort-by strokes --columns 1 --entry-width 2 --blank-every 0

  Resolve a polyphonic phrase via override:
    hanzi-sort -t 重庆 银行 --config ./override.toml

  Reverse + dedup:
    hanzi-sort -f names.txt -u -r

  Merge two files, dedup, then sort by strokes:
    hanzi-sort -f names.txt -f extra.txt -u --sort-by strokes

  Full pipeline (read a file, sort by strokes, lay out a 3-column grid, write a file):
    hanzi-sort -f names.txt -o sorted.txt --sort-by strokes --columns 3 --entry-width 6 --align left --blank-every 5

  Generate shell completions:
    hanzi-sort completions bash > /usr/local/etc/bash_completion.d/hanzi-sort";

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Sort Chinese text by Hanyu Pinyin or stroke count, with deterministic \
                  tie-breaking and phrase-level overrides for polyphonic characters.",
    after_help = AFTER_HELP,
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
    #[arg(
        short = 'f',
        long = "file",
        value_name = "FILE",
        help = "Input file path (can be multiple, '-' reads stdin)",
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

    #[arg(
        long = "columns",
        help = "Number of entries per row",
        default_value_t = FormatConfig::default().columns_per_row
    )]
    pub columns_per_row: usize,

    #[arg(
        long = "blank-every",
        help = "Insert a blank line every N rows; use 0 to disable"
    )]
    pub blank_per: Option<usize>,

    #[arg(
        long = "entry-width",
        help = "Pad each entry to this display width",
        default_value_t = FormatConfig::default().entry_width
    )]
    pub entry_width: usize,

    #[arg(
        long = "align",
        value_enum,
        help = "Alignment strategy",
        default_value_t = CliAlign::Center
    )]
    pub align: CliAlign,

    #[arg(
        long = "padding-char",
        help = "Character used for padding",
        default_value_t = FormatConfig::default().padding_char
    )]
    pub padding_char: char,

    #[arg(
        long = "separator",
        help = "Character used to separate entries (default: tab)"
    )]
    pub separator: Option<char>,

    #[arg(
        long = "line-ending",
        help = "Line ending character (default: newline)"
    )]
    pub line_ending: Option<char>,

    #[arg(
        long = "sort-by",
        value_enum,
        help = "Sort strategy",
        default_value_t = CliSortMode::Pinyin
    )]
    pub sort_by: CliSortMode,

    #[arg(
        short = 'r',
        long = "reverse",
        help = "Reverse the sorted output"
    )]
    pub reverse: bool,

    #[arg(
        short = 'u',
        long = "unique",
        help = "Remove adjacent duplicates from the sorted output (like sort -u)"
    )]
    pub unique: bool,
}

impl CliArgs {
    pub fn has_input(&self) -> bool {
        !self.input_files.is_empty() || !self.input_texts.is_empty()
    }

    pub fn into_runtime_parts(self) -> Result<(RuntimeConfig, Option<PathBuf>)> {
        let input = if !self.input_files.is_empty() {
            InputSource::Files(self.input_files)
        } else if !self.input_texts.is_empty() {
            InputSource::Text(self.input_texts)
        } else {
            InputSource::Stdin
        };

        let defaults = FormatConfig::default();
        let format = FormatConfig {
            columns_per_row: self.columns_per_row,
            blank_per: self.blank_per.map_or(defaults.blank_per, |n| (n > 0).then_some(n)),
            entry_width: self.entry_width,
            align: self.align.into(),
            padding_char: self.padding_char,
            separator: self.separator.unwrap_or(defaults.separator),
            line_ending: self.line_ending.unwrap_or(defaults.line_ending),
        };

        let collator = build_collator(self.sort_by, self.config_path.as_deref())?;
        let config = RuntimeConfig::new(input, format, collator)?
            .with_unique(self.unique)
            .with_reverse(self.reverse);
        Ok((config, self.output_path))
    }
}

fn build_collator(
    sort_by: CliSortMode,
    config_path: Option<&std::path::Path>,
) -> Result<AnyCollator> {
    let reject_override = |scheme: &str| -> HanziSortError {
        HanziSortError::InvalidArgument(format!(
            "--config is not supported with --sort-by {scheme}",
        ))
    };
    match sort_by {
        CliSortMode::Pinyin => match config_path {
            Some(path) => {
                let override_data = PinyinOverride::load_from_file(path)?;
                AnyCollator::pinyin_with_override(override_data)
            }
            None => Ok(AnyCollator::pinyin()),
        },
        CliSortMode::Strokes => {
            if config_path.is_some() {
                return Err(reject_override("strokes"));
            }
            Ok(AnyCollator::strokes())
        }
        #[cfg(feature = "collator-jyutping")]
        CliSortMode::Jyutping => match config_path {
            Some(path) => {
                let override_data = hanzi_sort::JyutpingOverride::load_from_file(path)?;
                AnyCollator::jyutping_with_override(override_data)
            }
            None => Ok(AnyCollator::jyutping()),
        },
        #[cfg(feature = "collator-zhuyin")]
        CliSortMode::Zhuyin => {
            if config_path.is_some() {
                return Err(reject_override("zhuyin"));
            }
            Ok(AnyCollator::zhuyin())
        }
        #[cfg(feature = "collator-radical")]
        CliSortMode::Radical => {
            if config_path.is_some() {
                return Err(reject_override("radical"));
            }
            Ok(AnyCollator::radical())
        }
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
        assert!(matches!(config.collator, AnyCollator::Pinyin(_)));
        assert_eq!(output_path, Some(PathBuf::from("sorted.txt")));
    }

    #[test]
    fn supports_stroke_sort_mode() {
        let args = CliArgs::parse_from(["hanzi-sort", "-t", "十", "一", "--sort-by", "strokes"]);
        let (config, _) = args
            .into_runtime_parts()
            .expect("runtime config should be created");

        assert!(matches!(config.collator, AnyCollator::Strokes(_)));
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

        // Behavior assertion: with the override, "重" sorts as "chong2", so
        // pairing it with a "y..." input would put "重要" before. We confirm
        // load by checking the constructed collator type and exercising it.
        assert!(matches!(config.collator, AnyCollator::Pinyin(_)));
        let sorted = config.collator.sort(vec!["银行".to_string(), "重要".to_string()]);
        assert_eq!(sorted, vec!["重要", "银行"]);
    }

    #[test]
    fn rejects_override_with_stroke_mode() {
        let temp = TempWorkspace::new();
        let override_path = temp.path().join("override.toml");
        fs::write(&override_path, "[char_override]\n'重' = 'chong2'\n")
            .expect("override file should be written");

        let args = CliArgs::parse_from([
            "hanzi-sort",
            "-t",
            "重",
            "--sort-by",
            "strokes",
            "--config",
            override_path.to_str().expect("path should be valid UTF-8"),
        ]);
        let error = args
            .into_runtime_parts()
            .expect_err("override + strokes should fail");
        assert!(
            error.to_string().contains("--config is not supported with --sort-by strokes"),
            "unexpected: {error}"
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
