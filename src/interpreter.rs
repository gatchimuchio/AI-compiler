use std::collections::BTreeMap;
use std::fmt;

use crate::diagnostic::Diagnostic;
use crate::ir::{InstructionKind, Module, TerminatorKind};
use crate::validate::{ValidatedModule, validate};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvalError {
    Invalid(Diagnostic),
    FuelExhausted,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(error) => error.fmt(f),
            Self::FuelExhausted => f.write_str("evaluation fuel exhausted"),
        }
    }
}

impl std::error::Error for EvalError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Value {
    U64(u64),
    Bool(bool),
}

pub fn evaluate(module: &Module, input: u64, fuel: u64) -> std::result::Result<u64, EvalError> {
    let validated = validate(module).map_err(EvalError::Invalid)?;
    evaluate_checked(module, &validated, input, fuel)
}

pub fn evaluate_many(
    module: &Module,
    inputs: &[u64],
    fuel: u64,
) -> std::result::Result<Vec<u64>, EvalError> {
    let validated = validate(module).map_err(EvalError::Invalid)?;
    inputs
        .iter()
        .map(|input| evaluate_checked(module, &validated, *input, fuel))
        .collect()
}

fn evaluate_checked(
    module: &Module,
    validated: &ValidatedModule,
    input: u64,
    fuel: u64,
) -> std::result::Result<u64, EvalError> {
    let mut values = BTreeMap::new();
    values.insert(module.parameter.clone(), Value::U64(input));
    let mut current = 0usize;
    let mut previous: Option<usize> = None;
    let mut remaining = fuel;

    loop {
        let block = &module.blocks[current];
        let phi_count = block
            .instructions
            .iter()
            .take_while(|instruction| instruction.is_phi())
            .count();
        if phi_count > 0 {
            let predecessor = previous.ok_or_else(|| {
                EvalError::Invalid(Diagnostic::at(
                    "E_PHI",
                    block.span,
                    "entry block cannot evaluate phi inputs",
                ))
            })?;
            let predecessor_id = &module.blocks[predecessor].id;
            let mut staged = Vec::with_capacity(phi_count);
            for instruction in block.instructions.iter().take(phi_count) {
                consume_fuel(&mut remaining)?;
                let InstructionKind::Phi { incomings } = &instruction.kind else {
                    unreachable!();
                };
                let incoming = incomings
                    .iter()
                    .find(|incoming| incoming.block == *predecessor_id)
                    .ok_or_else(|| {
                        EvalError::Invalid(Diagnostic::at(
                            "E_PHI",
                            instruction.span,
                            "missing input for predecessor edge",
                        ))
                    })?;
                let value = get_value(&values, &incoming.value, incoming.span)?;
                staged.push((instruction.id.clone(), value));
            }
            for (id, value) in staged {
                values.insert(id, value);
            }
        }

        for instruction in block.instructions.iter().skip(phi_count) {
            consume_fuel(&mut remaining)?;
            let value = match &instruction.kind {
                InstructionKind::Const(value) => Value::U64(*value),
                InstructionKind::Add { lhs, rhs } => Value::U64(
                    get_u64(&values, lhs, instruction.span)?.wrapping_add(get_u64(
                        &values,
                        rhs,
                        instruction.span,
                    )?),
                ),
                InstructionKind::Sub { lhs, rhs } => Value::U64(
                    get_u64(&values, lhs, instruction.span)?.wrapping_sub(get_u64(
                        &values,
                        rhs,
                        instruction.span,
                    )?),
                ),
                InstructionKind::Eq { lhs, rhs } => Value::Bool(
                    get_u64(&values, lhs, instruction.span)?
                        == get_u64(&values, rhs, instruction.span)?,
                ),
                InstructionKind::Phi { .. } => unreachable!(),
            };
            values.insert(instruction.id.clone(), value);
        }

        consume_fuel(&mut remaining)?;
        match &block.terminator.kind {
            TerminatorKind::Branch { target } => {
                previous = Some(current);
                current = validated.block_index[target];
            }
            TerminatorKind::BranchIf {
                condition,
                if_true,
                if_false,
            } => {
                let condition = match get_value(&values, condition, block.terminator.span)? {
                    Value::Bool(value) => value,
                    Value::U64(_) => {
                        return Err(EvalError::Invalid(Diagnostic::at(
                            "E_TYPE",
                            block.terminator.span,
                            "branch condition is not bool",
                        )));
                    }
                };
                previous = Some(current);
                current = validated.block_index[if condition { if_true } else { if_false }];
            }
            TerminatorKind::Return { value } => {
                return match get_value(&values, value, block.terminator.span)? {
                    Value::U64(result) => Ok(result),
                    Value::Bool(_) => Err(EvalError::Invalid(Diagnostic::at(
                        "E_TYPE",
                        block.terminator.span,
                        "return value is not u64",
                    ))),
                };
            }
        }
    }
}

fn consume_fuel(fuel: &mut u64) -> std::result::Result<(), EvalError> {
    if *fuel == 0 {
        return Err(EvalError::FuelExhausted);
    }
    *fuel -= 1;
    Ok(())
}

fn get_value(
    values: &BTreeMap<String, Value>,
    name: &str,
    span: crate::diagnostic::Span,
) -> std::result::Result<Value, EvalError> {
    values.get(name).copied().ok_or_else(|| {
        EvalError::Invalid(Diagnostic::at(
            "E_UNDEFINED_VALUE",
            span,
            format!("value %{} is not available during evaluation", name),
        ))
    })
}

fn get_u64(
    values: &BTreeMap<String, Value>,
    name: &str,
    span: crate::diagnostic::Span,
) -> std::result::Result<u64, EvalError> {
    match get_value(values, name, span)? {
        Value::U64(value) => Ok(value),
        Value::Bool(_) => Err(EvalError::Invalid(Diagnostic::at(
            "E_TYPE",
            span,
            format!("value %{} is not u64", name),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::validate::ValueType;

    fn eval(source: &str, input: u64) -> std::result::Result<u64, EvalError> {
        let module = parse(source.as_bytes()).unwrap();
        evaluate(&module, input, 10_000)
    }

    #[test]
    fn evaluates_wrapping_arithmetic_and_comparisons() {
        let source = "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %zero = const.u64 0\n %one = const.u64 1\n %decrement = sub.wrap.u64 %zero, %one\n %increment = add.wrap.u64 %x, %one\n %is_zero = eq.u64 %x, %zero\n br.if %is_zero, ^zero, ^other\n^zero:\n br ^join\n^other:\n br ^join\n^join:\n %result = phi.u64 [^zero, %decrement] [^other, %increment]\n ret.u64 %result\n}\n";
        assert_eq!(eval(source, 0), Ok(u64::MAX));
        assert_eq!(eval(source, 1), Ok(2));
        assert_eq!(eval(source, u64::MAX), Ok(0));
    }

    #[test]
    fn phi_values_are_assigned_in_parallel_across_loop_edges() {
        let source = "AIC 0.2\nfn aic_main(%n: u64) -> u64 {\n^entry:\n %zero = const.u64 0\n %one = const.u64 1\n br ^head\n^head:\n %i = phi.u64 [^entry, %zero] [^step, %next]\n %a = phi.u64 [^entry, %zero] [^step, %b]\n %b = phi.u64 [^entry, %one] [^step, %a]\n %done = eq.u64 %i, %n\n br.if %done, ^exit, ^step\n^step:\n %next = add.wrap.u64 %i, %one\n br ^head\n^exit:\n ret.u64 %a\n}\n";
        for (input, expected) in [(0, 0), (1, 1), (2, 0), (7, 1)] {
            assert_eq!(eval(source, input), Ok(expected));
        }
    }

    #[test]
    fn fuel_exhaustion_does_not_claim_termination() {
        let source =
            "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n br ^loop\n^loop:\n br ^loop\n}\n";
        let module = parse(source.as_bytes()).unwrap();
        assert_eq!(validate(&module).unwrap().value_types["x"], ValueType::U64);
        assert_eq!(evaluate(&module, 0, 10), Err(EvalError::FuelExhausted));
    }
}
