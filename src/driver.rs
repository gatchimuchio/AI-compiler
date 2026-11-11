use std::ffi::OsStr;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::diagnostic::{Diagnostic, Result};
use crate::elf::verify_elf64_rel_x86_64_bytes;
use crate::parser::parse;
use crate::validate::validate;
use crate::x86_64::emit_assembly;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TempDirectory(PathBuf);

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn compile_object(input: &[u8], target: &str, output: &Path) -> Result<()> {
    compile_object_with_assembler(input, target, output, OsStr::new("as"))
}

fn compile_object_with_assembler(
    input: &[u8],
    target: &str,
    output: &Path,
    assembler: &OsStr,
) -> Result<()> {
    let module = parse(input)?;
    let validated = validate(&module)?;
    if target != "x86_64-linux-gnu" {
        return Err(Diagnostic::global(
            "E_TARGET",
            "supported target is x86_64-linux-gnu",
        ));
    }
    verify_gnu_assembler(assembler)?;
    let assembly = emit_assembly(&module, &validated)?;

    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if !parent.is_dir() {
        return Err(Diagnostic::global(
            "E_OUTPUT",
            "output directory does not exist",
        ));
    }
    let temporary = reserve_temp_directory(parent)?;
    let assembly_path = temporary.0.join("input.s");
    let object_path = temporary.0.join("output.o");
    write_file(&assembly_path, assembly.as_bytes())?;

    let result = Command::new(assembler)
        .arg("--64")
        .arg("-march=generic64")
        .arg("-o")
        .arg(&object_path)
        .arg(&assembly_path)
        .output()
        .map_err(|error| {
            Diagnostic::global("E_TOOL", format!("could not start GNU assembler: {error}"))
        })?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let stderr = stderr.trim();
        let detail = if stderr.is_empty() {
            format!("GNU assembler exited with {}", result.status)
        } else {
            format!("GNU assembler exited with {}: {stderr}", result.status)
        };
        return Err(Diagnostic::global("E_TOOL", detail));
    }
    verify_elf64_rel_x86_64(&object_path)?;
    fs::rename(&object_path, output).map_err(|error| {
        Diagnostic::global(
            "E_OUTPUT",
            format!("could not atomically install output object: {error}"),
        )
    })?;
    Ok(())
}

fn verify_gnu_assembler(assembler: &OsStr) -> Result<()> {
    let output = Command::new(assembler)
        .env("LC_ALL", "C")
        .arg("--version")
        .output()
        .map_err(|error| {
            Diagnostic::global(
                "E_TOOL",
                format!("could not query GNU assembler version: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(Diagnostic::global(
            "E_TOOL",
            format!("GNU assembler version query exited with {}", output.status),
        ));
    }
    let banner = String::from_utf8_lossy(&output.stdout);
    if !banner
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("GNU assembler"))
    {
        return Err(Diagnostic::global(
            "E_TOOL",
            "selected assembler does not identify itself as GNU assembler",
        ));
    }
    Ok(())
}

fn reserve_temp_directory(parent: &Path) -> Result<TempDirectory> {
    for _ in 0..100 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".aic-{}-{sequence}", std::process::id()));
        let mut builder = DirBuilder::new();
        builder.mode(0o700);
        match builder.create(&path) {
            Ok(()) => return Ok(TempDirectory(path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(Diagnostic::global(
                    "E_OUTPUT",
                    format!("could not create private temporary directory: {error}"),
                ));
            }
        }
    }
    Err(Diagnostic::global(
        "E_OUTPUT",
        "could not reserve a unique private temporary directory",
    ))
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            Diagnostic::global(
                "E_OUTPUT",
                format!("could not write temporary assembly: {error}"),
            )
        })?;
    file.write_all(bytes).map_err(|error| {
        Diagnostic::global(
            "E_OUTPUT",
            format!("could not write temporary assembly: {error}"),
        )
    })
}

fn verify_elf64_rel_x86_64(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        Diagnostic::global(
            "E_OUTPUT",
            format!("could not inspect assembler output: {error}"),
        )
    })?;
    if !metadata.file_type().is_file() {
        return Err(Diagnostic::global(
            "E_OUTPUT",
            "assembler output is not a regular file",
        ));
    }
    let bytes = fs::read(path).map_err(|error| {
        Diagnostic::global(
            "E_OUTPUT",
            format!("could not read assembler output: {error}"),
        )
    })?;
    verify_elf64_rel_x86_64_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Do not create executable fixtures while another fixture is spawning:
    // an inherited writable descriptor can transiently make exec fail ETXTBSY.
    static ASSEMBLER_FIXTURE: Mutex<()> = Mutex::new(());

    struct TemporaryDirectory(PathBuf);

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn elf64_rel_header() -> [u8; 64] {
        let mut header = [0u8; 64];
        header[0..4].copy_from_slice(b"\x7fELF");
        header[4] = 2;
        header[5] = 1;
        header[6] = 1;
        header[16..18].copy_from_slice(&1u16.to_le_bytes());
        header[18..20].copy_from_slice(&62u16.to_le_bytes());
        header[20..24].copy_from_slice(&1u32.to_le_bytes());
        header[52..54].copy_from_slice(&64u16.to_le_bytes());
        header
    }

    #[test]
    fn rejects_header_only_elf_and_noncurrent_identification_version() {
        let mut header = elf64_rel_header();
        assert_eq!(
            verify_elf64_rel_x86_64_bytes(&header).unwrap_err().code,
            "E_OUTPUT"
        );

        header[6] = 0;
        assert_eq!(
            verify_elf64_rel_x86_64_bytes(&header).unwrap_err().code,
            "E_OUTPUT"
        );
    }

    #[test]
    fn rejects_unselected_targets_before_creating_artifacts() {
        let output = std::env::temp_dir().join(format!(
            "aic-invalid-target-{}-{}.o",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let error = compile_object(source, "aarch64-linux-gnu", &output).unwrap_err();
        assert_eq!(error.code, "E_TARGET");
        assert!(!output.exists());
    }

    #[test]
    fn malformed_input_preserves_existing_output() {
        let output = std::env::temp_dir().join(format!(
            "aic-preserve-{}-{}.o",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&output, b"previous artifact").unwrap();
        let error = compile_object(b"AIC 0.9\n", "x86_64-linux-gnu", &output).unwrap_err();
        assert_eq!(error.code, "E_VERSION");
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        fs::remove_file(output).unwrap();
    }

    #[test]
    fn temporary_directory_is_private_and_removed_on_drop() {
        use std::os::unix::fs::PermissionsExt;

        let parent = std::env::temp_dir().join(format!(
            "aic-temp-parent-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&parent).unwrap();
        let temporary = reserve_temp_directory(&parent).unwrap();
        let path = temporary.0.clone();
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "private directory mode was {mode:o}");
        drop(temporary);
        assert!(!path.exists());
        fs::remove_dir(parent).unwrap();
    }

    #[test]
    fn rejects_non_gnu_assembler_and_preserves_existing_output() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();

        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-non-gnu-as-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();
        let assembler = directory.0.join("portable-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nprintf 'Portable assembler 1.0\\n'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";

        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();

        assert_eq!(error.code, "E_TOOL");
        assert!(
            error
                .detail
                .contains("does not identify itself as GNU assembler"),
            "unexpected diagnostic: {error}"
        );
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);
    }

    #[test]
    fn assembler_failure_cleans_temporary_files_and_preserves_output() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();

        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-as-failure-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();
        let assembler = directory.0.join("failing-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\nprintf 'fixture assembler failure\\n' >&2\nexit 17\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";

        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();

        assert_eq!(error.code, "E_TOOL");
        assert!(error.detail.contains("fixture assembler failure"));
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);
    }

    #[test]
    fn invalid_elf_output_preserves_destination_and_cleans_temporary_files() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();

        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-invalid-elf-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();

        fs::write(directory.0.join("invalid-elf.o"), elf64_rel_header()).unwrap();

        let assembler = directory.0.join("invalid-elf-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\nif [ \"$1\" != \"--64\" ] || [ \"$2\" != \"-march=generic64\" ] || [ \"$3\" != \"-o\" ]; then\nprintf 'unexpected assembler arguments\\n' >&2\nexit 16\nfi\ncp \"${0%/*}/invalid-elf.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";

        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();

        assert_eq!(error.code, "E_OUTPUT", "unexpected diagnostic: {error}");
        assert!(error.detail.contains("section header"));
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 3);
    }

    #[test]
    fn unsupported_link_requirements_preserve_destination_and_clean_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-link-requirements-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();
        let assembler = directory.0.join("substituting-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\nif [ \"$1\" != \"--64\" ] || [ \"$2\" != \"-march=generic64\" ] || [ \"$3\" != \"-o\" ]; then\nexit 16\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let mut accepted = Vec::new();
        for (name, body, expected_inspection, expected_error) in [
            (
                "external call",
                "call fixture_external\nret\n",
                "R_X86_64_PLT32",
                "relocation sections are unsupported",
            ),
            (
                "local absolute address",
                ".Laic_local:\n.quad .Laic_local\nret\n",
                "R_X86_64_64",
                "relocation sections are unsupported",
            ),
            (
                "unused undefined symbol",
                "ret\n.globl fixture_external\n",
                "UND fixture_external",
                "undefined symbols are unsupported",
            ),
            (
                "common storage",
                "ret\n.comm fixture_common,8,8\n",
                "COM fixture_common",
                "common symbols are unsupported",
            ),
        ] {
            let assembly = format!(
                ".text\n.globl aic_main\n.type aic_main,@function\naic_main:\n{body}.size aic_main,.-aic_main\n.section .note.GNU-stack,\"\",@progbits\n"
            );
            let input = directory.0.join("fixture.s");
            let object = directory.0.join("fixture.o");
            fs::write(&input, assembly).unwrap();
            let assembled = Command::new("as")
                .args(["--64", "-march=generic64", "-o"])
                .arg(&object)
                .arg(&input)
                .output()
                .unwrap();
            assert!(assembled.status.success(), "{name}: {assembled:?}");
            let inspection = Command::new("readelf")
                .env("LC_ALL", "C")
                .args(["--wide", "--relocs", "--symbols"])
                .arg(&object)
                .output()
                .unwrap();
            assert!(inspection.status.success(), "{name}: {inspection:?}");
            let inspected = String::from_utf8(inspection.stdout).unwrap();
            assert!(
                inspected.contains(expected_inspection),
                "{name}: {inspected}"
            );

            match compile_object_with_assembler(
                source,
                "x86_64-linux-gnu",
                &output,
                assembler.as_os_str(),
            ) {
                Ok(()) => accepted.push(name),
                Err(error) => {
                    assert_eq!(error.code, "E_OUTPUT", "{name}: {error}");
                    assert!(error.detail.contains(expected_error), "{name}: {error}");
                    assert_eq!(fs::read(&output).unwrap(), b"previous artifact", "{name}");
                }
            }
            // Reset only the test's destination so all counterexamples can run.
            fs::write(&output, b"previous artifact").unwrap();
            assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 4, "{name}");
        }
        assert!(accepted.is_empty(), "unexpectedly installed: {accepted:?}");
    }

    #[test]
    fn rejects_symlink_assembler_output_and_preserves_destination() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();

        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-symlink-elf-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();
        fs::write(
            directory.0.join("matching-elf-header.o"),
            elf64_rel_header(),
        )
        .unwrap();

        let assembler = directory.0.join("symlink-elf-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\nln -s \"${0%/*}/matching-elf-header.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";

        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();

        assert_eq!(error.code, "E_OUTPUT", "unexpected diagnostic: {error}");
        assert!(error.detail.contains("regular file"));
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 3);
    }
}
