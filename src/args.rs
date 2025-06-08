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

	#[arg(long = "spaced-every", help = "Insert a blank line every N lines")]
	pub spaced_every: Option<usize>,

	#[arg(
		long = "fixed-width",
		help = "Set fixed width for each entry, useful for aligning columns"
	)]
	pub fixed_width: Option<usize>,

	#[arg(
		long = "align",
		default_value = "left",
		help = "Alignment of entries: left, center, right"
	)]
	pub align: String,
}