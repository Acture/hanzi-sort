use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::config::InputSource;
use crate::error::{PinyinSortError, Result};

pub fn read_input_lines(source: &InputSource) -> Result<Vec<String>> {
    match source {
        InputSource::Files(paths) => {
            let mut items = Vec::new();
            for path in paths {
                let metadata = std::fs::metadata(path).map_err(|source| {
                    PinyinSortError::io(
                        format!("failed to inspect input path {}", path.display()),
                        source,
                    )
                })?;
                if metadata.is_dir() {
                    return Err(PinyinSortError::InvalidArgument(format!(
                        "directory inputs are not supported: {}",
                        path.display()
                    )));
                }

                let file = File::open(path).map_err(|source| {
                    PinyinSortError::io(
                        format!("failed to read input file {}", path.display()),
                        source,
                    )
                })?;
                for line in BufReader::new(file).lines() {
                    let line = line.map_err(|source| {
                        PinyinSortError::io(
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
    }
}
