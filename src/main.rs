use relic::{compiler::Compiler, lexer::{Lexer, Token}, parser::Parser, typechecker::TypeChecker};
use std::{
    env,
    fs,
    io::{self, Write},
};

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
                relic::ast::Declaration::Function(f) => {
                    result.push_str(&format!("Defined function: {}\n", f.name));
                }
                relic::ast::Declaration::Method(m) => {
                    result.push_str(&format!("Defined method: {}\n", m.name));
                }
            }
        }

        Ok(result)
    }

    fn process_expression(&self, input: &str) -> relic::Result<String> {
        // Parse the expression
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer)?;
        
        // Try to parse as a single expression
        let expr = parser.parse_expression()?;
        
        // Ensure we consumed all tokens
        if parser.current_token != Token::Eof {
            return Err(relic::Error::Parser(relic::error::ParserError {
                message: format!("Unexpected token after expression: {:?}", parser.current_token),
                line: 1,
                column: 1,
            }));
        }
        
        // Type check the expression
        let expr_type = self.typechecker.check_expression(&expr)?;
        
        // Evaluate the expression
        let result = self.compiler.evaluate_expression(&expr)?;
        
        Ok(format!("→ {} : {:?}", result, expr_type))
    }

    fn process_construction(&self, input: &str) -> relic::Result<String> {
        // Simple pattern matching for value construction or function calls
        // Format: TypeName(value) or functionName(args)
        if let Some(paren_pos) = input.find('(') {
            if input.ends_with(')') {
                let name = &input[..paren_pos];
                let args_str = &input[paren_pos + 1..input.len() - 1];

                // Check if it's a function or method call - use the expression evaluator instead
                if self.compiler.get_registry().get_function(name).is_some() 
                    || self.compiler.get_registry().get_methods(name).is_some() {
                    return self.process_expression(input);
                }

                // Otherwise, it's a value construction
                // Try to parse the value
                let value: Box<dyn std::any::Any + Send + Sync> =
                    if let Ok(n) = args_str.parse::<i64>() {
                        Box::new(n)
                    } else {
                        // Remove quotes if present
                        let value_str = args_str.trim_matches('"');
                        Box::new(value_str.to_string())
                    };

                match self.compiler.get_registry().construct(name, value) {
                    Ok(_) => Ok(format!("✓ Created {} successfully", name)),
                    Err(e) => Ok(format!("✗ Failed to create {}: {}", name, e)),
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
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // File mode
        let filename = &args[1];
        match fs::read_to_string(filename) {
            Ok(contents) => {
                let mut repl = Repl::new();
                println!("Processing file: {}", filename);
                
                // Process the entire file as a program
                match repl.process_declaration(&contents) {
                    Ok(output) => {
                        println!("{}", output);
                        println!("\nFile processed successfully.");
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading file '{}': {}", filename, e);
                std::process::exit(1);
            }
        }
    } else {
        // REPL mode
        println!("Relic Language REPL v0.1.0");
        println!("Type 'exit' to quit");
        println!("Type 'help' for commands\n");

        let mut repl = Repl::new();

        loop {
        print!("relic> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF reached
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }

        let input = input.trim();

        match input {
            "exit" => break,
            "help" => {
                println!("Commands:");
                println!("  value TypeName(param: Type) {{ ... }}     - Define a value type");
                println!("  fn name(params) -> Type {{ ... }}         - Define a function");
                println!("  method name(params) -> Type {{ ... }}     - Define a method");
                println!("  TypeName(value)                           - Create a value instance");
                println!("  functionName(args)                        - Call a function");
                println!("  help                                      - Show this help");
                println!("  exit                                      - Exit the REPL");
            }
            "" => continue,
            _ => {
                // Determine if this is a declaration or expression
                let result = if input.starts_with("value ") || input.starts_with("fn ") || input.starts_with("method ") {
                    repl.process_declaration(input)
                } else {
                    // Try to parse as an expression first
                    match repl.process_expression(input) {
                        Ok(result) => Ok(result),
                        Err(_) => {
                            // If that fails and it looks like a construction, try that
                            if input.contains('(') && input.contains(')') {
                                repl.process_construction(input)
                            } else {
                                Err(relic::Error::Parser(relic::error::ParserError {
                                    message: "Invalid syntax. Type 'help' for commands.".to_string(),
                                    line: 1,
                                    column: 1,
                                }))
                            }
                        }
                    }
                };

                match result {
                    Ok(output) => println!("{}", output),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
    }
    }
}
