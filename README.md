# Relic

A value-oriented programming language with functional-relational foundations.

## Overview

Relic is an experimental programming language that combines:
- **Parse Don't Validate**: Value objects as immutable witnesses of valid construction
- **Functional-Relational Architecture**: Strict separation of essential state, logic, and effects
- **Multiple Dispatch**: Replace control flow with type-based method selection
- **Type-Level Relationships**: Encode constraints and relationships in the type system

## Current Implementation

This initial implementation includes:
- ✅ Value type declarations with validation predicates
- ✅ Parse-don't-validate constructor semantics
- ✅ Single construction path enforcement
- ✅ Basic type checker supporting value objects
- ✅ AST representation for value type declarations
- ✅ Lexer and recursive descent parser
- ✅ Compiler to generate value constructors from AST
- ✅ Interactive REPL

## Quick Start

### Building

```bash
cargo build
```

### Running the REPL

```bash
cargo run
```

### Example Usage

```relic
# Define a value type for email addresses
value EmailAddress(raw: String) {
    validate: raw contains "@" && raw.length > 3
    normalize: raw.toLowerCase()
}

# Define a value type for customer IDs
value CustomerId(id: Int) {
    validate: id > 0
    unique: true
}

# Create value instances (in REPL)
EmailAddress("test@example.com")  # ✓ Valid
EmailAddress("invalid")           # ✗ Invalid: Validation failed
CustomerId(123)                   # ✓ Valid
CustomerId(0)                     # ✗ Invalid: Validation failed
```

## Language Features

### Value Type Declarations

Value types are the core primitive in Relic. They encapsulate:
- **Validation**: Predicates that must be true for construction
- **Normalization**: Transformations applied during construction
- **Uniqueness**: Constraints for system-wide uniqueness

```relic
value EmailAddress(raw: String) {
    validate: raw contains "@" && raw.length > 3
    normalize: raw.toLowerCase()
}
```

### Type System

Relic currently supports:
- `String`: Text values
- `Int`: Integer values
- `Bool`: Boolean values
- User-defined value types

### Expressions

- **Logical**: `&&`, `||`, `!`
- **Comparison**: `==`, `!=`, `<`, `>`, `<=`, `>=`, `contains`
- **Arithmetic**: `+`, `-`, `*`, `/`
- **Member Access**: `object.property`
- **Method Calls**: `object.method(args)`
- **Pipeline**: `expr |> expr` - Functional composition
- **Let-bindings**: `let name = expr in body` - Local bindings
- **Pattern Matching**: `match expr { Pattern(binding) => result }` - Destructuring

## Examples

See the `examples/` directory for more examples:
- `email.relic`: Value type definitions
- `pipeline.relic`: Pipeline operator usage
- `let_bindings.relic`: Let-binding examples
- `pattern_matching.relic`: Pattern matching syntax (planned feature)

## Architecture

The implementation follows a traditional compiler architecture:
1. **Lexer** (`src/lexer.rs`): Tokenizes input
2. **Parser** (`src/parser.rs`): Builds AST
3. **Type Checker** (`src/typechecker.rs`): Validates types
4. **Compiler** (`src/compiler.rs`): Generates value constructors
5. **Runtime** (`src/value.rs`): Executes value construction

## Next Steps

See `IMPLEMENTATION_PLAN.md` for the full roadmap. Next phases include:
- Multiple dispatch system
- Functional-relational core
- Advanced type system with row types and refinements
- Effect system integration
- Standard library

## Contributing

This is an experimental research project. See `DESIGN.md` for the theoretical foundation and design principles.