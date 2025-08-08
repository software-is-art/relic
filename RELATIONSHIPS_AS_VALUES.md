# Relationships as Values in Relic

## Overview

This document explains how Relic models relationships between entities using its core principle that "everything is a value." Rather than introducing special syntax or constructs for relationships, Relic treats relationships as first-class value types that can be queried, validated, and composed like any other value.

## Philosophy

### Why Relationships as Values?

In traditional systems, relationships are often second-class citizens:
- SQL: Foreign keys and JOIN tables with special DDL syntax
- ORMs: Magic methods and lazy loading with hidden queries  
- Graph databases: Special edge types distinct from nodes

Relic takes a different approach: **relationships are just values**. This provides:
1. **Consistency**: One mental model for all data
2. **Explicitness**: Relationships are visible and queryable
3. **Flexibility**: Relationships can carry rich attributes
4. **Composability**: Standard functions work on relationships
5. **Type Safety**: Relationships are validated like any value

### Integration with Type-as-Relation

Since every value type forms an implicit relation of its instances, relationship types naturally form queryable collections:

```relic
// Person type - automatically tracked as a relation
value Person(id: Int, name: String) {
    validate: name.length > 0
    key: id
}

// Friendship type - also automatically tracked as a relation
value Friendship(person1: PersonId, person2: PersonId) {
    validate: person1 != person2
    normalize: person1 < person2 ? (person1, person2) : (person2, person1)
    unique: (person1, person2)
}

// Both can be queried the same way
let people = Person.all()
let friendships = Friendship.all()
```

## Basic Patterns

### Symmetric Relationships

For bidirectional relationships like friendships, use normalization to ensure consistency:

```relic
value Friendship(person1: PersonId, person2: PersonId) {
    // Normalize to ensure we don't get both (A,B) and (B,A)
    normalize: person1 < person2 ? (person1, person2) : (person2, person1)
    unique: (person1, person2)
}
```

### Directed Relationships

For relationships with direction, like reporting structures:

```relic
value Employment(employeeId: PersonId, managerId: PersonId) {
    validate: employeeId != managerId
    key: employeeId  // Each person has at most one manager
}
```

### Temporal Relationships

Relationships can include time dimensions:

```relic
value Employment(
    employeeId: PersonId, 
    managerId: PersonId,
    startDate: Date,
    endDate: Option[Date]
) {
    validate: employeeId != managerId
    validate: endDate.map(end => end > startDate).getOrElse(true)
}
```

### N-ary Relationships

Relationships can involve multiple entities:

```relic
value ProjectAssignment(
    projectId: ProjectId,
    personId: PersonId, 
    roleId: RoleId,
    allocation: Percentage
) {
    validate: allocation > 0 && allocation <= 100
    unique: (projectId, personId, roleId)
}
```

## Query Patterns

### Basic Relationship Queries

```relic
// Find all friendships for a person
fn friendshipsOf(person: Person) -> List[Friendship] {
    Friendship.where(f => 
        f.person1 == person.id || f.person2 == person.id
    )
}

// Find all friends of a person
fn friendsOf(person: Person) -> List[Person] {
    friendshipsOf(person)
        .map(f => f.person1 == person.id ? f.person2 : f.person1)
        .map(id => Person.find(p => p.id == id))
}
```

### Advanced Queries

```relic
// Mutual friends
fn mutualFriends(person1: Person, person2: Person) -> List[Person] {
    let friends1 = friendsOf(person1).map(_.id).toSet()
    let friends2 = friendsOf(person2).map(_.id).toSet()
    
    friends1.intersect(friends2)
        .map(id => Person.find(p => p.id == id))
}

// Friends of friends (excluding direct friends and self)
fn friendsOfFriends(person: Person) -> List[Person] {
    let directFriends = friendsOf(person).map(_.id).toSet()
    
    friendsOf(person)
        .flatMap(friend => friendsOf(friend))
        .filter(p => p.id != person.id && !directFriends.contains(p.id))
        .distinctBy(_.id)
}

// Reporting chain
fn reportingChain(person: Person) -> List[Person] {
    Employment.find(e => e.employeeId == person.id)
        .map(e => Person.find(p => p.id == e.managerId))
        .map(manager => [manager] ++ reportingChain(manager))
        .getOrElse([])
}
```

## Future Syntax Enhancements

### Pattern Matching on Relationships

Future versions of Relic may support pattern matching for cleaner queries:

```relic
// Potential future syntax
fn friendsOf(person: Person) -> List[Person] {
    Friendship.collect {
        case {person1: ^person.id, person2} => Person.get(person2)
        case {person1, person2: ^person.id} => Person.get(person1)
    }
}
```

### Relationship Methods

Adding methods to relationship types for common operations:

```relic
impl Friendship {
    fn involves(self, personId: PersonId) -> Bool {
        self.person1 == personId || self.person2 == personId
    }
    
    fn otherPerson(self, personId: PersonId) -> PersonId {
        if self.person1 == personId then self.person2 else self.person1
    }
}

// Cleaner queries
fn friendsOf(person: Person) -> List[Person] {
    Friendship.where(_.involves(person.id))
        .map(_.otherPerson(person.id))
        .map(Person.get)
}
```

### Symmetric Relationship Support

Special handling for symmetric relationships:

```relic
// Potential future syntax
value Friendship(person1: PersonId, person2: PersonId) symmetric {
    // Compiler understands this is bidirectional
}

// Enables special query methods
let friends = Friendship.connectedTo(person.id)
```

## Comparison with Other Approaches

### SQL Foreign Keys
```sql
-- SQL approach
CREATE TABLE friendships (
    person1_id INT REFERENCES persons(id),
    person2_id INT REFERENCES persons(id),
    CHECK (person1_id < person2_id),
    PRIMARY KEY (person1_id, person2_id)
);

-- Complex query for friends
SELECT p.* FROM persons p
JOIN friendships f ON (f.person1_id = ? AND f.person2_id = p.id)
                   OR (f.person2_id = ? AND f.person1_id = p.id);
```

**Relic advantages**:
- Validation logic is part of the type, not separate constraints
- Normalization happens automatically
- Queries compose with standard functions

### Graph Databases
```cypher
// Neo4j Cypher
MATCH (p1:Person)-[:FRIENDS_WITH]-(p2:Person)
WHERE p1.id = $personId
RETURN p2
```

**Relic advantages**:
- No special query language needed
- Relationships can carry rich, validated attributes
- Type safety throughout

### ORM Associations
```ruby
# Rails ActiveRecord
class Person < ApplicationRecord
  has_and_belongs_to_many :friends,
    class_name: "Person",
    join_table: "friendships",
    foreign_key: "person1_id",
    association_foreign_key: "person2_id"
end
```

**Relic advantages**:
- No magic or hidden queries
- Relationships are explicit and queryable
- No N+1 query problems

## Best Practices

### 1. Choose Meaningful Names
Name relationship types based on what they represent:
- ✅ `Friendship`, `Employment`, `ProjectAssignment`
- ❌ `PersonPerson`, `PersonProject`

### 2. Use Normalization for Symmetric Relations
Prevent duplicate entries with normalization:
```relic
normalize: person1 < person2 ? (person1, person2) : (person2, person1)
```

### 3. Include Relevant Attributes
Relationships often have their own properties:
```relic
value Friendship(
    person1: PersonId,
    person2: PersonId, 
    since: Date,
    context: Context  // How they met
)
```

### 4. Consider Temporal Aspects
Many relationships change over time:
```relic
value Employment(
    employeeId: PersonId,
    companyId: CompanyId,
    role: Role,
    startDate: Date,
    endDate: Option[Date],
    current: Bool  // Denormalized for performance
)
```

### 5. Create Helper Functions
Wrap common queries in well-named functions:
```relic
fn currentEmployer(person: Person) -> Option[Company] {
    Employment.find(e => e.employeeId == person.id && e.current)
        .map(e => Company.find(c => c.id == e.companyId))
}
```

## Performance Considerations

### Indexing
The Relic runtime can automatically index relationship fields:
- Primary keys are always indexed
- Fields used in `unique` constraints are indexed
- Common query patterns can trigger index creation

### Query Optimization
The sea of nodes compiler can optimize relationship queries:
- Fuse multiple operations into single passes
- Eliminate intermediate collections
- Use specialized join algorithms based on data characteristics

### Example Optimization
```relic
// This high-level code:
friendsOf(alice)
    .flatMap(friend => friendsOf(friend))
    .distinct()

// Can be optimized to:
// - Single pass through Friendship relation
// - Bitmap for duplicate detection
// - No intermediate List allocations
```

## Conclusion

Relationships as values is not a compromise or workaround - it's a powerful pattern that emerges naturally from Relic's core philosophy. By treating relationships as first-class values:

1. We maintain conceptual consistency
2. We gain flexibility and expressiveness  
3. We enable powerful optimizations
4. We avoid special-case complexity

This approach scales from simple binary relationships to complex temporal graph structures, all while using the same mental model and tools that work with any other Relic value type.