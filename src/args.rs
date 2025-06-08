use crate::format::Align;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
	#[arg(short = 'f', long = "file", help = "Input file or directory path (can be multiple)")]
	pub input_files: Vec<String>,

	#[arg(short = 't', long = "text", help = "Directly input text data (can be multiple)")]
	pub input_texts: Vec<String>,

	#[arg(short = 'o', long = "output", help = "Output file path, defaults to stdout")]
	pub output_path: Option<String>,

	#[arg(short = 'c', long = "config", help = "Override PinYin configuration file path")]
	pub config_path: Option<String>,

	#[arg(long = "columns", help = "Number of entries per row")]
	pub columns: Option<usize>,

	#[arg(long = "blank_every", help = "Insert a blank line every N lines")]
	pub blank_every: Option<usize>,

	#[arg(long = "entry-width", help = "Pad each entry to this width (excluding spacing)")]
	pub entry_width: Option<usize>,

	#[arg(
		long = "align",
		value_enum,
		default_value_t = Align::Left,
		help = "Alignment strategy: left, center, right, even"
	)]
	pub align: Align,

	#[arg(long = "cell-width", help = "Total width of each output cell (entry + spacing)")]
	pub cell_width: Option<usize>,

	#[arg(long = "pad-char", default_value = " ", help = "Character used for padding")]
	pub pad_char: char,

	#[arg(long = "line-ending", default_value = "\n", help = "Line ending character")]
	pub line_ending: char,
}