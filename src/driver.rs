use std::ffi::OsStr;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::diagnostic::{Diagnostic, Result};
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
        .arg("-o")
        .arg(&object_path)
        .arg(&assembly_path)
        .output()
        .map_err(|error| {
            Diagnostic::global("E_TOOL", format!("could not start GNU assembler: {error}"))
        })?;
    if !result.status.success() {
        return Err(Diagnostic::global(
            "E_TOOL",
            format!("GNU assembler exited with {}", result.status),
        ));
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
    let bytes = fs::read(path).map_err(|error| {
        Diagnostic::global(
            "E_OUTPUT",
            format!("could not read assembler output: {error}"),
        )
    })?;
    if bytes.len() < 64
        || bytes.get(0..4) != Some(b"\x7fELF")
        || bytes[4] != 2
        || bytes[5] != 1
        || u16::from_le_bytes([bytes[16], bytes[17]]) != 1
        || u16::from_le_bytes([bytes[18], bytes[19]]) != 62
        || u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) != 1
        || u16::from_le_bytes([bytes[52], bytes[53]]) != 64
    {
        return Err(Diagnostic::global(
            "E_OUTPUT",
            "assembler output is not an ELF64 little-endian x86-64 ET_REL object",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TemporaryDirectory(PathBuf);

    impl Drop for TemporaryDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
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
                .contains("does not identify itself as GNU assembler")
        );
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);
    }

    #[test]
    fn assembler_failure_cleans_temporary_files_and_preserves_output() {
        use std::os::unix::fs::PermissionsExt;

        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-as-failure-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();
        let assembler = directory.0.join("failing-as");
        fs::write(&assembler, b"#!/bin/sh\nexit 17\n").unwrap();
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
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);
    }
}
