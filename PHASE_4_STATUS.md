# Phase 4: Functional-Relational Core Status

## Overview
Phase 4 introduces the functional-relational programming paradigm to Relic, implementing relations as the sole storage for essential state following the "Out of the Tar Pit" architecture.

## Major Design Change: Pure Functional Approach
After analysis, we've decided to implement relations as regular values and query operations as normal functions, leveraging Relic's existing features (UFC, multiple dispatch) instead of adding special parser support. This aligns better with Relic's philosophy of functional composition.

## Summary
Phase 4 is **~15% complete** - Relation schema declarations are parsed, and we've refactored to remove special query syntax in favor of pure functional composition.

### Completed So Far
- ✅ Added relation keywords to lexer (`relation`, `key`, `foreign`, `references`, `unique`)
- ✅ Created AST nodes for relation declarations (`RelationDeclaration`, `RelationField`, `RelationConstraint`)
- ✅ Implemented parser for relation schema declarations
- ✅ **Removed special query syntax** - queries will be regular function calls
- ✅ Updated examples to show functional approach to queries
- ✅ Project builds successfully with simplified architecture

## Planned Tasks 📋

### 1. Value Generation from Relations
- [x] Design relation schema syntax
- [ ] Generate row value types from relation schemas
- [ ] Generate relation value types (collections)
- [ ] Generate field constants with type information
- [ ] Create constructor functions for relations
- [ ] Build validation for row value construction

### 2. Storage and Implementation
- [ ] Create in-memory storage backend for relation values
- [ ] Implement copy-on-write for efficient immutability
- [ ] Build indexes as part of relation values
- [ ] Add temporal tracking (valid time & transaction time)
- [ ] Support different storage strategies via multiple dispatch

### 3. Query Operations as Typed Functions
- [ ] Implement core query functions with proper types:
  ```relic
  fn where<T>(rel: Relation<T>, pred: Predicate<T>) -> Relation<T>
  fn select<T, U>(rel: Relation<T>, fields: Fields<T, U>) -> Relation<U>
  fn join<T, U>(left: Relation<T>, right: Relation<U>, on: JoinPred<T, U>) -> Relation<T, U>
  ```
- [ ] Type-safe field references instead of strings
- [ ] Predicate functions that work with row types
- [ ] Query optimization through multiple dispatch

### 3. Type-Level Relationships
- [ ] Encode relationships through type dependencies:
  ```relic
  value Post {
    author: User  // Direct dependency on User type
    content: String
    where author exists in Users
  }
  ```
- [ ] Replace foreign keys with type-level constraints
- [ ] Implement compile-time relationship validation
- [ ] Create inference for transitive relationships

### 4. Complete Relational Operations
All operations work with typed relations and row values:

#### MVP Operations (implement first)
- [ ] `fn where<T>(rel: Relation<T>, pred: T -> Bool) -> Relation<T>`
- [ ] `fn select<T, U>(rel: Relation<T>, ...fields: Field<?, T>) -> Relation<U>`
- [ ] `fn join<T, U, V>(left: Relation<T>, right: Relation<U>, on: (T, U) -> Bool) -> Relation<V>`
- [ ] `fn limit<T>(rel: Relation<T>, n: Int) -> Relation<T>`

#### Row Access Operations
- [ ] `fn find<T>(rel: Relation<T>, pred: T -> Bool) -> Option<T>`
- [ ] `fn first<T>(rel: Relation<T>) -> Option<T>`
- [ ] `fn forEach<T>(rel: Relation<T>, f: T -> Unit) -> Unit`
- [ ] `fn map<T, U>(rel: Relation<T>, f: T -> U) -> Relation<U>`

#### Set Operations (type-safe)
- [ ] `fn union<T>(r1: Relation<T>, r2: Relation<T>) -> Relation<T>`
- [ ] `fn intersect<T>(r1: Relation<T>, r2: Relation<T>) -> Relation<T>`
- [ ] `fn difference<T>(r1: Relation<T>, r2: Relation<T>) -> Relation<T>`
- [ ] `fn distinct<T>(rel: Relation<T>) -> Relation<T>`

#### Aggregation (returns values, not relations)
- [ ] `fn count<T>(rel: Relation<T>) -> Int`
- [ ] `fn sum<T>(rel: Relation<T>, field: Field<Number, T>) -> Number`
- [ ] `fn avg<T>(rel: Relation<T>, field: Field<Number, T>) -> Float`
- [ ] `fn min<T, U>(rel: Relation<T>, field: Field<U, T>) -> Option<U>`
- [ ] `fn max<T, U>(rel: Relation<T>, field: Field<U, T>) -> Option<U>`

### 5. Integration with Existing Features
- [ ] Ensure relations work with value types
- [ ] Support multiple dispatch on relation operations
- [ ] Enable UFC syntax for relation methods
- [ ] Integrate with pattern matching
- [ ] Support pipeline operator for query composition

## Design Philosophy: Relations as Value-Generating Constructs

The key insight is that relation declarations should generate multiple interconnected value types, maintaining Relic's "everything is a value" philosophy while providing type safety and relational semantics.

### What a Relation Declaration Generates

When you declare a relation, it creates:

1. **A Row Value Type** - Each record is an immutable value object
2. **A Relation Value Type** - The collection itself is an immutable value
3. **Field Constants** - Type-safe references to fields
4. **Constructor Functions** - For creating relation instances

```relic
// This declaration:
relation Users {
    id: UserId,
    name: String,
    age: Int
    
    key: id
}

// Automatically generates:

// 1. Row value type
value User(id: UserId, name: String, age: Int) {
    // Each row is a validated value
}

// 2. Relation value type  
value UsersRelation(rows: InternalStorage<User>) {
    // Immutable collection of User values
}

// 3. Field references
const User.id: Field<UserId, User>
const User.name: Field<String, User>
const User.age: Field<Int, User>

// 4. Constructor function
fn Users(rows: List<User>) -> UsersRelation {
    // Creates a new relation instance
}
```

### Why This Approach

1. **Parse Don't Validate** - Each row is a validated value object
2. **Immutability** - Both relations and rows are immutable values
3. **Type Safety** - Strong types for relations, rows, and fields
4. **Composability** - Relations transform functionally
5. **Multiple Dispatch** - Different storage strategies possible

## Design Decisions

### Syntax Design
Relations will be declared similarly to value types but with relational semantics:

```relic
relation Users {
  id: UserId,
  email: EmailAddress,
  age: Int,
  city: String
  
  key: id  // Primary key
  unique: email  // Uniqueness constraint
}

relation Orders {
  id: OrderId,
  userId: UserId,
  total: Money,
  date: Date
  
  key: id
  foreign: userId references Users.id
}
```

### Query Syntax
With the value-generating approach, queries become type-safe operations on relation values:

```relic
// Start with a relation value
let users: UsersRelation = loadUsers()

// Query operations transform relation values
let adults: UsersRelation = users.where(User.age > 18)

// Field references are type-safe
users
  .where(User.city == "New York")
  .where(User.age >= 21)
  .select(User.name, User.email)

// Can extract individual row values
match users.find(User.id == userId) {
    Some(user: User) => user.name,  // user is a value object
    None => "Unknown"
}

// Multiple dispatch on relation types
fn join(h: HashIndexed<User>, s: SortedRelation<Order>) -> JoinedRelation {
    // Use hash join algorithm
}

// Query functions work with typed relations
fn where(rel: UsersRelation, pred: Predicate<User>) -> UsersRelation {
    UsersRelation(rel.rows.filter(pred))
}

// Users can define type-safe custom operations
fn topUsers(users: UsersRelation, n: Int) -> UsersRelation {
    users.sort(User.score.desc()).limit(n)
}
```

### Immutability and Time
Time is a first-class concept in Relic's relational model. Every fact is automatically versioned with both valid time (when the fact was true in the real world) and transaction time (when the fact was recorded):

```relic
// Query state at a specific point in time
users.asOf("2024-01-01").where(age > 21)

// View complete history of changes
users
  .history()
  .where(email == "user@example.com")
  .select(email, age, validFrom, validTo, transactionTime)

// Bitemporal queries
orders
  .validAt("2024-01-01")        // When the order was placed
  .transactedBefore("2024-01-15") // As known by this date
  .where(status == "completed")

// Time-travel debugging
let oldState = appState.asOf(beforeBugIntroduced)
let newState = appState.asOf(afterBugIntroduced)
oldState.difference(newState)  // See what changed

// Temporal joins
users
  .asOf(orderDate)
  .join(orders, on: users.id == orders.userId)
  .select(userName, orderTotal)
```

This temporal model enables:
- Audit trails without extra code
- Time-travel debugging
- Historical reporting
- Retroactive corrections
- Compliance with data regulations

## Implementation Strategy

### Phase 4.1: Core Infrastructure (Current)
1. ✅ Design and implement relation declaration AST nodes
2. ✅ Extend parser to handle relation declarations
3. ✅ Refactor to pure functional approach (no special query syntax)
4. [ ] Create Relation value type
5. [ ] Build in-memory storage mechanism

### Phase 4.2: Basic Query Operations
1. [ ] Implement `where` and `select` functions
2. [ ] Add UFC support for natural syntax
3. [ ] Create basic join operation
4. [ ] Test query composition

### Phase 4.3: Advanced Operations
1. [ ] Add grouping and aggregation functions
2. [ ] Implement sorting and pagination
3. [ ] Use multiple dispatch for optimization
4. [ ] Build query performance tests

### Phase 4.4: Temporal Support
1. [ ] Add temporal tracking to all facts
2. [ ] Implement asOf queries
3. [ ] Create history functions
4. [ ] Build transaction support

## Design Advantages

Relic's pure relational algebra design avoids common pitfalls:

### No NULL Confusion
- Value types eliminate NULL handling complexity
- Pattern matching provides explicit handling of optional values
- No three-valued logic confusion

### No Implicit Behavior
- All operations are explicit and composable
- No hidden joins or magic variables
- Type system prevents nonsensical operations

### Relationship Safety
- Multiple dispatch prevents fan traps automatically
- Type-safe joins ensure schema compatibility
- Compile-time verification of field access

### Performance Transparency
- Multiple dispatch selects optimal algorithms
- Pure functions enable automatic parallelization
- Immutability allows aggressive caching

## Current Status

### What's Working
- ✅ Lexer recognizes relation keywords
- ✅ Relation declaration parser implemented and tested
- ✅ Simplified architecture - queries are just function calls
- ✅ Examples updated to show functional approach
- ✅ Project builds successfully

### Next Steps
1. **Create Relation value type** - Implement relations as first-class values
2. **Design storage layer** - In-memory storage with immutable facts
3. **Implement basic query functions** - `where`, `select`, `join`
4. **Add temporal support** - Time-travel queries from the start

## Test Plan

### Unit Tests
- ✅ Relation declaration parsing
- [ ] Relation value type construction
- [ ] Query function implementations
- [ ] Type checking for relations
- [ ] Multiple dispatch on query operations

### Integration Tests
- [ ] Relations with value types
- [ ] Query composition with UFC
- [ ] Pipeline operator with queries
- [ ] Performance benchmarks
- [ ] Temporal query tests

### Example Files
- ✅ `relations.relic` - Shows functional approach to queries
- [ ] `query_functions.relic` - Query function implementations
- [ ] `temporal_queries.relic` - Time-based query examples
- [ ] `custom_queries.relic` - User-defined query operations

## Success Criteria

Phase 4 will be considered complete when:
1. Relations can be declared with value type schemas
2. Basic queries (where, select) work correctly
3. Joins maintain type safety
4. Aggregation functions are implemented
5. Query composition via pipelines works
6. All tests pass
7. Documentation is complete

## Risks and Mitigation

1. **Performance of naive implementation** - Start simple, optimize later
2. **Query optimization complexity** - Begin with rule-based optimizer
3. **Type inference for complex queries** - Explicit types initially
4. **Integration with existing features** - Incremental integration

## References

- "Out of the Tar Pit" paper for functional-relational architecture
- Datomic for immutable fact storage
- Malloy for query composition patterns
- SQL for relational algebra foundations