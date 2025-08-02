# Relic Language Implementation Plan

This document outlines the implementation roadmap for Relic, a value-oriented programming language with functional-relational foundations as described in design.md.

## Current Status (February 2025)

✅ **Phase 1 COMPLETED**: Core value object foundation with parse-don't-validate semantics
🚧 **Phase 2 IN PROGRESS**: Parser, lexer, and basic language features (90% complete)
🔲 **Phase 3 PENDING**: Multiple dispatch system
🔲 **Phases 4-10**: Future work

### What's Working Now
- Full lexer and parser for value type declarations
- Type checker with validation and normalization support
- Compiler that generates value constructors from AST
- Interactive REPL with full pipeline (parse → typecheck → compile → execute)
- Pipeline operator `|>` for functional composition
- Let-bindings for intermediate calculations in validation predicates
- Line comments with `//` syntax
- File input support via command-line arguments
- Example files demonstrating all features
- Comprehensive test suite

### Recent Additions (February 2025)
- ✅ Let-bindings (`let x = expr in body`) fully implemented
- ✅ Support for nested let-bindings
- ✅ Line comment support (`//`)
- ✅ File input mode for processing `.relic` files
- ✅ Pipeline operator `|>` fully implemented
- ✅ Tests for all new features
- ✅ Example files for pipeline and let-bindings

### Next Steps
- Implement pattern matching on value types
- Add value type equality and structural hashing
- Begin Phase 3: Multiple dispatch system
- Add multi-line comment support

## Core Philosophy

Relic embodies four fundamental principles:
1. **Parse Don't Validate**: Value objects as immutable witnesses of valid construction
2. **Functional-Relational Architecture**: Strict separation of essential state, logic, and effects
3. **Multiple Dispatch**: Replace control flow with type-based method selection
4. **Type-Level Relationships**: Encode constraints and relationships in the type system

## Phase 1: Value Object Foundation (Weeks 1-4) ✅ COMPLETED

### 1.1 Core Value Object System
- [x] Implement value type declarations with validation predicates
- [x] Create parse-don't-validate constructor semantics
- [x] Build single construction path enforcement (static `From` methods)
- [x] Add normalization support in value constructors
- [x] Implement immutability guarantees at type level
- [ ] Create struct-based value objects for near-zero overhead (partial - generic implementation done)

### 1.2 Basic Type System Infrastructure
- [x] Implement type checker supporting value objects as primitives
- [x] Create AST representation for value type declarations
- [x] Build type inference engine with Hindley-Milner foundation
- [x] Add refinement type predicates for value constraints
- [x] Implement error reporting with source location tracking

### 1.3 Value Constructor Side Effects
- [x] Design side effect tracking for constructors
- [x] Implement constructor as sole boundary for effects
- [ ] Create effect type annotations for constructors (planned for Phase 5)
- [x] Build constructor failure handling with type safety
- [ ] Add uniqueness constraint checking at construction (syntax supported, implementation pending)

## Phase 2: Parser and Core Language (Weeks 5-8) ✅ MOSTLY COMPLETED

### 2.1 Concrete Syntax Design
- [x] Implement syntax matching design.md examples:
  ```
  value EmailAddress(raw: String) {
    validate: raw contains "@" && raw.length > 3
    normalize: raw.toLowerCase()
  }
  ```
- [x] Create lexer with proper token representation
- [x] Build recursive descent parser
- [x] Add syntax error recovery and reporting
- [x] Support pipeline operator `|>` for composition

### 2.2 Value Object Semantics
- [x] Parse value type declarations with validation blocks
- [x] Implement constructor code generation
- [x] Create type-level proof carrying for valid values
- [ ] Build exhaustive pattern matching on value types (pending)
- [ ] Add value type equality and structural hashing (pending)

### 2.3 Expression Evaluation
- [x] Implement pure expression evaluator
- [x] Add let-binding with value semantics
- [ ] Create function definitions as pure transformations (pending)
- [ ] Build pattern matching with value deconstruction (pending)
- [x] Enforce stratified architecture at language level

## Phase 3: Multiple Dispatch System (Weeks 9-12)

### 3.1 Multiple Dispatch Core
Based on Julia and CLOS research:
- [ ] Design method signature representation
- [ ] Implement dispatch table structure
- [ ] Create compile-time specialization for near-zero overhead
- [ ] Build method precedence based on type specificity
- [ ] Add ambiguity detection with clear error messages

### 3.2 Dispatch-Based Operations
- [ ] Implement relational operations via multiple dispatch:
  ```
  join(r1::HashRelation, r2::HashRelation) = hash_hash_join(r1, r2)
  join(r1::SortedRelation, r2::SortedRelation) = merge_join(r1, r2)
  ```
- [ ] Create extensible validation methods:
  ```
  validate(email: EmailAddress, domain: Domain) = 
    email.domain == domain
  ```
- [ ] Build dispatch caching for performance
- [ ] Add method introspection capabilities

### 3.3 Type-Safe Symmetric Operations
- [ ] Replace method ownership with symmetric dispatch
- [ ] Implement natural expression of binary operations
- [ ] Create specialized implementations per type combination
- [ ] Build performance profiling for dispatch overhead

## Phase 4: Functional-Relational Core (Weeks 13-16)

### 4.1 Essential State as Relations
Following "Out of the Tar Pit" architecture:
- [ ] Implement relations as sole storage for essential state
- [ ] Create relation types with value object schemas
- [ ] Build immutable fact storage (inspired by Datomic)
- [ ] Add time-based queries for historical data
- [ ] Enforce user-input data only in essential state

### 4.2 Relational Algebra with Value Types
- [ ] Implement type-safe relational operations
- [ ] Add Malloy-inspired source modeling:
  ```
  users 
    |> where(age > 21)
    |> join(orders, on: userId)
    |> group(by: city)
    |> select(city, orderCount: count())
  ```
- [ ] Create relationship-aware computing to prevent fan traps
- [ ] Build composable query blocks as first-class values
- [ ] Add automatic optimization based on value types

### 4.3 Type-Level Relationships
- [ ] Encode relationships through type dependencies:
  ```
  value Post {
    author: User  // Direct dependency on User type
    content: String
    where author exists in Users
  }
  ```
- [ ] Replace foreign keys with type-level constraints
- [ ] Implement compile-time relationship validation
- [ ] Create inference for transitive relationships

## Phase 5: Advanced Type System (Weeks 17-20)

### 5.1 Row Types and Extensible Records
Based on Ur/Web and PureScript:
- [ ] Implement row type representation
- [ ] Add record concatenation with type safety
- [ ] Create disjointness proofs for safe composition
- [ ] Build type-level record operations
- [ ] Support polymorphic extensible records

### 5.2 Refinement Types with SMT Solving
Following Liquid Haskell and F*:
- [ ] Implement refinement predicates: `{v:Int | v > 0}`
- [ ] Integrate Z3 or CVC5 for constraint solving
- [ ] Add automatic verification of value constraints
- [ ] Create dependent function types
- [ ] Build counterexample generation for failures

### 5.3 Effect System Integration
Inspired by F* and Links:
- [ ] Track side effects at type level
- [ ] Enforce effects only at value construction
- [ ] Create stratified architecture guarantees
- [ ] Build effect polymorphism
- [ ] Add effect handlers for controlled execution

## Phase 6: Constraint System (Weeks 21-24)

### 6.1 Declarative Constraints
- [ ] Implement automatic constraint maintenance
- [ ] Create constraint language integrated with types
- [ ] Build incremental constraint checking
- [ ] Add constraint debugging and visualization
- [ ] Support temporal constraints

### 6.2 Type-Safe Schema Evolution
- [ ] Design migration as type transformation
- [ ] Implement value type versioning
- [ ] Create automatic migration generation
- [ ] Build compatibility checking
- [ ] Add rollback support

## Phase 7: Standard Library (Weeks 25-28)

### 7.1 Core Value Types Library
- [ ] Implement common value types:
  - EmailAddress with RFC validation
  - URL with parsing and normalization  
  - Various ID types with uniqueness
  - Money with currency handling
  - DateRange with constraint checking
- [ ] Create numeric types with bounds
- [ ] Build collection value types
- [ ] Add validation combinator library

### 7.2 Relational Patterns Library
- [ ] Implement common query patterns
- [ ] Create audit log relations
- [ ] Build temporal relation utilities
- [ ] Add graph algorithms on relations
- [ ] Create statistical aggregations

### 7.3 Integration Adapters
- [ ] Build SQL compatibility layer
- [ ] Create JSON schema from value types
- [ ] Implement GraphQL type generation
- [ ] Add REST API generators
- [ ] Build event sourcing adapters

## Phase 8: Runtime and Performance (Weeks 29-32)

### 8.1 Memory Management
- [ ] Implement value object pooling
- [ ] Create zero-copy relation operations
- [ ] Build reference counting for values
- [ ] Add cycle detection for relations
- [ ] Optimize struct layout for cache

### 8.2 Query Optimization
- [ ] Implement cost-based optimizer
- [ ] Add value type-aware optimizations
- [ ] Create predicate pushdown
- [ ] Build join order optimization
- [ ] Add incremental view maintenance

### 8.3 Compilation Strategy
- [ ] Design compilation to efficient C/Rust
- [ ] Implement whole-program optimization
- [ ] Create dispatch specialization
- [ ] Build inlining for small values
- [ ] Add profile-guided optimization

## Phase 9: Developer Experience (Weeks 33-36)

### 9.1 Language Server Protocol
- [ ] Implement LSP with value type awareness
- [ ] Add constraint checking in real-time
- [ ] Create relationship visualization
- [ ] Build refactoring for value types
- [ ] Add migration suggestions

### 9.2 Development Tools
- [ ] Create REPL with value inspection
- [ ] Build time-travel debugger
- [ ] Add constraint solver traces
- [ ] Create query plan visualizer
- [ ] Build performance profiler

### 9.3 Documentation Generation
- [ ] Extract value constraints as documentation
- [ ] Generate relationship diagrams
- [ ] Create example-based docs
- [ ] Build interactive tutorials
- [ ] Add migration guides

## Phase 10: Ecosystem Integration (Weeks 37-40)

### 10.1 Database Adapters
- [ ] PostgreSQL adapter with custom types
- [ ] SQLite integration with value storage
- [ ] Datomic-compatible interface
- [ ] Redis adapter for caching
- [ ] S3 adapter for blob storage

### 10.2 Framework Integration
- [ ] Web framework with value types
- [ ] GraphQL server generation
- [ ] Event streaming integration
- [ ] Message queue adapters
- [ ] Microservice scaffolding

### 10.3 Migration Tools
- [ ] SQL schema to Relic types
- [ ] ORM model converters
- [ ] API spec importers
- [ ] Test data generators
- [ ] Legacy system adapters

## Success Metrics

### Correctness Guarantees
- 100% of invalid states unrepresentable
- All constraints verified at compile time
- Zero runtime validation in domain layer
- Complete effect tracking at boundaries

### Performance Targets
- Value construction: < 10ns overhead
- Multiple dispatch: < 5ns monomorphic  
- Query execution: Match hand-optimized SQL
- Memory usage: Comparable to C structs

### Developer Productivity
- Onboarding time: < 1 day for basics
- Error resolution: < 30s average
- Refactoring safety: 100% type-checked
- Test reduction: 50% fewer tests needed

## Implementation Strategy

### Technical Choices
- **Implementation Language**: Rust for performance and safety
- **Parser Generator**: Hand-written for better errors
- **Type Checker**: Bidirectional with elaboration
- **SMT Solver**: Z3 with CVC5 fallback
- **Backend**: LLVM for production, interpreter for development

### Development Process
1. **Prototype First**: Validate each concept before production implementation
2. **User Feedback**: Release previews after each major phase
3. **Dogfood Early**: Use Relic to build Relic tools
4. **Benchmark Continuously**: Performance regression tests
5. **Document Everything**: Implementation notes for contributors

### Risk Mitigation

**Technical Risks**:
- SMT solver performance → Incremental checking, caching
- Dispatch overhead → Compile-time specialization  
- Query optimization complexity → Start with rule-based optimizer
- Effect system usability → Make effects optional initially

**Adoption Risks**:
- Learning curve → Interactive tutorials, clear examples
- Ecosystem gap → Strong interop, adapter libraries
- Performance doubts → Public benchmarks, case studies
- Migration cost → Incremental adoption path

## Immediate Next Steps

### Completing Phase 2 (1-2 weeks)
1. **Pipeline Operator**: ✅ COMPLETED - Implement `|>` for functional composition
2. **Let Bindings**: Add support for `let x = value in expression`
3. **Pattern Matching**: Basic pattern matching on value types
4. **Value Equality**: Implement proper equality and hashing for value objects

### Starting Phase 3: Multiple Dispatch (3-4 weeks)
1. **Method Syntax**: Design and implement method declaration syntax
2. **Dispatch Table**: Create efficient dispatch table structure
3. **Type Specificity**: Implement method precedence rules
4. **Basic Dispatch**: Get simple multiple dispatch working
5. **Performance**: Add compile-time specialization

### Early Phase 4: Relational Foundation (2-3 weeks)
1. **Relation Types**: Design relation syntax and semantics
2. **Basic Queries**: Implement simple where/select operations
3. **Value Integration**: Ensure value types work seamlessly with relations

The goal is to demonstrate multiple dispatch and basic relational operations within the next 6-8 weeks, building on the solid value object foundation we've established.