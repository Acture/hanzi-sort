use crate::format::Align;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
	#[arg(short = 'f', long = "file", help = "Input file or directory path (can be multiple)")]
	pub input_files: Vec<String>,

	#[arg(short = 't', long = "text", help = "Directly input text data (can be multiple)")]
	pub input_texts: Vec<String>,

	#[arg(short = 'o', long = "output", help = "Output file path, defaults to stdout")]
	pub output_path: Option<String>,

	#[arg(short = 'c', long = "config", help = "Override PinYin configuration file path")]
	pub config_path: Option<PathBuf>,

	#[arg(long = "columns", help = "Number of entries per row")]
	pub columns_per_row: Option<usize>,

	#[arg(long = "blank-every", help = "Insert a blank line every N lines")]
	pub blank_per: Option<usize>,

	#[arg(long = "entry-width", help = "Pad each entry to this width (excluding spacing)")]
	pub entry_width: Option<usize>,

	#[arg(
		long = "align",
		value_enum,
		help = "Alignment strategy: left, center, right, even"
	)]
	pub align: Option<Align>,

	#[arg(long = "padding-char", default_value = " ", help = "Character used for padding")]
	pub padding_char: Option<char>,

	#[arg(long = "separator", default_value = "\t", help = "Character used to separate entries")]
	pub separator: Option<char>,

	#[arg(long = "line-ending", default_value = "\n", help = "Line ending character")]
	pub line_ending: Option<char>,
}