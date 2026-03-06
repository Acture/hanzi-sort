use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "pinyin-sort-test-{}-{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&path).expect("temporary directory should be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn binary_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pinyin-sort"))
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be valid UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be valid UTF-8")
}

#[test]
fn reads_file_inputs_line_by_line_and_ignores_blank_lines() {
    let temp = TempWorkspace::new();
    let input_path = temp.path().join("names.txt");
    fs::write(&input_path, "赵四\n\n张三\n汉字\n").expect("input file should be written");

    let mut command = binary_command();
    command.args(["-f"]);
    command.arg(&input_path);
    command.args(["--columns", "1", "--entry-width", "2", "--blank-every", "0"]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "汉字\n张三\n赵四");
}

#[test]
fn rejects_missing_input_file() {
    let temp = TempWorkspace::new();
    let missing_path = temp.path().join("missing.txt");

    let mut command = binary_command();
    command.args(["-f"]);
    command.arg(&missing_path);
    let output = command.output().expect("CLI command should run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to inspect input path"));
}

#[test]
fn rejects_directory_input() {
    let temp = TempWorkspace::new();
    let input_dir = temp.path().join("folder");
    fs::create_dir_all(&input_dir).expect("directory input should be created");

    let mut command = binary_command();
    command.args(["-f"]);
    command.arg(&input_dir);
    let output = command.output().expect("CLI command should run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("directory inputs are not supported"));
}

#[test]
fn writes_output_to_file_when_requested() {
    let temp = TempWorkspace::new();
    let output_path = temp.path().join("sorted.txt");

    let mut command = binary_command();
    command.args([
        "-t",
        "乙",
        "甲",
        "--columns",
        "1",
        "--entry-width",
        "2",
        "--blank-every",
        "0",
        "-o",
    ]);
    command.arg(&output_path);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success());
    assert!(stdout(&output).is_empty());
    assert_eq!(
        fs::read_to_string(&output_path).expect("output file should be written"),
        "甲\n乙"
    );
}

#[test]
fn rejects_invalid_override_config() {
    let temp = TempWorkspace::new();
    let override_path = temp.path().join("override.toml");
    fs::write(
        &override_path,
        "[phrase_override]\n\"重庆\" = [\"chong2\"]\n",
    )
    .expect("override file should be written");

    let mut command = binary_command();
    command.args(["-t", "重庆", "--config"]);
    command.arg(&override_path);
    let output = command.output().expect("CLI command should run");

    assert!(!output.status.success());
    assert!(
        stderr(&output)
            .contains("phrase_override entry '重庆' has 2 characters but 1 pinyin values")
    );
}

#[test]
fn rejects_missing_override_config_file() {
    let temp = TempWorkspace::new();
    let override_path = temp.path().join("missing.toml");

    let mut command = binary_command();
    command.args(["-t", "重庆", "--config"]);
    command.arg(&override_path);
    let output = command.output().expect("CLI command should run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to read override config"));
}

#[test]
fn phrase_override_changes_sort_order() {
    let temp = TempWorkspace::new();
    let override_path = temp.path().join("override.toml");
    fs::write(
        &override_path,
        "[phrase_override]\n\"重庆\" = [\"chong2\", \"qing4\"]\n",
    )
    .expect("override file should be written");

    let mut command = binary_command();
    command.args(["-t", "重庆", "银行", "--config"]);
    command.arg(&override_path);
    command.args(["--columns", "1", "--entry-width", "2", "--blank-every", "0"]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "重庆\n银行");
}
