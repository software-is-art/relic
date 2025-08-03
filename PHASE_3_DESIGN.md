# Phase 3: Multiple Dispatch Design

## Method Declaration Syntax

After analyzing various multiple dispatch systems (Julia, CLOS, Dylan), here's the proposed syntax for Relic:

### Basic Method Declaration

```relic
method area(shape: Circle) -> Float {
    3.14159 * shape.radius * shape.radius
}

method area(shape: Rectangle) -> Float {
    shape.width * shape.height
}

method area(shape: Triangle) -> Float {
    0.5 * shape.base * shape.height
}
```

### Multiple Parameter Dispatch

```relic
method combine(a: String, b: String) -> String {
    a + b
}

method combine(a: Int, b: Int) -> Int {
    a + b
}

method combine(a: List, b: List) -> List {
    a.concat(b)
}
```

### Type Constraints and Guards

```relic
method process(x: Int where x > 0, y: Int where y > 0) -> Int {
    x * y
}

method format(n: Float where n >= 0) -> String {
    "+" + n.toString()
}

method format(n: Float where n < 0) -> String {
    n.toString()
}
```

### Default/Fallback Methods

```relic
method display(x: Any) -> String {
    x.toString()
}

method display(x: String) -> String {
    "\"" + x + "\""
}

method display(x: Int) -> String {
    "Int(" + x.toString() + ")"
}
```

## Key Design Decisions

1. **`method` keyword**: Distinguishes multi-dispatch methods from regular functions
2. **Type annotations required**: All parameters must have type annotations for dispatch
3. **Guards with `where`**: Optional guards for runtime constraints
4. **Return type annotations**: Required for clarity and type checking
5. **`Any` type**: Acts as a fallback for generic dispatch

## Dispatch Resolution Rules

1. **Specificity**: More specific types win over less specific ones
   - `Int` is more specific than `Number`
   - `Number` is more specific than `Any`

2. **Left-to-right precedence**: When ambiguous, leftmost parameters take precedence

3. **Guard evaluation**: Guards are evaluated after type matching

4. **Ambiguity detection**: Compile-time error if no unique most-specific method exists

## Integration with UFC

The existing UFC syntax seamlessly works with methods:

```relic
let c = Circle(radius: 5)
let a1 = area(c)        // Direct call
let a2 = c.area()       // UFC style - desugars to area(c)

let result = 42.format()  // Calls format(42)
```

## Implementation Strategy

1. **AST Extension**: Add `MethodDecl` node similar to `FunctionDecl`
2. **Method Table**: Global table mapping (method_name, type_signature) -> implementation
3. **Dispatch Algorithm**: 
   - Collect all methods with matching name
   - Filter by type compatibility
   - Sort by specificity
   - Apply guards
   - Select most specific match

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
method area(c: Circle) -> Float {
    3.14159 * c.radius * c.radius
}

method area(r: Rectangle) -> Float {
    r.width * r.height
}

// Usage
let c = Circle(radius: 5)
let r = Rectangle(width: 10, height: 20)

println(c.area())  // 78.53975
println(r.area())  // 200

// Generic display
method display(x: Any) -> String {
    "<" + x.typeName() + ">"
}

method display(n: Int) -> String {
    "Integer: " + n.toString()
}

method display(s: String) -> String {
    "String: \"" + s + "\""
}
```