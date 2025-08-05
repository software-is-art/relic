# Phase 4: Functional-Relational Core Status

## Overview
Phase 4 introduces the functional-relational programming paradigm to Relic, implementing relations as the sole storage for essential state following the "Out of the Tar Pit" architecture.

## Summary
Phase 4 is **~10% complete** - AST infrastructure is in place. This phase will establish relations as first-class citizens in Relic, enabling type-safe relational operations with value objects.

### Completed So Far
- ✅ Added relation keywords to lexer (`relation`, `select`, `join`, `group`, `sort`, `on`, `by`, `key`, `foreign`, `references`)
- ✅ Created AST nodes for relation declarations (`RelationDeclaration`, `RelationField`, `RelationConstraint`)
- ✅ Created AST nodes for relational queries (`QueryExpression`, `SelectItem`, `AggregateItem`, etc.)
- ✅ Added placeholder implementations in compiler, evaluator, type checker, and specialization
- ✅ Project builds successfully with all Phase 4 infrastructure

## Planned Tasks 📋

### 1. Essential State as Relations
- [x] Design relation type syntax with value object schemas
- [ ] Implement relations as sole storage for essential state
- [ ] Create relation types with value object schemas
- [ ] Build immutable fact storage (inspired by Datomic)
- [ ] Add time-based queries for historical data
- [ ] Enforce user-input data only in essential state

### 2. Relational Algebra with Value Types
- [ ] Implement type-safe relational operations
- [ ] Add Malloy-inspired source modeling:
  ```relic
  users 
    |> where(age > 21)
    |> join(orders, on: userId)
    |> group(by: city)
    |> select(city, orderCount: count())
  ```
- [ ] Create relationship-aware computing to prevent fan traps
- [ ] Build composable query blocks as first-class values
- [ ] Add automatic optimization based on value types

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

#### MVP Operations (implement first)
- [ ] `where` - Filter rows based on predicates
- [ ] `select` - Project specific columns
- [ ] `join` - Inner join relations
- [ ] `limit` - Restrict result size

#### Essential Operations (implement second)
- [ ] `distinct` - Remove duplicate rows
- [ ] `union` - Combine relations (same schema)
- [ ] `group` - Group by columns with aggregations
- [ ] `sort` - Order results by columns
- [ ] `offset` - Skip rows for pagination

#### Set Operations
- [ ] `intersect` - Common rows between relations
- [ ] `difference` - Rows in first but not second relation
- [ ] `unionAll` - Union without removing duplicates

#### Advanced Join Operations
- [ ] `leftJoin` - Include all rows from left relation
- [ ] `rightJoin` - Include all rows from right relation
- [ ] `fullJoin` - Include all rows from both relations
- [ ] `naturalJoin` - Join on common column names
- [ ] `semiJoin` - Rows from left that have match in right
- [ ] `antiJoin` - Rows from left that have no match in right

#### Temporal Operations (first-class support)
- [ ] `asOf` - Query state at specific timestamp
- [ ] `history` - View all versions of an entity
- [ ] `between` - Query state between timestamps
- [ ] `validTime` - Query based on valid time
- [ ] `transactionTime` - Query based on transaction time

#### Aggregation Functions
- [ ] `count` - Count rows
- [ ] `sum` - Sum numeric values
- [ ] `avg` - Average numeric values
- [ ] `min` - Minimum value
- [ ] `max` - Maximum value
- [ ] Custom aggregates via multiple dispatch

### 5. Integration with Existing Features
- [ ] Ensure relations work with value types
- [ ] Support multiple dispatch on relation operations
- [ ] Enable UFC syntax for relation methods
- [ ] Integrate with pattern matching
- [ ] Support pipeline operator for query composition

## Relic's Relational Algebra Principles

Relic's relational algebra is designed from first principles, leveraging the language's unique features:

1. **Pure Functional Operations** - All relational operations are pure functions that return new relations without modifying existing ones. This enables:
   - Time-travel queries by preserving all states
   - Safe concurrent operations
   - Easy reasoning about query behavior

2. **UFC Syntax for Natural Query Flow** - Queries read naturally left-to-right:
   ```relic
   users
     .where(age > 21)
     .join(orders)
     .select(name, totalSpent: sum(amount))
     .sort(totalSpent.desc())
   ```

3. **Multiple Dispatch for Extensibility** - Operations dispatch on relation types:
   ```relic
   // Optimized joins based on relation storage
   fn join(r1: HashIndexedRelation, r2: SortedRelation) -> Relation {
     // Use hash join algorithm
   }
   
   // Custom aggregates via multiple dispatch
   fn aggregate(data: Relation, op: MedianOp) -> Float {
     // User-defined median calculation
   }
   ```

4. **Value Types Throughout** - No NULL values; use value types and pattern matching:
   ```relic
   // Instead of NULL, use value types
   value OptionalAge = Some(Int) | None
   
   users
     .select(name, age: OptionalAge)
     .where(age matches Some(n) where n > 21)
   ```

5. **Temporal as First-Class** - Time is built into the relational model:
   ```relic
   // All relations automatically track history
   users.asOf("2024-01-01")
   users.history().where(field == "email")
   
   // Bitemporal queries combine valid and transaction time
   orders
     .validAt("2024-01-01")
     .transactedBefore("2024-01-15")
   ```

6. **Composability via Pipelines** - All operations compose seamlessly:
   ```relic
   let activeUsers = users.where(active == true)
   let recentOrders = orders.where(date > lastMonth)
   
   activeUsers
     |> join(recentOrders)
     |> group(by: userId)
     |> having(count() > 5)
   ```

7. **Type-Safe Schema Evolution** - Relations evolve with type safety:
   ```relic
   // Migrations are type transformations
   relation UsersV2 extends UsersV1 {
     phoneNumber: PhoneNumber  // New field with value type
     
     migrate: phoneNumber = PhoneNumber.unknown()
   }
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
Queries leverage UFC syntax and pipeline operators for natural composition:

```relic
// Simple query with UFC
users.where(age > 21)

// Equivalent using pipeline
users |> where(age > 21)

// Complex query combining UFC and pipelines
users
  .where(city == "New York")
  .join(orders, on: users.id == orders.userId)
  .group(by: email)
  .select(email, totalSpent: sum(total))
  .sort(totalSpent.desc())
  .limit(10)

// Multiple dispatch enables custom operations
users
  .where(age > 21)
  .mapValues(normalizeEmail)  // Custom function via dispatch
  .distinct()

// Temporal queries are natural
users
  .asOf(yesterday)
  .where(status == "active")
  .history()
  .select(email, status, validFrom, validTo)
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

### Phase 4.1: Core Infrastructure (Weeks 1-2)
1. Design and implement relation type AST nodes
2. Extend parser to handle relation declarations
3. Create basic relation storage mechanism
4. Implement simple where/select operations

### Phase 4.2: Advanced Operations (Weeks 3-4)
1. Implement join operations with type safety
2. Add grouping and aggregation functions
3. Create query optimizer for basic operations
4. Support sorting and pagination

### Phase 4.3: Time and History (Week 5)
1. Add temporal aspects to relations
2. Implement asOf queries
3. Create history tracking
4. Build transaction support

### Phase 4.4: Integration and Testing (Week 6)
1. Integrate with existing type system
2. Ensure multiple dispatch works with relations
3. Create comprehensive test suite
4. Document all features

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
- Lexer recognizes all relation keywords
- AST structures for relations and queries defined
- Placeholder implementations allow project to build
- Basic infrastructure for Phase 4 is ready

### In Progress
- Parser implementation for relation declarations
- In-memory storage design for relations

### Blocked
- None

## Next Steps

1. ~~**Create relation AST nodes** - Define AST structures for relation declarations and queries~~ ✅ COMPLETED
2. ~~**Extend lexer** - Add keywords: `relation`, `where`, `select`, `join`, `group`, `sort`~~ ✅ COMPLETED
3. **Design storage layer** - Plan how relations will be stored in memory (IN PROGRESS)
4. **Implement parser** - Parse relation declarations and basic queries (NEXT)
5. **Create evaluator** - Execute simple where/select queries

## Test Plan

### Unit Tests
- Relation declaration parsing
- Query operation parsing
- Basic query execution
- Type checking for relations
- Join type safety

### Integration Tests
- Relations with value types
- Multiple dispatch on relations
- Pipeline composition
- Complex queries
- Performance benchmarks

### Example Files
- `relations.relic` - Basic relation examples
- `queries.relic` - Query operation examples
- `joins.relic` - Join and aggregation examples
- `temporal.relic` - Time-based query examples

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