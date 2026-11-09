pub mod diagnostic;
pub mod driver;
pub mod interpreter;
pub mod ir;
pub mod parser;
pub mod validate;
pub mod x86_64;

pub use diagnostic::{Diagnostic, Result, Span};
pub use interpreter::{EvalError, evaluate, evaluate_many};
pub use ir::{Block, Incoming, Instruction, InstructionKind, Module, Terminator, TerminatorKind};
pub use parser::parse;
pub use validate::validate;
