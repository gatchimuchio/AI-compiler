use crate::diagnostic::{Diagnostic, Result, Span};
use crate::ir::{
    Block, Incoming, Instruction, InstructionKind, Module, Terminator, TerminatorKind,
};
use crate::limits::{MAX_IDENTIFIER_BYTES, MAX_SOURCE_BYTES, check_budget};

#[derive(Clone, Debug, Eq, PartialEq)]
enum Kind {
    Word(String),
    Value(String),
    Block(String),
    Punct(char),
    Arrow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Token {
    kind: Kind,
    span: Span,
    end_column: usize,
}

impl Token {
    fn is_wordlike(&self) -> bool {
        matches!(self.kind, Kind::Word(_) | Kind::Value(_) | Kind::Block(_))
    }
}

struct Record {
    line: usize,
    tokens: Vec<Token>,
}

struct PendingBlock {
    id: String,
    span: Span,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
}

pub fn parse(input: &[u8]) -> Result<Module> {
    check_budget(input.len(), MAX_SOURCE_BYTES, "source bytes", None)?;
    let text = validate_encoding(input)?;
    let mut records = Vec::new();
    for (index, raw) in text.split('\n').enumerate() {
        let line_number = index + 1;
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let code = raw.split_once(';').map_or(raw, |(prefix, _)| prefix);
        let tokens = lex_line(code, line_number)?;
        if !tokens.is_empty() {
            records.push(Record {
                line: line_number,
                tokens,
            });
        }
    }

    let header = records
        .first()
        .ok_or_else(|| Diagnostic::at("E_SYNTAX", Span::new(1, 1), "missing AIC header"))?;
    if header.tokens.len() != 2 || !word_is(&header.tokens[0], "AIC") {
        return Err(Diagnostic::at(
            "E_SYNTAX",
            header.tokens[0].span,
            "expected AIC version header",
        ));
    }
    let version = match &header.tokens[1].kind {
        Kind::Word(value) if is_version(value) => value,
        _ => {
            return Err(Diagnostic::at(
                "E_SYNTAX",
                header.tokens[1].span,
                "malformed version token",
            ));
        }
    };
    if version != "0.2" {
        return Err(Diagnostic::at(
            "E_VERSION",
            header.tokens[1].span,
            "only AIC 0.2 is supported",
        ));
    }

    let function = records.get(1).ok_or_else(|| {
        Diagnostic::at(
            "E_SYNTAX",
            Span::new(header.line + 1, 1),
            "missing function",
        )
    })?;
    if function.tokens.len() != 10
        || !word_is(&function.tokens[0], "fn")
        || !word_is(&function.tokens[1], "aic_main")
        || !punct_is(&function.tokens[2], '(')
        || !punct_is(&function.tokens[4], ':')
        || !word_is(&function.tokens[5], "u64")
        || !punct_is(&function.tokens[6], ')')
        || !matches!(function.tokens[7].kind, Kind::Arrow)
        || !word_is(&function.tokens[8], "u64")
        || !punct_is(&function.tokens[9], '{')
    {
        return Err(Diagnostic::at(
            "E_SYNTAX",
            function.tokens[0].span,
            "expected fn aic_main(%arg: u64) -> u64 {",
        ));
    }
    let parameter = value_id(&function.tokens[3]).ok_or_else(|| {
        Diagnostic::at(
            "E_SYNTAX",
            function.tokens[3].span,
            "expected one u64 parameter",
        )
    })?;
    let parameter_span = function.tokens[3].span;

    let mut pending: Option<PendingBlock> = None;
    let mut blocks = Vec::new();
    let mut closed = false;
    for record in records.iter().skip(2) {
        let tokens = &record.tokens;
        if closed {
            return Err(Diagnostic::at(
                "E_SYNTAX",
                tokens[0].span,
                "content after function end",
            ));
        }
        if tokens.len() == 1 && punct_is(&tokens[0], '}') {
            finish_block(&mut pending, &mut blocks, tokens[0].span)?;
            closed = true;
            continue;
        }
        if tokens.len() == 2 && block_id(&tokens[0]).is_some() && punct_is(&tokens[1], ':') {
            if let Some(previous) = pending.as_ref()
                && previous.terminator.is_none()
            {
                return Err(Diagnostic::at(
                    "E_CFG",
                    tokens[0].span,
                    "block ended without a terminator",
                ));
            }
            finish_block(&mut pending, &mut blocks, tokens[0].span)?;
            pending = Some(PendingBlock {
                id: block_id(&tokens[0]).expect("checked above").to_owned(),
                span: tokens[0].span,
                instructions: Vec::new(),
                terminator: None,
            });
            continue;
        }
        let current = pending.as_mut().ok_or_else(|| {
            Diagnostic::at(
                "E_SYNTAX",
                tokens[0].span,
                "expected a block label or function end",
            )
        })?;
        if current.terminator.is_some() {
            return Err(Diagnostic::at(
                "E_SYNTAX",
                tokens[0].span,
                "content after block terminator",
            ));
        }
        if matches!(tokens[0].kind, Kind::Word(_)) {
            current.terminator = Some(parse_terminator(tokens, record.line)?);
        } else {
            current
                .instructions
                .push(parse_instruction(tokens, record.line)?);
        }
    }
    if !closed {
        let eof = Span::new(text.lines().count() + 1, 1);
        return Err(Diagnostic::at(
            "E_SYNTAX",
            eof,
            "missing function closing brace",
        ));
    }

    Ok(Module {
        parameter: parameter.to_owned(),
        parameter_span,
        blocks,
    })
}

fn validate_encoding(input: &[u8]) -> Result<&str> {
    let mut line = 1usize;
    let mut column = 1usize;
    for (index, byte) in input.iter().copied().enumerate() {
        let span = Span::new(line, column);
        if byte == 0
            || byte > 0x7f
            || (byte < 0x20 && !matches!(byte, b'\t' | b'\n' | b'\r'))
            || byte == 0x7f
        {
            return Err(Diagnostic::at(
                "E_ENCODING",
                span,
                "input must contain printable ASCII, horizontal whitespace, and line endings only",
            ));
        }
        if byte == b'\r' && input.get(index + 1) != Some(&b'\n') {
            return Err(Diagnostic::at(
                "E_ENCODING",
                span,
                "bare CR is not permitted",
            ));
        }
        if byte == b'\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    std::str::from_utf8(input)
        .map_err(|_| Diagnostic::global("E_ENCODING", "input is not valid ASCII text"))
}

fn lex_line(line: &str, line_number: usize) -> Result<Vec<Token>> {
    let bytes = line.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' || bytes[i] == b'\t' {
            i += 1;
            continue;
        }
        let start = i;
        let kind = match bytes[i] {
            b'%' | b'^' => {
                let prefix = bytes[i];
                i += 1;
                if i >= bytes.len() || !bytes[i].is_ascii_lowercase() {
                    return Err(Diagnostic::at(
                        "E_SYNTAX",
                        Span::new(line_number, start + 1),
                        "value and block references require a lowercase identifier",
                    ));
                }
                let id_start = i;
                i += 1;
                while i < bytes.len()
                    && (bytes[i].is_ascii_lowercase()
                        || bytes[i].is_ascii_digit()
                        || bytes[i] == b'_')
                {
                    i += 1;
                }
                if i - id_start > MAX_IDENTIFIER_BYTES {
                    return Err(Diagnostic::at(
                        "E_SYNTAX",
                        Span::new(line_number, start + 1),
                        "identifier exceeds the 63-byte limit",
                    ));
                }
                let id = line[id_start..i].to_owned();
                if prefix == b'%' {
                    Kind::Value(id)
                } else {
                    Kind::Block(id)
                }
            }
            b'=' | b':' | b'(' | b')' | b'[' | b']' | b',' | b'}' | b'{' => {
                i += 1;
                Kind::Punct(bytes[start] as char)
            }
            b'-' if bytes.get(i + 1) == Some(&b'>') => {
                i += 2;
                Kind::Arrow
            }
            byte if byte.is_ascii_alphanumeric() || byte == b'_' => {
                i += 1;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.')
                {
                    i += 1;
                }
                Kind::Word(line[start..i].to_owned())
            }
            _ => {
                return Err(Diagnostic::at(
                    "E_SYNTAX",
                    Span::new(line_number, start + 1),
                    "invalid character or token",
                ));
            }
        };
        let token = Token {
            kind,
            span: Span::new(line_number, start + 1),
            end_column: i + 1,
        };
        if tokens.last().is_some_and(|previous: &Token| {
            previous.is_wordlike()
                && token.is_wordlike()
                && previous.end_column == token.span.column
        }) {
            return Err(Diagnostic::at(
                "E_SYNTAX",
                token.span,
                "adjacent word tokens require horizontal whitespace",
            ));
        }
        tokens.push(token);
    }
    Ok(tokens)
}

fn parse_instruction(tokens: &[Token], line: usize) -> Result<Instruction> {
    let span = tokens
        .first()
        .map_or(Span::new(line, 1), |token| token.span);
    let id = tokens
        .first()
        .and_then(value_id)
        .ok_or_else(|| Diagnostic::at("E_SYNTAX", span, "instruction must define a value"))?
        .to_owned();
    if tokens.len() < 3 || !punct_is(&tokens[1], '=') {
        return Err(Diagnostic::at(
            "E_SYNTAX",
            span,
            "expected value definition and operation",
        ));
    }
    let operation = match &tokens[2].kind {
        Kind::Word(operation) => operation.as_str(),
        _ => {
            return Err(Diagnostic::at(
                "E_SYNTAX",
                tokens[2].span,
                "expected operation name",
            ));
        }
    };
    if !is_operation_name(operation) {
        return Err(Diagnostic::at(
            "E_SYNTAX",
            tokens[2].span,
            "malformed operation token",
        ));
    }
    let kind = match operation {
        "const.u64" => {
            if tokens.len() != 4 {
                return Err(Diagnostic::at(
                    "E_SYNTAX",
                    span,
                    "const.u64 expects one integer",
                ));
            }
            InstructionKind::Const(parse_integer(&tokens[3])?)
        }
        "add.wrap.u64" | "sub.wrap.u64" | "eq.u64" => {
            if tokens.len() != 6 || !punct_is(&tokens[4], ',') {
                return Err(Diagnostic::at(
                    "E_SYNTAX",
                    span,
                    "binary operation expects two values separated by a comma",
                ));
            }
            let lhs = value_id(&tokens[3]).ok_or_else(|| {
                Diagnostic::at("E_SYNTAX", tokens[3].span, "expected value operand")
            })?;
            let rhs = value_id(&tokens[5]).ok_or_else(|| {
                Diagnostic::at("E_SYNTAX", tokens[5].span, "expected value operand")
            })?;
            match operation {
                "add.wrap.u64" => InstructionKind::Add {
                    lhs: lhs.to_owned(),
                    rhs: rhs.to_owned(),
                },
                "sub.wrap.u64" => InstructionKind::Sub {
                    lhs: lhs.to_owned(),
                    rhs: rhs.to_owned(),
                },
                _ => InstructionKind::Eq {
                    lhs: lhs.to_owned(),
                    rhs: rhs.to_owned(),
                },
            }
        }
        "phi.u64" => {
            let mut incomings = Vec::new();
            let mut i = 3;
            while i < tokens.len() {
                if i + 4 >= tokens.len()
                    || !punct_is(&tokens[i], '[')
                    || !punct_is(&tokens[i + 2], ',')
                    || !punct_is(&tokens[i + 4], ']')
                {
                    return Err(Diagnostic::at(
                        "E_PHI",
                        tokens[i].span,
                        "expected [^predecessor, %value] incoming",
                    ));
                }
                let block = block_id(&tokens[i + 1]).ok_or_else(|| {
                    Diagnostic::at("E_PHI", tokens[i + 1].span, "expected predecessor block")
                })?;
                let value = value_id(&tokens[i + 3]).ok_or_else(|| {
                    Diagnostic::at("E_PHI", tokens[i + 3].span, "expected incoming value")
                })?;
                incomings.push(Incoming {
                    block: block.to_owned(),
                    value: value.to_owned(),
                    span: tokens[i].span,
                });
                i += 5;
            }
            if incomings.is_empty() {
                return Err(Diagnostic::at(
                    "E_PHI",
                    span,
                    "phi.u64 requires at least one incoming",
                ));
            }
            InstructionKind::Phi { incomings }
        }
        _ => {
            return Err(Diagnostic::at(
                "E_UNSUPPORTED_OP",
                tokens[2].span,
                format!("unsupported operation {operation}"),
            ));
        }
    };
    Ok(Instruction { id, kind, span })
}

fn parse_terminator(tokens: &[Token], line: usize) -> Result<Terminator> {
    let span = tokens[0].span;
    let operation = match &tokens[0].kind {
        Kind::Word(operation) => operation.as_str(),
        _ => unreachable!(),
    };
    if !is_operation_name(operation) {
        return Err(Diagnostic::at(
            "E_SYNTAX",
            span,
            "malformed terminator token",
        ));
    }
    let kind = match operation {
        "br" if tokens.len() == 2 => {
            let target = block_id(&tokens[1]).ok_or_else(|| {
                Diagnostic::at("E_SYNTAX", tokens[1].span, "expected branch target")
            })?;
            TerminatorKind::Branch {
                target: target.to_owned(),
            }
        }
        "br.if" if tokens.len() == 6 && punct_is(&tokens[2], ',') && punct_is(&tokens[4], ',') => {
            let condition = value_id(&tokens[1]).ok_or_else(|| {
                Diagnostic::at("E_SYNTAX", tokens[1].span, "expected bool condition")
            })?;
            let if_true = block_id(&tokens[3]).ok_or_else(|| {
                Diagnostic::at("E_SYNTAX", tokens[3].span, "expected true branch target")
            })?;
            let if_false = block_id(&tokens[5]).ok_or_else(|| {
                Diagnostic::at("E_SYNTAX", tokens[5].span, "expected false branch target")
            })?;
            TerminatorKind::BranchIf {
                condition: condition.to_owned(),
                if_true: if_true.to_owned(),
                if_false: if_false.to_owned(),
            }
        }
        "ret.u64" if tokens.len() == 2 => {
            let value = value_id(&tokens[1]).ok_or_else(|| {
                Diagnostic::at("E_SYNTAX", tokens[1].span, "expected return value")
            })?;
            TerminatorKind::Return {
                value: value.to_owned(),
            }
        }
        _ => {
            let code = if operation.contains('.') {
                "E_UNSUPPORTED_OP"
            } else if operation == "br" || operation == "br.if" || operation == "ret.u64" {
                "E_SYNTAX"
            } else {
                "E_UNSUPPORTED_OP"
            };
            return Err(Diagnostic::at(
                code,
                span,
                format!("invalid or unsupported terminator {operation} on line {line}"),
            ));
        }
    };
    Ok(Terminator { kind, span })
}

fn parse_integer(token: &Token) -> Result<u64> {
    let text = match &token.kind {
        Kind::Word(text) => text.as_str(),
        _ => {
            return Err(Diagnostic::at(
                "E_SYNTAX",
                token.span,
                "expected unsigned integer",
            ));
        }
    };
    if let Some(hex) = text.strip_prefix("0x") {
        if hex.is_empty() || hex.len() > 16 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Diagnostic::at(
                "E_SYNTAX",
                token.span,
                "invalid hexadecimal u64 literal",
            ));
        }
        u64::from_str_radix(hex, 16)
            .map_err(|_| Diagnostic::at("E_SYNTAX", token.span, "u64 literal is out of range"))
    } else if text == "0"
        || (!text.starts_with('0') && text.len() <= 20 && text.bytes().all(|b| b.is_ascii_digit()))
    {
        text.parse::<u64>()
            .map_err(|_| Diagnostic::at("E_SYNTAX", token.span, "u64 literal is out of range"))
    } else {
        Err(Diagnostic::at(
            "E_SYNTAX",
            token.span,
            "invalid decimal u64 literal",
        ))
    }
}

fn finish_block(
    pending: &mut Option<PendingBlock>,
    blocks: &mut Vec<Block>,
    span: Span,
) -> Result<()> {
    let Some(current) = pending.take() else {
        return Ok(());
    };
    let terminator = current
        .terminator
        .ok_or_else(|| Diagnostic::at("E_CFG", span, "block ended without a terminator"))?;
    blocks.push(Block {
        id: current.id,
        span: current.span,
        instructions: current.instructions,
        terminator,
    });
    Ok(())
}

fn value_id(token: &Token) -> Option<&str> {
    match &token.kind {
        Kind::Value(id) => Some(id),
        _ => None,
    }
}

fn block_id(token: &Token) -> Option<&str> {
    match &token.kind {
        Kind::Block(id) => Some(id),
        _ => None,
    }
}

fn word_is(token: &Token, expected: &str) -> bool {
    matches!(&token.kind, Kind::Word(word) if word == expected)
}

fn punct_is(token: &Token, expected: char) -> bool {
    matches!(token.kind, Kind::Punct(value) if value == expected)
}

fn is_version(value: &str) -> bool {
    let Some((major, minor)) = value.split_once('.') else {
        return false;
    };
    !major.is_empty()
        && !minor.is_empty()
        && major.bytes().all(|b| b.is_ascii_digit())
        && minor.bytes().all(|b| b.is_ascii_digit())
        && !minor.contains('.')
}

fn is_operation_name(value: &str) -> bool {
    value.split('.').all(|part| {
        let bytes = part.as_bytes();
        !bytes.is_empty()
            && bytes.len() <= MAX_IDENTIFIER_BYTES
            && bytes[0].is_ascii_lowercase()
            && bytes[1..]
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::validate;

    const SIMPLE: &[u8] = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n  %one = const.u64 1\n  %y = add.wrap.u64 %x, %one\n  ret.u64 %y\n}\n";

    #[test]
    fn accepts_lf_crlf_comments_tabs_and_no_final_newline() {
        let crlf = b"; header\r\nAIC\t0.2\r\nfn aic_main(%x:u64)->u64 {\r\n^entry:\r\n\t%one=const.u64 1 ; one\r\n%y=add.wrap.u64 %x,%one\r\nret.u64 %y\r\n}\r\n";
        assert!(parse(crlf).is_ok());
        assert!(parse(&SIMPLE[..SIMPLE.len() - 1]).is_ok());
    }

    #[test]
    fn rejects_bad_encoding_and_versions_with_stable_codes() {
        assert_eq!(
            parse(b"\xef\xbb\xbfAIC 0.2\n").unwrap_err().code,
            "E_ENCODING"
        );
        assert_eq!(parse(b"AIC 0.3\n").unwrap_err().code, "E_VERSION");
    }

    #[test]
    fn parses_variable_length_phi_inputs() {
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n  br ^join\n^join:\n  %result = phi.u64 [^entry, %x]\n  ret.u64 %result\n}\n";
        let module = parse(source).unwrap();
        let InstructionKind::Phi { incomings } = &module.blocks[1].instructions[0].kind else {
            panic!("expected phi");
        };
        assert_eq!(incomings.len(), 1);
    }

    #[test]
    fn reports_unsupported_operation() {
        let source = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n  %y = mul.u64 %x, %x\n  ret.u64 %y\n}\n";
        let error = parse(source).unwrap_err();
        assert_eq!(error.code, "E_UNSUPPORTED_OP");
        assert_eq!(error.span, Some(Span::new(4, 8)));
    }

    #[test]
    fn checks_u64_literal_boundaries_and_decimal_spelling() {
        for (literal, accepted) in [
            ("0", true),
            ("18446744073709551615", true),
            ("0xffffffffffffffff", true),
            ("18446744073709551616", false),
            ("00", false),
            ("0x", false),
            ("0Xff", false),
            ("-1", false),
        ] {
            let source = format!(
                "AIC 0.2\nfn aic_main(%x: u64) -> u64 {{\n^entry:\n %v = const.u64 {literal}\n ret.u64 %v\n}}\n"
            );
            assert_eq!(
                parse(source.as_bytes()).is_ok(),
                accepted,
                "literal {literal}"
            );
        }
    }

    #[test]
    fn rejects_bare_cr_non_ascii_and_missing_line_delimiters() {
        assert_eq!(
            parse(b"AIC 0.2\rfn aic_main(%x: u64) -> u64 {\n}\n")
                .unwrap_err()
                .code,
            "E_ENCODING"
        );
        assert_eq!(
            parse(
                "AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\n;注".as_bytes()
            )
            .unwrap_err()
            .code,
            "E_ENCODING"
        );
        let split_instruction = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %v = add.wrap.u64\n %x, %x\n ret.u64 %v\n}\n";
        assert_eq!(parse(split_instruction).unwrap_err().code, "E_SYNTAX");
    }

    #[test]
    fn enforces_control_byte_and_identifier_limits() {
        for byte in [0x01, 0x0b, 0x0c, 0x1f, 0x7f] {
            let mut source = SIMPLE.to_vec();
            source.insert(source.len() - 1, byte);
            assert_eq!(parse(&source).unwrap_err().code, "E_ENCODING", "{byte:#x}");
        }

        let accepted = "a".repeat(63);
        let source = format!(
            "AIC 0.2\nfn aic_main(%{accepted}: u64) -> u64 {{\n^entry:\n ret.u64 %{accepted}\n}}\n"
        );
        assert!(parse(source.as_bytes()).is_ok());

        let rejected = "a".repeat(64);
        let source = format!(
            "AIC 0.2\nfn aic_main(%{rejected}: u64) -> u64 {{\n^entry:\n ret.u64 %{rejected}\n}}\n"
        );
        assert_eq!(parse(source.as_bytes()).unwrap_err().code, "E_SYNTAX");
    }

    #[test]
    fn separates_malformed_operation_tokens_from_unsupported_operations() {
        let malformed = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %y = add..wrap.u64 %x, %x\n ret.u64 %x\n}\n";
        assert_eq!(parse(malformed).unwrap_err().code, "E_SYNTAX");

        let unsupported = b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n %y = mul.u64 %x, %x\n ret.u64 %x\n}\n";
        assert_eq!(parse(unsupported).unwrap_err().code, "E_UNSUPPORTED_OP");
    }

    #[test]
    fn rejects_misplaced_records_and_incomplete_blocks() {
        let cases: &[(&[u8], &str)] = &[
            (b"", "E_SYNTAX"),
            (b"AIC 0.2\n", "E_SYNTAX"),
            (
                b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n %v = const.u64 1\n}",
                "E_SYNTAX",
            ),
            (
                b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n}\n",
                "E_CFG",
            ),
            (
                b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n br ^next\n^next:\n ret.u64 %x\n",
                "E_SYNTAX",
            ),
            (
                b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n ret.u64 %x\n}\n",
                "E_SYNTAX",
            ),
            (
                b"AIC 0.2\nfn aic_main(%x: u64) -> u64 {\n^entry:\n ret.u64 %x\n}\nret.u64 %x\n",
                "E_SYNTAX",
            ),
        ];
        for (source, code) in cases {
            assert_eq!(parse(source).unwrap_err().code, *code, "{source:?}");
        }
    }

    #[test]
    fn arbitrary_byte_inputs_never_panic_the_parser() {
        let mut state = 0x8f31_2d79_c4a5_611b_u64;
        for _ in 0..5_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let length = (state as usize) % 96;
            let mut bytes = Vec::with_capacity(length);
            for _ in 0..length {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                bytes.push((state >> 32) as u8);
            }
            let result = std::panic::catch_unwind(|| parse(&bytes));
            assert!(result.is_ok(), "parser panicked for {bytes:?}");
        }
    }

    #[test]
    fn single_byte_mutations_and_truncations_do_not_panic_parse_or_validate() {
        const REPLACEMENTS: &[u8] = &[
            0, b'\t', b'\n', b'\r', b' ', b'%', b'^', b'_', b'0', b'9', b'a', b'Z', b';', b'=',
            b':', b',', b'[', b']', b'{', b'}', b'-', b'>', b'.', 0x7f,
        ];
        let fixtures: &[&[u8]] = &[
            include_bytes!("../examples/branch_phi.aic"),
            include_bytes!("../examples/loop_swap.aic"),
        ];

        for fixture in fixtures {
            for index in 0..fixture.len() {
                for replacement in REPLACEMENTS {
                    let mut mutated = fixture.to_vec();
                    mutated[index] = *replacement;
                    let checked = std::panic::catch_unwind(|| {
                        if let Ok(module) = parse(&mutated) {
                            let _ = validate(&module);
                        }
                    });
                    assert!(
                        checked.is_ok(),
                        "panic after replacing byte {index} with {replacement:#x}"
                    );
                }

                let mut deleted = fixture.to_vec();
                deleted.remove(index);
                let checked = std::panic::catch_unwind(|| {
                    if let Ok(module) = parse(&deleted) {
                        let _ = validate(&module);
                    }
                });
                assert!(checked.is_ok(), "panic after deleting byte {index}");
            }

            for end in 0..=fixture.len() {
                let truncated = &fixture[..end];
                let checked = std::panic::catch_unwind(|| {
                    if let Ok(module) = parse(truncated) {
                        let _ = validate(&module);
                    }
                });
                assert!(checked.is_ok(), "panic after truncating at byte {end}");
            }
        }
    }
}
