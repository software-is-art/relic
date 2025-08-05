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

### 1. Relation Values and Storage
- [x] Design relation schema syntax
- [ ] Implement Relation as a value type
- [ ] Create in-memory storage backend for relations
- [ ] Build immutable fact storage (inspired by Datomic)
- [ ] Add temporal tracking to all facts
- [ ] Create relation constructor functions

### 2. Query Operations as Functions
- [ ] Implement core query functions using multiple dispatch:
  ```relic
  fn where(rel: Relation, predicate: Expression) -> Relation { ... }
  fn select(rel: Relation, fields: List) -> Relation { ... }
  fn join(left: Relation, right: Relation, on: Expression) -> Relation { ... }
  ```
- [ ] Leverage UFC for natural query syntax
- [ ] Use multiple dispatch for optimization based on relation types
- [ ] Create composable query operations
- [ ] Build standard library of query functions

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
All operations will be implemented as regular functions, not special syntax:

#### MVP Operations (implement first)
- [ ] `fn where(rel: Relation, pred: Expression) -> Relation`
- [ ] `fn select(rel: Relation, fields: List) -> Relation`
- [ ] `fn join(left: Relation, right: Relation, on: Expression) -> Relation`
- [ ] `fn limit(rel: Relation, n: Int) -> Relation`

#### Essential Operations (implement second)
- [ ] `fn distinct(rel: Relation) -> Relation`
- [ ] `fn union(rel1: Relation, rel2: Relation) -> Relation`
- [ ] `fn group(rel: Relation, by: List) -> GroupedRelation`
- [ ] `fn sort(rel: Relation, by: List) -> Relation`
- [ ] `fn offset(rel: Relation, n: Int) -> Relation`

#### Aggregation Functions
- [ ] `fn count(grouped: GroupedRelation) -> Relation`
- [ ] `fn sum(grouped: GroupedRelation, field: String) -> Relation`
- [ ] `fn avg(grouped: GroupedRelation, field: String) -> Relation`
- [ ] `fn min(grouped: GroupedRelation, field: String) -> Relation`
- [ ] `fn max(grouped: GroupedRelation, field: String) -> Relation`

### 5. Integration with Existing Features
- [ ] Ensure relations work with value types
- [ ] Support multiple dispatch on relation operations
- [ ] Enable UFC syntax for relation methods
- [ ] Integrate with pattern matching
- [ ] Support pipeline operator for query composition

## Design Philosophy: Relations as Values

The key insight is that relations should be implemented as regular Relic values, with query operations as normal functions. This approach:

1. **Maintains Language Consistency** - Everything uses the same mechanisms (values, functions, UFC)
2. **Enables User Extensions** - Users can define custom query operations
3. **Leverages Multiple Dispatch** - Different relation types can have optimized implementations
4. **Simplifies the Parser** - No special syntax rules needed
5. **Promotes Composability** - Queries are just function compositions

Example of how it works:
```relic
// Relations are values
value Relation(data: InternalRelationData) {
    // Internal storage implementation
}

// Query operations are functions with multiple dispatch
fn where(rel: Relation, predicate: Expression) -> Relation {
    // Filter implementation
}

// UFC makes it natural
users.where(age > 21).select([name, email])
// Desugars to: select(where(users, age > 21), [name, email])
```

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
Queries are just function calls with UFC syntax:

```relic
// Simple query - just a function call
users.where(age > 21)

// Chaining - natural with UFC
users
  .where(city == "New York")
  .where(age >= 21)
  .select([name, email])

// Pipeline operator also works
users |> where(age > 21) |> select([name])

// Multiple dispatch optimizes based on types
fn join(h: HashRelation, s: SortedRelation) -> Relation {
    // Use hash join algorithm
}

fn join(s1: SortedRelation, s2: SortedRelation) -> Relation {
    // Use merge join algorithm
}

// Users can define custom operations
fn topN(rel: Relation, n: Int, by: String) -> Relation {
    rel.sort([by]).limit(n)
}

users.topN(10, "score")
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