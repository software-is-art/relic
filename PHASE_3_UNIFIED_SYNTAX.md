# Phase 3: Unified Function Syntax

## Overview

After careful consideration and analysis of user experience, we've decided to unify function syntax in Relic. This eliminates the distinction between `fn` and `method`, making the language simpler and more elegant while maintaining all the power of multiple dispatch.

## Rationale

### The Problem with Separate Syntax

1. **Cognitive Load**: Users must decide upfront whether a function might need multiple implementations
2. **Evolution Friction**: Converting `fn` to `method` when adding specializations is cumbersome
3. **Inconsistent Mental Model**: UFC makes both look identical to users anyway
4. **Implementation Detail Leaked**: The distinction exposes compiler internals to users

### The Julia Inspiration

Julia demonstrates that every function can potentially have multiple dispatch without special syntax:

```julia
# Just define functions - no special syntax needed
area(c::Circle) = π * c.radius^2
area(r::Rectangle) = r.width * r.height
```

## Unified Syntax Design

### Before (Current Implementation)
```relic
// Single implementation requires 'fn'
fn format(x: Int) -> String {
    "Int: " + x.toString()
}

// Multiple implementations require 'method'
method process(x: Int) -> Int { x * 2 }
method process(x: String) -> String { x.toUpperCase() }
```

### After (Unified Syntax)
```relic
// All functions use 'fn' - compiler handles dispatch automatically
fn format(x: Int) -> String {
    "Int: " + x.toString()
}

fn process(x: Int) -> Int { x * 2 }
fn process(x: String) -> String { x.toUpperCase() }

// Can add specializations naturally
fn format(x: String) -> String {
    "String: \"" + x + "\""
}
```

## Implementation Strategy

### Phase 1: Parser Unification (Backward Compatible)
1. Make `method` keyword an alias for `fn`
2. Both keywords produce identical AST nodes
3. Existing code continues to work

### Phase 2: Type System Changes
1. Unify function and method storage in TypeEnvironment
2. Collect all functions with the same name automatically
3. Type checker treats all functions as potentially multi-dispatch

### Phase 3: Runtime Changes
1. Evaluator checks number of implementations for each function name
2. Single implementation → direct call (optimized)
3. Multiple implementations → dispatch table (flexible)

### Phase 4: Deprecation (Future)
1. Add gentle warnings for `method` keyword usage
2. Provide automated migration tool
3. Eventually remove `method` keyword entirely

## Benefits

### For Users
- **Simpler Language**: One concept instead of two
- **Natural Evolution**: Add specializations without syntax changes
- **Better Ergonomics**: No upfront dispatch decisions
- **Cleaner Code**: Less syntax noise

### For Implementation
- **Unified Code Paths**: Simpler compiler internals
- **Better Optimization**: Compiler can choose dispatch strategy
- **Easier Maintenance**: Less duplication in parser/type checker

## Examples

### Basic Functions
```relic
// Simple function - no dispatch needed
fn double(x: Int) -> Int {
    x * 2
}

// Later, add specialization for strings
fn double(s: String) -> String {
    s + s
}

// UFC works identically
42.double()      // Returns 84
"hi".double()    // Returns "hihi"
```

### Type-Based Algorithms
```relic
// Define different algorithms for different types
fn sort(list: List<Int>) -> List<Int> {
    // Use counting sort for integers
}

fn sort(list: List<String>) -> List<String> {
    // Use radix sort for strings
}

fn sort(list: List<Any>) -> List<Any> {
    // Use comparison sort for generic types
}
```

### Extensible Operations
```relic
// Core library defines base cases
fn equals(a: Int, b: Int) -> Bool { a == b }
fn equals(a: String, b: String) -> Bool { a == b }
fn equals(a: Any, b: Any) -> Bool { false }

// User code adds specializations
value Point(x: Int, y: Int)

fn equals(p1: Point, p2: Point) -> Bool {
    p1.x == p2.x && p1.y == p2.y
}
```

## Migration Guide

### For Existing Code
```relic
// Old style - still works
method area(c: Circle) -> Float { ... }

// New style - preferred
fn area(c: Circle) -> Float { ... }
```

### Compiler Behavior
1. Both syntaxes produce identical behavior
2. No performance difference
3. Same dispatch resolution rules apply

## Future Considerations

### Optimization Opportunities
- Compiler can inline single-implementation functions
- Multi-implementation functions can use various dispatch strategies
- Profile-guided optimization can specialize hot paths

### Language Evolution
- Opens door for gradual typing (functions can gain type specificity over time)
- Enables better metaprogramming (all functions are first-class)
- Simplifies language specification

## Conclusion

Unifying function syntax makes Relic more elegant, user-friendly, and powerful. By following Julia's lead and making every function potentially multi-dispatch, we remove unnecessary complexity while maintaining all the benefits of multiple dispatch. This change aligns with Relic's philosophy of simplicity and expressiveness.