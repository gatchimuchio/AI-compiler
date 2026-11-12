use std::ffi::OsStr;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::diagnostic::{Diagnostic, Result};
use crate::elf::verify_elf64_rel_x86_64_bytes;
use crate::limits::{MAX_ASSEMBLER_RUNTIME_SECONDS, MAX_ASSEMBLER_STREAM_BYTES, MAX_OBJECT_BYTES};
use crate::parser::parse;
use crate::validate::validate;
use crate::x86_64::emit_validated_assembly;

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
    compile_object_with_assembler_runtime_limit(
        input,
        target,
        output,
        assembler,
        Duration::from_secs(MAX_ASSEMBLER_RUNTIME_SECONDS),
    )
}

fn compile_object_with_assembler_runtime_limit(
    input: &[u8],
    target: &str,
    output: &Path,
    assembler: &OsStr,
    runtime_limit: Duration,
) -> Result<()> {
    let module = parse(input)?;
    let validated = validate(&module)?;
    if target != "x86_64-linux-gnu" {
        return Err(Diagnostic::global(
            "E_TARGET",
            "supported target is x86_64-linux-gnu",
        ));
    }
    verify_gnu_assembler(assembler, runtime_limit)?;
    let assembly = emit_validated_assembly(&module, &validated)?;

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

    let result = run_bounded_tool(
        Command::new(assembler)
            .arg("--64")
            .arg("-march=generic64")
            .arg("-o")
            .arg(&object_path)
            .arg(&assembly_path),
        "GNU assembler",
        MAX_ASSEMBLER_STREAM_BYTES,
        runtime_limit,
    )?;
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

fn verify_gnu_assembler(assembler: &OsStr, runtime_limit: Duration) -> Result<()> {
    let output = run_bounded_tool(
        Command::new(assembler).env("LC_ALL", "C").arg("--version"),
        "GNU assembler version query",
        MAX_ASSEMBLER_STREAM_BYTES,
        runtime_limit,
    )?;
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

fn stop_child(child: &mut Child, original: Diagnostic, operation: &str) -> Diagnostic {
    let _ = child.kill();
    match child.wait() {
        Ok(_) => original,
        Err(error) => Diagnostic::global(
            original.code,
            format!("{}; could not reap {operation}: {error}", original.detail),
        ),
    }
}

fn read_available(
    stream: &mut UnixStream,
    captured: &mut Vec<u8>,
    stream_name: &str,
    operation: &str,
    limit: usize,
) -> Result<bool> {
    loop {
        let remaining = limit
            .checked_add(1)
            .and_then(|bound| bound.checked_sub(captured.len()))
            .ok_or_else(|| {
                Diagnostic::global(
                    "E_LIMIT",
                    format!("{operation} {stream_name} exceeded the {limit}-byte capture limit"),
                )
            })?;
        if remaining == 0 {
            return Err(Diagnostic::global(
                "E_LIMIT",
                format!("{operation} {stream_name} exceeded the {limit}-byte capture limit"),
            ));
        }
        let mut chunk = [0u8; 4096];
        let read_limit = remaining.min(chunk.len());
        match stream.read(&mut chunk[..read_limit]) {
            Ok(0) => return Ok(true),
            Ok(count) => {
                let capture_count = count.min(limit.saturating_sub(captured.len()));
                captured.extend_from_slice(&chunk[..capture_count]);
                if capture_count < count {
                    return Err(Diagnostic::global(
                        "E_LIMIT",
                        format!(
                            "{operation} {stream_name} exceeded the {limit}-byte capture limit"
                        ),
                    ));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Err(Diagnostic::global(
                    "E_TOOL",
                    format!("could not read {operation} {stream_name}: {error}"),
                ));
            }
        }
    }
}

fn run_bounded_tool(
    command: &mut Command,
    operation: &str,
    stream_limit: usize,
    runtime_limit: Duration,
) -> Result<Output> {
    let (mut stdout, child_stdout) = UnixStream::pair().map_err(|error| {
        Diagnostic::global(
            "E_TOOL",
            format!("could not capture {operation} stdout: {error}"),
        )
    })?;
    let (mut stderr, child_stderr) = UnixStream::pair().map_err(|error| {
        Diagnostic::global(
            "E_TOOL",
            format!("could not capture {operation} stderr: {error}"),
        )
    })?;
    stdout.set_nonblocking(true).map_err(|error| {
        Diagnostic::global(
            "E_TOOL",
            format!("could not configure {operation} stdout: {error}"),
        )
    })?;
    stderr.set_nonblocking(true).map_err(|error| {
        Diagnostic::global(
            "E_TOOL",
            format!("could not configure {operation} stderr: {error}"),
        )
    })?;
    let stdout_fd: OwnedFd = child_stdout.into();
    let stderr_fd: OwnedFd = child_stderr.into();
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_fd))
        .stderr(Stdio::from(stderr_fd))
        .spawn()
        .map_err(|error| {
            Diagnostic::global("E_TOOL", format!("could not start {operation}: {error}"))
        })?;

    let started = Instant::now();
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    let mut stdout_eof = false;
    let mut stderr_eof = false;
    loop {
        if !stdout_eof {
            stdout_eof = match read_available(
                &mut stdout,
                &mut stdout_bytes,
                "stdout",
                operation,
                stream_limit,
            ) {
                Ok(done) => done,
                Err(error) => return Err(stop_child(&mut child, error, operation)),
            };
        }

        if !stderr_eof {
            stderr_eof = match read_available(
                &mut stderr,
                &mut stderr_bytes,
                "stderr",
                operation,
                stream_limit,
            ) {
                Ok(done) => done,
                Err(error) => return Err(stop_child(&mut child, error, operation)),
            };
        }

        let status = match child.try_wait() {
            Ok(status) => status,
            Err(error) => {
                let failure = Diagnostic::global(
                    "E_TOOL",
                    format!("could not inspect {operation} status: {error}"),
                );
                return Err(stop_child(&mut child, failure, operation));
            }
        };
        if let Some(status) = status {
            // Read currently available bytes after exit; inherited descendant streams
            // are outside this direct-child lifecycle and are not awaited.
            for (stream, bytes, name, eof) in [
                (&mut stdout, &mut stdout_bytes, "stdout", &mut stdout_eof),
                (&mut stderr, &mut stderr_bytes, "stderr", &mut stderr_eof),
            ] {
                if !*eof {
                    *eof = read_available(stream, bytes, name, operation, stream_limit)?;
                }
            }
            return Ok(Output {
                status,
                stdout: stdout_bytes,
                stderr: stderr_bytes,
            });
        }
        if started.elapsed() >= runtime_limit {
            let failure = Diagnostic::global(
                "E_TOOL",
                format!(
                    "{operation} exceeded the runtime limit ({} ms)",
                    runtime_limit.as_millis()
                ),
            );
            return Err(stop_child(&mut child, failure, operation));
        }
        thread::sleep(Duration::from_millis(2));
    }
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
    // Bound the observed bytes even if a regular file grows after inspection.
    // The verifier rejects the extra byte instead of accepting a truncated ELF.
    let bytes = fs::File::open(path)
        .and_then(|file| {
            let mut bytes = Vec::new();
            file.take((MAX_OBJECT_BYTES + 1) as u64)
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        })
        .map_err(|error| {
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
    fn bounded_tool_capture_accepts_exact_limit_on_both_streams() {
        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let limit = MAX_ASSEMBLER_STREAM_BYTES.to_string();
        let runtime = Duration::from_secs(2);
        let mut stdout_command = Command::new("head");
        stdout_command.args(["-c", &limit, "/dev/zero"]);
        let stdout = run_bounded_tool(
            &mut stdout_command,
            "capture fixture",
            MAX_ASSEMBLER_STREAM_BYTES,
            runtime,
        )
        .unwrap();
        assert!(stdout.status.success());
        assert_eq!(stdout.stdout.len(), MAX_ASSEMBLER_STREAM_BYTES);
        assert!(stdout.stderr.is_empty());

        let mut stderr_command = Command::new("sh");
        stderr_command.args(["-c", &format!("exec head -c {limit} /dev/zero >&2")]);
        let stderr = run_bounded_tool(
            &mut stderr_command,
            "capture fixture",
            MAX_ASSEMBLER_STREAM_BYTES,
            runtime,
        )
        .unwrap();
        assert!(stderr.status.success());
        assert!(stderr.stdout.is_empty());
        assert_eq!(stderr.stderr.len(), MAX_ASSEMBLER_STREAM_BYTES);
    }

    #[test]
    fn bounded_tool_capture_drains_both_streams_without_deadlock() {
        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let limit = MAX_ASSEMBLER_STREAM_BYTES.to_string();
        let mut command = Command::new("sh");
        command.args([
            "-c",
            &format!("head -c {limit} /dev/zero & head -c {limit} /dev/zero >&2 & wait"),
        ]);
        let output = run_bounded_tool(
            &mut command,
            "simultaneous capture fixture",
            MAX_ASSEMBLER_STREAM_BYTES,
            Duration::from_secs(2),
        )
        .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout.len(), MAX_ASSEMBLER_STREAM_BYTES);
        assert_eq!(output.stderr.len(), MAX_ASSEMBLER_STREAM_BYTES);
    }

    #[test]
    fn bounded_tool_capture_rejects_excess_on_both_streams() {
        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let limit = (MAX_ASSEMBLER_STREAM_BYTES + 1).to_string();
        let mut stdout_command = Command::new("head");
        stdout_command.args(["-c", &limit, "/dev/zero"]);
        let stdout = run_bounded_tool(
            &mut stdout_command,
            "capture fixture",
            MAX_ASSEMBLER_STREAM_BYTES,
            Duration::from_secs(2),
        )
        .unwrap_err();
        assert_eq!(stdout.code, "E_LIMIT");
        assert!(stdout.detail.contains("stdout"));

        let mut stderr_command = Command::new("sh");
        stderr_command.args(["-c", &format!("exec head -c {limit} /dev/zero >&2")]);
        let stderr = run_bounded_tool(
            &mut stderr_command,
            "capture fixture",
            MAX_ASSEMBLER_STREAM_BYTES,
            Duration::from_secs(2),
        )
        .unwrap_err();
        assert_eq!(stderr.code, "E_LIMIT");
        assert!(stderr.detail.contains("stderr"));
    }

    #[test]
    fn bounded_tool_runtime_limit_kills_and_reaps_the_child() {
        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let mut command = Command::new("sleep");
        command.arg("2");
        let error = run_bounded_tool(
            &mut command,
            "runtime fixture",
            MAX_ASSEMBLER_STREAM_BYTES,
            Duration::from_millis(50),
        )
        .unwrap_err();
        assert_eq!(error.code, "E_TOOL");
        assert!(error.detail.contains("runtime limit"));
    }

    #[test]
    fn assembler_capture_limits_preserve_output_and_clean_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-as-capture-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();
        let assembler = directory.0.join("capturing-as");
        let excessive_count = (MAX_ASSEMBLER_STREAM_BYTES + 1).to_string();
        let overflowing_query = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexec head -c {excessive_count} /dev/zero\nfi\nexit 0\n"
        );
        fs::write(&assembler, overflowing_query.as_bytes()).unwrap();
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
        assert_eq!(error.code, "E_LIMIT");
        assert!(error.detail.contains("stdout"));
        assert!(error.detail.contains("version query"));
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);

        let overflowing_assembly = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\nexec head -c {excessive_count} /dev/zero\n"
        );
        fs::write(&assembler, overflowing_assembly.as_bytes()).unwrap();
        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();
        assert_eq!(error.code, "E_LIMIT");
        assert!(error.detail.contains("stdout"));
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);
    }

    #[test]
    fn assembler_runtime_limits_preserve_output_and_clean_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-as-runtime-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();
        let assembler = directory.0.join("sleeping-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nexec sleep 2\nfi\nexit 0\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let runtime_limit = Duration::from_millis(50);

        let error = compile_object_with_assembler_runtime_limit(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
            runtime_limit,
        )
        .unwrap_err();
        assert_eq!(error.code, "E_TOOL");
        assert!(error.detail.contains("version query"), "{error}");
        assert!(error.detail.contains("runtime limit"), "{error}");
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 2);

        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\nexec sleep 2\n",
        )
        .unwrap();
        let error = compile_object_with_assembler_runtime_limit(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
            runtime_limit,
        )
        .unwrap_err();
        assert_eq!(error.code, "E_TOOL");
        assert!(!error.detail.contains("version query"), "{error}");
        assert!(error.detail.contains("runtime limit"), "{error}");
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
    fn unsupported_initialization_and_termination_arrays_preserve_output_and_clean_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-init-termination-arrays-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let input = directory.0.join("fixture.s");
        let fixture = directory.0.join("fixture.o");
        let output = directory.0.join("preserved.o");
        let assembler = directory.0.join("substituting-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let mut accepted = Vec::new();

        for (section, section_type) in [
            ("preinit_array", "@preinit_array"),
            ("init_array", "@init_array"),
            ("fini_array", "@fini_array"),
        ] {
            let assembly = format!(
                ".text\n.globl aic_main\n.type aic_main,@function\naic_main:\nret\n.size aic_main,.-aic_main\n.section .{section},\"aw\",{section_type}\n.p2align 3\n.quad 0\n.section .note.GNU-stack,\"\",@progbits\n"
            );
            fs::write(&input, assembly).unwrap();
            let assembled = Command::new("as")
                .args(["--64", "-march=generic64", "-o"])
                .arg(&fixture)
                .arg(&input)
                .output()
                .unwrap();
            assert!(assembled.status.success(), "{section}: {assembled:?}");

            let inspection = Command::new("readelf")
                .env("LC_ALL", "C")
                .args(["--wide", "--section-headers"])
                .arg(&fixture)
                .output()
                .unwrap();
            assert!(inspection.status.success(), "{section}: {inspection:?}");
            let inspected = String::from_utf8(inspection.stdout).unwrap();
            let section_type_name = section.to_ascii_uppercase();
            assert!(
                inspected.lines().any(|line| {
                    line.contains(&format!(".{section}")) && line.contains(&section_type_name)
                }),
                "{section}: {inspected}"
            );

            fs::write(&output, b"previous artifact").unwrap();
            match compile_object_with_assembler(
                source,
                "x86_64-linux-gnu",
                &output,
                assembler.as_os_str(),
            ) {
                Ok(()) => accepted.push(section),
                Err(error) => {
                    assert_eq!(error.code, "E_OUTPUT", "{section}: {error}");
                    assert!(error.detail.starts_with("unsupported assembler output:"));
                    assert!(
                        error
                            .detail
                            .contains("initialization and termination arrays are unsupported"),
                        "{section}: {error}"
                    );
                    assert_eq!(
                        fs::read(&output).unwrap(),
                        b"previous artifact",
                        "{section}"
                    );
                }
            }
            assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 4, "{section}");
        }
        assert!(accepted.is_empty(), "unexpectedly installed: {accepted:?}");
    }

    #[test]
    fn unsupported_thread_local_sections_preserve_output_and_clean_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-tls-sections-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let input = directory.0.join("fixture.s");
        let fixture = directory.0.join("fixture.o");
        let output = directory.0.join("preserved.o");
        let assembler = directory.0.join("substituting-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let mut accepted = Vec::new();

        for (section, section_type, directive, payload) in [
            ("tdata", "PROGBITS", "@progbits", ".quad 0"),
            ("tbss", "NOBITS", "@nobits", ".zero 8"),
        ] {
            let assembly = format!(
                ".text\n.globl aic_main\n.type aic_main,@function\naic_main:\nret\n.size aic_main,.-aic_main\n.section .{section},\"awT\",{directive}\n.p2align 3\n{payload}\n.section .note.GNU-stack,\"\",@progbits\n"
            );
            fs::write(&input, assembly).unwrap();
            let assembled = Command::new("as")
                .args(["--64", "-march=generic64", "-o"])
                .arg(&fixture)
                .arg(&input)
                .output()
                .unwrap();
            assert!(assembled.status.success(), "{section}: {assembled:?}");

            let inspection = Command::new("readelf")
                .env("LC_ALL", "C")
                .args(["--wide", "--section-headers"])
                .arg(&fixture)
                .output()
                .unwrap();
            assert!(inspection.status.success(), "{section}: {inspection:?}");
            let inspected = String::from_utf8(inspection.stdout).unwrap();
            assert!(
                inspected.lines().any(|line| {
                    line.contains(&format!(".{section}"))
                        && line.contains(section_type)
                        && line.split_whitespace().any(|field| field == "WAT")
                }),
                "{section}: {inspected}"
            );

            fs::write(&output, b"previous artifact").unwrap();
            match compile_object_with_assembler(
                source,
                "x86_64-linux-gnu",
                &output,
                assembler.as_os_str(),
            ) {
                Ok(()) => accepted.push(section),
                Err(error) => {
                    assert_eq!(error.code, "E_OUTPUT", "{section}: {error}");
                    assert!(error.detail.starts_with("unsupported assembler output:"));
                    assert!(
                        error.detail.contains("thread-local storage is unsupported"),
                        "{section}: {error}"
                    );
                    assert_eq!(
                        fs::read(&output).unwrap(),
                        b"previous artifact",
                        "{section}"
                    );
                }
            }
            assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 4, "{section}");
        }
        assert!(accepted.is_empty(), "unexpectedly installed: {accepted:?}");
    }

    #[test]
    fn unsupported_allocated_data_sections_preserve_output_and_clean_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-allocated-data-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let input = directory.0.join("fixture.s");
        let fixture = directory.0.join("fixture.o");
        let output = directory.0.join("preserved.o");
        let assembler = directory.0.join("substituting-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let mut accepted = Vec::new();

        for (section, section_type, flags, assembler_flags, directive, payload) in [
            (".data", "PROGBITS", "WA", "aw", "@progbits", ".quad 42"),
            (".rodata", "PROGBITS", "A", "a", "@progbits", ".quad 42"),
            (".bss", "NOBITS", "WA", "aw", "@nobits", ".zero 8"),
        ] {
            let assembly = format!(
                ".text\n.globl aic_main\n.type aic_main,@function\naic_main:\nret\n.size aic_main,.-aic_main\n.section {section},\"{assembler_flags}\",{directive}\n.p2align 3\n{payload}\n.section .note.GNU-stack,\"\",@progbits\n"
            );
            fs::write(&input, assembly).unwrap();
            let assembled = Command::new("as")
                .args(["--64", "-march=generic64", "-o"])
                .arg(&fixture)
                .arg(&input)
                .output()
                .unwrap();
            assert!(assembled.status.success(), "{section}: {assembled:?}");

            let inspection = Command::new("readelf")
                .env("LC_ALL", "C")
                .args(["--wide", "--section-headers"])
                .arg(&fixture)
                .output()
                .unwrap();
            assert!(inspection.status.success(), "{section}: {inspection:?}");
            let inspected = String::from_utf8(inspection.stdout).unwrap();
            let row = inspected
                .lines()
                .find(|line| line.split_whitespace().nth(2) == Some(section))
                .unwrap_or_else(|| panic!("{section}: {inspected}"));
            let columns: Vec<_> = row.split_whitespace().collect();
            assert_eq!(columns.get(3).copied(), Some(section_type), "{row}");
            assert_eq!(columns.get(6).copied(), Some("000008"), "{row}");
            assert_eq!(columns.get(8).copied(), Some(flags), "{row}");

            fs::write(&output, b"previous artifact").unwrap();
            match compile_object_with_assembler(
                source,
                "x86_64-linux-gnu",
                &output,
                assembler.as_os_str(),
            ) {
                Ok(()) => accepted.push(section),
                Err(error) => {
                    assert_eq!(error.code, "E_OUTPUT", "{section}: {error}");
                    assert!(error.detail.starts_with("unsupported assembler output:"));
                    assert!(
                        error
                            .detail
                            .contains("writable or non-code allocated sections are unsupported"),
                        "{section}: {error}"
                    );
                    assert_eq!(
                        fs::read(&output).unwrap(),
                        b"previous artifact",
                        "{section}"
                    );
                }
            }
            assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 4, "{section}");
        }
        assert!(accepted.is_empty(), "unexpectedly installed: {accepted:?}");
    }

    #[test]
    fn excluded_code_section_preserves_output_and_cleans_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-excluded-section-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let input = directory.0.join("fixture.s");
        let fixture = directory.0.join("fixture.o");
        let output = directory.0.join("preserved.o");
        let assembler = directory.0.join("substituting-as");
        fs::write(
            &input,
            ".section .text.aic_main,\"axe\",@progbits\n.globl aic_main\n.type aic_main,@function\naic_main:\nmovq %rdi,%rax\nret\n.size aic_main,.-aic_main\n.section .note.GNU-stack,\"\",@progbits\n",
        )
        .unwrap();
        let assembled = Command::new("as")
            .args(["--64", "-march=generic64", "-o"])
            .arg(&fixture)
            .arg(&input)
            .output()
            .unwrap();
        assert!(assembled.status.success(), "{assembled:?}");

        let inspection = Command::new("readelf")
            .env("LC_ALL", "C")
            .args(["--wide", "--section-headers"])
            .arg(&fixture)
            .output()
            .unwrap();
        assert!(inspection.status.success(), "{inspection:?}");
        let inspected = String::from_utf8(inspection.stdout).unwrap();
        let row = inspected
            .lines()
            .find(|line| line.split_whitespace().nth(2) == Some(".text.aic_main"))
            .unwrap_or_else(|| panic!("{inspected}"));
        let columns: Vec<_> = row.split_whitespace().collect();
        assert_eq!(columns.get(3).copied(), Some("PROGBITS"), "{row}");
        assert_eq!(columns.get(6).copied(), Some("000004"), "{row}");
        assert_eq!(columns.get(8).copied(), Some("AXE"), "{row}");

        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        fs::write(&output, b"previous artifact").unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();
        assert_eq!(error.code, "E_OUTPUT");
        assert!(error.detail.starts_with("unsupported assembler output:"));
        assert!(error.detail.contains("excluded sections are unsupported"));
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 4);
    }

    #[test]
    fn mergeable_code_section_preserves_output_and_cleans_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-mergeable-section-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let input = directory.0.join("fixture.s");
        let fixture = directory.0.join("fixture.o");
        let output = directory.0.join("preserved.o");
        let assembler = directory.0.join("substituting-as");
        fs::write(
            &input,
            ".section .text.aic_main,\"axM\",@progbits,1\n.globl aic_main\n.type aic_main,@function\naic_main:\nmovl $0x01010101,%eax\nret\n.size aic_main,.-aic_main\n.section .note.GNU-stack,\"\",@progbits\n",
        )
        .unwrap();
        let assembled = Command::new("as")
            .args(["--64", "-march=generic64", "-o"])
            .arg(&fixture)
            .arg(&input)
            .output()
            .unwrap();
        assert!(assembled.status.success(), "{assembled:?}");

        let inspection = Command::new("readelf")
            .env("LC_ALL", "C")
            .args(["--wide", "--section-headers"])
            .arg(&fixture)
            .output()
            .unwrap();
        assert!(inspection.status.success(), "{inspection:?}");
        let inspected = String::from_utf8(inspection.stdout).unwrap();
        let row = inspected
            .lines()
            .find(|line| line.split_whitespace().nth(2) == Some(".text.aic_main"))
            .unwrap_or_else(|| panic!("{inspected}"));
        let columns: Vec<_> = row.split_whitespace().collect();
        assert_eq!(columns.get(3).copied(), Some("PROGBITS"), "{row}");
        assert_eq!(columns.get(6).copied(), Some("000006"), "{row}");
        assert_eq!(columns.get(7).copied(), Some("01"), "{row}");
        assert_eq!(columns.get(8).copied(), Some("AXM"), "{row}");

        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        fs::write(&output, b"previous artifact").unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();
        assert_eq!(error.code, "E_OUTPUT");
        assert!(error.detail.starts_with("unsupported assembler output:"));
        assert!(error.detail.contains("mergeable sections are unsupported"));
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 4);
    }

    #[test]
    fn link_order_code_section_preserves_output_and_cleans_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-link-order-section-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let input = directory.0.join("fixture.s");
        let fixture = directory.0.join("fixture.o");
        let output = directory.0.join("preserved.o");
        let assembler = directory.0.join("substituting-as");
        fs::write(
            &input,
            ".section .text.anchor,\"ax\",@progbits\n.globl anchor\n.type anchor,@function\nanchor:\nret\n.size anchor,.-anchor\n.section .text.aic_main,\"axo\",@progbits,anchor\n.globl aic_main\n.type aic_main,@function\naic_main:\nmovq %rdi,%rax\nret\n.size aic_main,.-aic_main\n.section .note.GNU-stack,\"\",@progbits\n",
        )
        .unwrap();
        let assembled = Command::new("as")
            .args(["--64", "-march=generic64", "-o"])
            .arg(&fixture)
            .arg(&input)
            .output()
            .unwrap();
        assert!(assembled.status.success(), "{assembled:?}");

        let inspection = Command::new("readelf")
            .env("LC_ALL", "C")
            .args(["--wide", "--section-headers"])
            .arg(&fixture)
            .output()
            .unwrap();
        assert!(inspection.status.success(), "{inspection:?}");
        let inspected = String::from_utf8(inspection.stdout).unwrap();
        let row = inspected
            .lines()
            .find(|line| line.split_whitespace().nth(2) == Some(".text.aic_main"))
            .unwrap_or_else(|| panic!("{inspected}"));
        let columns: Vec<_> = row.split_whitespace().collect();
        assert_eq!(columns.get(3).copied(), Some("PROGBITS"), "{row}");
        assert_eq!(columns.get(6).copied(), Some("000004"), "{row}");
        assert_eq!(columns.get(8).copied(), Some("AXL"), "{row}");
        let anchor_row = inspected
            .lines()
            .find(|line| line.split_whitespace().nth(2) == Some(".text.anchor"))
            .unwrap_or_else(|| panic!("{inspected}"));
        let anchor_index = anchor_row
            .split_whitespace()
            .nth(1)
            .expect("section index")
            .trim_matches(['[', ']']);
        assert_eq!(columns.get(9).copied(), Some(anchor_index), "{row}");

        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        fs::write(&output, b"previous artifact").unwrap();
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();
        assert_eq!(error.code, "E_OUTPUT");
        assert!(error.detail.starts_with("unsupported assembler output:"));
        assert!(error.detail.contains("link-order sections are unsupported"));
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 4);
    }

    #[test]
    fn unsupported_entry_and_program_headers_preserve_output_and_clean_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-object-profile-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let fixture = directory.0.join("fixture.o");
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        compile_object(source, "x86_64-linux-gnu", &fixture).unwrap();
        let original = fs::read(&fixture).unwrap();
        let assembler = directory.0.join("substituting-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let output = directory.0.join("preserved.o");
        let mut accepted = Vec::new();

        for (name, program_table, expected_detail) in [
            ("entry address", false, "entry points"),
            ("bounded PT_NULL table", true, "program headers"),
        ] {
            let mut bytes = original.clone();
            if program_table {
                let table_offset = bytes.len().next_multiple_of(8);
                bytes[32..40].copy_from_slice(&(table_offset as u64).to_le_bytes());
                bytes[54..56].copy_from_slice(&56u16.to_le_bytes());
                bytes[56..58].copy_from_slice(&1u16.to_le_bytes());
                bytes.resize(table_offset + 56, 0); // One bounded PT_NULL entry.
            } else {
                bytes[24..32].copy_from_slice(&1u64.to_le_bytes());
            }
            fs::write(&fixture, &bytes).unwrap();
            let inspection = Command::new("readelf")
                .env("LC_ALL", "C")
                .args(["--wide", "--file-header", "--program-headers"])
                .arg(&fixture)
                .output()
                .unwrap();
            assert!(inspection.status.success(), "{name}: {inspection:?}");
            let inspected = String::from_utf8(inspection.stdout).unwrap();
            if program_table {
                assert!(
                    inspected
                        .lines()
                        .any(|line| line.trim_start().starts_with("NULL")),
                    "{name}: {inspected}"
                );
            } else {
                assert!(
                    inspected.lines().any(|line| line
                        .trim_start()
                        .strip_prefix("Entry point address:")
                        .is_some_and(|value| value.trim() == "0x1")),
                    "{name}: {inspected}"
                );
            }
            fs::write(&output, b"previous artifact").unwrap();
            match compile_object_with_assembler(
                source,
                "x86_64-linux-gnu",
                &output,
                assembler.as_os_str(),
            ) {
                Ok(()) => accepted.push(name),
                Err(error) => {
                    assert_eq!(error.code, "E_OUTPUT", "{name}: {error}");
                    assert!(error.detail.starts_with("unsupported assembler output:"));
                    assert!(error.detail.contains(expected_detail), "{name}: {error}");
                    assert_eq!(fs::read(&output).unwrap(), b"previous artifact", "{name}");
                }
            }
            assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 3, "{name}");
        }
        assert!(accepted.is_empty(), "unexpectedly installed: {accepted:?}");
    }

    #[test]
    fn misaligned_section_address_preserves_output_and_cleans_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-section-alignment-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let fixture = directory.0.join("fixture.o");
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        compile_object(source, "x86_64-linux-gnu", &fixture).unwrap();
        let mut bytes = fs::read(&fixture).unwrap();
        let section_offset = u64::from_le_bytes(bytes[40..48].try_into().unwrap()) as usize;
        let section_count = u16::from_le_bytes(bytes[60..62].try_into().unwrap()) as usize;
        let code_header = (0..section_count)
            .map(|index| section_offset + index * 64)
            .find(|offset| {
                let flags = u64::from_le_bytes(bytes[offset + 8..offset + 16].try_into().unwrap());
                flags & 6 == 6
            })
            .expect("GNU as output has an allocated executable section");
        bytes[code_header + 16..code_header + 24].copy_from_slice(&1u64.to_le_bytes());
        bytes[code_header + 48..code_header + 56].copy_from_slice(&8u64.to_le_bytes());
        fs::write(&fixture, &bytes).unwrap();

        let inspection = Command::new("readelf")
            .env("LC_ALL", "C")
            .args(["--wide", "--section-headers"])
            .arg(&fixture)
            .output()
            .unwrap();
        assert!(inspection.status.success(), "{inspection:?}");
        let inspected = String::from_utf8(inspection.stdout).unwrap();
        assert!(
            inspected.lines().any(|line| {
                line.contains(".text")
                    && line.contains("0000000000000001")
                    && line.trim_end().ends_with('8')
            }),
            "{inspected}"
        );

        let assembler = directory.0.join("substituting-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();

        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();
        assert_eq!(error.code, "E_OUTPUT", "{error}");
        assert!(error.detail.starts_with("invalid ELF64 object:"));
        assert!(
            error
                .detail
                .contains("section address violates its alignment")
        );
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 3);
    }

    #[test]
    fn comdat_section_group_preserves_output_and_cleans_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-section-group-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let assembly = directory.0.join("fixture.s");
        let fixture = directory.0.join("fixture.o");
        fs::write(
            &assembly,
            ".section .text.aic_main,\"axG\",@progbits,aic_main,comdat\n.globl aic_main\n.type aic_main,@function\naic_main:\n  movq %rdi,%rax\n  ret\n.size aic_main,.-aic_main\n.section .note.GNU-stack,\"\",@progbits\n",
        )
        .unwrap();
        let assembled = Command::new("as")
            .args(["--64", "-march=generic64", "-o"])
            .arg(&fixture)
            .arg(&assembly)
            .output()
            .unwrap();
        assert!(assembled.status.success(), "{assembled:?}");
        let inspection = Command::new("readelf")
            .env("LC_ALL", "C")
            .args([
                "--wide",
                "--section-headers",
                "--section-groups",
                "--symbols",
            ])
            .arg(&fixture)
            .output()
            .unwrap();
        assert!(inspection.status.success(), "{inspection:?}");
        let inspected = String::from_utf8(inspection.stdout).unwrap();
        assert!(inspected.contains("COMDAT group"), "{inspected}");
        assert!(inspected.contains(".text.aic_main"), "{inspected}");
        assert!(inspected.contains("aic_main"), "{inspected}");
        fs::remove_file(&assembly).unwrap();

        let assembler = directory.0.join("substituting-as");
        fs::write(
            &assembler,
            b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let output = directory.0.join("preserved.o");
        fs::write(&output, b"previous artifact").unwrap();

        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        let error = compile_object_with_assembler(
            source,
            "x86_64-linux-gnu",
            &output,
            assembler.as_os_str(),
        )
        .unwrap_err();
        assert_eq!(error.code, "E_OUTPUT", "{error}");
        assert!(error.detail.starts_with("unsupported assembler output:"));
        assert!(error.detail.contains("section groups are unsupported"));
        assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 3);
    }

    #[test]
    fn object_byte_budget_preserves_destination_and_cleans_staging() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ASSEMBLER_FIXTURE.lock().unwrap();
        let directory = TemporaryDirectory(std::env::temp_dir().join(format!(
            "aic-object-budget-{}-{}",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&directory.0).unwrap();
        let input = directory.0.join("fixture.s");
        let fixture = directory.0.join("fixture.o");
        fs::write(&input, b".text\n.globl aic_main\n.type aic_main,@function\naic_main:\nmovq %rdi,%rax\nret\n.size aic_main,.-aic_main\n.section .note.GNU-stack,\"\",@progbits\n").unwrap();
        let assembled = Command::new("as")
            .args(["--64", "-march=generic64", "-o"])
            .arg(&fixture)
            .arg(&input)
            .output()
            .unwrap();
        assert!(assembled.status.success(), "{assembled:?}");
        let assembler = directory.0.join("substituting-as");
        fs::write(&assembler, b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\nprintf 'GNU assembler fixture\\n'\nexit 0\nfi\nif [ \"$1\" != \"--64\" ] || [ \"$2\" != \"-march=generic64\" ] || [ \"$3\" != \"-o\" ]; then\nexit 16\nfi\ncp \"${0%/*}/fixture.o\" \"$4\"\n").unwrap();
        let mut permissions = fs::metadata(&assembler).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&assembler, permissions).unwrap();
        let output = directory.0.join("preserved.o");
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n";
        for size in [MAX_OBJECT_BYTES, MAX_OBJECT_BYTES + 1] {
            fs::OpenOptions::new()
                .write(true)
                .open(&fixture)
                .unwrap()
                .set_len(size as u64)
                .unwrap();
            fs::write(&output, b"previous artifact").unwrap();
            let result = compile_object_with_assembler(
                source,
                "x86_64-linux-gnu",
                &output,
                assembler.as_os_str(),
            );
            if size == MAX_OBJECT_BYTES {
                result.unwrap();
                assert_eq!(fs::metadata(&output).unwrap().len(), size as u64);
            } else {
                assert_eq!(result.err().map(|error| error.code), Some("E_LIMIT"));
                assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
            }
            assert_eq!(fs::read_dir(&directory.0).unwrap().count(), 4);
        }
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
