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
#[error("Encountered error:\n{error_kind}\n at {line}:{col} while processing token \"{token}\"")]
struct LexerError<'a> {
    error_kind: LexerErrorKind,
    line: usize,
    col: usize,
    token: &'a str,
}

fn make_token(lexeme: &str, line: usize, col: usize) -> Result<Token, LexerError> {
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
                    Err(LexerError {
                        line,
                        col,
                        token: lexeme,
                        error_kind: LexerErrorKind::InvalidToken,
                    })
                }
            } else if let Some(stripped) = lexeme.strip_prefix("0b") {
                if let Ok(parsed_int) = i64::from_str_radix(stripped, 2) {
                    Ok(Token {
                        line,
                        col,
                        kind: TokenKind::IntegerLiteral(parsed_int),
                    })
                } else {
                    Err(LexerError {
                        line,
                        col,
                        token: lexeme,
                        error_kind: LexerErrorKind::InvalidToken,
                    })
                }
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
                    Err(LexerError {
                        line,
                        col,
                        error_kind: LexerErrorKind::InvalidToken,
                        token: lexeme,
                    })
                }
            }
        }
    }
}

fn tokenize(code: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = vec![];
    let mut iter = code.as_bytes().iter().enumerate();
    let (mut line, mut col) = (1, 1);
    let mut start = 0;
    while let Some((idx, chr)) = iter.next() {
        let scanned_token = match chr {
            b' ' | b'\t' => {
                col += 1;
                continue;
            }
            b'\n' => {
                col = 1;
                line += 1;
                continue;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => {
                if let Some(next) = code.as_bytes().get(idx + 1) {
                    match next {
                        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' => {
                            col += 1;
                            continue;
                        }
                        _ => {
                            make_token(&code[start..idx+1], line, col)
                        }
                    }
                } else {
                    make_token(&code[start..idx], line, col)
                }
            }

            b',' => Ok(Token {
                kind: TokenKind::Comma,
                line,
                col: col + 1,
            }),
            b':' => Ok(Token {
                kind: TokenKind::Colon,
                line,
                col: col + 1,
            }),
            b'=' => Ok(Token {
                kind: TokenKind::Equals,
                line,
                col: col + 1,
            }),
            b';' => Ok(Token {
                kind: TokenKind::SemiColon,
                line,
                col: col + 1,
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
            b'"' => loop {
                if let Some((x, chr)) = iter.next() {
                    if *chr == b'"' {
                        break Ok(Token {
                            kind: TokenKind::StringLiteral(&code[start..x]),
                            line,
                            col,
                        });
                    }
                } else {
                    break Err(LexerError {
                        line,
                        col,
                        error_kind: LexerErrorKind::UnexpectedEof("\"".to_owned()),
                        token: &code[idx..],
                    });
                }
            },
            _ => Err(LexerError {
                error_kind: LexerErrorKind::UnexpectedCharacter(*chr as char),
                line,
                col,
                token: &code[idx..idx + 1],
            }),
        };
        if let Ok(token) = scanned_token {
            tokens.push(token);
            start = idx;
            col += 1;
        }
    }
    Ok(tokens)
}

fn main() -> std::io::Result<()> {
    let mut file = std::fs::File::open("test.jasm")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    match tokenize(&buf) {
        Ok(tokens) => {
            println!("{:?}", tokens);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
    Ok(())
}
