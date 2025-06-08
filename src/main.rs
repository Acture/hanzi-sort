mod config;
mod format;
mod pinyin;
mod sort;

mod generated;
mod r#override;
mod args;

use crate::args::Args;
use clap::Parser;

fn main() {
	let args = Args::parse();
	println!("Parsed arguments: {:?}", args);
}
