mod args;

use std::io::Write;
use std::path::Path;

use clap::{CommandFactory, Parser};
use pinyin_sort::{Result, app};

use crate::args::CliArgs;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = CliArgs::parse();
    if !args.has_input() {
        let mut command = CliArgs::command();
        command
            .print_help()
            .map_err(|source| pinyin_sort::PinyinSortError::io("failed to print help", source))?;
        println!();
        return Ok(());
    }

    let (config, output_path) = args.into_runtime_parts()?;
    let output = app::render(config)?;
    write_output(output_path.as_deref(), &output)
}

fn write_output(path: Option<&Path>, output: &str) -> Result<()> {
    if let Some(path) = path {
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
