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

fn assert_amd64_baseline_instruction_subset(object: &Path) {
    let output = Command::new("objdump")
        .env("LC_ALL", "C")
        .arg("-d")
        .arg(object)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "objdump failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let disassembly = String::from_utf8(output.stdout).unwrap();
    assert!(disassembly.contains("<aic_main>"));

    const BASELINE_SUBSET: &[&str] = &[
        "add", "cmp", "cmpq", "jmp", "jne", "leave", "mov", "movabs", "movzbq", "push", "ret",
        "sete", "sub",
    ];
    let mut instruction_count = 0;
    for line in disassembly.lines() {
        let Some((_, encoded)) = line.split_once('\t') else {
            continue;
        };
        let Some((_, instruction)) = encoded.split_once('\t') else {
            continue;
        };
        let Some(mnemonic) = instruction.split_whitespace().next() else {
            continue;
        };
        assert!(
            BASELINE_SUBSET.contains(&mnemonic),
            "instruction `{mnemonic}` is outside the reviewed baseline subset: {line}"
        );
        instruction_count += 1;
    }
    assert!(instruction_count > 0, "objdump found no instructions");
}

fn link_harness(dir: &TestDir, object: &Path) -> PathBuf {
    let executable = dir.path("harness");
    let result = Command::new("g++")
        .arg("-std=c++20")
        .arg("-O2")
        .arg("-mno-red-zone")
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
    // Fuel bounds the interpreter only. Bound native fixture execution separately.
    let child = Command::new("timeout")
        .args(["--kill-after=1s", "30s"])
        .arg(executable)
        .stdin(Stdio::from(File::open(input_path).unwrap()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "harness failed ({}): {}",
        output.status,
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
    assert_amd64_baseline_instruction_subset(&object);
    let executable = link_harness(&dir, &object);

    let mut inputs: Vec<u64> = (0..=u16::MAX).map(u64::from).collect();
    inputs.extend([0, 1, 2, u64::MAX - 1, u64::MAX, 1 << 63, (1 << 63) - 1]);
    let actual = run_harness(&dir, &executable, &inputs);
    let expected = evaluate_many(&module, &inputs, 100).unwrap();
    let independent: Vec<u64> = inputs
        .iter()
        .map(|value| {
            if *value == 0 {
                value.wrapping_sub(1)
            } else {
                value.wrapping_add(1)
            }
        })
        .collect();
    assert_eq!(actual, independent);
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
    let independent: Vec<u64> = inputs.iter().map(|value| value & 1).collect();
    assert_eq!(actual, independent);
    assert_eq!(actual, expected);

    let inspection = Command::new("readelf")
        .env("LC_ALL", "C")
        .args(["--lint", "--wide", "--all"])
        .arg(&object)
        .output()
        .unwrap();
    assert!(
        inspection.status.success(),
        "readelf inspection failed: {}",
        String::from_utf8_lossy(&inspection.stderr)
    );
    // GNU as intentionally emits these empty sections. ELF permits zero-size
    // sections; retain readelf's other lint diagnostics as test failures.
    for warning in String::from_utf8(inspection.stderr).unwrap().lines() {
        assert!(
            [
                "readelf: Warning: Section '.data': has a size of zero - is this intended ?",
                "readelf: Warning: Section '.note.GNU-stack': has a size of zero - is this intended ?",
            ]
            .contains(&warning),
            "unexpected readelf lint diagnostic: {warning}"
        );
    }
    let inspection = String::from_utf8(inspection.stdout).unwrap();
    for expected in [
        "ELF64",
        "REL (Relocatable file)",
        "Advanced Micro Devices X86-64",
        ".text",
        ".symtab",
        ".strtab",
        "GLOBAL",
        "aic_main",
        "There are no relocations",
    ] {
        assert!(inspection.contains(expected), "missing {expected}");
    }

    assert_amd64_baseline_instruction_subset(&object);
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

#[test]
fn long_stack_frame_with_disp32_operands_matches_reference() {
    let mut source = String::from("AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n");
    let mut accumulator = String::from("x");
    let mut state = 0x7c31_8ad4_52e6_b90f_u64;
    let mut operations = Vec::with_capacity(48);
    for index in 0..48 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        source.push_str(&format!(" %k{index} = const.u64 {state}\n"));
        let operation = if index % 2 == 0 {
            "add.wrap.u64"
        } else {
            "sub.wrap.u64"
        };
        operations.push((operation == "add.wrap.u64", state));
        source.push_str(&format!(
            " %v{index} = {operation} %{accumulator}, %k{index}\n"
        ));
        source.push_str(&format!(" %equal{index} = eq.u64 %v{index}, %x\n"));
        accumulator = format!("v{index}");
    }
    source.push_str(&format!(" ret.u64 %{accumulator}\n}}\n"));

    let module = parse(source.as_bytes()).unwrap();
    let dir = TestDir::new();
    let object = compile_source(&dir, &source);
    let executable = link_harness(&dir, &object);
    let mut inputs = vec![0, 1, 2, u64::MAX, u64::MAX - 1, 1 << 63, (1 << 63) - 1];
    for _ in 0..512 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        inputs.push(state);
    }

    let actual = run_harness(&dir, &executable, &inputs);
    let expected = evaluate_many(&module, &inputs, 1_000).unwrap();
    let independent: Vec<u64> = inputs
        .iter()
        .map(|input| {
            operations.iter().fold(*input, |value, (is_add, constant)| {
                if *is_add {
                    value.wrapping_add(*constant)
                } else {
                    value.wrapping_sub(*constant)
                }
            })
        })
        .collect();
    assert_eq!(actual, independent);
    assert_eq!(actual, expected);

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
            .contains("-0x100(%rbp)"),
        "expected stack accesses beyond the disp8 range"
    );
}

#[test]
fn generated_multiway_phi_control_flow_matches_reference() {
    const STAGES: usize = 24;
    let mut source = String::from("AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n");
    let mut thresholds = Vec::with_capacity(STAGES * 2);
    let mut stages = Vec::with_capacity(STAGES);
    let mut random = 0x91d4_2ac7_6e3b_580f_u64;

    for stage in 0..STAGES {
        let threshold_a = (stage as u64) * 4 + 1;
        let threshold_b = threshold_a + 1;
        thresholds.extend([threshold_a, threshold_b]);

        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let delta_a = if stage % 3 == 0 { u64::MAX } else { random };
        let delta_b = if stage % 3 == 1 {
            u64::MAX
        } else {
            random.rotate_left(17)
        };
        let delta_default = if stage % 2 == 0 { 1 } else { u64::MAX };
        stages.push((threshold_a, threshold_b, delta_a, delta_b, delta_default));

        source.push_str(&format!(
            " %ta{stage} = const.u64 {threshold_a}\n %tb{stage} = const.u64 {threshold_b}\n %da{stage} = const.u64 {delta_a}\n %db{stage} = const.u64 {delta_b}\n %dc{stage} = const.u64 {delta_default}\n"
        ));
    }
    source.push_str(" br ^test0\n");

    for stage in 0..STAGES {
        let current = if stage == 0 {
            String::from("x")
        } else {
            format!("v{}", stage - 1)
        };
        source.push_str(&format!(
            "^test{stage}:\n %ca{stage} = eq.u64 %x, %ta{stage}\n br.if %ca{stage}, ^arma{stage}, ^split{stage}\n"
        ));
        source.push_str(&format!(
            "^split{stage}:\n %cb{stage} = eq.u64 %x, %tb{stage}\n br.if %cb{stage}, ^armb{stage}, ^armc{stage}\n"
        ));
        source.push_str(&format!(
            "^arma{stage}:\n %va{stage} = add.wrap.u64 %{current}, %da{stage}\n br ^join{stage}\n"
        ));
        source.push_str(&format!(
            "^armb{stage}:\n %vb{stage} = sub.wrap.u64 %{current}, %db{stage}\n br ^join{stage}\n"
        ));
        source.push_str(&format!(
            "^armc{stage}:\n %vc{stage} = add.wrap.u64 %{current}, %dc{stage}\n br ^join{stage}\n"
        ));
        source.push_str(&format!(
            "^join{stage}:\n %v{stage} = phi.u64 [^arma{stage}, %va{stage}] [^armb{stage}, %vb{stage}] [^armc{stage}, %vc{stage}]\n"
        ));
        if stage + 1 == STAGES {
            source.push_str(&format!(" ret.u64 %v{stage}\n"));
        } else {
            source.push_str(&format!(" br ^test{}\n", stage + 1));
        }
    }
    source.push_str("}\n");

    let module = parse(source.as_bytes()).unwrap();
    let dir = TestDir::new();
    let object = compile_source(&dir, &source);
    let executable = link_harness(&dir, &object);
    let mut inputs = vec![0, 1, 2, u64::MAX, u64::MAX - 1, 1 << 63, (1 << 63) - 1];
    inputs.extend(thresholds);
    for _ in 0..512 {
        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        inputs.push(random);
    }

    let actual = run_harness(&dir, &executable, &inputs);
    let expected = evaluate_many(&module, &inputs, 10_000).unwrap();
    let independent: Vec<u64> = inputs
        .iter()
        .map(|input| {
            stages.iter().fold(*input, |value, stage| {
                let (threshold_a, threshold_b, delta_a, delta_b, delta_default) = stage;
                if *input == *threshold_a {
                    value.wrapping_add(*delta_a)
                } else if *input == *threshold_b {
                    value.wrapping_sub(*delta_b)
                } else {
                    value.wrapping_add(*delta_default)
                }
            })
        })
        .collect();
    assert_eq!(actual, independent);
    assert_eq!(actual, expected);
}

#[test]
fn loop_header_parallel_phi_with_two_backedges_matches_reference() {
    let source = "AIC 0.2\nfn aic_main(%n: u64) -> u64 {\n^entry:\n %zero = const.u64 0\n %one = const.u64 1\n %two = const.u64 2\n %a0 = const.u64 17\n %b0 = const.u64 29\n br ^head\n^head:\n %i = phi.u64 [^entry, %zero] [^step_one, %next_one] [^step_two, %next_two]\n %a = phi.u64 [^entry, %a0] [^step_one, %b] [^step_two, %a]\n %b = phi.u64 [^entry, %b0] [^step_one, %a] [^step_two, %b]\n %done = eq.u64 %i, %n\n br.if %done, ^exit, ^select\n^select:\n %use_one = eq.u64 %n, %one\n br.if %use_one, ^step_one, ^step_two\n^step_one:\n %next_one = add.wrap.u64 %i, %one\n br ^head\n^step_two:\n %next_two = add.wrap.u64 %i, %two\n br ^head\n^exit:\n ret.u64 %a\n}\n";
    let module = parse(source.as_bytes()).unwrap();
    let dir = TestDir::new();
    let object = compile_source(&dir, source);
    let executable = link_harness(&dir, &object);
    let mut inputs = vec![0, 1];
    inputs.extend((1..=256).map(|value| value * 2));

    let actual = run_harness(&dir, &executable, &inputs);
    let expected = evaluate_many(&module, &inputs, 5_000).unwrap();
    assert_eq!(actual, expected);
    let independent: Vec<u64> = inputs
        .iter()
        .map(|input| if *input == 1 { 29 } else { 17 })
        .collect();
    assert_eq!(actual, independent);
}

#[test]
fn two_entry_cycle_phi_matches_independent_gold() {
    // Neither left nor right dominates the other: entry can enter either node
    // of this SCC. Each cross-edge must carry the predecessor's UPDATED value.
    let source = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %zero = const.u64 0\n %one = const.u64 1\n %limit = const.u64 5\n %delta_l = const.u64 17\n %delta_r = const.u64 29\n %choose = eq.u64 %x, %zero\n br.if %choose, ^left, ^right\n^left:\n %li = phi.u64 [^entry, %zero] [^right, %rn]\n %lv = phi.u64 [^entry, %x] [^right, %ru]\n %lu = add.wrap.u64 %lv, %delta_l\n %ln = add.wrap.u64 %li, %one\n %ld = eq.u64 %ln, %limit\n br.if %ld, ^exit, ^right\n^right:\n %ri = phi.u64 [^entry, %zero] [^left, %ln]\n %rv = phi.u64 [^entry, %x] [^left, %lu]\n %ru = sub.wrap.u64 %rv, %delta_r\n %rn = add.wrap.u64 %ri, %one\n %rd = eq.u64 %rn, %limit\n br.if %rd, ^exit, ^left\n^exit:\n %result = phi.u64 [^left, %lu] [^right, %ru]\n ret.u64 %result\n}\n";
    let inputs = cyclic_cfg_inputs();
    // Five visits: L/R/L/R/L adds 3*17-2*29; R/L/R/L/R adds 2*17-3*29.
    let gold: Vec<_> = inputs
        .iter()
        .map(|x| x.wrapping_sub(if *x == 0 { 7 } else { 53 }))
        .collect();
    let module = parse(source.as_bytes()).unwrap();
    assert_eq!(evaluate_many(&module, &inputs, 200).unwrap(), gold);
    let dir = TestDir::new();
    let object = compile_source(&dir, source);
    let executable = link_harness(&dir, &object);
    assert_eq!(run_harness(&dir, &executable, &inputs), gold);
}

fn cyclic_cfg_inputs() -> Vec<u64> {
    let mut inputs = vec![0, 1, 2, u64::MAX, u64::MAX - 1, 1 << 63, (1 << 63) - 1];
    let mut state = 0x48dc_1e72_a905_3fb6_u64;
    for _ in 0..256 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        inputs.push(state);
    }
    inputs
}

#[test]
fn alternating_latch_phi_maps_match_independent_gold_and_changed_controls() {
    let source = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %zero = const.u64 0\n %one = const.u64 1\n %limit = const.u64 5\n %plus = add.wrap.u64 %x, %one\n %minus = sub.wrap.u64 %x, %one\n br ^head\n^head:\n %i = phi.u64 [^entry, %zero] [^rotate, %rnext] [^adjust, %anext]\n %phase = phi.u64 [^entry, %zero] [^rotate, %one] [^adjust, %zero]\n %a = phi.u64 [^entry, %x] [^rotate, %b] [^adjust, %adjusted]\n %b = phi.u64 [^entry, %plus] [^rotate, %c] [^adjust, %b]\n %c = phi.u64 [^entry, %minus] [^rotate, %a] [^adjust, %c]\n %done = eq.u64 %i, %limit\n br.if %done, ^exit, ^select\n^select:\n %use_rotate = eq.u64 %phase, %zero\n br.if %use_rotate, ^rotate, ^adjust\n^rotate:\n %rnext = add.wrap.u64 %i, %one\n br ^head\n^adjust:\n %adjusted = sub.wrap.u64 %a, %one\n %anext = add.wrap.u64 %i, %one\n br ^head\n^exit:\n %twob = add.wrap.u64 %b, %b\n %twoc = add.wrap.u64 %c, %c\n %fourc = add.wrap.u64 %twoc, %twoc\n %ab = add.wrap.u64 %a, %twob\n %result = add.wrap.u64 %ab, %fourc\n ret.u64 %result\n}\n";
    let phase_reversed = source.replace(
        "%phase = phi.u64 [^entry, %zero]",
        "%phase = phi.u64 [^entry, %one]",
    );
    // Change A=(a-1,b,c) to A'=(a,b,c-1), including its destination mapping.
    let decrement_c = source
        .replace("[^adjust, %adjusted]", "[^adjust, %a]")
        .replace(
            "[^rotate, %a] [^adjust, %c]",
            "[^rotate, %a] [^adjust, %adjusted]",
        )
        .replace(
            "%adjusted = sub.wrap.u64 %a, %one",
            "%adjusted = sub.wrap.u64 %c, %one",
        );
    assert_ne!(phase_reversed, source);
    assert_ne!(decrement_c, source);
    let inputs = cyclic_cfg_inputs();
    // R=(b,c,a), A=(a-1,b,c). R/A/R/A/R yields (x,x,x-2).
    // Return a+2b+4c. Unequal weights distinguish the two changed controls.
    for (name, source, delta) in [
        ("rotate_first", source, 8),
        ("phase_reversed", phase_reversed.as_str(), 4),
        ("decrement_c", decrement_c.as_str(), 5),
    ] {
        let gold: Vec<_> = inputs
            .iter()
            .map(|x| x.wrapping_mul(7).wrapping_sub(delta))
            .collect();
        let module = parse(source.as_bytes()).unwrap();
        assert_eq!(
            evaluate_many(&module, &inputs, 200).unwrap(),
            gold,
            "{name}"
        );
        let dir = TestDir::new();
        let object = compile_source(&dir, source);
        let executable = link_harness(&dir, &object);
        assert_eq!(run_harness(&dir, &executable, &inputs), gold, "{name}");
    }
}

#[test]
fn self_edge_phi_permutations_match_independent_gold_and_native_deadline() {
    // Rearrange literal source records without using the compiler's AST, maps,
    // or lowering. Keep entry first and ordinary instruction order unchanged.
    fn rearrange(source: &str, variant: u8) -> String {
        let (prefix, body) = source.split_once("^entry:\n").unwrap();
        let mut blocks = vec![vec![String::from("^entry:")]];
        for line in body.lines().filter(|line| *line != "}") {
            if line.starts_with('^') {
                blocks.push(vec![line.to_owned()]);
            } else {
                blocks.last_mut().unwrap().push(line.to_owned());
            }
        }
        if variant & 1 != 0 {
            blocks[1..].reverse();
        }
        for block in &mut blocks {
            let phis = block[1..]
                .iter()
                .take_while(|line| line.contains("= phi.u64 "))
                .count();
            if variant & 2 != 0 {
                block[1..1 + phis].reverse();
            }
            if variant & 4 != 0 {
                for line in block {
                    if let Some((left, right)) = line.split_once("phi.u64 ") {
                        let mut incomings: Vec<_> =
                            right.split_inclusive(']').map(str::trim).collect();
                        incomings.reverse();
                        *line = format!("{left}phi.u64 {}", incomings.join(" "));
                    }
                }
            }
        }
        let result = format!(
            "{prefix}{}\n}}\n",
            blocks.into_iter().flatten().collect::<Vec<_>>().join("\n")
        );
        if variant & 8 == 0 {
            return result;
        }
        let mut renamed = String::new();
        let mut chars = result.chars().peekable();
        while let Some(ch) = chars.next() {
            renamed.push(ch);
            if ch == '%' || ch == '^' {
                let mut id = String::new();
                while chars
                    .peek()
                    .is_some_and(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                {
                    id.push(chars.next().unwrap());
                }
                if ch != '^' || id != "entry" {
                    renamed.push_str("renamed_");
                }
                renamed.push_str(&id);
            }
        }
        renamed
    }

    let base = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %zero = const.u64 0\n %one = const.u64 1\n %limit = const.u64 5\n %plus = add.wrap.u64 %x, %one\n %minus = sub.wrap.u64 %x, %one\n br ^head\n^head:\n %i = phi.u64 [^entry, %zero] [^head, %next]\n %a = phi.u64 [^entry, %x] [^head, %b]\n %b = phi.u64 [^entry, %plus] [^head, %c]\n %c = phi.u64 [^entry, %minus] [^head, %a]\n %held = phi.u64 [^entry, %x] [^head, %held]\n %fan = phi.u64 [^entry, %x] [^head, %a]\n %next = add.wrap.u64 %i, %one\n %done = eq.u64 %i, %limit\n br.if %done, ^exit, ^head\n^exit:\n %ab = add.wrap.u64 %a, %b\n %abc = sub.wrap.u64 %ab, %c\n %with_hold = add.wrap.u64 %abc, %held\n %result = sub.wrap.u64 %with_hold, %fan\n ret.u64 %result\n}\n";
    let mut inputs = vec![0, 1, 2, u64::MAX, u64::MAX - 1, 1 << 63, (1 << 63) - 1];
    let mut random = 0x5b3e_107a_d924_6fc1_u64;
    for _ in 0..256 {
        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        inputs.push(random);
    }
    for trips in [0, 1, 5] {
        let source = base.replace("const.u64 5", &format!("const.u64 {trips}"));
        // Starting from (x,x+1,x-1), rotate (a,b,c) on every self edge.
        // held stays x; fan receives the PREVIOUS a. These closed forms are
        // derived from that tuple, independently of parse/validation/codegen.
        let gold: Vec<_> = inputs
            .iter()
            .map(|x| match trips {
                0 => x.wrapping_add(2),
                1 => *x,
                5 => x.wrapping_sub(3),
                _ => unreachable!(),
            })
            .collect();
        for variant in 0..16 {
            let source = rearrange(&source, variant);
            let module = parse(source.as_bytes()).unwrap();
            assert_eq!(
                evaluate_many(&module, &inputs, 200).unwrap(),
                gold,
                "trips={trips} variant={variant}"
            );
            let dir = TestDir::new();
            let object = compile_source(&dir, &source);
            let executable = link_harness(&dir, &object);
            assert_eq!(
                run_harness(&dir, &executable, &inputs),
                gold,
                "trips={trips} variant={variant}"
            );
        }
    }

    // A valid but meaning-changing control breaks the three-cycle. Both
    // consumers must expose its changed result, rather than pass the old gold.
    let changed = base.replace(
        "%a = phi.u64 [^entry, %x] [^head, %b]",
        "%a = phi.u64 [^entry, %x] [^head, %a]",
    );
    assert_ne!(changed, base);
    let module = parse(changed.as_bytes()).unwrap();
    assert_eq!(evaluate_many(&module, &inputs, 200).unwrap(), inputs);
    let dir = TestDir::new();
    let object = compile_source(&dir, &changed);
    let executable = link_harness(&dir, &object);
    assert_eq!(run_harness(&dir, &executable, &inputs), inputs);

    // A valid nonterminating function is not a compile rejection. Check the
    // runner's deadline independently of the interpreter's fuel classification.
    let forever =
        "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n br ^loop\n^loop:\n br ^loop\n}\n";
    let module = parse(forever.as_bytes()).unwrap();
    assert_eq!(
        evaluate_many(&module, &[0], 20),
        Err(ai_compiler::EvalError::FuelExhausted)
    );
    let dir = TestDir::new();
    let object = compile_source(&dir, forever);
    let executable = link_harness(&dir, &object);
    let input = dir.path("forever.in");
    fs::write(&input, b"0\n").unwrap();
    let result = Command::new("timeout")
        .args(["--kill-after=1s", "0.1s"])
        .arg(executable)
        .stdin(Stdio::from(File::open(input).unwrap()))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(124), "{result:?}");
    assert!(result.stdout.is_empty());
}

#[test]
fn abi_probe_checks_preserved_controls_and_restores_caller_state() {
    let cases = [
        (
            "entry_and_rounding_modes",
            "leaq 8(%rsp), %rax\n testq $15, %rax\n jne .Lbad\n pushfq\n popq %rax\n testq $0x400, %rax\n jne .Lbad\n fnstcw -8(%rsp)\n movzwl -8(%rsp), %eax\n andl $0x0c00, %eax\n cmpl $0x0400, %eax\n jne .Lbad\n stmxcsr -4(%rsp)\n movl -4(%rsp), %eax\n andl $0x6000, %eax\n cmpl $0x4000, %eax\n jne .Lbad\n movq %rdi, %rax\n ret\n.Lbad:\n movq %rdi, %rax\n notq %rax\n ret\n",
            0,
        ),
        ("rbp", "movq $0, %rbp\n movq %rdi, %rax\n ret\n", 2),
        ("direction_flag", "std\n movq %rdi, %rax\n ret\n", 2),
        (
            "x87_control",
            "fnstcw -8(%rsp)\n xorw $0x0400, -8(%rsp)\n fldcw -8(%rsp)\n movq %rdi, %rax\n ret\n",
            2,
        ),
        (
            "mxcsr_control",
            "stmxcsr -4(%rsp)\n xorl $0x2000, -4(%rsp)\n ldmxcsr -4(%rsp)\n movq %rdi, %rax\n ret\n",
            2,
        ),
        (
            "mxcsr_status",
            "stmxcsr -4(%rsp)\n xorl $0x20, -4(%rsp)\n ldmxcsr -4(%rsp)\n movq %rdi, %rax\n ret\n",
            0,
        ),
    ];
    for (name, body, expected_status) in cases {
        let dir = TestDir::new();
        let assembly_path = dir.path("abi-case.s");
        let object = dir.path("abi-case.o");
        fs::write(
            &assembly_path,
            format!(
                ".text\n.globl aic_main\n.type aic_main, @function\naic_main:\n {body}.size aic_main, .-aic_main\n.section .note.GNU-stack,\"\",@progbits\n"
            ),
        )
        .unwrap();
        let assembled = Command::new("as")
            .arg("--64")
            .arg("-march=generic64")
            .arg("-o")
            .arg(&object)
            .arg(&assembly_path)
            .output()
            .unwrap();
        assert!(
            assembled.status.success(),
            "assembler failed for {name}: {}",
            String::from_utf8_lossy(&assembled.stderr)
        );
        let executable = link_harness(&dir, &object);
        let input_path = dir.path("input.txt");
        fs::write(&input_path, "37\n").unwrap();
        let result = Command::new(executable)
            .stdin(Stdio::from(File::open(input_path).unwrap()))
            .output()
            .unwrap();

        assert_eq!(result.status.code(), Some(expected_status), "{name}");
        assert_eq!(
            result.stdout,
            if expected_status == 0 {
                b"37\n".as_slice()
            } else {
                b"".as_slice()
            },
            "{name}"
        );
    }
}
