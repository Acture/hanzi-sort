use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::config::InputSource;
use crate::error::{HanziSortError, Result};

pub(crate) fn read_input_lines(source: &InputSource) -> Result<Vec<String>> {
    match source {
        InputSource::Files(paths) => {
            let mut items = Vec::new();
            for path in paths {
                if path.as_os_str() == "-" {
                    read_stdin_into(&mut items)?;
                    continue;
                }
                let metadata = std::fs::metadata(path).map_err(|source| {
                    HanziSortError::io(
                        format!("failed to inspect input path {}", path.display()),
                        source,
                    )
                })?;
                if metadata.is_dir() {
                    return Err(HanziSortError::InvalidArgument(format!(
                        "directory inputs are not supported: {}",
                        path.display()
                    )));
                }

                let file = File::open(path).map_err(|source| {
                    HanziSortError::io(
                        format!("failed to read input file {}", path.display()),
                        source,
                    )
                })?;
                for line in BufReader::new(file).lines() {
                    let line = line.map_err(|source| {
                        HanziSortError::io(
                            format!("failed to read line from {}", path.display()),
                            source,
                        )
                    })?;
                    if !line.trim().is_empty() {
                        items.push(line);
                    }
                }
            }
            Ok(items)
        }
        InputSource::Text(items) => Ok(items
            .iter()
            .filter(|item| !item.trim().is_empty())
            .cloned()
            .collect()),
        InputSource::Stdin => {
            let mut items = Vec::new();
            read_stdin_into(&mut items)?;
            Ok(items)
        }
    }
}

fn read_stdin_into(items: &mut Vec<String>) -> Result<()> {
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    for line in handle.lines() {
        let line = line.map_err(|source| HanziSortError::io("failed to read stdin", source))?;
        if !line.trim().is_empty() {
            items.push(line);
        }
    }
    Ok(())
}
