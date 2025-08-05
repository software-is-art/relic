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
Phase 4 is **~30% complete** - We've designed the type-as-relation architecture and are ready to implement instance tracking and query operations.

### Design Evolution Timeline
1. ✅ **First approach**: Special `relation` syntax with code generation
2. ✅ **Second approach**: Relations as value constructors (`relationOf`)
3. ✅ **Current approach**: Type-as-Relation - types ARE relations

## Current Design: Type-as-Relation

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

// Query the type directly - no special relation needed
let allUsers = User.all()                    // Get all User instances
let user = User.find(u => u.id == 1)        // Find single instance
let adults = User.where(u => u.age >= 18)    // Filter instances
let count = User.count()                     // Count instances
```

### Key Benefits
1. **Ultimate Simplicity**: No relation syntax or special types
2. **True Unity**: Values and relations are the same thing
3. **Type Safety**: The type system ensures schema consistency
4. **Sea of Nodes Ready**: Each value is a node, types are node collections

## Implementation Plan

### Phase 4.1: Instance Tracking Infrastructure
- [ ] Modify ValueRegistry to track all instances by type
- [ ] Add instance registration during value construction
- [ ] Implement memory management strategy (strong vs weak references)
- [ ] Handle key and unique constraint validation

### Phase 4.2: Type-Level Query Methods
- [ ] Implement `Type.all()` - return all instances of a type
- [ ] Implement `Type.where(predicate)` - filter instances
- [ ] Implement `Type.find(predicate)` - find single instance
- [ ] Implement `Type.count()` - count instances
- [ ] Add support for type methods in evaluator

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

// Query types directly
let users = User.all()
let highValueOrders = Order.where(o => o.amount > 100)

// Join types naturally
let userOrders = User.all()
    |> join(Order.all(), (u, o) => u.id == o.userId)
    |> select(u => {name: u.name, amount: o.amount})

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

## Success Criteria

Phase 4 will be considered complete when:
1. Value types automatically maintain relations of their instances
2. Type-level query methods work: `all()`, `where()`, `find()`, `count()`
3. Joins between types are supported
4. Memory management is configurable
5. All tests pass
6. Documentation is updated

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