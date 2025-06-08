mod config;
mod format;
mod pinyin;
mod sort;

mod generated;
mod r#override;
mod args;

use crate::args::CliArgs;
use clap::{CommandFactory, Parser};
use std::sync::OnceLock;


static PINYIN_OVERRIDE: OnceLock<Option<r#override::PinyinOverride>> = OnceLock::new();

fn main() {
	let args = CliArgs::parse();

	if args.input_files.is_empty() && args.input_texts.is_empty() {
		CliArgs::command().print_help().unwrap();
		std::process::exit(0);
	}

	let format_config = format::FormatConfig::with_overrides(&args);

	let r#override = if let Some(config_path) = args.config_path {
		let overrides = r#override::PinyinOverride::load_from_file(&config_path);
		match overrides {
			Ok(overrides) => Some(overrides),
			Err(e) => {
				eprintln!("Error loading override config: {}", e);
				None
			}
		}
	} else {
		None
	};

	PINYIN_OVERRIDE.set(r#override).unwrap_or_else(|_| {
		eprintln!("Failed to set PINYIN_OVERRIDE");
	});

	println!("Format Config: {:?}", format_config);
}
