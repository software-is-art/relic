# Relational Power in Relic: Beyond Queries

## The Philosophical Breakthrough

Relic doesn't have "queries" - it has functions. This isn't a limitation; it's a breakthrough. By unifying data manipulation and computation into a single paradigm, we've created something more powerful than traditional query languages while preserving full relational capabilities.

## Key Insight: No Queries, Just Composition

In traditional systems:
```sql
-- This is a "query" - a special sublanguage
SELECT u.name, COUNT(o.id) 
FROM users u 
LEFT JOIN orders o ON u.id = o.user_id 
WHERE u.age > 18 
GROUP BY u.name
```

In Relic:
```relic
// This is just function composition - no special syntax
User.where(u => u.age > 18)
    .leftJoin(Order, (u, o) => u.id == o.userId)
    .groupBy(pair => pair.0.name)
    .map(group => {name: group.key, orderCount: group.values.count()})
```

The crucial difference: In Relic, these are just regular functions that compose. There's no boundary between "query language" and "programming language."

## Full Relational Algebra Mapping

### Core Operations

#### Selection (σ) - WHERE
```relic
// Relational: σ(age > 18)(Users)
// SQL: SELECT * FROM users WHERE age > 18
// Relic:
User.where(u => u.age > 18)
// Or explicitly: all(User).filter(u => u.age > 18)
```

#### Projection (π) - SELECT columns
```relic
// Relational: π(name, email)(Users)
// SQL: SELECT name, email FROM users
// Relic:
User.all().map(u => {name: u.name, email: u.email})
```

#### Cartesian Product (×)
```relic
// Relational: Users × Orders
// SQL: SELECT * FROM users, orders
// Relic:
fn product(t1: Type, t2: Type) -> List[(t1, t2)] {
    all(t1).flatMap(x => all(t2).map(y => (x, y)))
}
```

#### Natural Join (⋈)
```relic
// Relational: Users ⋈ Orders
// SQL: SELECT * FROM users JOIN orders ON users.id = orders.user_id
// Relic:
fn join(t1: Type, t2: Type, on: fn(t1, t2) -> Bool) -> List[(t1, t2)] {
    all(t1).flatMap(x => 
        all(t2).filter(y => on(x, y)).map(y => (x, y))
    )
}
```

#### Union (∪)
```relic
// Relational: ActiveUsers ∪ PremiumUsers
// SQL: SELECT * FROM active_users UNION SELECT * FROM premium_users
// Relic:
ActiveUser.all().concat(PremiumUser.all()).distinct()
```

#### Difference (-)
```relic
// Relational: AllUsers - BannedUsers
// SQL: SELECT * FROM users WHERE id NOT IN (SELECT id FROM banned_users)
// Relic:
User.all().filter(u => !BannedUser.exists(b => b.id == u.id))
```

#### Intersection (∩)
```relic
// Relational: AdminUsers ∩ ActiveUsers
// SQL: SELECT * FROM admin_users INTERSECT SELECT * FROM active_users
// Relic:
AdminUser.all().filter(a => ActiveUser.exists(u => u.id == a.id))
```

### Advanced Operations

#### Aggregation (GROUP BY)
```relic
// SQL: SELECT city, COUNT(*), AVG(age) FROM users GROUP BY city
// Relic:
User.all()
    .groupBy(u => u.city)
    .map(group => {
        city: group.key,
        count: group.values.length(),
        avgAge: group.values.map(u => u.age).average()
    })
```

#### Left/Right/Outer Joins
```relic
// Left Join
fn leftJoin(t1: Type, t2: Type, on: fn(t1, t2) -> Bool) -> List[(t1, Option[t2])] {
    all(t1).map(x => {
        let matches = all(t2).filter(y => on(x, y))
        (x, matches.first())
    })
}

// Full Outer Join
fn outerJoin(t1: Type, t2: Type, on: fn(t1, t2) -> Bool) -> List[(Option[t1], Option[t2])] {
    let leftJoined = leftJoin(t1, t2, on).map(pair => (Some(pair.0), pair.1))
    let rightOnly = all(t2).filter(y => !all(t1).exists(x => on(x, y)))
                          .map(y => (None, Some(y)))
    leftJoined.concat(rightOnly)
}
```

#### Window Functions
```relic
// SQL: ROW_NUMBER() OVER (PARTITION BY dept ORDER BY salary DESC)
// Relic:
Employee.all()
    .groupBy(e => e.dept)
    .flatMap(group => 
        group.values
            .sortBy(e => -e.salary)
            .zipWithIndex()
            .map((e, idx) => {...e, rank: idx + 1})
    )
```

#### Recursive Queries
```relic
// SQL: WITH RECURSIVE ... (complex syntax)
// Relic: Just regular recursion!
fn reportsUnder(managerId: Int) -> List[Employee] {
    let direct = Employee.where(e => e.managerId == managerId)
    direct.concat(
        direct.flatMap(e => reportsUnder(e.id))
    )
}
```

## What We've Gained: Beyond SQL

### 1. Arbitrary Computation in "Queries"
```relic
// Impossible in pure SQL without stored procedures
User.where(u => isPrime(u.id))
    .map(u => {
        name: u.name,
        score: complexScoringAlgorithm(u),
        category: ml_model.classify(u)
    })
```

### 2. First-Class Composition
```relic
// Queries are values you can pass around and compose
let activeUsers = User.where(u => u.lastLogin > lastWeek)
let premiumFilter = (users: List[User]) => users.filter(u => u.tier == "premium")

// Compose them
let activePremium = activeUsers |> premiumFilter

// Or create higher-order query functions
fn withAge(minAge: Int) -> fn(List[User]) -> List[User] {
    users => users.filter(u => u.age >= minAge)
}
```

### 3. Custom Relational Operators
```relic
// Define your own relational algebra extensions
fn semiJoin(t1: Type, t2: Type, on: fn(t1, t2) -> Bool) -> List[t1] {
    all(t1).filter(x => all(t2).exists(y => on(x, y)))
}

fn antiJoin(t1: Type, t2: Type, on: fn(t1, t2) -> Bool) -> List[t1] {
    all(t1).filter(x => !all(t2).exists(y => on(x, y)))
}

// Temporal operations
fn asOf(t: Type, time: Timestamp) -> List[t] {
    all(t).filter(x => x.validFrom <= time && x.validTo > time)
}
```

### 4. Type-Safe Heterogeneous Queries
```relic
// Mix different types safely in a single "query"
let results = User.all()
    .flatMap(u => Order.where(o => o.userId == u.id)
                       .map(o => Payment.where(p => p.orderId == o.id)
                                        .map(p => (u, o, p))))
    .filter(triple => triple.2.amount > 100)
```

### 5. Incremental Query Building
```relic
// Build queries programmatically
fn buildQuery(filters: List[FilterSpec]) -> List[User] {
    filters.fold(User.all(), (users, filter) => 
        match filter {
            AgeFilter(min) => users.filter(u => u.age >= min),
            CityFilter(city) => users.filter(u => u.city == city),
            ActiveFilter => users.filter(u => u.active)
        }
    )
}
```

## Optimization: Whole Program vs Query

Traditional query optimizers work in isolation:
```sql
-- Optimizer only sees this query
SELECT * FROM users WHERE age > 18 AND city = 'NYC'
```

Relic's sea of nodes compiler sees everything:
```relic
let minAge = computeMinAge(config)  // Compiler can inline this
let targetCity = getTargetCity()    // And this

User.where(u => u.age > minAge && u.city == targetCity)

// Compiler can:
// - Inline the functions
// - Recognize constant values
// - Optimize across function boundaries
// - Fuse operations
// - Use indexes when available
```

## The Deeper Insight

By eliminating the concept of "queries", we've achieved something profound:

1. **Unified Mental Model**: Developers think in one language, not two
2. **Composability**: Any function that works on Lists works in "queries"
3. **Extensibility**: New relational operations are just functions
4. **Optimization**: Whole-program optimization exceeds query optimization
5. **Type Safety**: The full type system applies to all data operations

## Conclusion

Relic doesn't have queries because it doesn't need them. When your entire language is built on:
- Immutable values
- Pure functions  
- Strong types
- Functional composition

...then "queries" are just function composition patterns. We haven't lost relational power - we've generalized it into something more fundamental and more powerful.

This is the future of data manipulation: not special query languages, but general-purpose languages powerful enough to express relational operations naturally.