mod args;

use std::io::Write;

use clap::{CommandFactory, Parser};
use pinyin_sort::{
    FormatConfig, InputSource, PinyinContext, PinyinOverride, Result, RuntimeConfig, format_items,
    read_input_lines, sort_strings,
};

use crate::args::CliArgs;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = CliArgs::parse();
    if args.input_files.is_empty() && args.input_texts.is_empty() {
        let mut command = CliArgs::command();
        command
            .print_help()
            .map_err(|source| pinyin_sort::PinyinSortError::io("failed to print help", source))?;
        println!();
        return Ok(());
    }

    let input = if !args.input_files.is_empty() {
        InputSource::Files(args.input_files)
    } else {
        InputSource::Text(args.input_texts)
    };

    let mut format = FormatConfig::default();
    if let Some(value) = args.columns_per_row {
        format.columns_per_row = value;
    }
    if let Some(value) = args.blank_per {
        format.blank_per = (value > 0).then_some(value);
    }
    if let Some(value) = args.entry_width {
        format.entry_width = value;
    }
    if let Some(value) = args.align {
        format.align = value;
    }
    if let Some(value) = args.padding_char {
        format.padding_char = value;
    }
    if let Some(value) = args.separator {
        format.separator = value;
    }
    if let Some(value) = args.line_ending {
        format.line_ending = value;
    }

    let config = RuntimeConfig::new(input, args.output_path, args.config_path, format)?;
    let override_data = if let Some(path) = config.override_path.as_ref() {
        Some(PinyinOverride::load_from_file(path)?)
    } else {
        None
    };
    let context = PinyinContext::new(override_data);
    let input_data = read_input_lines(&config.input)?;
    let sorted = sort_strings(input_data, &context);
    let output = format_items(&sorted, &config.format);

    if let Some(path) = config.output_path.as_ref() {
        std::fs::write(path, output).map_err(|source| {
            pinyin_sort::PinyinSortError::io(
                format!("failed to write output file {}", path.display()),
                source,
            )
        })?;
    } else {
        let mut stdout = std::io::stdout().lock();
        stdout
            .write_all(output.as_bytes())
            .map_err(|source| pinyin_sort::PinyinSortError::io("failed to write stdout", source))?;
    }

    Ok(())
}
