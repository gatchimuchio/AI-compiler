use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use ai_compiler::Diagnostic;
use ai_compiler::driver::compile_object;

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let mut args = env::args_os().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return 2;
    };
    let command = command.to_string_lossy();
    if command == "--help" || command == "-h" {
        print_usage();
        return 0;
    }
    if command == "--version" {
        println!("aic {}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    if command != "compile" {
        eprintln!(
            "{}",
            Diagnostic::global("E_USAGE", "expected compile command")
        );
        print_usage();
        return 2;
    }

    let mut target: Option<String> = None;
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut options_enabled = true;
    while let Some(argument) = args.next() {
        if options_enabled && argument == "--" {
            options_enabled = false;
        } else if options_enabled && argument == "--target" {
            if target.is_some() {
                eprintln!(
                    "{}",
                    Diagnostic::global("E_USAGE", "--target may be specified only once")
                );
                return 2;
            }
            let Some(value) = args.next() else {
                eprintln!(
                    "{}",
                    Diagnostic::global("E_USAGE", "--target requires a value")
                );
                return 2;
            };
            target = Some(value.to_string_lossy().into_owned());
        } else if options_enabled && argument == "-o" {
            if output.is_some() {
                eprintln!(
                    "{}",
                    Diagnostic::global("E_USAGE", "-o may be specified only once")
                );
                return 2;
            }
            let Some(value) = args.next() else {
                eprintln!("{}", Diagnostic::global("E_USAGE", "-o requires a path"));
                return 2;
            };
            output = Some(PathBuf::from(value));
        } else if input.is_none() {
            if options_enabled
                && argument
                    .to_str()
                    .is_some_and(|argument| argument.starts_with('-'))
            {
                eprintln!(
                    "{}",
                    Diagnostic::global(
                        "E_USAGE",
                        "unknown option; use -- before an input path beginning with '-'"
                    )
                );
                return 2;
            }
            input = Some(PathBuf::from(argument));
        } else {
            eprintln!(
                "{}",
                Diagnostic::global("E_USAGE", "unexpected extra argument")
            );
            return 2;
        }
    }
    let (Some(target), Some(input_path), Some(output_path)) = (target, input, output) else {
        eprintln!(
            "{}",
            Diagnostic::global("E_USAGE", "target, input, and -o are required")
        );
        print_usage();
        return 2;
    };
    if same_path(&input_path, &output_path) {
        eprintln!(
            "{}",
            Diagnostic::global("E_OUTPUT", "input and output paths must differ")
        );
        return 2;
    }
    let source = match fs::read(&input_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!(
                "{}",
                Diagnostic::global("E_INPUT", format!("could not read source: {error}"))
            );
            return 2;
        }
    };
    match compile_object(&source, &target, &output_path) {
        Ok(()) => {
            println!("wrote ELF64 ET_REL object for {target}");
            0
        }
        Err(error) => {
            eprintln!("{error}");
            2
        }
    }
}

fn same_path(input: &Path, output: &Path) -> bool {
    if let (Ok(input), Ok(output)) = (fs::canonicalize(input), fs::canonicalize(output)) {
        return input == output;
    }
    let input = fs::canonicalize(input).unwrap_or_else(|_| absolute_path(input));
    let output = absolute_path(output);
    input == output
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        env::current_dir()
            .map(|current| current.join(path))
            .unwrap_or_else(|_| path.to_owned())
    }
}

fn print_usage() {
    println!("Usage: aic compile --target x86_64-linux-gnu INPUT -o OUTPUT");
    println!("       aic --version");
}
