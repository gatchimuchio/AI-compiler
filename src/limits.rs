//! Current implementation budgets, not a beta resource or performance guarantee.

use crate::diagnostic::{Diagnostic, Result, Span};
use crate::ir::{InstructionKind, Module, TerminatorKind};

pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;
pub const MAX_OBJECT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_BLOCKS: usize = 256;
pub const MAX_INSTRUCTIONS: usize = 16_384;
pub const MAX_PHI_INCOMINGS: usize = 65_536;
pub const MAX_IDENTIFIER_BYTES: usize = 63;

pub(crate) fn check_budget(
    count: usize,
    maximum: usize,
    resource: &str,
    span: Option<Span>,
) -> Result<()> {
    if count <= maximum {
        return Ok(());
    }
    Err(limit_error(maximum, resource, span))
}

fn limit_error(maximum: usize, resource: &str, span: Option<Span>) -> Diagnostic {
    let detail = format!("implementation limit exceeded: {resource} (maximum {maximum})");
    match span {
        Some(span) => Diagnostic::at("E_LIMIT", span, detail),
        None => Diagnostic::global("E_LIMIT", detail),
    }
}

fn add_budget(
    total: &mut usize,
    count: usize,
    maximum: usize,
    resource: &str,
    span: Span,
) -> Result<()> {
    *total = total
        .checked_add(count)
        .filter(|total| *total <= maximum)
        .ok_or_else(|| limit_error(maximum, resource, Some(span)))?;
    Ok(())
}

pub(crate) fn check_module_budget(module: &Module) -> Result<()> {
    check_budget(module.blocks.len(), MAX_BLOCKS, "blocks", None)?;
    let mut instructions = 0;
    for block in &module.blocks {
        add_budget(
            &mut instructions,
            block.instructions.len(),
            MAX_INSTRUCTIONS,
            "instructions",
            block.span,
        )?;
    }
    let mut incomings = 0;
    for block in &module.blocks {
        for instruction in &block.instructions {
            if let InstructionKind::Phi { incomings: inputs } = &instruction.kind {
                add_budget(
                    &mut incomings,
                    inputs.len(),
                    MAX_PHI_INCOMINGS,
                    "Phi incoming records",
                    instruction.span,
                )?;
            }
        }
    }

    // Count first, then inspect strings before cloning names into validation maps.
    // Do not include an unbounded caller-provided name in the error detail.
    let identifier = |name: &str, span| {
        check_budget(
            name.len(),
            MAX_IDENTIFIER_BYTES,
            "identifier bytes",
            Some(span),
        )
    };
    identifier(&module.parameter, module.parameter_span)?;
    for block in &module.blocks {
        identifier(&block.id, block.span)?;
        for instruction in &block.instructions {
            identifier(&instruction.id, instruction.span)?;
            match &instruction.kind {
                InstructionKind::Const(_) => {}
                InstructionKind::Add { lhs, rhs }
                | InstructionKind::Sub { lhs, rhs }
                | InstructionKind::Eq { lhs, rhs } => {
                    identifier(lhs, instruction.span)?;
                    identifier(rhs, instruction.span)?;
                }
                InstructionKind::Phi { incomings } => {
                    for incoming in incomings {
                        identifier(&incoming.block, incoming.span)?;
                        identifier(&incoming.value, incoming.span)?;
                    }
                }
            }
        }
        match &block.terminator.kind {
            TerminatorKind::Branch { target } => identifier(target, block.terminator.span)?,
            TerminatorKind::BranchIf {
                condition,
                if_true,
                if_false,
            } => {
                identifier(condition, block.terminator.span)?;
                identifier(if_true, block.terminator.span)?;
                identifier(if_false, block.terminator.span)?;
            }
            TerminatorKind::Return { value } => identifier(value, block.terminator.span)?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{
        Block, Incoming, Instruction, InstructionKind, Module, Terminator, TerminatorKind,
    };
    use crate::{Span, parse, validate};

    fn block(id: String, kind: TerminatorKind) -> Block {
        Block {
            id,
            span: Span::new(1, 1),
            instructions: Vec::new(),
            terminator: Terminator {
                kind,
                span: Span::new(1, 1),
            },
        }
    }

    fn identity() -> Module {
        parse(b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n").unwrap()
    }

    fn instruction(id: String, kind: InstructionKind) -> Instruction {
        Instruction {
            id,
            kind,
            span: Span::new(1, 1),
        }
    }

    #[test]
    fn budget_arithmetic_checks_zero_upper_bound_and_overflow() {
        assert!(check_budget(0, 0, "test", None).is_ok());
        assert!(check_budget(usize::MAX, usize::MAX, "test", None).is_ok());
        let mut total = usize::MAX;
        assert_eq!(
            add_budget(&mut total, 1, usize::MAX, "test", Span::new(1, 1))
                .err()
                .map(|error| error.code),
            Some("E_LIMIT")
        );
        assert_eq!(total, usize::MAX);
    }

    #[test]
    fn source_byte_budget_includes_comments_and_precedes_encoding() {
        let mut source =
            b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n;".to_vec();
        source.resize(MAX_SOURCE_BYTES, b'x');
        assert!(parse(&source).is_ok());
        source.push(b'x');
        assert_eq!(
            parse(&source).err().map(|error| error.code),
            Some("E_LIMIT")
        );
        *source.last_mut().unwrap() = 0;
        assert_eq!(
            parse(&source).err().map(|error| error.code),
            Some("E_LIMIT")
        );
    }

    #[test]
    fn block_budget_precedes_dominator_allocation() {
        let mut module = identity();
        module.blocks.clear();
        for index in 0..MAX_BLOCKS {
            let id = if index == 0 {
                "entry".to_owned()
            } else {
                format!("b{index}")
            };
            let terminator = if index + 1 == MAX_BLOCKS {
                TerminatorKind::Return {
                    value: "x".to_owned(),
                }
            } else {
                TerminatorKind::Branch {
                    target: format!("b{}", index + 1),
                }
            };
            module.blocks.push(block(id, terminator));
        }
        assert!(validate(&module).is_ok());
        module.blocks.last_mut().unwrap().terminator.kind = TerminatorKind::Branch {
            target: "extra".to_owned(),
        };
        module.blocks.push(block(
            "extra".to_owned(),
            TerminatorKind::Return {
                value: "x".to_owned(),
            },
        ));
        assert_eq!(
            validate(&module).err().map(|error| error.code),
            Some("E_LIMIT")
        );
    }

    #[test]
    fn instruction_budget_covers_public_validation_evaluation_and_emission() {
        let mut module = identity();
        for index in 0..MAX_INSTRUCTIONS {
            module.blocks[0]
                .instructions
                .push(instruction(format!("v{index}"), InstructionKind::Const(0)));
        }
        let metadata = validate(&module).unwrap();
        module.blocks[0]
            .instructions
            .push(instruction("extra".to_owned(), InstructionKind::Const(0)));
        assert_eq!(
            validate(&module).err().map(|error| error.code),
            Some("E_LIMIT")
        );
        assert!(
            matches!(crate::evaluate(&module, 0, 0), Err(crate::EvalError::Invalid(error)) if error.code == "E_LIMIT")
        );
        assert!(
            matches!(crate::evaluate_many(&module, &[], 0), Err(crate::EvalError::Invalid(error)) if error.code == "E_LIMIT")
        );
        assert_eq!(
            crate::x86_64::emit_assembly(&module, &metadata)
                .err()
                .map(|error| error.code),
            Some("E_LIMIT")
        );
    }

    #[test]
    fn phi_budget_counts_all_incoming_records() {
        let mut module = identity();
        module.blocks.clear();
        for index in 0..15 {
            let id = if index == 0 {
                "entry".to_owned()
            } else {
                format!("b{index}")
            };
            let terminator = if index < 7 {
                TerminatorKind::BranchIf {
                    condition: "condition".to_owned(),
                    if_true: format!("b{}", index * 2 + 1),
                    if_false: format!("b{}", index * 2 + 2),
                }
            } else {
                TerminatorKind::Branch {
                    target: "join".to_owned(),
                }
            };
            module.blocks.push(block(id, terminator));
        }
        module.blocks[0].instructions.push(instruction(
            "condition".to_owned(),
            InstructionKind::Eq {
                lhs: "x".to_owned(),
                rhs: "x".to_owned(),
            },
        ));
        let mut join = block(
            "join".to_owned(),
            TerminatorKind::Return {
                value: "x".to_owned(),
            },
        );
        for index in 0..MAX_PHI_INCOMINGS / 8 {
            let incomings = (7..15)
                .map(|predecessor| Incoming {
                    block: format!("b{predecessor}"),
                    value: "x".to_owned(),
                    span: Span::new(1, 1),
                })
                .collect();
            join.instructions.push(instruction(
                format!("v{index}"),
                InstructionKind::Phi { incomings },
            ));
        }
        module.blocks.push(join);
        assert!(validate(&module).is_ok());
        module.blocks.last_mut().unwrap().terminator.kind = TerminatorKind::Branch {
            target: "exit".to_owned(),
        };
        let mut exit = block(
            "exit".to_owned(),
            TerminatorKind::Return {
                value: "x".to_owned(),
            },
        );
        exit.instructions.push(instruction(
            "extra".to_owned(),
            InstructionKind::Phi {
                incomings: vec![Incoming {
                    block: "join".to_owned(),
                    value: "x".to_owned(),
                    span: Span::new(1, 1),
                }],
            },
        ));
        module.blocks.push(exit);
        assert_eq!(
            validate(&module).err().map(|error| error.code),
            Some("E_LIMIT")
        );
    }

    #[test]
    fn public_ir_identifier_budget_covers_definitions_and_references() {
        let mut module = identity();
        module.parameter = "x".repeat(MAX_IDENTIFIER_BYTES);
        module.blocks[0].terminator.kind = TerminatorKind::Return {
            value: module.parameter.clone(),
        };
        assert!(validate(&module).is_ok());
        let long = "x".repeat(MAX_IDENTIFIER_BYTES + 1);
        let mut cases = Vec::new();
        let mut parameter = identity();
        parameter.parameter = long.clone();
        cases.push(parameter);
        let mut block_id = identity();
        block_id.blocks[0].id = long.clone();
        cases.push(block_id);
        for kind in [
            InstructionKind::Const(0),
            InstructionKind::Add {
                lhs: long.clone(),
                rhs: "x".to_owned(),
            },
            InstructionKind::Sub {
                lhs: "x".to_owned(),
                rhs: long.clone(),
            },
            InstructionKind::Eq {
                lhs: long.clone(),
                rhs: "x".to_owned(),
            },
            InstructionKind::Phi {
                incomings: vec![Incoming {
                    block: long.clone(),
                    value: "x".to_owned(),
                    span: Span::new(1, 1),
                }],
            },
            InstructionKind::Phi {
                incomings: vec![Incoming {
                    block: "entry".to_owned(),
                    value: long.clone(),
                    span: Span::new(1, 1),
                }],
            },
        ] {
            let mut module = identity();
            let id = if matches!(kind, InstructionKind::Const(_)) {
                long.clone()
            } else {
                "v".to_owned()
            };
            module.blocks[0].instructions.push(instruction(id, kind));
            cases.push(module);
        }
        for kind in [
            TerminatorKind::Return {
                value: long.clone(),
            },
            TerminatorKind::Branch {
                target: long.clone(),
            },
            TerminatorKind::BranchIf {
                condition: long.clone(),
                if_true: "a".to_owned(),
                if_false: "b".to_owned(),
            },
            TerminatorKind::BranchIf {
                condition: "c".to_owned(),
                if_true: long.clone(),
                if_false: "b".to_owned(),
            },
            TerminatorKind::BranchIf {
                condition: "c".to_owned(),
                if_true: "a".to_owned(),
                if_false: long.clone(),
            },
        ] {
            let mut module = identity();
            module.blocks[0].terminator.kind = kind;
            cases.push(module);
        }
        for module in cases {
            let Err(error) = validate(&module) else {
                panic!("over-budget IR must be rejected");
            };
            assert_eq!(error.code, "E_LIMIT");
            assert!(!error.detail.contains(&long));
        }
    }
}
