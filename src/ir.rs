use crate::diagnostic::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Module {
    pub parameter: String,
    pub parameter_span: Span,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    pub id: String,
    pub span: Span,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Instruction {
    pub id: String,
    pub kind: InstructionKind,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstructionKind {
    Const(u64),
    Add { lhs: String, rhs: String },
    Sub { lhs: String, rhs: String },
    Eq { lhs: String, rhs: String },
    Phi { incomings: Vec<Incoming> },
}

impl Instruction {
    pub fn is_phi(&self) -> bool {
        matches!(self.kind, InstructionKind::Phi { .. })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Incoming {
    pub block: String,
    pub value: String,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Terminator {
    pub kind: TerminatorKind,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminatorKind {
    Branch {
        target: String,
    },
    BranchIf {
        condition: String,
        if_true: String,
        if_false: String,
    },
    Return {
        value: String,
    },
}
