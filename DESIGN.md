# Designing a value-oriented programming language with functional-relational foundations

This research explores concepts for designing a new programming language that combines value objects as core primitives, functional-relational programming paradigms, and multiple dispatch to create a powerful yet elegant system for data manipulation and constraint enforcement.

## The foundation: Parse don't validate with value objects

The Vogen C# library exemplifies how **"parse don't validate"** principles can transform programming language design. At its core, this pattern advocates for parsing data into structured representations that carry proof of validity in their types, rather than repeatedly validating raw data throughout an application. Vogen achieves this through single construction paths - value objects can only be created through static `From` methods that enforce validation, making illegal states unrepresentable at compile time.

For our language design, this suggests **value constructors should be the sole gateway for creating data**, with all validation and side effects concentrated at these boundaries. Once constructed, values become immutable witnesses of their valid creation, carrying type-level proof that eliminates the need for defensive programming elsewhere in the system.

The performance implications are significant: validation happens exactly once at system boundaries, with struct-based value objects showing virtually identical performance to primitives. This architectural decision creates stratified programs where parsing layers handle all input validation while domain layers work with guaranteed valid data.

## Functional-relational architecture from "Out of the Tar Pit"

Ben Moseley and Peter Marks' seminal paper provides the theoretical foundation for managing complexity through functional-relational programming. Their key insight is that **state and control flow are the primary sources of accidental complexity** in software systems. They propose an architecture that strictly separates:

- **Essential State**: Relations containing only user-input data that must be retained
- **Essential Logic**: Pure functions and relational algebra expressing business rules  
- **Accidental State and Control**: Performance optimizations kept separate from core logic
- **Input/Output**: Minimal interface components (feeders and observers)

This separation suggests our language should store all essential state as relations rather than objects, with logic expressed through relational algebra extended with pure functions. The paper's emphasis on **declarative constraints maintained automatically** aligns perfectly with encoding relationships at the type level rather than through foreign key constraints.

Real-world implementations like Datomic and Project:M36 demonstrate the viability of these concepts, showing how immutable facts and time-based queries can provide both correctness and performance.

## Malloy's innovations in data transformation

Google's Malloy language offers crucial insights for making relational programming more intuitive and composable. Its semantic modeling approach treats **sources as first-class objects** that combine tables with their associated computations and relationships. This maps directly to our concept of value objects that encapsulate both data and validation logic.

Malloy's key innovations include:
- **Composable queries as building blocks** that can be nested and combined
- **Relationship-aware computing** that prevents common SQL errors like fan and chasm traps
- **Pipeline operations** using the `->` operator for clear data transformation flows
- **Avoiding "tables of primitives"** by binding behavior to data at the type level

The language demonstrates that query operations can be both more powerful and easier to use through careful attention to composability and type safety - principles directly applicable to our value-oriented design.

## Multiple dispatch as control flow replacement

Julia, Common Lisp's CLOS, and other multiple dispatch languages show how method selection based on all argument types can replace traditional control flow. For relational algebra, this is particularly powerful since operations like joins naturally depend on both operands rather than belonging to either one.

Multiple dispatch enables:
- **Type-safe relational operations** where algorithm selection happens automatically
- **Extensible systems** where new types and operations can be added independently
- **Performance optimization** through specialized implementations for different type combinations
- **Natural expression of symmetric operations** without artificial ownership

Consider how join operations could leverage multiple dispatch:
```julia
join(r1::HashRelation, r2::HashRelation) = hash_hash_join(r1, r2)
join(r1::SortedRelation, r2::SortedRelation) = merge_join(r1, r2)
join(r1::HashRelation, r2::SortedRelation) = hash_sort_join(r1, r2)
```

The research shows this approach can achieve near-zero dispatch overhead through compile-time specialization while enabling unprecedented extensibility.

### Uniform Function Call Syntax with Multiple Dispatch

A key insight is that UFC syntax is purely syntactic sugar that doesn't conflict with multiple dispatch semantics:

```
// These are semantically identical:
users.join(orders)                    // UFC syntax
join(users, orders)                   // Traditional syntax

// Both dispatch based on BOTH argument types:
// - If users is HashIndexed and orders is Sorted, calls hash_sort_join
// - If both are Sorted, calls merge_join
// - etc.
```

UFC syntax makes relational operations more discoverable and natural to write while preserving the power of multiple dispatch. The syntax transformation is simple: `x.f(y, z)` becomes `f(x, y, z)`, maintaining all dispatch semantics.

## Prior art synthesis: Type systems and constraints

The research reveals several mature approaches for encoding constraints and relationships at the type level:

**Row Types and Extensible Records** (from Ur/Web, Links, PureScript):
- Enable flexible schema representation with compile-time safety
- Support type-level record operations like concatenation and projection
- Provide disjointness proofs to ensure safe composition

**Refinement Types** (from Liquid Haskell, F*):
- Encode value constraints as predicates: `{v:Int | v > 0}`
- SMT solver integration for automatic verification
- Dependent function types that track relationships between values

**Effect Systems** (from F*, Links):
- Track side effects at the type level
- Enable reasoning about where and when effects occur
- Support stratified architectures with clear boundaries

## A unified design vision

Synthesizing these concepts suggests a language architecture where:

### Core Primitives: Value Objects
Every piece of data enters the system through value constructors that parse, validate, and witness the creation of valid data:

```
value EmailAddress(raw: String) {
  validate: raw contains "@" && raw.length > 3
  normalize: raw.toLowerCase()
}

value CustomerId(id: Int) {
  validate: id > 0
  unique: true  // Type-level constraint
}
```

### Side Effects Through Construction
Value constructors are the **only** place where side effects occur - they represent the boundary between the external world and the pure, typed interior:

```
// Constructor performs database insertion as side effect
let customer = Customer(
  id: CustomerId(123),      // Validates and may check uniqueness
  email: EmailAddress(input),  // Parses and normalizes
  age: Age(25)              // Validates age constraints
)
// Result is immutable witness of successful creation
```

### Relational Composition Without Foreign Keys
Instead of foreign key constraints, relationships are encoded through type dependencies:

```
value Post {
  author: User  // Direct dependency on User type
  content: String
  
  // Implicit relation through type system
  where author exists in Users
}
```

### Multiple Dispatch for Operations
All operations use multiple dispatch, eliminating control flow:

```
// Different implementations based on value types
validate(email: EmailAddress, domain: Domain) = 
  email.domain == domain

validate(age: Age, requirement: AgeRequirement) = 
  age.value >= requirement.minimum

// Relational operations dispatch on storage strategies
join(users: HashIndexed[User], posts: SortedBy[Post, date]) = 
  hashJoinSorted(users, posts)
```

### Type-as-Relation: The Ultimate Unity
The key innovation in Relic is that **every value type implicitly forms a relation of all its instances**. There is no separate relation construct - types ARE relations:

```
// Define a value type - this automatically creates a relation
value User(id: UserId, name: String, age: Int) {
  validate: age >= 0
  key: id           // Primary key for the implicit relation
  unique: email     // Unique constraint on the implicit relation
}

// Creating values automatically adds them to the type's relation
let alice = User(1, "Alice", 30)
let bob = User(2, "Bob", 25)

// Query the type directly - no special relation object needed
let adults = User.where(u => u.age >= 18)
let user = User.find(u => u.id == 1)
let count = User.count()
```

This approach ensures:
- No distinction between values and relations
- Type safety guarantees schema consistency
- All instances are automatically indexed
- Natural integration with the type system

### Functional-Relational Query Composition
With Type-as-Relation and UFC syntax, queries become natural transformations on types:

```
User.all()
  .where(u => u.age > 21)
  .join(Order.all(), (u, o) => u.id == o.userId)
  .groupBy(u => u.city)
  .select(city, orderCount: count(), avgAmount: avg(o => o.amount))
  .where(r => r.orderCount > 10)
```

Note that we query types directly - `User.all()` returns all User instances, not a separate relation object.

## Uniform Function Call Syntax as a Core Feature

UFC syntax allows any function to be called as a method on its first argument, transforming `f(x, y)` into `x.f(y)`. This syntactic convenience provides several benefits for Relic:

### Enhancing Readability Without Compromising Semantics

```
// Traditional functional style
let validated = validate(normalize(trim(input)))

// With UFC - same semantics, better readability
let validated = input.trim().normalize().validate()

// Relational operations flow naturally
let results = users
  .filter(u => u.age > 21)
  .join(orders, on: (u, o) => u.id == o.userId)
  .group(by: u => u.city)
  .select(city, revenue: sum(o => o.amount))
```

### UFC and Value Objects

UFC makes value object transformations feel natural while maintaining immutability:

```
// Each operation returns a new value
let email = rawInput
  .trim()
  .toLowerCase()
  .validateEmail()
  .normalizeEmail()

// Equivalent to:
let email = normalizeEmail(validateEmail(toLowerCase(trim(rawInput))))
```

### UFC with Multiple Dispatch

Critically, UFC doesn't interfere with multiple dispatch - it's purely syntactic:

```
// Both forms use the same multiple dispatch resolution
users.join(orders)  // Dispatches on types of both users AND orders
join(users, orders) // Identical dispatch behavior

// The implementation selected depends on both arguments
// UFC just provides a more natural way to write it
```

This means developers get the readability benefits of method chaining while the language maintains the power and flexibility of multiple dispatch for selecting optimal implementations.

## Compilation strategy: Sea of nodes for zero-overhead abstractions

Relic's compilation strategy leverages a sea of nodes intermediate representation to achieve near-zero overhead for its high-level abstractions. This graph-based IR is particularly well-suited to Relic's design principles.

### Why sea of nodes aligns with parse-don't-validate

The parse-don't-validate pattern creates natural boundaries in the dataflow graph:

1. **Value construction nodes** mark where raw data enters the system
2. **Validation nodes** can be optimized away when types are statically known
3. **Immutability** means each value has exactly one definition, mapping perfectly to SSA form
4. **Effect boundaries** are explicit in the graph, enabling aggressive pure code optimization

Example optimization:
```relic
// Original code
fn processEmail(raw: String) -> EmailAddress {
    EmailAddress(raw.toLowerCase())
}

// After optimization when called with a literal
processEmail("USER@EXAMPLE.COM")
// Validation is proven at compile time, constructor inlined
// Result: Direct allocation of EmailAddress("user@example.com")
```

### Multiple dispatch optimization through specialization

The sea of nodes representation enables powerful optimizations for multiple dispatch:

1. **Type flow analysis** tracks concrete types through the graph
2. **Dispatch nodes** can be replaced with direct calls when types are known
3. **Speculative optimization** with deoptimization guards for dynamic cases
4. **Method inlining** across dispatch boundaries

This means Relic can offer the flexibility of multiple dispatch with the performance of static dispatch in most cases.

### Relational operations as graph transformations

With Type-as-Relation, functional-relational operations map naturally to dataflow graphs:

```relic
User.all()
  .where(u => u.age > 21)
  .join(Order.all(), (u, o) => u.id == o.userId)
  .select(u => {name: u.name, total: o.amount})
```

In the graph representation:
- Each value instance is a node in the graph
- Type relations are node collections
- Query operations are pure transformation nodes
- Type information flows through the graph enabling optimization
- Common subexpressions are automatically shared

The Type-as-Relation approach provides optimization opportunities:
- No impedance mismatch between values and relations
- Direct mapping to sea of nodes architecture
- Instance tracking enables efficient indexing
- Immutable values enable aggressive parallelization

### Performance implications

The sea of nodes architecture enables Relic to achieve:

1. **Zero-overhead value types** - Construction and validation optimized away
2. **Free abstractions** - High-level code compiles to optimal low-level code  
3. **Predictable performance** - Graph structure makes costs visible
4. **Adaptive optimization** - Runtime profiling guides specialization

This compilation strategy ensures that Relic's elegant abstractions don't come at the cost of performance, making it suitable for both high-level domain modeling and systems programming.

## Implementation recommendations

Based on the research, a practical implementation should:

1. **Start with Type-as-Relation** where value types automatically track their instances
2. **Use immutable value objects everywhere** - values are automatically part of their type's relation
3. **Implement multiple dispatch** with compile-time specialization for performance
4. **Provide pure functional query operations** without special syntax - just functions and UFC
5. **Type-safe field references** eliminating string-based field access
6. **Enforce stratified architecture** through language-level separation of concerns
7. **Include SMT solver integration** for automatic constraint verification
8. **Support incremental adoption** through interop with existing databases and languages

The key innovation is that **types ARE relations**, perfectly unifying Relic's philosophy:
- Parse don't validate: Every row is a validated value object
- Everything is a value: Relations, rows, and fields
- Functional composition: Queries are pure function applications
- Type safety: Field access is compile-time verified
- Multiple dispatch: Storage strategies can be optimized per type

This creates a programming paradigm where working with relational data is as natural, safe, and performant as working with simple values - because relations ARE values that generate other values.