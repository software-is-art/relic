# Changelog

All notable changes to the Relic programming language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-02-08

### Summary
Phase 2 is now 97% complete with the addition of value type equality and hashing support. The language continues to mature with essential features for practical use.

## [Unreleased]

### Added
- **Let Bindings**
  - Full lexer support for `let` and `in` keywords
  - Parser implementation for let-binding expressions
  - Type checker support with proper scoping rules
  - Compiler support for evaluating let-bindings in validation predicates
  - Support for nested let-bindings
  - Comprehensive test suite for let-binding functionality
  - Example file demonstrating let-binding usage (`examples/let_bindings.relic`)

- **Comment Support**
  - Line comments using `//` syntax
  - Comments are properly skipped during lexical analysis

- **File Input Support**
  - Command-line argument support for processing `.relic` files
  - Example: `cargo run examples/file.relic`

- **Pipeline Operator `|>`**
  - Full lexer support for distinguishing `|>` from `||`
  - Parser implementation with correct precedence (lower than logical operators)
  - Type checker support for pipeline expressions
  - Compiler support for evaluating pipeline expressions in validation
  - Comprehensive test suite for pipeline functionality
  - Example file demonstrating pipeline usage (`examples/pipeline.relic`)

- **Pattern Matching** (February 2025)
  - Lexer support for `match` keyword and `=>` arrow operator
  - AST representation with `Match` expressions, `MatchArm`, and `Pattern` types
  - Parser implementation for match expressions with constructor patterns
  - Type checker ensures patterns match value types and all arms return same type
  - Pattern variables properly bound in arm scopes
  - Test suite for pattern matching functionality
  - Example file showing pattern matching syntax (`examples/pattern_matching.relic`)

- **Value Type Equality** (February 8, 2025)
  - Implemented equality (`==`) and inequality (`!=`) operators for validation expressions
  - Added `equals()` and `hash_value()` methods to `ValueObject` trait
  - Support for structural equality comparison of value objects
  - Hashing implementation for value objects (enables use in HashSet/HashMap)
  - Display trait implementation for better debugging
  - Comprehensive test suite for equality and hashing
  - Example file demonstrating equality operators (`examples/value_equality.relic`)
  - Compiler support for all comparison operators (>, <, >=, <=, ==, !=)

### Changed
- Updated parser to support functional composition via pipelines
- Enhanced expression evaluation to handle chained transformations
- Improved REPL to support file processing mode
- Extended AST to support match expressions and patterns
- Type checker now handles pattern matching with proper scoping
- Enhanced ValueObject trait with equality and hashing capabilities
- Compiler now evaluates all comparison operators in validation expressions

### Examples
```relic
// Let bindings for intermediate calculations
value Temperature(celsius: Int) {
    validate: let fahrenheit = celsius * 9 / 5 + 32 in 
              fahrenheit > -459 && fahrenheit < 1000
}

// Pipeline operator chains transformations
value ProcessedText(raw: String) {
    validate: raw |> toLowerCase |> trim |> length > 0
    normalize: raw |> trim |> toLowerCase
}

// Pattern matching for value deconstruction
value Result(status: Status) {
    validate: match status {
        Status(code) => code == 200 || (code >= 400 && code < 500)
    }
}

// Value equality checks
value Username(name: String) {
    validate: name != "admin" && name != "root"
    normalize: name.toLowerCase()
}
```

## [0.1.0] - 2024-12-XX

### Added
- **Core Language Features**
  - Value type declarations with validation predicates
  - Parse-don't-validate constructor semantics
  - Single construction path enforcement
  - Validation expressions with logical, comparison, and arithmetic operators
  - Normalization support in value constructors
  - Unique constraint syntax (parsing only)

- **Type System**
  - Basic type checker supporting String, Int, Bool, and user-defined value types
  - Type inference for expressions
  - Validation that ensures validate expressions return Bool
  - Validation that ensures normalize expressions return the parameter type

- **Parser and Lexer**
  - Complete lexer with all necessary tokens
  - Recursive descent parser for value declarations
  - Support for complex expressions including method calls and member access
  - Error reporting with line and column information

- **Compiler and Runtime**
  - Compiler that generates value constructors from AST
  - Runtime validation of value constraints
  - Value registry for managing constructors
  - Generic value object implementation

- **Developer Tools**
  - Interactive REPL with full compilation pipeline
  - Support for defining value types and creating instances
  - Helpful error messages for validation failures
  - Example files demonstrating language features

### Examples
```relic
value EmailAddress(raw: String) {
    validate: raw contains "@" && raw.length > 3
    normalize: raw.toLowerCase()
}

value CustomerId(id: Int) {
    validate: id > 0
    unique: true
}
```

### Known Limitations
- Pattern matching currently limited to validation expressions (not yet in standalone expressions)
- Uniqueness constraints parsed but not enforced
- Multiple dispatch system not yet implemented
- No relational features yet
- Pipeline operator currently limited (no proper function application on right side)
- No support for multi-line comments (only `//` line comments)
- Function definitions not yet implemented

### Internal
- Project structure following Rust best practices
- Modular architecture with separate lexer, parser, typechecker, and compiler
- Comprehensive error handling throughout
- Examples demonstrating each stage of compilation