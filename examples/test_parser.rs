use relic::{lexer::Lexer, parser::Parser};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("examples/email.relic")?;
    println!("Parsing file:\n{}\n", content);

    let lexer = Lexer::new(content);
    let mut parser = Parser::new(lexer)?;
    let program = parser.parse_program()?;

    println!("Parsed AST:");
    for (i, decl) in program.declarations.iter().enumerate() {
        println!("Declaration {}: {:?}", i + 1, decl);
    }

    Ok(())
}
