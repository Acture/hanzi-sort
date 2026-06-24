use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::config::{HeaderSpec, InputSource};
use crate::error::{HanziSortError, Result};

/// Records read from an input source, split into the (optionally retained)
/// header lines and the body to be sorted.
pub(crate) struct Input {
    pub headers: Vec<String>,
    pub body: Vec<String>,
}

pub(crate) fn read_input_lines(source: &InputSource, header: &HeaderSpec) -> Result<Input> {
    let mut headers = Vec::new();
    let mut body = Vec::new();

    match source {
        InputSource::Files(paths) => {
            for path in paths {
                if path.as_os_str() == "-" {
                    let lines = read_stdin_lines()?;
                    split_header(lines, header, &mut headers, &mut body);
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
                let mut lines = Vec::new();
                for line in BufReader::new(file).lines() {
                    let line = line.map_err(|source| {
                        HanziSortError::io(
                            format!("failed to read line from {}", path.display()),
                            source,
                        )
                    })?;
                    lines.push(line);
                }
                split_header(lines, header, &mut headers, &mut body);
            }
        }
        InputSource::Text(items) => {
            // Header handling does not apply to inline `--text` records; the CLI
            // rejects the combination, so `header.lines` is 0 on this path.
            body.extend(
                items
                    .iter()
                    .filter(|item| !item.trim().is_empty())
                    .cloned(),
            );
        }
        InputSource::Stdin => {
            let lines = read_stdin_lines()?;
            split_header(lines, header, &mut headers, &mut body);
        }
    }

    Ok(Input { headers, body })
}

/// Split one source's physical lines into header and body.
///
/// The first `spec.lines` lines are the header region: dropped, or collected
/// into `headers` when `spec.keep` is set. The remaining lines are blank-line
/// filtered into `body`, matching the long-standing read behavior.
fn split_header(
    lines: Vec<String>,
    spec: &HeaderSpec,
    headers: &mut Vec<String>,
    body: &mut Vec<String>,
) {
    let mut iter = lines.into_iter();
    for _ in 0..spec.lines {
        match iter.next() {
            Some(line) => {
                if spec.keep {
                    headers.push(line);
                }
            }
            None => break,
        }
    }
    for line in iter {
        if !line.trim().is_empty() {
            body.push(line);
        }
    }
}

fn read_stdin_lines() -> Result<Vec<String>> {
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let mut lines = Vec::new();
    for line in handle.lines() {
        let line = line.map_err(|source| HanziSortError::io("failed to read stdin", source))?;
        lines.push(line);
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(lines: &[&str], spec: HeaderSpec) -> (Vec<String>, Vec<String>) {
        let mut headers = Vec::new();
        let mut body = Vec::new();
        let owned: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        split_header(owned, &spec, &mut headers, &mut body);
        (headers, body)
    }

    #[test]
    fn drops_first_n_lines() {
        let (headers, body) = run(
            &["name", "赵四", "汉字"],
            HeaderSpec { lines: 1, keep: false },
        );
        assert!(headers.is_empty());
        assert_eq!(body, vec!["赵四", "汉字"]);
    }

    #[test]
    fn keeps_first_n_lines_verbatim() {
        let (headers, body) = run(
            &["col1", "col2", "赵四", "汉字"],
            HeaderSpec { lines: 2, keep: true },
        );
        assert_eq!(headers, vec!["col1", "col2"]);
        assert_eq!(body, vec!["赵四", "汉字"]);
    }

    #[test]
    fn header_larger_than_input_consumes_everything() {
        let (headers, body) = run(&["only"], HeaderSpec { lines: 3, keep: false });
        assert!(headers.is_empty());
        assert!(body.is_empty());
    }

    #[test]
    fn blank_lines_in_header_region_are_consumed_blanks_in_body_filtered() {
        // First two physical lines are the header (one of them blank); a blank
        // line in the body is filtered as before.
        let (headers, body) = run(
            &["name", "", "赵四", "", "汉字"],
            HeaderSpec { lines: 2, keep: true },
        );
        assert_eq!(headers, vec!["name", ""]);
        assert_eq!(body, vec!["赵四", "汉字"]);
    }

    #[test]
    fn zero_lines_is_a_noop_with_blank_filtering() {
        let (headers, body) = run(&["赵四", "", "汉字"], HeaderSpec::default());
        assert!(headers.is_empty());
        assert_eq!(body, vec!["赵四", "汉字"]);
    }
}
