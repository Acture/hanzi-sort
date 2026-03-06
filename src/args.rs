use clap::Parser;
use std::path::PathBuf;

use pinyin_sort::Align;

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
        help = "Override PinYin configuration file path"
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
    pub align: Option<Align>,

    #[arg(long = "padding-char", help = "Character used for padding")]
    pub padding_char: Option<char>,

    #[arg(long = "separator", help = "Character used to separate entries")]
    pub separator: Option<char>,

    #[arg(long = "line-ending", help = "Line ending character")]
    pub line_ending: Option<char>,
}
