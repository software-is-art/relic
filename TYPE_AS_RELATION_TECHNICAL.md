# Type-as-Relation Technical Documentation

## Overview

Type-as-Relation is a revolutionary approach in Relic where every value type automatically forms a relation of all its instances. The implementation uses a minimal built-in approach where only `all(t: Type)` is built-in, with all other relational operations implemented in pure Relic.

## Architecture

### Core Concept
```relic
// When you define a value type...
value User(name: String) {
    validate: name.length > 0
}

// Only ONE built-in function:
fn all(t: Type) -> List[t]  // Returns all instances of type t

// Everything else is pure Relic:
fn count(t: Type) -> Int { all(t).length() }
fn where(t: Type, pred: fn(t) -> Bool) -> List[t] { all(t).filter(pred) }

// Usage with UFC:
User.count()  // count(User) -> all(User).length()
User.all()    // all(User)
```

### Implementation Details

#### 1. Instance Tracking in ValueRegistry

The `ValueRegistry` struct has been extended with instance tracking:

```rust
pub struct ValueRegistry {
    pub(crate) constructors: HashMap<String, ValueConstructor>,
    functions: HashMap<String, Vec<FunctionDeclaration>>,
    // New: Track all instances by type name
    instances: Arc<RwLock<HashMap<String, Vec<Weak<dyn ValueObject>>>>>,
}
```

Key design decisions:
- Uses `Arc<RwLock<...>>` for thread-safe access
- Stores `Weak<dyn ValueObject>` references to allow garbage collection
- Automatically cleans up dead references on access

#### 2. Value Construction Changes

The `construct` method now returns `Arc` instead of `Box`:

```rust
pub fn construct(
    &self,
    type_name: &str,
    input: Box<dyn Any + Send + Sync>,
) -> Result<Arc<dyn ValueObject>> {
    // ... validation ...
    let value = self.create_value_object(type_name, input)?;
    let value_arc = Arc::from(value);
    
    // Register the instance for Type-as-Relation
    self.register_instance(type_name, Arc::downgrade(&value_arc));
    
    Ok(value_arc)
}
```

#### 3. Type Method Implementation

New methods on ValueRegistry:

```rust
// Get all living instances of a type
pub fn get_all_instances(&self, type_name: &str) -> Vec<Arc<dyn ValueObject>>

// Count living instances of a type
pub fn count_instances(&self, type_name: &str) -> usize
```

The `get_all_instances` method also performs cleanup of dead weak references.

#### 4. Type Checker Updates

The type checker now recognizes type names as valid identifiers:

```rust
Expression::Identifier(name) => {
    if let Some(ty) = self.locals.get(name) {
        Ok(ty.clone())
    } else if self.env.is_type_name(name) {
        // Type names are valid identifiers
        Ok(Type::String)  // Temporary until we have a Type type
    } else {
        Err(Error::Type(TypeError {
            message: format!("Undefined identifier: {}", name),
        }))
    }
}
```

Method calls on types are also handled:

```rust
Expression::MethodCall(object, method, args) => {
    // Check if this is a Type method call (e.g., User.all())
    if let Expression::Identifier(type_name) = &**object {
        if self.env.is_type_name(type_name) {
            match method.as_str() {
                "all" if args.is_empty() => return Ok(Type::String),
                "count" if args.is_empty() => return Ok(Type::Int),
                // ... other type methods ...
            }
        }
    }
    // ... regular method handling ...
}
```

#### 5. Evaluator Updates

The evaluator handles type identifiers and type method calls:

```rust
// Type names evaluate to a special marker
Expression::Identifier(name) => {
    if let Some(value) = context.get(name) {
        Ok(value.clone())
    } else if registry.constructors.contains_key(name) {
        // Type identifier
        Ok(EvalValue::String(format!("<type {}>", name)))
    } else {
        Err(...)
    }
}

// Type method calls
Expression::MethodCall(obj, method, args) => {
    if let Expression::Identifier(type_name) = &**obj {
        if registry.constructors.contains_key(type_name) {
            match method.as_str() {
                "count" => {
                    let count = registry.count_instances(type_name);
                    Ok(EvalValue::Integer(count as i64))
                }
                "all" => {
                    let instances = registry.get_all_instances(type_name);
                    // TODO: Return proper list type
                    Ok(EvalValue::String(format!("{} instances", instances.len())))
                }
                // ... other methods ...
            }
        }
    }
    // ... regular method handling ...
}
```

## Memory Management

### Weak References
- Instances are stored as `Weak<dyn ValueObject>` to allow garbage collection
- When no strong references remain, the instance can be dropped
- Dead weak references are cleaned up lazily on access

### Example Behavior
```relic
User("Alice")     // Creates instance, immediately dropped (count = 0)
let u = User("Bob")  // Creates instance, stored in variable (count = 1)
User.count()      // Returns 1
// When u goes out of scope, count becomes 0
```

## Current Limitations

1. **No Persistence**: Instances only exist in memory during REPL session
2. **No Collection Types**: `all()` returns a string representation instead of a proper list
3. **No Predicates**: `where()` and `find()` not yet implemented
4. **No Indexes**: All queries scan the full instance list
5. **Single-threaded**: While using Arc/RwLock, REPL is single-threaded

## Minimal Built-in Design

### Philosophy
Instead of implementing type methods as special cases, we use a minimal built-in approach:

1. **Single Built-in**: Only `all(t: Type) -> List[t]` is implemented in Rust
2. **Everything Else in Relic**: All other operations are pure Relic functions
3. **Composability**: Complex queries built through function composition
4. **No Special Cases**: No type method handling in evaluator or type checker

### Benefits
- **Simplicity**: Only one function needs special registry access
- **Extensibility**: Users can define their own type-level functions
- **Consistency**: All operations use the same function dispatch mechanism
- **Education**: Shows that relational operations are just functional transformations

### Example: Building Queries
```relic
// Basic operations
fn count(t: Type) -> Int { all(t).length() }
fn exists(t: Type, pred: fn(t) -> Bool) -> Bool { all(t).any(pred) }
fn first(t: Type) -> Option[t] { all(t).head() }

// Joins are just nested operations
fn join(t1: Type, t2: Type, on: fn(t1, t2) -> Bool) -> List[(t1, t2)] {
    all(t1).flatMap(x => 
        all(t2).filter(y => on(x, y))
               .map(y => (x, y))
    )
}

// Complex queries through composition
User.where(u => u.age > 18)
    .map(u => u.email)
    .distinct()
    .count()
```

## Future Enhancements

### Phase 4.2: Implement Minimal Built-in
- Add Type as first-class type
- Implement `all(t: Type)` built-in
- Create List type with essential methods
- Remove all special-case handling

### Phase 4.3: Standard Library
- Implement query functions in pure Relic
- Add join operations
- Support aggregations

### Phase 4.4: Performance
- Add indexing for key/unique fields
- Optimize `all()` for large datasets
- Consider lazy evaluation

### Integration with Sea of Nodes
- Each value instance is naturally a node
- Type relations are node collections
- Queries become graph traversals
- `all()` returns a node collection

## Usage Examples

### Basic Usage
```relic
// Define a type
value User(id: Int, name: String) {
    validate: id > 0
    key: id  // Future: will create index
}

// Create instances
let alice = User(1, "Alice")
let bob = User(2, "Bob")

// Query the type
User.count()  // Returns 2
User.all()    // Returns all instances

// Future: filtered queries
User.where(u => u.name contains "A")  // Would return [alice]
User.find(u => u.id == 2)             // Would return bob
```

### Memory Management Example
```relic
// Temporary instances
User(3, "Charlie")  // Created and immediately dropped

// Persistent instances
let users = [
    User(4, "David"),
    User(5, "Eve")
]

User.count()  // Returns 2 (only David and Eve)
```

## Implementation Files

- `src/value.rs` - Core instance tracking implementation
- `src/typechecker.rs` - Type method validation
- `src/evaluator.rs` - Type method execution
- `src/types.rs` - Type environment extensions
- `examples/type_as_relation.relic` - Usage examples

## Testing

Current test coverage includes:
- Unit test for instance tracking (in comments)
- REPL integration testing
- Manual verification of count/all methods

Future tests needed:
- Comprehensive unit tests for all type methods
- Memory management verification
- Thread safety tests
- Performance benchmarks