use relic::{compiler::Compiler, lexer::Lexer, parser::Parser, typechecker::TypeChecker};
use std::io::{self, Write};

struct Repl {
    compiler: Compiler,
    typechecker: TypeChecker,
}

impl Repl {
    fn new() -> Self {
        Self {
            compiler: Compiler::new(),
            typechecker: TypeChecker::new(),
        }
    }

    fn process_declaration(&mut self, input: &str) -> relic::Result<String> {
        // Parse
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer)?;
        let program = parser.parse_program()?;

        // Type check
        self.typechecker.check_program(&program)?;

        // Compile
        self.compiler.compile_program(&program)?;

        let mut result = String::new();
        for decl in &program.declarations {
            match decl {
                relic::ast::Declaration::Value(v) => {
                    result.push_str(&format!("Defined value type: {}\n", v.name));
                }
            }
        }

        Ok(result)
    }

    fn process_construction(&self, input: &str) -> relic::Result<String> {
        // Simple pattern matching for value construction
        // Format: TypeName(value)
        if let Some(paren_pos) = input.find('(') {
            if input.ends_with(')') {
                let type_name = &input[..paren_pos];
                let value_str = &input[paren_pos + 1..input.len() - 1];

                // Try to parse the value
                let value: Box<dyn std::any::Any + Send + Sync> =
                    if let Ok(n) = value_str.parse::<i64>() {
                        Box::new(n)
                    } else {
                        // Remove quotes if present
                        let value_str = value_str.trim_matches('"');
                        Box::new(value_str.to_string())
                    };

                match self.compiler.get_registry().construct(type_name, value) {
                    Ok(_) => Ok(format!("✓ Created {} successfully", type_name)),
                    Err(e) => Ok(format!("✗ Failed to create {}: {}", type_name, e)),
                }
            } else {
                Err(relic::Error::Parser(relic::error::ParserError {
                    message: "Invalid construction syntax".to_string(),
                    line: 1,
                    column: 1,
                }))
            }
        } else {
            Err(relic::Error::Parser(relic::error::ParserError {
                message: "Expected value construction: TypeName(value)".to_string(),
                line: 1,
                column: 1,
            }))
        }
    }
}

fn main() {
    println!("Relic Language REPL v0.1.0");
    println!("Type 'exit' to quit");
    println!("Type 'help' for commands\n");

    let mut repl = Repl::new();

    loop {
        print!("relic> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();

        match input {
            "exit" => break,
            "help" => {
                println!("Commands:");
                println!("  value TypeName(param: Type) {{ ... }} - Define a value type");
                println!("  TypeName(value)                       - Create a value instance");
                println!("  help                                  - Show this help");
                println!("  exit                                  - Exit the REPL");
            }
            "" => continue,
            _ => {
                // Determine if this is a declaration or construction
                let result = if input.starts_with("value ") {
                    repl.process_declaration(input)
                } else if input.contains('(') && input.contains(')') {
                    repl.process_construction(input)
                } else {
                    Err(relic::Error::Parser(relic::error::ParserError {
                        message: "Invalid syntax. Type 'help' for commands.".to_string(),
                        line: 1,
                        column: 1,
                    }))
                };

                match result {
                    Ok(output) => print!("{}", output),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
    }
}
