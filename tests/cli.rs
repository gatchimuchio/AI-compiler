use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use ai_compiler::limits::MAX_SOURCE_BYTES;

static NEXT_TEST: AtomicU64 = AtomicU64::new(0);
const TARGET: &str = "x86_64-linux-gnu";

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let sequence = NEXT_TEST.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("aic-cli-{}-{sequence}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_aic"))
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn cli_rejects_unknown_and_duplicate_options_as_usage_errors() {
    let dir = TestDir::new();
    let input = dir.path("input.aic");
    let output = dir.path("output.o");
    let input = input.to_str().unwrap();
    let output = output.to_str().unwrap();
    let target_and_paths = ["compile", "--target", TARGET, input, "-o", output];

    let unknown = vec!["compile", "--mystery"];
    let mut duplicate_target = target_and_paths.to_vec();
    duplicate_target.extend(["--target", TARGET]);
    let mut duplicate_output = target_and_paths.to_vec();
    duplicate_output.extend(["-o", output]);

    for arguments in [unknown, duplicate_target, duplicate_output] {
        let result = run(&arguments);
        assert_eq!(result.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("error[E_USAGE]"),
            "unexpected diagnostic: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn cli_refuses_to_replace_the_input_path() {
    let dir = TestDir::new();
    let input = dir.path("same.aic");
    let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
    fs::write(&input, source).unwrap();
    let input_text = input.to_str().unwrap();
    let result = run(&["compile", "--target", TARGET, input_text, "-o", input_text]);

    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("error[E_OUTPUT]"));
    assert_eq!(fs::read(input).unwrap(), source);
}

#[test]
fn cli_supports_a_positional_path_starting_with_a_dash_after_double_dash() {
    let dir = TestDir::new();
    let input_name = "-input.aic";
    fs::write(
        dir.path(input_name),
        b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n",
    )
    .unwrap();
    let output = dir.path("output.o");
    let result = Command::new(env!("CARGO_BIN_EXE_aic"))
        .current_dir(&dir.0)
        .args(["compile", "--target", TARGET, "-o"])
        .arg(&output)
        .args(["--", input_name])
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "unexpected diagnostic: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.is_file());
}

#[test]
fn cli_enforces_source_budget_and_preserves_files_before_tool_invocation() {
    let dir = TestDir::new();
    let input = dir.path("input.aic");
    let output = dir.path("output.o");
    let mut source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n;".to_vec();
    source.resize(MAX_SOURCE_BYTES, b'x');
    fs::write(&input, &source).unwrap();
    let arguments = [
        "compile",
        "--target",
        TARGET,
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ];
    let accepted = run(&arguments);
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );

    source.push(b'x');
    fs::write(&input, &source).unwrap();
    fs::write(&output, b"previous artifact").unwrap();
    let rejected = Command::new(env!("CARGO_BIN_EXE_aic"))
        .args(arguments)
        .env("PATH", &dir.0) // No assembler is available in this directory.
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("error[E_LIMIT]"));
    assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
    assert_eq!(fs::read(&input).unwrap(), source);
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 2);

    let same_input = run(&[
        "compile",
        "--target",
        TARGET,
        input.to_str().unwrap(),
        "-o",
        input.to_str().unwrap(),
    ]);
    assert_eq!(same_input.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&same_input.stderr).contains("error[E_OUTPUT]"));
    assert_eq!(fs::read(&input).unwrap(), source);
}

#[test]
fn cli_bounds_reads_without_waiting_for_end_of_stream() {
    let dir = TestDir::new();
    let output = dir.path("output.o");
    fs::write(&output, b"previous artifact").unwrap();
    let mut child = Command::new("timeout")
        .args([
            "--kill-after=1s",
            "5s",
            env!("CARGO_BIN_EXE_aic"),
            "compile",
            "--target",
            TARGET,
            "/dev/stdin",
            "-o",
        ])
        .arg(&output)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Feed only a finite amount and keep the pipe open: a missing read bound
    // waits for EOF and times out, rather than consuming unbounded test memory.
    let mut writer = child.stdin.take().unwrap();
    writer.write_all(&vec![0; MAX_SOURCE_BYTES + 1]).unwrap();
    let result = child.wait_with_output().unwrap();
    drop(writer);
    assert_eq!(
        result.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains("error[E_LIMIT]"));
    assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
}
