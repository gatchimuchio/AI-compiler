use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

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
