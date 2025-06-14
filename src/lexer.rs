use logos::{Logos, Span};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum LexError {
    #[error("[L001] Unterminated string literal")]
    UnterminatedString { span: (usize, usize) },
    
    #[error("[L002] Invalid character")]
    InvalidCharacter { char: char, span: (usize, usize) },
    
    #[error("[L003] Invalid escape sequence")]
    InvalidEscapeSequence { sequence: String, span: (usize, usize) },
    
    #[error("[L004] Empty string literal")]
    EmptyStringLiteral { span: (usize, usize) },
}

impl LexError {
    pub fn span(&self) -> (usize, usize) {
        match self {
            Self::UnterminatedString { span } => *span,
            Self::InvalidCharacter { span, .. } => *span,
            Self::InvalidEscapeSequence { span, .. } => *span,
            Self::EmptyStringLiteral { span } => *span,
        }
    }
    
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::UnterminatedString { .. } => "L001",
            Self::InvalidCharacter { .. } => "L002",
            Self::InvalidEscapeSequence { .. } => "L003",
            Self::EmptyStringLiteral { .. } => "L004",
        }
    }
}

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
pub enum Token {
    #[regex(r"@\[", priority = 10)]
    StyleBlockStart,

    #[regex(r"&\[", priority = 10)]
    ScriptBlockStart,

    #[regex(r"%[a-zA-Z_][a-zA-Z0-9_]*", priority = 5)]
    SpecialFunction,

    #[regex(r"[a-zA-Z_$][a-zA-Z0-9_$]*", priority = 1)]
    Ident,

    #[regex(r"[a-zA-Z]+(-[a-zA-Z]+)+", priority = 2)]
    CodeBlock,

    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,

    #[regex(r"[0-9]+")]
    Number,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token("=")]
    Equals,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("@")]
    At,
    #[token("*")]
    Asterisk,
    #[token("#")]
    Hash,
    #[token(">")]
    Greater,
    #[token("+")]
    Plus,
    #[token("~")]
    Tilde,
    #[token("'")]
    SingleQuote,
    #[token("-")]
    Minus,

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

#[derive(Debug)]
struct LineMap {
    offsets: Vec<usize>, // byte offset at the start of each line
}

impl LineMap {
    fn new(input: &str) -> Self {
        let mut offsets = vec![0];
        for (i, b) in input.bytes().enumerate() {
            if b == b'\n' {
                offsets.push(i + 1);
            }
        }
        Self { offsets }
    }

    fn line_col(&self, pos: usize) -> (usize, usize) {
        match self.offsets.binary_search(&pos) {
            Ok(line) => (line + 1, 1),
            Err(line) => {
                let line_start = self.offsets.get(line.wrapping_sub(1)).copied().unwrap_or(0);
                (line + 1, pos - line_start + 1)
            }
        }
    }
}

pub fn lex_with_positions(input: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut lexer = Token::lexer(input);
    let mut result = vec![];
    let line_map = LineMap::new(input);

    while let Some(res) = lexer.next() {
        match res {
            Ok(token) => {
                let span = lexer.span();
                let slice = input[span.clone()].to_string();
                
                
  
                let decoded_slice = if matches!(token, Token::StringLiteral) {
                    validate_string_literal(&slice, span.clone())?;
                    
         
                    let inner = &slice[1..slice.len() - 1];
                    let decoded_inner = decode_escape_sequences(inner, span.clone())?;
                    format!("\"{}\"", decoded_inner)
                } else {
                    validate_token(&token, &slice, span.clone())?;
                    slice
                };
                
                let (line, column) = line_map.line_col(span.start);

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
                let invalid_char = input.chars().nth(span.start).unwrap_or('\0');
                return Err(LexError::InvalidCharacter { 
                    char: invalid_char, 
                    span: (span.start, span.end) 
                });
            }
        }
    }

    let (line, column) = line_map.line_col(input.len());
    let span = lexer.span();
    result.push(SpannedToken {
        token: Token::Eof,
        slice: "".to_string(),
        line,
        column,
        span: span,
    });

    Ok(result)
}

fn decode_escape_sequences(input: &str, span: Span) -> Result<String, LexError> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some(other) => {
                    return Err(LexError::InvalidEscapeSequence {
                        sequence: format!("\\{}", other),
                        span: (span.start, span.end),
                    });
                }
                None => {
                    return Err(LexError::InvalidEscapeSequence {
                        sequence: "\\".to_string(),
                        span: (span.start, span.end),
                    });
                }
            }
        } else {
            result.push(ch);
        }
    }
    
    Ok(result)
}

fn validate_string_literal(slice: &str, span: Span) -> Result<(), LexError> {

    if !slice.starts_with('"') || !slice.ends_with('"') {
        return Err(LexError::UnterminatedString {
            span: (span.start, span.end),
        });
    }

    if slice.len() < 2 {
        return Err(LexError::EmptyStringLiteral {
            span: (span.start, span.end),
        });
    }

    let inner = &slice[1..slice.len()-1];
    let mut chars = inner.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if chars.peek().is_none() {
                return Err(LexError::UnterminatedString {
                    span: (span.start, span.end),
                });
            }
        }
    }
    
    Ok(())
}

fn validate_token(token: &Token, slice: &str, span: Span) -> Result<(), LexError> {
    match token {
        Token::Ident => {
            if slice.is_empty() {
                return Err(LexError::InvalidCharacter {
                    char: '\0',
                    span: (span.start, span.end),
                });
            }
        }
        Token::Number => {

            if slice.is_empty() || !slice.chars().all(|c| c.is_ascii_digit()) {
                return Err(LexError::InvalidCharacter {
                    char: slice.chars().next().unwrap_or('\0'),
                    span: (span.start, span.end),
                });
            }
        }
        _ => {} 
    }
    
    Ok(())
}