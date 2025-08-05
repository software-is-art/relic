# CLAUDE.md

This file provides comprehensive guidance to Claude Code (claude.ai/code) when working with the Relic programming language codebase.

## Project Overview

Relic is an experimental programming language that demonstrates how **parse-don't-validate** principles, **functional-relational programming**, and **multiple dispatch** can create a more elegant and correct approach to software development.

### Core Philosophy

1. **Parse Don't Validate**: Value objects serve as immutable witnesses of valid construction
2. **Unified Function Syntax**: All functions can have multiple dispatch (inspired by Julia)
3. **Functional-Relational Architecture**: Strict separation of essential state, logic, and effects
4. **Type-Level Relationships**: Constraints and relationships encoded in the type system

## Current Implementation Status

### Phase 1-2: Core Language ✅ Complete
- Value type declarations with validation predicates
- Expression evaluation with let-bindings
- Function definitions and evaluation
- Uniform Function Call (UFC) syntax
- Pattern matching on value types
- Pipeline operator (`|>`)
- Interactive REPL with file support

### Phase 3: Multiple Dispatch ~80% Complete
- ✅ Multiple dispatch with type-based precedence
- ✅ Compile-time ambiguity detection
- ✅ UFC integration with dispatch
- ✅ Unified function syntax design (all functions can dispatch)
- 🚧 Parameter guards in dispatch
- 🚧 Compile-time specialization

### Phase 4+: Future Work
- Functional-relational core with query operations
- Advanced type system features (row types, refinement types)
- Sea of nodes compiler architecture
- Effect system integration

## Development Commands

### Build and Test
```bash
cargo build              # Build the project
cargo test              # Run all tests
cargo test lexer        # Test specific component
cargo fmt               # Format code
cargo clippy            # Run linter
```

### Running Relic
```bash
cargo run                           # Start REPL
cargo run examples/values.relic     # Run a file
cargo build --release               # Build optimized version
```

**Important:** Always pass filenames as arguments, never use `cargo run < file.relic`

## Architecture

### Compiler Pipeline
1. **Lexer** (`src/lexer.rs`) - Tokenizes source code
2. **Parser** (`src/parser.rs`) - Builds AST using recursive descent
3. **Type Checker** (`src/typechecker.rs`) - Validates types and collects functions
4. **Evaluator** (`src/evaluator.rs`) - Interprets expressions with multiple dispatch
5. **Compiler** (`src/compiler.rs`) - Generates value constructors
6. **Value Registry** (`src/value.rs`) - Stores types, functions, and methods

### Key Design Decisions

#### Unified Function Syntax
- No distinction between `fn` and `method` (though `method` is temporarily supported)
- All functions can have multiple implementations
- Compiler automatically handles dispatch based on number of implementations
- Single implementation = direct call, multiple = dispatch table

#### Multiple Dispatch
- Functions dispatch on ALL argument types, not just the first
- Type-based precedence rules (concrete types > Any type)
- Compile-time ambiguity detection
- Runtime dispatch with specificity scoring

#### Value Objects
- Immutable by design
- Single construction path through validation
- Carry type-level proof of validity
- Support for normalization during construction

## Language Features

### Value Types
```relic
value EmailAddress(raw: String) {
    validate: raw contains "@" && raw.length > 3
    normalize: raw.toLowerCase()
}
```

### Functions with Multiple Dispatch
```relic
// All functions use 'fn' - dispatch is automatic
fn area(c: Circle) -> Float {
    3.14159 * c.radius * c.radius
}

fn area(r: Rectangle) -> Float {
    r.width * r.height
}

// Add specializations naturally
fn area(t: Triangle) -> Float {
    0.5 * t.base * t.height
}
```

### Uniform Function Call (UFC)
```relic
// These are equivalent:
area(shape)       // Traditional call
shape.area()      // UFC syntax

// Enables natural chaining:
input
  .trim()
  .toLowerCase()
  .validate()
```

### Let Bindings and Pipelines
```relic
// Let bindings for local values
let x = computeValue() in
let y = transform(x) in
combine(x, y)

// Pipeline operator for composition
data |> filter(x => x > 0) |> map(double) |> sum()
```

### Pattern Matching
```relic
match result {
    Success(value) => process(value),
    Error(msg) => handleError(msg)
}
```

## Code Style Guidelines

### General Principles
- Follow Rust idioms and conventions
- Prefer clarity over cleverness
- Keep functions small and focused
- Use descriptive names

### Relic-Specific Guidelines
- Value type names should be nouns (e.g., `EmailAddress`, `CustomerId`)
- Validation predicates should be simple and composable
- Use multiple dispatch instead of if/else chains
- Leverage UFC for readable data transformations

### Error Messages
- Be specific about what went wrong
- Include the invalid value when appropriate
- Suggest fixes when possible
- Maintain consistent formatting

## Testing Strategy

### Unit Tests
- Each compiler phase has dedicated tests
- Use `#[test]` annotations in Rust modules
- Test both success and failure cases

### Integration Tests
- End-to-end tests in `examples/` directory
- REPL interaction tests
- File processing tests

### Example Files
Key examples to understand the language:
- `values.relic` - Value type declarations
- `functions.relic` - Function definitions
- `pipeline.relic` - Pipeline and UFC usage
- `pattern_matching.relic` - Pattern matching examples

## Important Files

### Documentation
- `DESIGN.md` - Theoretical foundations and philosophy
- `IMPLEMENTATION_PLAN.md` - Detailed roadmap
- `PHASE_3_UNIFIED_SYNTAX.md` - Unified function syntax design
- `README.md` - User-facing documentation

### Core Implementation
- `src/ast.rs` - AST definitions
- `src/types.rs` - Type system
- `src/evaluator.rs` - Expression evaluation and dispatch
- `src/main.rs` - REPL implementation

## Current Limitations

1. **Single-parameter value types** - Multi-field values not yet supported
2. **No string concatenation** - String operations limited
3. **Basic pattern matching** - Only simple constructor patterns
4. **No parameter guards** - Guards parsed but not evaluated
5. **Limited built-in functions** - Minimal standard library

## Contributing Guidelines

When making changes:
1. Ensure all tests pass (`cargo test`)
2. Run formatter (`cargo fmt`)
3. Check linter warnings (`cargo clippy`)
4. Update relevant documentation
5. Add tests for new functionality
6. Consider backward compatibility

## Debug Tips

- Use `cargo test -- --nocapture` to see println! output
- The REPL shows type information for expressions
- Check `PHASE_*_STATUS.md` files for implementation progress
- Enable verbose parsing with debug prints in parser.rs

## Future Directions

The language is evolving toward:
- Full functional-relational programming with built-in query operations
- Advanced type system with refinement types and row polymorphism
- Optimizing compiler using sea of nodes architecture
- Effect system for controlled side effects
- Rich standard library of value types

Remember: Relic aims to make invalid states unrepresentable through its type system while maintaining elegance and performance.