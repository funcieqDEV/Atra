use logos::{Logos, Span};

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
pub enum Token {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("=")]
    Equals,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,

    #[regex(r"[ \t\n\r]+", logos::skip)]
    Whitespace,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub slice: String,
    pub line: usize,
    pub column: usize,
    pub span: Span,
}

fn line_col(input: &str, pos: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, c) in input.char_indices() {
        if i == pos {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

pub fn lex_with_positions(input: &str) -> Vec<SpannedToken> {
    let mut lexer = Token::lexer(input);
    let mut result = vec![];

    while let Some(res) = lexer.next() {
        match res {
            Ok(token) => {
                let span = lexer.span();
                let slice = input[span.clone()].to_string();
                let decoded_slice = if matches!(token, Token::StringLiteral) {
                    // Usunięcie pierwszego i ostatniego znaku
                    let trimmed_slice = slice[1..slice.len() - 1].to_string();
                    decode_escape_sequences(&trimmed_slice)
                } else {
                    slice
                };
                let (line, column) = line_col(input, span.start);

                result.push(SpannedToken {
                    token,
                    slice: decoded_slice,
                    line,
                    column,
                    span,
                });
            }
            Err(_) => {
                let span = lexer.span();
                let (line, column) = line_col(input, span.start);
                eprintln!("Unexpected token at line {}, column {}", line, column);
            }
        }
    }

    let (line, column) = line_col(input, input.len());
    let span = lexer.span();
    result.push(SpannedToken {
        token: Token::Eof,
        slice: "".to_string(),
        line,
        column,
        span: span,
    });

    result
}

fn decode_escape_sequences(input: &str) -> String {
    input.replace("\\\"", "\"")
         .replace("\\\\", "\\")
         .replace("\\n", "\n")
         .replace("\\t", "\t")
         .replace("\\r", "\r")
}