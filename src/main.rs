mod args;

use std::io::{IsTerminal, Write};
use std::path::Path;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use hanzi_sort::{Result, app};

use crate::args::{CliArgs, CliCommand};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = CliArgs::parse();

    if let Some(CliCommand::Completions { shell }) = args.command {
        let mut command = CliArgs::command();
        let bin_name = command.get_name().to_string();
        generate(shell, &mut command, bin_name, &mut std::io::stdout());
        return Ok(());
    }

    // No explicit input: read stdin if it's piped, otherwise show help.
    if !args.has_input() && std::io::stdin().is_terminal() {
        let mut command = CliArgs::command();
        command
            .print_help()
            .map_err(|source| hanzi_sort::HanziSortError::io("failed to print help", source))?;
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
            hanzi_sort::HanziSortError::io(
                format!("failed to write output file {}", path.display()),
                source,
            )
        })?;
    } else {
        let mut stdout = std::io::stdout().lock();
        stdout
            .write_all(output.as_bytes())
            .map_err(|source| hanzi_sort::HanziSortError::io("failed to write stdout", source))?;
    }

    Ok(())
}
