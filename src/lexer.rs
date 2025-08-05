use crate::error::{Error, LexerError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Value,
    Fn,
    Method,
    Validate,
    Normalize,
    Unique,
    True,
    False,
    Contains,
    Let,
    In,
    Match,
    Where,
    Arrow,        // => for match arms
    ReturnArrow,  // -> for function return types

    // Identifiers and literals
    Identifier(String),
    String(String),
    Integer(i64),

    // Operators
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Colon,
    Dot,
    Comma,

    // Comparison operators
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // Assignment
    Assign,

    // Logical operators
    And,
    Or,
    Not,

    // Arithmetic operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,  // modulo operator

    // Pipeline operator
    Pipeline,

    // Special
    Eof,
}

pub struct Lexer {
    input: String,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            current_char: None,
            line: 1,
            column: 0,
        };
        lexer.current_char = lexer.input.chars().next();
        lexer
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        match self.current_char {
            None => Ok(Token::Eof),
            Some(ch) => match ch {
                '(' => {
                    self.advance();
                    Ok(Token::LeftParen)
                }
                ')' => {
                    self.advance();
                    Ok(Token::RightParen)
                }
                '{' => {
                    self.advance();
                    Ok(Token::LeftBrace)
                }
                '}' => {
                    self.advance();
                    Ok(Token::RightBrace)
                }
                ':' => {
                    self.advance();
                    Ok(Token::Colon)
                }
                '.' => {
                    self.advance();
                    Ok(Token::Dot)
                }
                ',' => {
                    self.advance();
                    Ok(Token::Comma)
                }
                '+' => {
                    self.advance();
                    Ok(Token::Plus)
                }
                '-' => {
                    self.advance();
                    if self.current_char == Some('>') {
                        self.advance();
                        Ok(Token::ReturnArrow)
                    } else {
                        Ok(Token::Minus)
                    }
                }
                '*' => {
                    self.advance();
                    Ok(Token::Star)
                }
                '/' => {
                    self.advance();
                    if self.current_char == Some('/') {
                        // Line comment - skip to end of line
                        self.advance();
                        while self.current_char.is_some() && self.current_char != Some('\n') {
                            self.advance();
                        }
                        self.next_token()
                    } else if self.current_char == Some('*') {
                        // Multi-line comment - skip until */
                        self.advance();
                        self.skip_multiline_comment()?;
                        self.next_token()
                    } else {
                        Ok(Token::Slash)
                    }
                }
                '%' => {
                    self.advance();
                    Ok(Token::Percent)
                }
                '=' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Ok(Token::Equal)
                    } else if self.current_char == Some('>') {
                        self.advance();
                        Ok(Token::Arrow)
                    } else {
                        Ok(Token::Assign)
                    }
                }
                '!' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Ok(Token::NotEqual)
                    } else {
                        Ok(Token::Not)
                    }
                }
                '<' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Ok(Token::LessEqual)
                    } else {
                        Ok(Token::Less)
                    }
                }
                '>' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Ok(Token::GreaterEqual)
                    } else {
                        Ok(Token::Greater)
                    }
                }
                '&' => {
                    self.advance();
                    if self.current_char == Some('&') {
                        self.advance();
                        Ok(Token::And)
                    } else {
                        Err(Error::Lexer(LexerError {
                            message: "Unexpected character '&', did you mean '&&'?".to_string(),
                            line: self.line,
                            column: self.column,
                        }))
                    }
                }
                '|' => {
                    self.advance();
                    if self.current_char == Some('|') {
                        self.advance();
                        Ok(Token::Or)
                    } else if self.current_char == Some('>') {
                        self.advance();
                        Ok(Token::Pipeline)
                    } else {
                        Err(Error::Lexer(LexerError {
                            message: "Unexpected character '|', did you mean '||' or '|>'?"
                                .to_string(),
                            line: self.line,
                            column: self.column,
                        }))
                    }
                }
                '"' => self.read_string(),
                _ if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
                _ if ch.is_numeric() => self.read_number(),
                _ => Err(Error::Lexer(LexerError {
                    message: format!("Unexpected character '{}'", ch),
                    line: self.line,
                    column: self.column,
                })),
            },
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.column += 1;

        if self.position >= self.input.len() {
            self.current_char = None;
        } else {
            self.current_char = self.input.chars().nth(self.position);
            if self.current_char == Some('\n') {
                self.line += 1;
                self.column = 0;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_multiline_comment(&mut self) -> Result<()> {
        let mut depth = 1; // Support nested comments
        
        while depth > 0 && self.current_char.is_some() {
            if self.current_char == Some('*') {
                self.advance();
                if self.current_char == Some('/') {
                    self.advance();
                    depth -= 1;
                }
            } else if self.current_char == Some('/') {
                self.advance();
                if self.current_char == Some('*') {
                    self.advance();
                    depth += 1;
                }
            } else {
                self.advance();
            }
        }
        
        if depth > 0 {
            Err(Error::Lexer(LexerError {
                message: "Unterminated multi-line comment".to_string(),
                line: self.line,
                column: self.column,
            }))
        } else {
            Ok(())
        }
    }

    fn read_identifier(&mut self) -> Result<Token> {
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let identifier = &self.input[start..self.position];

        let token = match identifier {
            "value" => Token::Value,
            "fn" => Token::Fn,
            "method" => Token::Method,
            "validate" => Token::Validate,
            "normalize" => Token::Normalize,
            "unique" => Token::Unique,
            "true" => Token::True,
            "false" => Token::False,
            "contains" => Token::Contains,
            "let" => Token::Let,
            "in" => Token::In,
            "match" => Token::Match,
            "where" => Token::Where,
            _ => Token::Identifier(identifier.to_string()),
        };

        Ok(token)
    }

    fn read_number(&mut self) -> Result<Token> {
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch.is_numeric() {
                self.advance();
            } else {
                break;
            }
        }

        let number_str = &self.input[start..self.position];
        let number = number_str.parse::<i64>().map_err(|_| {
            Error::Lexer(LexerError {
                message: format!("Invalid number: {}", number_str),
                line: self.line,
                column: self.column - number_str.len(),
            })
        })?;

        Ok(Token::Integer(number))
    }

    fn read_string(&mut self) -> Result<Token> {
        self.advance(); // Skip opening quote
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch == '"' {
                let string = self.input[start..self.position].to_string();
                self.advance(); // Skip closing quote
                return Ok(Token::String(string));
            } else if ch == '\\' {
                self.advance();
                if self.current_char.is_none() {
                    return Err(Error::Lexer(LexerError {
                        message: "Unexpected end of string".to_string(),
                        line: self.line,
                        column: self.column,
                    }));
                }
                self.advance();
            } else {
                self.advance();
            }
        }

        Err(Error::Lexer(LexerError {
            message: "Unclosed string literal".to_string(),
            line: self.line,
            column: self.column,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_operator() {
        let mut lexer = Lexer::new("x |> f".to_string());

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("x".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Pipeline);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("f".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_pipeline_vs_or() {
        let mut lexer = Lexer::new("a || b |> c".to_string());

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("a".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Or);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("b".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Pipeline);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("c".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_let_keywords() {
        let mut lexer = Lexer::new("let x = 5 in x + 1".to_string());
        assert_eq!(lexer.next_token().unwrap(), Token::Let);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("x".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Assign);
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(5));
        assert_eq!(lexer.next_token().unwrap(), Token::In);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("x".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Plus);
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(1));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_match_keywords() {
        let mut lexer = Lexer::new("match x { Status(c) => c > 0 }".to_string());
        assert_eq!(lexer.next_token().unwrap(), Token::Match);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("x".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::LeftBrace);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("Status".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("c".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
        assert_eq!(lexer.next_token().unwrap(), Token::Arrow);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("c".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Greater);
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(0));
        assert_eq!(lexer.next_token().unwrap(), Token::RightBrace);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }
}
