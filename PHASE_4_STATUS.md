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
Phase 4 is **~50% complete** - We've implemented the core Type-as-Relation infrastructure with instance tracking and basic query operations (count, all).

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
- [x] Implement memory management strategy (using Arc/Weak references)
- [ ] Handle key and unique constraint validation

### Phase 4.2: Minimal Built-in Approach - IN PROGRESS
- [ ] Add `Type` as a first-class type in the type system
- [ ] Implement `all(t: Type) -> List[t]` as the ONLY built-in
- [ ] Create minimal List type with essential methods
- [ ] Remove special-case type method handling
- [ ] Implement other functions in pure Relic:
  - `count(t: Type) -> Int { all(t).length() }`
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

### What's Working
1. **Instance Tracking**: ValueRegistry tracks all instances using `Arc<RwLock<HashMap<String, Vec<Weak<dyn ValueObject>>>>>`
2. **Automatic Registration**: When `construct()` is called, instances are automatically registered with weak references
3. **Memory Management**: Weak references allow instances to be garbage collected when no longer referenced

### Current Special-Case Implementation (To Be Replaced)
- Type methods handled as special cases in evaluator
- Type checker has special logic for type method calls
- This will be replaced with the minimal built-in approach

### Next Steps: Minimal Built-in Approach
1. **Add Type type**: Type identifiers will evaluate to Type values
2. **Single built-in**: `all(t: Type) -> List[t]` accesses the registry
3. **Pure Relic functions**: Everything else built on top of `all()`
4. **Remove special cases**: No more type method handling in evaluator/typechecker

### Known Limitations
- Instances created without storing in variables are immediately dropped (expected with weak refs)
- No persistence between REPL sessions
- List type not yet implemented
- Current implementation uses special cases (to be removed)

## Success Criteria

Phase 4 will be considered complete when:
1. ✅ Value types automatically maintain relations of their instances
2. 🔶 Type-level query methods work: `all()` ✅, `where()` ❌, `find()` ❌, `count()` ✅
3. ❌ Joins between types are supported
4. ✅ Memory management is configurable (using Arc/Weak)
5. 🔶 All tests pass (need more tests)
6. 🔶 Documentation is updated (in progress)

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