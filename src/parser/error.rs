
use thiserror::Error;
use crate::lexer::Token;

#[allow(dead_code)]
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("[E001] Unexpected token")]
    UnexpectedToken {
        found: Token,
        span: (usize, usize),
        expected: Option<String>,
    },

    #[error("[E002] Expected token")]
    ExpectedToken {
        expected: String,
        found: Token,
        span: (usize, usize),
    },

    #[error("[E003] Unexpected end of input")]
    UnexpectedEof { 
        expected: String,
        span: (usize, usize),
    },

    #[error("[E004] Missing closing symbol")]
    UnclosedBlock {
        symbol: char,
        start_span: (usize, usize),
        span: (usize, usize),
    },

    #[error("[E005] Invalid CSS property syntax")]
    InvalidCssProperty { 
        span: (usize, usize),
    },

    #[error("[E006] Invalid attribute syntax")]
    InvalidAttribute { 
        span: (usize, usize),
    },

    #[error("[E007] Component cannot have both definition and call arguments")]
    AmbiguousComponent {
        name: String,
        span: (usize, usize),
    },

    #[error("[E008] Special function requires arguments")]
    MissingSpecialFunctionArgs {
        name: String,
        span: (usize, usize),
    },

    #[error("[E009] Text element requires string literal")]
    InvalidTextElement { 
        span: (usize, usize),
    },

    #[error("[E010] Duplicate element found")]
    DuplicateElement {
        element_type: String,
        span: (usize, usize),
    },

    #[error("[E011] Unclosed string literal")]
    UnclosedString {
        span: (usize, usize),
    },

    #[error("[E012] Unmatched closing bracket")]
    UnmatchedClosingBracket {
        bracket_type: char,
        span: (usize, usize),
    },

    #[error("[E013] Too many closing brackets")]
    TooManyClosingBrackets {
        bracket_type: char,
        span: (usize, usize),
    },

    #[error("[E014] Mismatched bracket types")]
    MismatchedBrackets {
        expected: char,
        found: char,
        start_span: (usize, usize),
        span: (usize, usize),
    },

    #[error("[E015] Empty block not allowed")]
    EmptyBlock {
        block_type: String,
        span: (usize, usize),
    },

    #[error("[E016] Invalid token in context")]
    InvalidTokenInContext {
        token: Token,
        context: String,
        span: (usize, usize),
    },

    #[error("[E017] Missing semicolon")]
    MissingSemicolon {
        span: (usize, usize),
    },

    #[error("[E018] Invalid escape sequence")]
    InvalidEscapeSequence {
        sequence: String,
        span: (usize, usize),
    },

    #[error("[E019] Nested blocks not allowed")]
    NestedBlocksNotAllowed {
        block_type: String,
        span: (usize, usize),
    },

    #[error("[E020] Invalid identifier")]
    InvalidIdentifier {
        name: String,
        span: (usize, usize),
    },
}

#[allow(dead_code)]
impl ParseError {
    pub fn unexpected_token(found: Token, span: (usize, usize), expected: Option<String>) -> Self {
        Self::UnexpectedToken { found, span, expected }
    }

    pub fn expected_token(expected: String, found: Token, span: (usize, usize)) -> Self {
        Self::ExpectedToken {
            expected,
            found,
            span,
        }
    }

    pub fn unexpected_eof(expected: &str, span: (usize, usize)) -> Self {
        Self::UnexpectedEof {
            expected: expected.to_string(),
            span,
        }
    }

    pub fn unclosed_block(symbol: char, start_span: (usize, usize), span: (usize, usize)) -> Self {
        Self::UnclosedBlock {
            symbol,
            start_span,
            span,
        }
    }

    pub fn unmatched_closing_bracket(bracket_type: char, span: (usize, usize)) -> Self {
        Self::UnmatchedClosingBracket { bracket_type, span }
    }

    pub fn too_many_closing_brackets(bracket_type: char, span: (usize, usize)) -> Self {
        Self::TooManyClosingBrackets { bracket_type, span }
    }

    pub fn mismatched_brackets(expected: char, found: char, start_span: (usize, usize), span: (usize, usize)) -> Self {
        Self::MismatchedBrackets { expected, found, start_span, span }
    }

    pub fn empty_block(block_type: &str, span: (usize, usize)) -> Self {
        Self::EmptyBlock { block_type: block_type.to_string(), span }
    }

    pub fn invalid_token_in_context(token: Token, context: &str, span: (usize, usize)) -> Self {
        Self::InvalidTokenInContext { token, context: context.to_string(), span }
    }

    pub fn missing_semicolon(span: (usize, usize)) -> Self {
        Self::MissingSemicolon { span }
    }

    pub fn invalid_escape_sequence(sequence: String, span: (usize, usize)) -> Self {
        Self::InvalidEscapeSequence { sequence, span }
    }

    pub fn nested_blocks_not_allowed(block_type: &str, span: (usize, usize)) -> Self {
        Self::NestedBlocksNotAllowed { block_type: block_type.to_string(), span }
    }

    pub fn invalid_identifier(name: String, span: (usize, usize)) -> Self {
        Self::InvalidIdentifier { name, span }
    }

    pub fn invalid_css_property(span: (usize, usize)) -> Self {
        Self::InvalidCssProperty { span }
    }

    pub fn invalid_attribute(span: (usize, usize)) -> Self {
        Self::InvalidAttribute { span }
    }

    pub fn ambiguous_component(name: String, span: (usize, usize)) -> Self {
        Self::AmbiguousComponent { name, span }
    }

    pub fn missing_special_function_args(name: String, span: (usize, usize)) -> Self {
        Self::MissingSpecialFunctionArgs { name, span }
    }

    pub fn invalid_text_element(span: (usize, usize)) -> Self {
        Self::InvalidTextElement { span }
    }

    pub fn duplicate_element(element_type: &str, span: (usize, usize)) -> Self {
        Self::DuplicateElement {
            element_type: element_type.to_string(),
            span,
        }
    }

    pub fn unclosed_string(span: (usize, usize)) -> Self {
        Self::UnclosedString { span }
    }

    pub fn span(&self) -> (usize, usize) {
        match self {
            Self::UnexpectedToken { span, .. } => *span,
            Self::ExpectedToken { span, .. } => *span,
            Self::UnexpectedEof { span, .. } => *span,
            Self::UnclosedBlock { span, .. } => *span,
            Self::InvalidCssProperty { span } => *span,
            Self::InvalidAttribute { span } => *span,
            Self::AmbiguousComponent { span, .. } => *span,
            Self::MissingSpecialFunctionArgs { span, .. } => *span,
            Self::InvalidTextElement { span } => *span,
            Self::DuplicateElement { span, .. } => *span,
            Self::UnclosedString { span } => *span,
            Self::UnmatchedClosingBracket { span, .. } => *span,
            Self::TooManyClosingBrackets { span, .. } => *span,
            Self::MismatchedBrackets { span, .. } => *span,
            Self::EmptyBlock { span, .. } => *span,
            Self::InvalidTokenInContext { span, .. } => *span,
            Self::MissingSemicolon { span } => *span,
            Self::InvalidEscapeSequence { span, .. } => *span,
            Self::NestedBlocksNotAllowed { span, .. } => *span,
            Self::InvalidIdentifier { span, .. } => *span,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::UnexpectedToken { .. } => "E001",
            Self::ExpectedToken { .. } => "E002",
            Self::UnexpectedEof { .. } => "E003",
            Self::UnclosedBlock { .. } => "E004",
            Self::InvalidCssProperty { .. } => "E005",
            Self::InvalidAttribute { .. } => "E006",
            Self::AmbiguousComponent { .. } => "E007",
            Self::MissingSpecialFunctionArgs { .. } => "E008",
            Self::InvalidTextElement { .. } => "E009",
            Self::DuplicateElement { .. } => "E010",
            Self::UnclosedString { .. } => "E011",
            Self::UnmatchedClosingBracket { .. } => "E012",
            Self::TooManyClosingBrackets { .. } => "E013",
            Self::MismatchedBrackets { .. } => "E014",
            Self::EmptyBlock { .. } => "E015",
            Self::InvalidTokenInContext { .. } => "E016",
            Self::MissingSemicolon { .. } => "E017",
            Self::InvalidEscapeSequence { .. } => "E018",
            Self::NestedBlocksNotAllowed { .. } => "E019",
            Self::InvalidIdentifier { .. } => "E020",
        }
    }
}
