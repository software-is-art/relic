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

### Functional-Relational Query Composition
Borrowing from Malloy's pipeline syntax and FRP's mathematical foundations:

```
users 
  |> where(age > 21)
  |> join(orders, on: userId)
  |> group(by: city)
  |> select(city, orderCount: count(), avgAmount: mean(amount))
  |> where(orderCount > 10)
```

## Implementation recommendations

Based on the research, a practical implementation should:

1. **Start with a strong type system** featuring row types, refinement types, and effect tracking
2. **Use immutable value objects** as the core data abstraction with parse-don't-validate semantics
3. **Implement multiple dispatch** with compile-time specialization for performance
4. **Provide built-in relational algebra** operations that understand value types and their relationships
5. **Enforce stratified architecture** through language-level separation of concerns
6. **Include SMT solver integration** for automatic constraint verification
7. **Support incremental adoption** through interop with existing databases and languages

The convergence of these ideas - value objects embodying parse-don't-validate, functional-relational programming minimizing complexity, Malloy's intuitive data transformations, and multiple dispatch replacing control flow - creates a compelling vision for a new programming paradigm that is simultaneously more correct, more performant, and more expressive than current approaches.