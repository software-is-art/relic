# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Relic is an experimental programming language written in Rust that implements value-oriented programming with functional-relational foundations. The language focuses on the "parse don't validate" paradigm, where value objects serve as immutable witnesses of valid construction.

## Development Commands

### Build and Run
- `cargo build` - Build the project
- `cargo run` - Run the REPL
- `cargo run <filename>` - Run a Relic source file (e.g., `cargo run examples/let_bindings.relic`)
- `cargo test` - Run all tests
- `cargo fmt` - Format code
- `cargo clippy` - Run linter
- `cargo build --release` - Build optimized release version

**Important:** Do NOT use `cargo run < file.relic` as this will pipe the file to stdin and cause the REPL to process each line separately. Always pass the filename as a command-line argument.

### Testing Specific Components
- `cargo test lexer` - Test lexer only
- `cargo test parser` - Test parser only
- `cargo test typechecker` - Test type checker only
- `cargo test -- --nocapture` - Run tests with println! output visible

## Architecture Overview

The compiler follows a traditional pipeline architecture:

1. **Lexer** (`src/lexer.rs`) - Tokenizes source code
2. **Parser** (`src/parser.rs`) - Constructs AST from tokens
3. **Type Checker** (`src/typechecker.rs`) - Validates types and constraints
4. **Compiler** (`src/compiler.rs`) - Generates value constructors
5. **Runtime** (`src/value.rs`) - Executes value objects

Key architectural decisions:
- Value declarations define immutable types with validation predicates
- Constructor functions automatically validate inputs according to predicates
- The type system enforces parse-don't-validate semantics at compile time
- Multiple dispatch is planned as the primary control flow mechanism

## Core Language Features

### Value Declarations
Values are the fundamental building blocks, defined with validation predicates:
```relic
value EmailAddress(raw: String) {
    validate: raw contains "@" && raw.length > 3
    normalize: raw.toLowerCase()
}
```

### Current Implementation Status
- Phase 1-2 (100% complete): Basic value types, validation, REPL, functions, UFC, pattern matching
- Phase 3 (~40% complete): Multiple dispatch system - method parsing implemented, dispatch mechanism in progress
- Phase 4+ planned: Functional-relational core, query system, optimizations

## Important Design Documents
- `DESIGN.md` - Theoretical foundations and design principles
- `IMPLEMENTATION_PLAN.md` - Detailed 10-phase development roadmap
- `README.md` - Usage examples and quick start guide

## Code Style Guidelines
- Follow Rust conventions and idioms
- Use descriptive names for value types and predicates
- Keep validation logic simple and composable
- Error messages should be helpful and specific to the validation failure