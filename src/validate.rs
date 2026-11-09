use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{InstructionKind, Module, TerminatorKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueType {
    U64,
    Bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Definition {
    block: usize,
    ordinal: usize,
    ty: ValueType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedModule {
    pub block_index: BTreeMap<String, usize>,
    pub predecessors: Vec<Vec<usize>>,
    pub value_types: BTreeMap<String, ValueType>,
}

pub fn validate(module: &Module) -> Result<ValidatedModule> {
    if module.blocks.is_empty() {
        return Err(Diagnostic::global(
            "E_CFG",
            "function must contain an entry block",
        ));
    }

    let mut block_index = BTreeMap::new();
    for (index, block) in module.blocks.iter().enumerate() {
        if block_index.insert(block.id.clone(), index).is_some() {
            return Err(Diagnostic::at(
                "E_DUPLICATE_ID",
                block.span,
                format!("duplicate block ^{}", block.id),
            ));
        }
    }
    if module.blocks[0].id != "entry" {
        return Err(Diagnostic::at(
            "E_CFG",
            module.blocks[0].span,
            "the first block must be ^entry",
        ));
    }

    let mut definitions = BTreeMap::new();
    let mut value_types = BTreeMap::new();
    definitions.insert(
        module.parameter.clone(),
        Definition {
            block: 0,
            ordinal: 0,
            ty: ValueType::U64,
        },
    );
    value_types.insert(module.parameter.clone(), ValueType::U64);
    for (block_number, block) in module.blocks.iter().enumerate() {
        let mut saw_non_phi = false;
        for (instruction_number, instruction) in block.instructions.iter().enumerate() {
            if instruction.is_phi() {
                if saw_non_phi {
                    return Err(Diagnostic::at(
                        "E_PHI",
                        instruction.span,
                        "phi instructions must form a prefix of the block",
                    ));
                }
            } else {
                saw_non_phi = true;
            }
            let ty = match instruction.kind {
                InstructionKind::Eq { .. } => ValueType::Bool,
                _ => ValueType::U64,
            };
            if definitions
                .insert(
                    instruction.id.clone(),
                    Definition {
                        block: block_number,
                        ordinal: instruction_number + 1,
                        ty,
                    },
                )
                .is_some()
            {
                return Err(Diagnostic::at(
                    "E_DUPLICATE_ID",
                    instruction.span,
                    format!("duplicate value %{}", instruction.id),
                ));
            }
            value_types.insert(instruction.id.clone(), ty);
        }
    }

    check_value_types(module, &definitions)?;

    let mut predecessors = vec![Vec::new(); module.blocks.len()];
    let mut successors = vec![Vec::new(); module.blocks.len()];
    for (source, block) in module.blocks.iter().enumerate() {
        let destinations: Vec<&str> = match &block.terminator.kind {
            TerminatorKind::Branch { target } => vec![target],
            TerminatorKind::BranchIf {
                if_true, if_false, ..
            } => {
                if if_true == if_false {
                    return Err(Diagnostic::at(
                        "E_CFG",
                        block.terminator.span,
                        "conditional branch destinations must be distinct",
                    ));
                }
                vec![if_true, if_false]
            }
            TerminatorKind::Return { .. } => Vec::new(),
        };
        for target in destinations {
            let target_index = *block_index.get(target).ok_or_else(|| {
                Diagnostic::at(
                    "E_UNDEFINED_BLOCK",
                    block.terminator.span,
                    format!("unknown branch target ^{target}"),
                )
            })?;
            successors[source].push(target_index);
            predecessors[target_index].push(source);
        }
    }
    if !predecessors[0].is_empty() {
        return Err(Diagnostic::at(
            "E_CFG",
            module.blocks[0].span,
            "^entry must not have incoming edges",
        ));
    }

    check_phi_edges(module, &block_index, &predecessors)?;
    check_reachability(module, &successors)?;
    let dominators = compute_dominators(&predecessors);
    check_dominance(module, &definitions, &block_index, &dominators)?;

    Ok(ValidatedModule {
        block_index,
        predecessors,
        value_types,
    })
}

fn check_value_types(module: &Module, definitions: &BTreeMap<String, Definition>) -> Result<()> {
    for block in &module.blocks {
        for instruction in &block.instructions {
            match &instruction.kind {
                InstructionKind::Const(_) => {}
                InstructionKind::Add { lhs, rhs }
                | InstructionKind::Sub { lhs, rhs }
                | InstructionKind::Eq { lhs, rhs } => {
                    require_type(lhs, ValueType::U64, instruction.span, definitions)?;
                    require_type(rhs, ValueType::U64, instruction.span, definitions)?;
                }
                InstructionKind::Phi { incomings } => {
                    for incoming in incomings {
                        require_type(
                            incoming.value.as_str(),
                            ValueType::U64,
                            incoming.span,
                            definitions,
                        )?;
                    }
                }
            }
        }
        match &block.terminator.kind {
            TerminatorKind::Branch { .. } => {}
            TerminatorKind::BranchIf { condition, .. } => {
                require_type(
                    condition,
                    ValueType::Bool,
                    block.terminator.span,
                    definitions,
                )?;
            }
            TerminatorKind::Return { value } => {
                require_type(value, ValueType::U64, block.terminator.span, definitions)?;
            }
        }
    }
    Ok(())
}

fn require_type(
    name: &str,
    expected: ValueType,
    span: crate::diagnostic::Span,
    definitions: &BTreeMap<String, Definition>,
) -> Result<()> {
    let definition = definitions.get(name).ok_or_else(|| {
        Diagnostic::at(
            "E_UNDEFINED_VALUE",
            span,
            format!("value %{} is not defined", name),
        )
    })?;
    if definition.ty != expected {
        return Err(Diagnostic::at(
            "E_TYPE",
            span,
            format!(
                "value %{} has type {:?}, expected {:?}",
                name, definition.ty, expected
            ),
        ));
    }
    Ok(())
}

fn check_phi_edges(
    module: &Module,
    block_index: &BTreeMap<String, usize>,
    predecessors: &[Vec<usize>],
) -> Result<()> {
    for (target_index, block) in module.blocks.iter().enumerate() {
        let expected: BTreeSet<usize> = predecessors[target_index].iter().copied().collect();
        for instruction in &block.instructions {
            let InstructionKind::Phi { incomings } = &instruction.kind else {
                continue;
            };
            let mut actual = BTreeSet::new();
            for incoming in incomings {
                let predecessor = *block_index.get(&incoming.block).ok_or_else(|| {
                    Diagnostic::at(
                        "E_UNDEFINED_BLOCK",
                        incoming.span,
                        format!("unknown phi predecessor ^{}", incoming.block),
                    )
                })?;
                if !actual.insert(predecessor) {
                    return Err(Diagnostic::at(
                        "E_PHI",
                        incoming.span,
                        format!("duplicate phi predecessor ^{}", incoming.block),
                    ));
                }
                if !expected.contains(&predecessor) {
                    return Err(Diagnostic::at(
                        "E_PHI",
                        incoming.span,
                        format!("^{} is not a predecessor of ^{}", incoming.block, block.id),
                    ));
                }
            }
            if actual != expected {
                return Err(Diagnostic::at(
                    "E_PHI",
                    instruction.span,
                    format!(
                        "phi in ^{} must name every predecessor exactly once",
                        block.id
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn check_reachability(module: &Module, successors: &[Vec<usize>]) -> Result<()> {
    let mut reached = vec![false; module.blocks.len()];
    let mut queue = VecDeque::from([0]);
    reached[0] = true;
    while let Some(block) = queue.pop_front() {
        for successor in &successors[block] {
            if !reached[*successor] {
                reached[*successor] = true;
                queue.push_back(*successor);
            }
        }
    }
    for (index, block) in module.blocks.iter().enumerate() {
        if !reached[index] {
            return Err(Diagnostic::at(
                "E_UNREACHABLE",
                block.span,
                format!("block ^{} is unreachable from ^entry", block.id),
            ));
        }
    }
    Ok(())
}

fn compute_dominators(predecessors: &[Vec<usize>]) -> Vec<BTreeSet<usize>> {
    let count = predecessors.len();
    let all: BTreeSet<usize> = (0..count).collect();
    let mut dominators = vec![all.clone(); count];
    dominators[0] = BTreeSet::from([0]);
    loop {
        let mut changed = false;
        for block in 1..count {
            let mut next = all.clone();
            for predecessor in &predecessors[block] {
                next = next
                    .intersection(&dominators[*predecessor])
                    .copied()
                    .collect();
            }
            next.insert(block);
            if next != dominators[block] {
                dominators[block] = next;
                changed = true;
            }
        }
        if !changed {
            return dominators;
        }
    }
}

fn check_dominance(
    module: &Module,
    definitions: &BTreeMap<String, Definition>,
    block_index: &BTreeMap<String, usize>,
    dominators: &[BTreeSet<usize>],
) -> Result<()> {
    for (block_number, block) in module.blocks.iter().enumerate() {
        for (instruction_number, instruction) in block.instructions.iter().enumerate() {
            let use_ordinal = instruction_number + 1;
            match &instruction.kind {
                InstructionKind::Const(_) => {}
                InstructionKind::Add { lhs, rhs }
                | InstructionKind::Sub { lhs, rhs }
                | InstructionKind::Eq { lhs, rhs } => {
                    check_dominates(
                        lhs,
                        block_number,
                        use_ordinal,
                        instruction.span,
                        definitions,
                        dominators,
                    )?;
                    check_dominates(
                        rhs,
                        block_number,
                        use_ordinal,
                        instruction.span,
                        definitions,
                        dominators,
                    )?;
                }
                InstructionKind::Phi { incomings } => {
                    for incoming in incomings {
                        let predecessor = block_index[&incoming.block];
                        let end_ordinal = module.blocks[predecessor].instructions.len() + 1;
                        check_dominates(
                            &incoming.value,
                            predecessor,
                            end_ordinal,
                            incoming.span,
                            definitions,
                            dominators,
                        )?;
                    }
                }
            }
        }
        let terminator_ordinal = block.instructions.len() + 1;
        let value = match &block.terminator.kind {
            TerminatorKind::Branch { .. } => None,
            TerminatorKind::BranchIf { condition, .. } => Some(condition),
            TerminatorKind::Return { value } => Some(value),
        };
        if let Some(value) = value {
            check_dominates(
                value,
                block_number,
                terminator_ordinal,
                block.terminator.span,
                definitions,
                dominators,
            )?;
        }
    }
    Ok(())
}

fn check_dominates(
    name: &str,
    use_block: usize,
    use_ordinal: usize,
    span: crate::diagnostic::Span,
    definitions: &BTreeMap<String, Definition>,
    dominators: &[BTreeSet<usize>],
) -> Result<()> {
    let definition = &definitions[name];
    let dominates = if definition.block == use_block {
        definition.ordinal < use_ordinal
    } else {
        dominators[use_block].contains(&definition.block)
    };
    if !dominates {
        return Err(Diagnostic::at(
            "E_DOMINANCE",
            span,
            format!("definition of %{} does not dominate this use", name),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn validate_source(source: &str) -> Result<ValidatedModule> {
        let module = parse(source.as_bytes())?;
        validate(&module)
    }

    #[test]
    fn accepts_loop_phi_swap_and_rejects_duplicate_edge() {
        let source = "AIC 0.2\nfn aic_main(%n: u64) -> u64 {\n^entry:\n %z = const.u64 0\n %one = const.u64 1\n br ^head\n^head:\n %i = phi.u64 [^entry, %z] [^step, %next]\n %a = phi.u64 [^entry, %z] [^step, %b]\n %b = phi.u64 [^entry, %one] [^step, %a]\n %done = eq.u64 %i, %n\n br.if %done, ^exit, ^step\n^step:\n %next = add.wrap.u64 %i, %one\n br ^head\n^exit:\n ret.u64 %a\n}\n";
        assert!(validate_source(source).is_ok());

        let duplicate = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %b = eq.u64 %x, %x\n br.if %b, ^join, ^join\n^join:\n %v = phi.u64 [^entry, %x]\n ret.u64 %v\n}\n";
        assert_eq!(validate_source(duplicate).unwrap_err().code, "E_CFG");
    }

    #[test]
    fn rejects_unreachable_blocks_and_bad_phi_coverage() {
        let unreachable = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n^ghost:\n ret.u64 %x\n}\n";
        assert_eq!(
            validate_source(unreachable).unwrap_err().code,
            "E_UNREACHABLE"
        );

        let bad_phi = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n br ^join\n^join:\n %v = phi.u64 [^join, %x]\n ret.u64 %v\n}\n";
        assert_eq!(validate_source(bad_phi).unwrap_err().code, "E_PHI");
    }

    #[test]
    fn rejects_type_and_dominance_errors() {
        let bad_type = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %b = eq.u64 %x, %x\n ret.u64 %b\n}\n";
        assert_eq!(validate_source(bad_type).unwrap_err().code, "E_TYPE");

        let bad_dominance = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %b = eq.u64 %x, %x\n br.if %b, ^left, ^right\n^left:\n %v = const.u64 1\n br ^join\n^right:\n br ^join\n^join:\n ret.u64 %v\n}\n";
        assert_eq!(
            validate_source(bad_dominance).unwrap_err().code,
            "E_DOMINANCE"
        );
    }

    #[test]
    fn reports_duplicate_and_undefined_names() {
        let duplicate_value = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %v = const.u64 1\n %v = const.u64 2\n ret.u64 %v\n}\n";
        assert_eq!(
            validate_source(duplicate_value).unwrap_err().code,
            "E_DUPLICATE_ID"
        );

        let duplicate_block = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n br ^next\n^next:\n br ^entry\n^next:\n ret.u64 %x\n}\n";
        assert_eq!(
            validate_source(duplicate_block).unwrap_err().code,
            "E_DUPLICATE_ID"
        );

        let missing_value =
            "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %missing\n}\n";
        assert_eq!(
            validate_source(missing_value).unwrap_err().code,
            "E_UNDEFINED_VALUE"
        );

        let missing_block = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n br ^missing\n}\n";
        assert_eq!(
            validate_source(missing_block).unwrap_err().code,
            "E_UNDEFINED_BLOCK"
        );
    }

    #[test]
    fn rejects_phi_placement_duplicate_inputs_and_bad_edge_uses() {
        let phi_after_operation = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %one = const.u64 1\n br ^join\n^join:\n %two = add.wrap.u64 %x, %one\n %v = phi.u64 [^entry, %x]\n ret.u64 %two\n}\n";
        assert_eq!(
            validate_source(phi_after_operation).unwrap_err().code,
            "E_PHI"
        );

        let duplicate_input = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n br ^join\n^join:\n %v = phi.u64 [^entry, %x] [^entry, %x]\n ret.u64 %v\n}\n";
        assert_eq!(validate_source(duplicate_input).unwrap_err().code, "E_PHI");

        let non_dominating_phi_use = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %b = eq.u64 %x, %x\n br.if %b, ^left, ^right\n^left:\n %v = const.u64 1\n br ^join\n^right:\n br ^join\n^join:\n %r = phi.u64 [^left, %v] [^right, %v]\n ret.u64 %r\n}\n";
        assert_eq!(
            validate_source(non_dominating_phi_use).unwrap_err().code,
            "E_DOMINANCE"
        );
    }
}
