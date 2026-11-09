use std::fmt;

pub type Result<T> = std::result::Result<T, Diagnostic>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub span: Option<Span>,
    pub detail: String,
}

impl Diagnostic {
    pub fn at(code: &'static str, span: Span, detail: impl Into<String>) -> Self {
        Self {
            code,
            span: Some(span),
            detail: detail.into(),
        }
    }

    pub fn global(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            span: None,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => write!(
                f,
                "error[{}] at {}:{}: {}",
                self.code, span.line, span.column, self.detail
            ),
            None => write!(f, "error[{}]: {}", self.code, self.detail),
        }
    }
}

impl std::error::Error for Diagnostic {}
