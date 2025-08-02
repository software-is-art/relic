use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Lexer(LexerError),
    Parser(ParserError),
    Type(TypeError),
    Validation(ValidationError),
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub value_type: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lexer(e) => write!(f, "Lexer error at {}:{}: {}", e.line, e.column, e.message),
            Error::Parser(e) => write!(f, "Parser error at {}:{}: {}", e.line, e.column, e.message),
            Error::Type(e) => write!(f, "Type error: {}", e.message),
            Error::Validation(e) => {
                write!(f, "Validation error in {}: {}", e.value_type, e.message)
            }
        }
    }
}

impl std::error::Error for Error {}
