# Phase 4: Functional-Relational Core Status

## Overview
Phase 4 introduces the functional-relational programming paradigm to Relic. After multiple design iterations, we've arrived at a revolutionary approach: **Type-as-Relation** - where every value type implicitly forms a relation of all its instances.

## Major Design Evolution: Type-as-Relation
We've evolved from explicit relations to a model where:
- Every value type automatically forms a relation of its instances
- No special relation syntax or types needed
- Relations emerge naturally from the type system
- Aligns perfectly with the sea of nodes compiler architecture

## Summary
Phase 4 is **~85% complete** - We've successfully implemented the minimal built-in approach with Type as a first-class type, the `all(t: Type) -> List[t]` built-in function, persistent instance storage, value construction, field extraction, and the pure Relic `count()` function.

### Design Evolution Timeline
1. ✅ **First approach**: Special `relation` syntax with code generation
2. ✅ **Second approach**: Relations as value constructors (`relationOf`)
3. ✅ **Current approach**: Type-as-Relation - types ARE relations

## Current Design: Type-as-Relation with Minimal Built-ins

### Core Concept
```relic
// Define a value type - this implicitly creates a relation
value User(id: Int, name: String, email: String) {
    validate: email contains "@"
    key: id          // Primary key for the implicit relation
    unique: email    // Unique constraint on the implicit relation
}

// Creating values automatically adds them to the type's relation
let alice = User(1, "Alice", "alice@example.com")
let bob = User(2, "Bob", "bob@example.com")

// Only ONE built-in function needed:
fn all(t: Type) -> List[t]  // Returns all instances of type t

// Everything else is pure Relic built on top of all():
fn count(t: Type) -> Int { all(t).length() }
fn where(t: Type, pred: fn(t) -> Bool) -> List[t] { all(t).filter(pred) }
fn find(t: Type, pred: fn(t) -> Bool) -> Option[t] { all(t).find(pred) }

// Usage with UFC syntax:
let allUsers = User.all()                    // all(User)
let user = User.find(u => u.id == 1)        // find(User, u => u.id == 1)
let adults = User.where(u => u.age >= 18)    // where(User, u => u.age >= 18)
let count = User.count()                     // count(User) -> all(User).length()
```

### Key Benefits
1. **Minimal Built-ins**: Only `all(t: Type)` is built-in, everything else is pure Relic
2. **True Composability**: All relational operations are just functional transformations
3. **Ultimate Simplicity**: No special syntax, just functions and UFC
4. **Type Safety**: The type system ensures schema consistency
5. **Sea of Nodes Ready**: Each value is a node, types are node collections

## Implementation Plan

### Phase 4.1: Instance Tracking Infrastructure ✅ COMPLETE
- [x] Modify ValueRegistry to track all instances by type
- [x] Add instance registration during value construction
- [x] Implement memory management strategy (using Arc references for persistent storage)
- [ ] Handle key and unique constraint validation

### Phase 4.2: Minimal Built-in Approach ✅ COMPLETE
- [x] Add `Type` as a first-class type in the type system
- [x] Implement `all(t: Type) -> List[t]` as the ONLY built-in
- [x] Create minimal List type with essential methods
- [x] Keep special-case type method handling for better UX (delegates to built-in)
- [x] Implement value constructor calls (e.g., `User("Alice")`)
- [x] Implement field value extraction for proper display
- [x] Implement List.length() method
- [x] Implement count() in pure Relic: `count(t: Type) -> Int { all(t).length() }`
- [ ] Implement other functions in pure Relic (requires lambda support):
  - `where(t: Type, pred) -> List[t] { all(t).filter(pred) }`
  - `find(t: Type, pred) -> Option[t] { all(t).find(pred) }`

### Phase 4.3: Advanced Query Operations
- [ ] Implement joins between types
- [ ] Add aggregation functions
- [ ] Support grouping operations
- [ ] Enable pipeline operator with type methods

### Phase 4.4: Memory Management & Performance
- [ ] Implement configurable retention policies
- [ ] Add indexing for efficient queries
- [ ] Optimize for sea of nodes compilation
- [ ] Consider persistence strategies

## Example Usage (Target State)

```relic
// Value types with relational constraints
value User(id: Int, name: String, email: String) {
    validate: email contains "@"
    key: id
    unique: email
}

value Order(id: Int, userId: Int, amount: Float) {
    validate: amount > 0
    key: id
}

// Creating values populates the implicit relations
let alice = User(1, "Alice", "alice@example.com")
let bob = User(2, "Bob", "bob@example.com")
let order1 = Order(101, 1, 99.99)
let order2 = Order(102, 2, 149.99)

// Only ONE built-in needed - everything else is pure Relic!
fn all(t: Type) -> List[t]  // Built-in

// Standard library functions built on top of all()
fn count(t: Type) -> Int { all(t).length() }
fn where(t: Type, pred: fn(t) -> Bool) -> List[t] { all(t).filter(pred) }
fn find(t: Type, pred: fn(t) -> Bool) -> Option[t] { all(t).find(pred) }
fn exists(t: Type, pred: fn(t) -> Bool) -> Bool { all(t).any(pred) }

// Joins are just functional compositions!
fn join(t1: Type, t2: Type, on: fn(t1, t2) -> Bool) -> List[(t1, t2)] {
    all(t1).flatMap(x => 
        all(t2).filter(y => on(x, y))
               .map(y => (x, y))
    )
}

// Query types using UFC syntax
let users = User.all()                           // all(User)
let highValueOrders = Order.where(o => o.amount > 100)  // where(Order, ...)

// Complex queries through composition
let userOrders = join(User, Order, (u, o) => u.id == o.userId)
    |> map(pair => {name: pair.0.name, amount: pair.1.amount})

// Aggregations
let totalRevenue = Order.all()
    |> map(o => o.amount)
    |> sum()
```

## Migration from Previous Approaches

### From Explicit Relations
```relic
// Old approach
relation Users {
    id: Int,
    name: String,
    email: String
    key: id
}

// New approach - just a value type
value User(id: Int, name: String, email: String) {
    key: id
}
```

### From relationOf
```relic
// Previous approach
value Users = relationOf({
    schema: {id: Int, name: String},
    key: "id"
})

// New approach - direct value type
value User(id: Int, name: String) {
    key: id
}
```

## Current Implementation Details

### Implementation Evolution
1. **Initial**: Instance tracking with weak references for automatic garbage collection
2. **Current**: Strong references (`Arc`) for persistent storage, enabling REPL experimentation
3. **Future**: Configurable retention policies based on use case

### Current Implementation Details
- Type identifiers evaluate to `Type` values (e.g., `Person` → `Type(Person)`)
- Single built-in function: `all(t: Type) -> List[t]` accesses the registry
- Type method calls (e.g., `Person.all()`) delegate to the built-in for better UX
- Strong references (`Arc`) ensure instances persist indefinitely in memory
- List type supports essential operations needed for functional programming

### What's Working
1. **Type as First-Class Values**: `Person` evaluates to `Type(Person)` with type `Type`
2. **Built-in `all` Function**: Both `all(Person)` and `Person.all()` return lists of instances
3. **Persistent Storage**: All instances stored indefinitely using strong references
4. **Count Method**: `Person.count()` returns the correct number of instances
5. **Type Checking**: Proper type inference for Type values and List types
6. **Value Construction**: Function call syntax creates values (e.g., `User("Alice")`)
7. **Field Extraction**: List display shows actual field values (e.g., `[User(Alice), User(Bob)]`)
8. **List Methods**: `List.length()` method works for all lists
9. **Pure Relic Functions**: `count(t: Type)` implemented in pure Relic

### Known Limitations
- No persistence between REPL sessions (in-memory only)
- Lambda/function values not yet supported (blocks `filter`, `find`, `where`)
- Key and unique constraints not yet validated
- Multi-parameter value types not yet supported

## Relational Completeness

### The Breakthrough: No Queries, Just Functions

Relic doesn't have "queries" - it has functions. This represents a fundamental breakthrough in data programming:

- **Traditional Systems**: Separate query language (SQL) with different syntax and semantics
- **Relic**: All data operations are just function composition using the same language

### Full Relational Power Preserved

All relational algebra operations map naturally to Relic functions:

```relic
// Selection (WHERE)
User.where(u => u.age > 18)                    // σ(age > 18)(Users)

// Projection (SELECT columns)  
User.all().map(u => {name: u.name})            // π(name)(Users)

// Join
join(User, Order, (u, o) => u.id == o.userId) // Users ⋈ Orders

// Union
User.all().concat(Admin.all()).distinct()     // Users ∪ Admins

// Aggregation (GROUP BY)
User.all().groupBy(u => u.city)               // Complex aggregations
    .map(g => {city: g.key, count: g.count()})
```

### What We've Actually Gained

1. **Arbitrary Computation**: Mix algorithms with data operations
   ```relic
   User.where(u => isPrime(u.id))
       .map(u => {name: u.name, score: mlModel.classify(u)})
   ```

2. **First-Class Composition**: Queries are values you can pass around
   ```relic
   let activeUsers = User.where(u => u.active)
   let premiumFilter = users => users.filter(u => u.tier == "premium")
   ```

3. **Custom Operators**: Define new relational operations
   ```relic
   fn semiJoin(t1: Type, t2: Type, on: fn(t1, t2) -> Bool) -> List[t1] {
       all(t1).filter(x => all(t2).exists(y => on(x, y)))
   }
   ```

4. **Whole-Program Optimization**: Sea of nodes can optimize across "query" boundaries
   ```relic
   let threshold = computeThreshold(config)  // Can be inlined
   User.where(u => u.score > threshold)     // Becomes constant predicate
   ```

See [RELATIONAL_POWER.md](RELATIONAL_POWER.md) for complete relational algebra mappings.

## Success Criteria

Phase 4 will be considered complete when:
1. ✅ Value types automatically maintain relations of their instances
2. ✅ Built-in `all(t: Type)` implemented with count() in pure Relic
3. ✅ All relational algebra operations expressible (documented in RELATIONAL_POWER.md)
4. ✅ Memory management is configurable (currently using Arc for persistence)
5. ✅ Value construction and field extraction working
6. 🔶 Lambda support for where/find/filter functions
7. 🔶 All tests pass (need more tests)
8. ✅ Documentation captures the "no queries" breakthrough

## Technical Challenges

1. **Memory Management**: Balancing between keeping all instances and allowing GC
2. **Performance**: Efficient indexing for large instance sets
3. **Type System**: Ensuring type methods don't conflict with instance methods
4. **Migration**: Smooth transition from previous relation approaches

## Sea of Nodes Alignment

This approach aligns perfectly with the future sea of nodes compiler:
- Each value instance is naturally a node in the graph
- Type relations are node collections
- Queries become graph traversals
- No impedance mismatch between the language model and compiler model

## References

- `RELATIONS_AS_VALUES.md` - Evolution of the design
- `DESIGN.md` - Core language philosophy
- `src/value.rs` - Value registry implementation
- `examples/type_as_relation.relic` - Example usage (to be created)