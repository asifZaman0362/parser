extern crate regex;

enum TokenKind<'a> {
    Plus,
    Minus,
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(&'a str),
    Identifier(&'a str),
    Let,
    Var,
    Func,
    Def,
    Struct,
    End,
    Repeat,
    Until
}

struct Token<'a> {
    kind: TokenKind<'a>,
    line: usize,
    col: usize,
}

enum LexerErrorKind {
    UnexpectEof,
    InvalidToken
}

struct LexerError<'a> {
    error_kind: LexerErrorKind,
    line_number: usize,
    column: usize,
    token: &'a str,
}

fn make_token(lexeme: &str, line: usize, col: usize) -> Result<Token, LexerError> {
    match lexeme {
        "let" => Ok(Token{line, col, kind: TokenKind::Let}),
        "def" => Ok(Token{line, col, kind: TokenKind::Def}),
        "struct" => Ok(Token{line, col, kind: TokenKind::Def}),
        "func" => Ok(Token{line, col, kind: TokenKind::Func}),
        "var" => Ok(Token{line, col, kind: TokenKind::Var}),
        "repeat" => Ok(Token{line, col, kind: TokenKind::Repeat}),
        "until" => Ok(Token{line, col, kind: TokenKind::Until}),
        "end" => Ok(Token{line, col, kind: TokenKind::End}),
        _ => {
            let ident_regex = regex::Regex::new(r#"^w+(w,d)*$"#).unwrap();
            if ident_regex.is_match(lexeme) {
                Ok(Token{line, col, kind: TokenKind::Identifier(lexeme)})
            } else {
                Err(LexerError{line_number: line, column: col, error_kind: LexerErrorKind::InvalidToken, token: lexeme})
            }
        }
    }
}

fn tokenize<'a>(code: &'a str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = vec![];
    let mut iter = code.as_bytes().iter().enumerate();
    let (mut line, mut column) = (0, 0);
    let mut start= 0;
    while let Some((idx, chr)) = iter.next() {
        match chr {
            b' ' | b'\t' => {
                tokens.push(make_token(&code[start..idx], line, column)?);
                start = idx;
            }
            b'\n' => {
                line += 1;
                column = 0;
            }
            b'0'..=b'9' => {}
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {}
            _ => {}
        }
    }
    Ok(vec![])
}

fn main() -> std::io::Result<()> {
    Ok(())
}
