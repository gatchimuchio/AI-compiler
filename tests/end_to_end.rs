use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use ai_compiler::{evaluate_many, parse};

static NEXT_TEST: AtomicU64 = AtomicU64::new(0);
const TARGET: &str = "x86_64-linux-gnu";

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let sequence = NEXT_TEST.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("aic-e2e-{}-{sequence}", std::process::id()));
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

fn compile_source(dir: &TestDir, source: &str) -> PathBuf {
    let input = dir.path("input.aic");
    let object = dir.path("input.o");
    fs::write(&input, source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_aic"))
        .arg("compile")
        .arg("--target")
        .arg(TARGET)
        .arg(&input)
        .arg("-o")
        .arg(&object)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "compiler failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    object
}

fn link_harness(dir: &TestDir, object: &Path) -> PathBuf {
    let executable = dir.path("harness");
    let result = Command::new("g++")
        .arg("-std=c++20")
        .arg("-O2")
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-Werror")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/harness.cpp"))
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/abi_probe.S"))
        .arg(object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "harness link failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    executable
}

fn run_harness(dir: &TestDir, executable: &Path, inputs: &[u64]) -> Vec<u64> {
    let input_path = dir.path("harness-input.txt");
    let mut input_file = fs::File::create(&input_path).unwrap();
    for input in inputs {
        writeln!(input_file, "{input}").unwrap();
    }
    drop(input_file);
    let child = Command::new(executable)
        .stdin(Stdio::from(File::open(input_path).unwrap()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "harness failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| line.parse::<u64>().unwrap())
        .collect()
}

#[test]
fn branch_phi_object_matches_reference_for_exhaustive_u16_inputs_and_boundaries() {
    let source = include_str!("../examples/branch_phi.aic");
    let module = parse(source.as_bytes()).unwrap();
    let dir = TestDir::new();
    let object = compile_source(&dir, source);
    let executable = link_harness(&dir, &object);

    let mut inputs: Vec<u64> = (0..=u16::MAX).map(u64::from).collect();
    inputs.extend([0, 1, 2, u64::MAX - 1, u64::MAX, 1 << 63, (1 << 63) - 1]);
    let actual = run_harness(&dir, &executable, &inputs);
    let expected = evaluate_many(&module, &inputs, 100).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn loop_phi_cycle_object_matches_reference_and_exposes_valid_elf() {
    let source = include_str!("../examples/loop_swap.aic");
    let module = parse(source.as_bytes()).unwrap();
    let dir = TestDir::new();
    let object = compile_source(&dir, source);
    let executable = link_harness(&dir, &object);

    let inputs: Vec<u64> = (0..=255).collect();
    let actual = run_harness(&dir, &executable, &inputs);
    let expected = evaluate_many(&module, &inputs, 10_000).unwrap();
    assert_eq!(actual, expected);

    let header = Command::new("readelf")
        .env("LC_ALL", "C")
        .arg("-h")
        .arg(&object)
        .output()
        .unwrap();
    assert!(header.status.success());
    let header = String::from_utf8(header.stdout).unwrap();
    assert!(header.contains("ELF64"));
    assert!(header.contains("REL (Relocatable file)"));
    assert!(header.contains("Advanced Micro Devices X86-64"));

    let symbols = Command::new("readelf")
        .env("LC_ALL", "C")
        .arg("-s")
        .arg(&object)
        .output()
        .unwrap();
    assert!(symbols.status.success());
    let symbols = String::from_utf8(symbols.stdout).unwrap();
    assert!(symbols.contains("GLOBAL"));
    assert!(symbols.contains("aic_main"));

    let relocations = Command::new("readelf")
        .env("LC_ALL", "C")
        .arg("-r")
        .arg(&object)
        .output()
        .unwrap();
    assert!(relocations.status.success());
    assert!(
        String::from_utf8(relocations.stdout)
            .unwrap()
            .contains("There are no relocations")
    );

    let disassembly = Command::new("objdump")
        .env("LC_ALL", "C")
        .arg("-dr")
        .arg(&object)
        .output()
        .unwrap();
    assert!(disassembly.status.success());
    assert!(
        String::from_utf8(disassembly.stdout)
            .unwrap()
            .contains("<aic_main>")
    );
}

#[test]
fn repeated_compilation_is_byte_for_byte_deterministic() {
    let dir = TestDir::new();
    let input = dir.path("stable.aic");
    let first = dir.path("first.o");
    let second = dir.path("second.o");
    fs::write(&input, include_str!("../examples/branch_phi.aic")).unwrap();

    for output in [&first, &second] {
        let result = Command::new(env!("CARGO_BIN_EXE_aic"))
            .arg("compile")
            .arg("--target")
            .arg(TARGET)
            .arg(&input)
            .arg("-o")
            .arg(output)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "compiler failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    assert_eq!(fs::read(first).unwrap(), fs::read(second).unwrap());
}
