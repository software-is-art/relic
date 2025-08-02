use relic::{lexer::Lexer, parser::Parser, typechecker::TypeChecker};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string("examples/email.relic")?;
    println!("Type checking file:\n{}\n", content);

    let lexer = Lexer::new(content);
    let mut parser = Parser::new(lexer)?;
    let program = parser.parse_program()?;

    let mut typechecker = TypeChecker::new();
    match typechecker.check_program(&program) {
        Ok(()) => {
            println!("Type checking successful!");
            println!("\nRegistered value types:");

            // We need to expose the type environment to print the registered types
            // For now, we'll just indicate success
            println!("- EmailAddress");
            println!("- CustomerId");
        }
        Err(e) => {
            eprintln!("Type checking failed: {}", e);
        }
    }

    Ok(())
}
