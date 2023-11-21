extern crate regex;
extern crate thiserror;

use std::io::Read;

#[derive(PartialEq, Debug)]
enum TokenKind<'a> {
    Plus,
    Minus,
    Divide,
    Multiply,
    Modulo,
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(&'a str),
    Identifier(&'a str),
    Let,
    Var,
    Def,
    Func,
    Struct,
    End,
    Repeat,
    Until,
    Comma,
    SemiColon,
    Colon,
    Equals,
    Or,
    And,
    Xor,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Token<'a> {
    kind: TokenKind<'a>,
    line: usize,
    col: usize,
}

#[derive(Debug, thiserror::Error)]
enum LexerErrorKind {
    #[error("Unexpectedly reached end of file! Expected {0}")]
    UnexpectedEof(String),
    #[error("Invalid token")]
    InvalidToken,
    #[error("Unexpected character '{0}'")]
    UnexpectedCharacter(char),
}

#[derive(Debug, thiserror::Error)]
#[error("{filename}:{line}:{col}\nSyntax error:\n\t{snippet}\n\t{error_kind}")]
struct LexerError<'a> {
    error_kind: LexerErrorKind,
    line: usize,
    col: usize,
    filename: &'a str,
    snippet: &'a str,
}

fn make_token<'a>(lexeme: &'a str, line: usize, col: usize) -> Result<Token<'a>, LexerErrorKind> {
    let col = col - (lexeme.len() - 1);
    match lexeme {
        "let" => Ok(Token {
            line,
            col,
            kind: TokenKind::Let,
        }),
        "def" => Ok(Token {
            line,
            col,
            kind: TokenKind::Def,
        }),
        "struct" => Ok(Token {
            line,
            col,
            kind: TokenKind::Struct,
        }),
        "func" => Ok(Token {
            line,
            col,
            kind: TokenKind::Func,
        }),
        "var" => Ok(Token {
            line,
            col,
            kind: TokenKind::Var,
        }),
        "repeat" => Ok(Token {
            line,
            col,
            kind: TokenKind::Repeat,
        }),
        "until" => Ok(Token {
            line,
            col,
            kind: TokenKind::Until,
        }),
        "end" => Ok(Token {
            line,
            col,
            kind: TokenKind::End,
        }),
        _ => {
            if let Some(stripped) = lexeme.strip_prefix("0x") {
                if let Ok(parsed_int) = i64::from_str_radix(stripped, 16) {
                    Ok(Token {
                        line,
                        col,
                        kind: TokenKind::IntegerLiteral(parsed_int),
                    })
                } else {
                    Err(LexerErrorKind::InvalidToken)
                }
            } else if let Some(stripped) = lexeme.strip_prefix("0b") {
                if let Ok(parsed_int) = i64::from_str_radix(stripped, 2) {
                    Ok(Token {
                        line,
                        col,
                        kind: TokenKind::IntegerLiteral(parsed_int),
                    })
                } else {
                    Err(LexerErrorKind::InvalidToken)
                }
            } else if let Ok(parsed) = lexeme.parse::<i64>() {
                Ok(Token {
                    line,
                    col,
                    kind: TokenKind::IntegerLiteral(parsed),
                })
            } else if let Ok(parsed_float) = lexeme.parse::<f64>() {
                Ok(Token {
                    line,
                    col,
                    kind: TokenKind::FloatLiteral(parsed_float),
                })
            } else {
                let ident_regex = regex::Regex::new(r#"^[a-zA-Z_]+\w*$"#).unwrap();
                if ident_regex.is_match(lexeme) {
                    Ok(Token {
                        line,
                        col,
                        kind: TokenKind::Identifier(lexeme),
                    })
                } else {
                    Err(LexerErrorKind::InvalidToken)
                }
            }
        }
    }
}

fn tokenize<'a>(code: &'a str, filename: &'a str) -> Result<Vec<Token<'a>>, LexerError<'a>> {
    let mut tokens = vec![];
    let mut iter = code.as_bytes().iter().enumerate().peekable();
    let (mut line, mut col) = (1, 1);
    let mut start = 0;
    let mut i = 0;
    while let Some((idx, scanned)) = iter.next() {
        i = idx;
        let scanned_token = match scanned {
            b' ' | b'\t' => {
                col += 1;
                start = idx + 1;
                continue;
            }
            b'\n' => {
                col = 1;
                line += 1;
                start = idx + 1;
                continue;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' | b'.' => match iter.peek() {
                Some((next_idx, lookahead)) => match lookahead {
                    b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.' => {
                        col += 1;
                        continue;
                    }
                    _ => make_token(&code[start..*next_idx], line, col),
                },
                None => make_token(&code[start..idx], line, col),
            },
            b',' => Ok(Token {
                kind: TokenKind::Comma,
                line,
                col,
            }),
            b':' => Ok(Token {
                kind: TokenKind::Colon,
                line,
                col,
            }),
            b'=' => Ok(Token {
                kind: TokenKind::Equals,
                line,
                col,
            }),
            b';' => Ok(Token {
                kind: TokenKind::SemiColon,
                line,
                col,
            }),
            b'|' => Ok(Token {
                kind: TokenKind::Or,
                line,
                col,
            }),
            b'&' => Ok(Token {
                kind: TokenKind::And,
                line,
                col,
            }),
            b'^' => Ok(Token {
                kind: TokenKind::Xor,
                line,
                col,
            }),
            b'/' => Ok(Token {
                kind: TokenKind::Divide,
                line,
                col,
            }),
            b'*' => Ok(Token {
                kind: TokenKind::Multiply,
                line,
                col,
            }),
            b'+' => Ok(Token {
                kind: TokenKind::Plus,
                line,
                col,
            }),
            b'-' => Ok(Token {
                kind: TokenKind::Minus,
                line,
                col,
            }),
            b'%' => Ok(Token {
                kind: TokenKind::Modulo,
                line,
                col,
            }),
            b'"' | b'\'' => loop {
                // TODO: escape sequence
                if let Some((x, chr)) = iter.next() {
                    col += 1;
                    i += 1;
                    if chr == scanned {
                        let lexeme = &code[start + 1..x];
                        let len = lexeme.len() + 1;
                        break Ok(Token {
                            kind: TokenKind::StringLiteral(lexeme),
                            line,
                            col: col - len,
                        });
                    }
                } else {
                    break Err(LexerErrorKind::UnexpectedEof(format!(
                        "a '{}'",
                        *scanned as char
                    )));
                }
            },
            _ => Err(LexerErrorKind::UnexpectedCharacter(*scanned as char)),
        };
        match scanned_token {
            Ok(token) => tokens.push(token),
            Err(error_kind) => {
                return Err(LexerError {
                    error_kind,
                    line,
                    col,
                    filename,
                    snippet: &code[i - col + 1..=i],
                })
            }
        };
        start = idx;
        col += 1;
    }
    Ok(tokens)
}

fn main() -> std::io::Result<()> {
    let mut file = std::fs::File::open("test.jasm")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    match tokenize(&buf, "test.jasm") {
        Ok(tokens) => {
            println!("{:?}", tokens);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
    Ok(())
}
