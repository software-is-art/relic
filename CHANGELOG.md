# Changelog

All notable changes to the Relic programming language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Pipeline Operator `|>`**
  - Full lexer support for distinguishing `|>` from `||`
  - Parser implementation with correct precedence (lower than logical operators)
  - Type checker support for pipeline expressions
  - Compiler support for evaluating pipeline expressions in validation
  - Comprehensive test suite for pipeline functionality
  - Example file demonstrating pipeline usage (`examples/pipeline.relic`)

### Changed
- Updated parser to support functional composition via pipelines
- Enhanced expression evaluation to handle chained transformations

### Examples
```relic
value ProcessedText(raw: String) {
    // Pipeline operator chains transformations
    validate: raw |> toLowerCase |> trim |> length > 0
    normalize: raw |> trim |> toLowerCase
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
- Pattern matching not yet implemented
- Let bindings not yet implemented
- Uniqueness constraints parsed but not enforced
- Multiple dispatch system not yet implemented
- No relational features yet
- Pipeline operator currently limited (no proper function application on right side)

### Internal
- Project structure following Rust best practices
- Modular architecture with separate lexer, parser, typechecker, and compiler
- Comprehensive error handling throughout
- Examples demonstrating each stage of compilation