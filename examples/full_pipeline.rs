use relic::{compiler::Compiler, lexer::Lexer, parser::Parser, typechecker::TypeChecker};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the source file
    let content = fs::read_to_string("examples/email.relic")?;
    println!("=== Source Code ===");
    println!("{}", content);

    // Parse
    println!("\n=== Parsing ===");
    let lexer = Lexer::new(content);
    let mut parser = Parser::new(lexer)?;
    let program = parser.parse_program()?;
    println!("✓ Parsing successful");

    // Type check
    println!("\n=== Type Checking ===");
    let mut typechecker = TypeChecker::new();
    typechecker.check_program(&program)?;
    println!("✓ Type checking successful");

    // Compile
    println!("\n=== Compiling ===");
    let mut compiler = Compiler::new();
    compiler.compile_program(&program)?;
    let registry = compiler.into_registry();
    println!("✓ Compilation successful");

    // Test value construction
    println!("\n=== Testing Value Constructors ===");

    // Test EmailAddress
    println!("\nTesting EmailAddress:");
    test_email_construction(&registry, "test@example.com")?;
    test_email_construction(&registry, "TEST@EXAMPLE.COM")?;
    test_email_construction(&registry, "invalid")?;
    test_email_construction(&registry, "@")?;

    // Test CustomerId
    println!("\nTesting CustomerId:");
    test_customer_id(&registry, 123)?;
    test_customer_id(&registry, 1)?;
    test_customer_id(&registry, 0)?;
    test_customer_id(&registry, -5)?;

    Ok(())
}

fn test_email_construction(
    registry: &relic::value::ValueRegistry,
    input: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    print!("  '{}' -> ", input);
    match registry.construct("EmailAddress", Box::new(input.to_string())) {
        Ok(_) => println!("✓ Valid"),
        Err(e) => println!("✗ Invalid: {}", e),
    }
    Ok(())
}

fn test_customer_id(
    registry: &relic::value::ValueRegistry,
    input: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    print!("  {} -> ", input);
    match registry.construct("CustomerId", Box::new(input)) {
        Ok(_) => println!("✓ Valid"),
        Err(e) => println!("✗ Invalid: {}", e),
    }
    Ok(())
}
