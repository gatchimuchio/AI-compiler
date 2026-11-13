use std::env;
use std::fs;
use std::process;

use ai_compiler::{InstructionKind, TerminatorKind, parse, validate};

fn main() {
    let Some(path) = env::args_os().nth(1) else {
        eprintln!("usage: g3-parser-adapter SOURCE.aic");
        process::exit(2);
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("could not read normalized AIC source: {error}");
            process::exit(2);
        }
    };
    let module = match parse(&bytes) {
        Ok(module) => module,
        Err(error) => {
            println!("PARSE_ERROR");
            eprintln!("{error}");
            process::exit(3);
        }
    };
    println!("PARSE_OK");
    if let Err(error) = validate(&module) {
        println!("VALIDATION_ERROR");
        eprintln!("{error}");
        process::exit(4);
    }
    println!("VALIDATION_OK");

    if bytes.contains(&b';')
        || module.parameter != "x"
        || module.blocks.len() != 1
        || module.blocks[0].id != "entry"
        || module.blocks[0].instructions.iter().any(|instruction| {
            !matches!(
                &instruction.kind,
                InstructionKind::Const(_) | InstructionKind::Add { .. } | InstructionKind::Sub { .. }
            )
        })
        || !matches!(&module.blocks[0].terminator.kind, TerminatorKind::Return { .. })
    {
        println!("PROFILE_ERROR");
        eprintln!("source is outside the fixed single-block pilot profile");
        process::exit(5);
    }
    println!("PROFILE_OK");
    println!("PARAM\t{}", module.parameter);
    for instruction in &module.blocks[0].instructions {
        match &instruction.kind {
            InstructionKind::Const(value) => println!("CONST\t{}\t{}", instruction.id, value),
            InstructionKind::Add { lhs, rhs } => {
                println!("ADD\t{}\t{}\t{}", instruction.id, lhs, rhs)
            }
            InstructionKind::Sub { lhs, rhs } => {
                println!("SUB\t{}\t{}\t{}", instruction.id, lhs, rhs)
            }
            _ => unreachable!("profile was checked above"),
        }
    }
    let TerminatorKind::Return { value } = &module.blocks[0].terminator.kind else {
        unreachable!("profile was checked above")
    };
    println!("RET\t{value}");
}
