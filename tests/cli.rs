use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
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
        let path =
            std::env::temp_dir().join(format!("hanzi-sort-test-{}-{}", std::process::id(), unique));
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
    Command::new(env!("CARGO_BIN_EXE_hanzi-sort"))
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be valid UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be valid UTF-8")
}

#[test]
fn empty_stdin_produces_empty_output_and_exits_successfully() {
    // Mimics `hanzi-sort < /dev/null`: stdin is non-TTY, empty.
    // Like `sort`, hanzi-sort should treat this as "no input, no output"
    // and exit 0 — matching Unix filter conventions.
    let mut command = binary_command();
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let output = command.output().expect("CLI command should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        stderr(&output)
    );
    assert_eq!(stdout(&output), "");
}

#[test]
fn reads_stdin_when_no_explicit_input_provided() {
    let mut command = binary_command();
    command.args(["--columns", "1", "--entry-width", "2", "--blank-every", "0"]);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().expect("CLI command should spawn");
    {
        let stdin = child.stdin.as_mut().expect("stdin should be piped");
        stdin
            .write_all(b"\xe8\xb5\xb5\xe5\x9b\x9b\n\xe5\xbc\xa0\xe4\xb8\x89\n\xe6\xb1\x89\xe5\xad\x97\n")
            .expect("write stdin");
        // 赵四 / 张三 / 汉字 in UTF-8
    }
    let output = child.wait_with_output().expect("CLI command should finish");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "汉字\n张三\n赵四");
}

#[test]
fn dash_file_arg_reads_stdin() {
    let mut command = binary_command();
    command.args(["-f", "-", "--columns", "1", "--entry-width", "2", "--blank-every", "0"]);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().expect("CLI command should spawn");
    {
        let stdin = child.stdin.as_mut().expect("stdin should be piped");
        stdin
            .write_all("乙\n甲\n".as_bytes())
            .expect("write stdin");
    }
    let output = child.wait_with_output().expect("CLI command should finish");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "甲\n乙");
}

#[test]
fn reverse_flag_inverts_sort_order() {
    let mut command = binary_command();
    command.args([
        "-t", "汉字", "张三", "赵四",
        "--reverse",
        "--columns", "1", "--entry-width", "2", "--blank-every", "0",
    ]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "赵四\n张三\n汉字");
}

#[test]
fn unique_flag_removes_duplicates() {
    let mut command = binary_command();
    command.args([
        "-t", "张三", "汉字", "张三", "赵四", "张三",
        "--unique",
        "--columns", "1", "--entry-width", "2", "--blank-every", "0",
    ]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "汉字\n张三\n赵四");
}

#[test]
fn unique_and_reverse_compose_correctly() {
    // -u removes duplicates first, then -r reverses the deduped output.
    let mut command = binary_command();
    command.args([
        "-t", "张三", "汉字", "张三", "赵四",
        "--unique", "--reverse",
        "--columns", "1", "--entry-width", "2", "--blank-every", "0",
    ]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "赵四\n张三\n汉字");
}

#[test]
fn completions_subcommand_emits_bash_script() {
    let mut command = binary_command();
    command.args(["completions", "bash"]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let body = stdout(&output);
    assert!(
        body.contains("_hanzi-sort()"),
        "bash completion should define _hanzi-sort function: {body}"
    );
    assert!(
        body.contains("--reverse") && body.contains("--unique"),
        "completion should mention all top-level flags"
    );
}

#[test]
fn completions_subcommand_emits_zsh_script() {
    let mut command = binary_command();
    command.args(["completions", "zsh"]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let body = stdout(&output);
    assert!(
        body.contains("#compdef hanzi-sort"),
        "zsh completion should declare compdef header: {body}"
    );
}

#[test]
fn help_includes_examples_section() {
    let mut command = binary_command();
    command.arg("--help");
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success());
    let help = stdout(&output);
    assert!(help.contains("EXAMPLES:"), "help should include examples section");
    assert!(
        help.contains("cat names.txt | hanzi-sort"),
        "help should show stdin example"
    );
    assert!(
        help.contains("hanzi-sort completions bash"),
        "help should advertise the completions subcommand"
    );
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
fn rejects_mixing_file_and_text_inputs() {
    let temp = TempWorkspace::new();
    let input_path = temp.path().join("names.txt");
    fs::write(&input_path, "张三\n").expect("input file should be written");

    let mut command = binary_command();
    command.args(["-f"]);
    command.arg(&input_path);
    command.args(["-t", "赵四"]);
    let output = command.output().expect("CLI command should run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("cannot be used with"));
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
fn supports_stroke_sorting_from_cli() {
    let mut command = binary_command();
    command.args([
        "-t",
        "天",
        "一",
        "十",
        "--sort-by",
        "strokes",
        "--columns",
        "1",
        "--entry-width",
        "2",
        "--blank-every",
        "0",
    ]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "一\n十\n天");
}

#[cfg(feature = "collator-radical")]
#[test]
fn radical_sort_works_via_cli() {
    let mut command = binary_command();
    command.args([
        "-t",
        "一",
        "中",
        "汉",
        "--sort-by",
        "radical",
        "--columns",
        "1",
        "--entry-width",
        "2",
        "--blank-every",
        "0",
    ]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "一\n中\n汉");
}

#[cfg(feature = "collator-zhuyin")]
#[test]
fn supports_zhuyin_sorting_from_cli() {
    let mut command = binary_command();
    command.args([
        "-t",
        "一",
        "中",
        "汉",
        "--sort-by",
        "zhuyin",
        "--columns",
        "1",
        "--entry-width",
        "2",
        "--blank-every",
        "0",
    ]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "汉\n中\n一");
}

#[test]
fn rejects_output_path_that_is_a_directory() {
    let temp = TempWorkspace::new();
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&output_dir).expect("output directory should be created");

    let mut command = binary_command();
    command.args(["-t", "乙", "甲", "-o"]);
    command.arg(&output_dir);
    let output = command.output().expect("CLI command should run");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("failed to write output file"));
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

#[test]
fn char_override_changes_single_character_sort_order() {
    let temp = TempWorkspace::new();
    let override_path = temp.path().join("override.toml");
    fs::write(&override_path, "[char_override]\n'重' = 'chong2'\n")
        .expect("override file should be written");

    let mut command = binary_command();
    command.args(["-t", "重要", "银行", "--config"]);
    command.arg(&override_path);
    command.args(["--columns", "1", "--entry-width", "2", "--blank-every", "0"]);
    let output = command.output().expect("CLI command should run");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "重要\n银行");
}
