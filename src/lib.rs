pub mod ast;
pub mod compiler;
pub mod error;
pub mod evaluator;
pub mod lexer;
pub mod parser;
pub mod typechecker;
pub mod types;
pub mod value;

#[cfg(test)]
mod test_value_equality;

pub use error::{Error, Result};
