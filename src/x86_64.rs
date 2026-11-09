use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::diagnostic::{Diagnostic, Result};
use crate::ir::{InstructionKind, Module, TerminatorKind};
use crate::validate::ValidatedModule;

struct Edge {
    source: usize,
    target: usize,
    label: usize,
}

pub fn emit_assembly(module: &Module, validated: &ValidatedModule) -> Result<String> {
    let mut slots = BTreeMap::new();
    let mut next_slot = 1usize;
    slots.insert(module.parameter.clone(), next_slot);
    next_slot += 1;
    for block in &module.blocks {
        for instruction in &block.instructions {
            slots.insert(instruction.id.clone(), next_slot);
            next_slot += 1;
        }
    }
    let scratch_count = module
        .blocks
        .iter()
        .map(|block| {
            block
                .instructions
                .iter()
                .take_while(|instruction| instruction.is_phi())
                .count()
        })
        .max()
        .unwrap_or(0);
    let slot_count = next_slot
        .checked_sub(1)
        .and_then(|count| count.checked_add(scratch_count))
        .ok_or_else(|| Diagnostic::global("E_OUTPUT", "stack frame size overflow"))?;
    let raw_frame = slot_count
        .checked_mul(8)
        .ok_or_else(|| Diagnostic::global("E_OUTPUT", "stack frame size overflow"))?;
    let frame_size = raw_frame
        .checked_add(15)
        .map(|size| size & !15)
        .ok_or_else(|| Diagnostic::global("E_OUTPUT", "stack frame size overflow"))?;
    if frame_size > i32::MAX as usize {
        return Err(Diagnostic::global(
            "E_OUTPUT",
            "stack frame exceeds x86-64 signed displacement range",
        ));
    }

    let mut edges = Vec::new();
    let mut edge_labels = BTreeMap::new();
    for (source, block) in module.blocks.iter().enumerate() {
        let targets: Vec<&str> = match &block.terminator.kind {
            TerminatorKind::Branch { target } => vec![target],
            TerminatorKind::BranchIf {
                if_true, if_false, ..
            } => vec![if_true, if_false],
            TerminatorKind::Return { .. } => Vec::new(),
        };
        for target in targets {
            let target_index = validated.block_index[target];
            let label = edges.len();
            edges.push(Edge {
                source,
                target: target_index,
                label,
            });
            edge_labels.insert((source, target_index), label);
        }
    }

    let mut assembly = String::new();
    writeln!(assembly, ".text").unwrap();
    writeln!(assembly, ".globl aic_main").unwrap();
    writeln!(assembly, ".type aic_main, @function").unwrap();
    writeln!(assembly, "aic_main:").unwrap();
    writeln!(assembly, "  pushq %rbp").unwrap();
    writeln!(assembly, "  movq %rsp, %rbp").unwrap();
    if frame_size > 0 {
        writeln!(assembly, concat!("  subq $", "{}", ", %rsp"), frame_size).unwrap();
    }
    emit_store(
        &mut assembly,
        "%rdi",
        slot_offset(&slots, &module.parameter)?,
    )?;
    writeln!(assembly, "  jmp .Laic_block_0").unwrap();

    for (block_number, block) in module.blocks.iter().enumerate() {
        writeln!(assembly, ".Laic_block_{block_number}:").unwrap();
        for instruction in block
            .instructions
            .iter()
            .filter(|instruction| !instruction.is_phi())
        {
            match &instruction.kind {
                InstructionKind::Const(value) => {
                    let offset = slot_offset(&slots, &instruction.id)?;
                    writeln!(
                        assembly,
                        concat!("  movabsq $", "{}", ", %rax"),
                        *value as i64
                    )
                    .unwrap();
                    emit_store(&mut assembly, "%rax", offset)?;
                }
                InstructionKind::Add { lhs, rhs } => {
                    emit_load(&mut assembly, slot_offset(&slots, lhs)?, "%rax")?;
                    writeln!(assembly, "  addq {}(%rbp), %rax", slot_offset(&slots, rhs)?).unwrap();
                    emit_store(&mut assembly, "%rax", slot_offset(&slots, &instruction.id)?)?;
                }
                InstructionKind::Sub { lhs, rhs } => {
                    emit_load(&mut assembly, slot_offset(&slots, lhs)?, "%rax")?;
                    writeln!(assembly, "  subq {}(%rbp), %rax", slot_offset(&slots, rhs)?).unwrap();
                    emit_store(&mut assembly, "%rax", slot_offset(&slots, &instruction.id)?)?;
                }
                InstructionKind::Eq { lhs, rhs } => {
                    emit_load(&mut assembly, slot_offset(&slots, lhs)?, "%rax")?;
                    writeln!(assembly, "  cmpq {}(%rbp), %rax", slot_offset(&slots, rhs)?).unwrap();
                    writeln!(assembly, "  sete %al").unwrap();
                    writeln!(assembly, "  movzbq %al, %rax").unwrap();
                    emit_store(&mut assembly, "%rax", slot_offset(&slots, &instruction.id)?)?;
                }
                InstructionKind::Phi { .. } => unreachable!(),
            }
        }
        match &block.terminator.kind {
            TerminatorKind::Branch { target } => {
                let target = validated.block_index[target];
                writeln!(
                    assembly,
                    "  jmp .Laic_edge_{}",
                    edge_labels[&(block_number, target)]
                )
                .unwrap();
            }
            TerminatorKind::BranchIf {
                condition,
                if_true,
                if_false,
            } => {
                writeln!(
                    assembly,
                    "  cmpq $0, {}(%rbp)",
                    slot_offset(&slots, condition)?
                )
                .unwrap();
                let true_target = validated.block_index[if_true];
                let false_target = validated.block_index[if_false];
                writeln!(
                    assembly,
                    "  jne .Laic_edge_{}",
                    edge_labels[&(block_number, true_target)]
                )
                .unwrap();
                writeln!(
                    assembly,
                    "  jmp .Laic_edge_{}",
                    edge_labels[&(block_number, false_target)]
                )
                .unwrap();
            }
            TerminatorKind::Return { value } => {
                emit_load(&mut assembly, slot_offset(&slots, value)?, "%rax")?;
                writeln!(assembly, "  leave").unwrap();
                writeln!(assembly, "  ret").unwrap();
            }
        }
    }

    for edge in &edges {
        writeln!(assembly, ".Laic_edge_{}:", edge.label).unwrap();
        let target = &module.blocks[edge.target];
        let phis: Vec<_> = target
            .instructions
            .iter()
            .take_while(|instruction| instruction.is_phi())
            .collect();
        for (index, instruction) in phis.iter().enumerate() {
            let InstructionKind::Phi { incomings } = &instruction.kind else {
                unreachable!();
            };
            let incoming = incomings
                .iter()
                .find(|incoming| incoming.block == module.blocks[edge.source].id)
                .ok_or_else(|| {
                    Diagnostic::at(
                        "E_PHI",
                        instruction.span,
                        "missing phi operand for incoming edge",
                    )
                })?;
            let source_offset = slot_offset(&slots, &incoming.value)?;
            let scratch_offset = scratch_offset(scratch_count, slots.len(), index)?;
            emit_load(&mut assembly, source_offset, "%rax")?;
            emit_store(&mut assembly, "%rax", scratch_offset)?;
        }
        for (index, instruction) in phis.iter().enumerate() {
            let destination_offset = slot_offset(&slots, &instruction.id)?;
            let scratch_offset = scratch_offset(scratch_count, slots.len(), index)?;
            emit_load(&mut assembly, scratch_offset, "%rax")?;
            emit_store(&mut assembly, "%rax", destination_offset)?;
        }
        writeln!(assembly, "  jmp .Laic_block_{}", edge.target).unwrap();
    }
    writeln!(assembly, ".size aic_main, .-aic_main").unwrap();
    writeln!(assembly, ".section .note.GNU-stack,\"\",@progbits").unwrap();
    Ok(assembly)
}

fn slot_offset(slots: &BTreeMap<String, usize>, name: &str) -> Result<i64> {
    let slot = slots.get(name).ok_or_else(|| {
        Diagnostic::global("E_OUTPUT", format!("missing stack slot for %{}", name))
    })?;
    Ok(-((*slot as i64) * 8))
}

fn scratch_offset(scratch_count: usize, value_slots: usize, index: usize) -> Result<i64> {
    if index >= scratch_count {
        return Err(Diagnostic::global(
            "E_OUTPUT",
            "phi scratch slot index overflow",
        ));
    }
    let slot = value_slots
        .checked_add(index + 1)
        .ok_or_else(|| Diagnostic::global("E_OUTPUT", "phi scratch slot overflow"))?;
    Ok(-((slot as i64) * 8))
}

fn emit_load(assembly: &mut String, offset: i64, register: &str) -> Result<()> {
    writeln!(assembly, "  movq {offset}(%rbp), {register}").unwrap();
    Ok(())
}

fn emit_store(assembly: &mut String, register: &str, offset: i64) -> Result<()> {
    writeln!(assembly, "  movq {register}, {offset}(%rbp)").unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::validate::validate;

    #[test]
    fn emits_edge_staged_phi_copies() {
        let source = "AIC 0.2\nfn aic_main(%n: u64) -> u64 {\n^entry:\n %z = const.u64 0\n %one = const.u64 1\n %max = const.u64 18446744073709551615\n br ^head\n^head:\n %i = phi.u64 [^entry, %z] [^step, %next]\n %a = phi.u64 [^entry, %z] [^step, %b]\n %b = phi.u64 [^entry, %one] [^step, %a]\n %done = eq.u64 %i, %n\n br.if %done, ^exit, ^step\n^step:\n %next = add.wrap.u64 %i, %one\n br ^head\n^exit:\n ret.u64 %a\n}\n";
        let module = parse(source.as_bytes()).unwrap();
        let validated = validate(&module).unwrap();
        let assembly = emit_assembly(&module, &validated).unwrap();
        let edge_stub = assembly.split(".Laic_edge_3:").nth(1).unwrap();
        let final_stage = edge_stub.find("movq -48(%rbp), %rax").unwrap();
        let first_commit = edge_stub.find("movq -80(%rbp), %rax").unwrap();
        assert!(final_stage < first_commit);
        assert!(assembly.contains("movabsq $-1, %rax"));
    }
}
