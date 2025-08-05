# Phase 3: Multiple Dispatch Design (Revised with Unified Syntax)

## Unified Function Declaration Syntax

After analyzing various multiple dispatch systems (Julia, CLOS, Dylan) and considering user experience, we've decided to follow Julia's approach of unified function syntax. All functions in Relic can potentially have multiple dispatch:

### Basic Function Declaration with Multiple Dispatch

```relic
fn area(shape: Circle) -> Float {
    3.14159 * shape.radius * shape.radius
}

fn area(shape: Rectangle) -> Float {
    shape.width * shape.height
}

fn area(shape: Triangle) -> Float {
    0.5 * shape.base * shape.height
}
```

### Multiple Parameter Dispatch

```relic
fn combine(a: String, b: String) -> String {
    a + b
}

fn combine(a: Int, b: Int) -> Int {
    a + b
}

fn combine(a: List, b: List) -> List {
    a.concat(b)
}
```

### Type Constraints and Guards

```relic
fn process(x: Int where x > 0, y: Int where y > 0) -> Int {
    x * y
}

fn format(n: Float where n >= 0) -> String {
    "+" + n.toString()
}

fn format(n: Float where n < 0) -> String {
    n.toString()
}
```

### Default/Fallback Functions

```relic
fn display(x: Any) -> String {
    x.toString()
}

fn display(x: String) -> String {
    "\"" + x + "\""
}

fn display(x: Int) -> String {
    "Int(" + x.toString() + ")"
}
```

## Key Design Decisions

1. **Unified `fn` syntax**: All functions can have multiple dispatch (no separate `method` keyword)
2. **Type annotations required**: All parameters must have type annotations for dispatch
3. **Guards with `where`**: Optional guards for runtime constraints
4. **Return type annotations**: Required for clarity and type checking
5. **`Any` type**: Acts as a fallback for generic dispatch
6. **Compiler optimization**: Single-implementation functions are optimized to direct calls

## Dispatch Resolution Rules

1. **Specificity**: More specific types win over less specific ones
   - `Int` is more specific than `Number`
   - `Number` is more specific than `Any`

2. **Left-to-right precedence**: When ambiguous, leftmost parameters take precedence

3. **Guard evaluation**: Guards are evaluated after type matching

4. **Ambiguity detection**: Compile-time error if no unique most-specific method exists

## Integration with UFC

The UFC syntax seamlessly works with all functions:

```relic
let c = Circle(radius: 5)
let a1 = area(c)        // Direct call
let a2 = c.area()       // UFC style - desugars to area(c)

let result = 42.format()  // Calls format(42)

// UFC makes no distinction between single or multi-dispatch functions
```

## Implementation Strategy

1. **Unified AST**: All functions use `FunctionDecl` node (method becomes optional alias)
2. **Function Registry**: Global table mapping function_name -> list of implementations
3. **Dispatch Algorithm**: 
   - Collect all functions with matching name
   - If single implementation: direct call (optimized path)
   - If multiple implementations:
     - Filter by type compatibility
     - Sort by specificity
     - Apply guards
     - Select most specific match
4. **Backward Compatibility**: `method` keyword accepted as alias for `fn`

## Examples for Phase 3 Implementation

```relic
// Shape hierarchy
value Circle(radius: Float) {
    validate: radius > 0
}

value Rectangle(width: Float, height: Float) {
    validate: width > 0 && height > 0
}

// Area calculation with multiple dispatch
fn area(c: Circle) -> Float {
    3.14159 * c.radius * c.radius
}

fn area(r: Rectangle) -> Float {
    r.width * r.height
}

// Usage
let c = Circle(radius: 5)
let r = Rectangle(width: 10, height: 20)

println(c.area())  // 78.53975
println(r.area())  // 200

// Generic display with fallback
fn display(x: Any) -> String {
    "<" + x.typeName() + ">"
}

fn display(n: Int) -> String {
    "Integer: " + n.toString()
}

fn display(s: String) -> String {
    "String: \"" + s + "\""
}
```