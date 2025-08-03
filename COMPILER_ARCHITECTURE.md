# Relic Compiler Architecture: Sea of Nodes

## Overview

Relic uses a sea of nodes intermediate representation (IR) for its optimizing compiler. This architecture, pioneered by the HotSpot JVM and used in modern compilers like V8's TurboFan and Graal, represents programs as directed graphs where nodes represent operations and edges represent dependencies.

## Why Sea of Nodes for Relic

### Natural Alignment with Language Design

1. **Immutability and SSA**
   - Relic's immutable values map directly to Static Single Assignment (SSA) form
   - No mutation means simpler dependency tracking
   - Each value has exactly one definition point

2. **Parse-Don't-Validate Pattern**
   - Value construction creates clear dataflow boundaries
   - Validation predicates become explicit nodes in the graph
   - Enables optimization of validation checks

3. **Functional-Relational Architecture**
   - Relational operations naturally form dataflow graphs
   - Pure functions have no side effects, simplifying optimization
   - Clear separation between essential state and computation

4. **Multiple Dispatch Optimization**
   - Type information flows through the graph
   - Enables aggressive specialization and devirtualization
   - Can eliminate dispatch overhead for known types

## Node Types

### 1. Value Nodes
Represent immutable values in the program:
```
- ConstantNode: Compile-time constants
- ParameterNode: Function parameters
- PhiNode: SSA phi functions for control flow joins
```

### 2. Value Construction Nodes
Handle the parse-don't-validate pattern:
```
- ConstructNode: Create a new value object
- ValidateNode: Execute validation predicate
- NormalizeNode: Apply normalization function
- ParseBoundaryNode: Mark system boundary crossings
```

### 3. Computation Nodes
Pure computational operations:
```
- ArithmeticNode: +, -, *, /, %
- ComparisonNode: ==, !=, <, >, <=, >=
- LogicalNode: &&, ||, !
- StringOpNode: concat, substring, toLowerCase, etc.
```

### 4. Function and Dispatch Nodes
Handle function calls and multiple dispatch:
```
- CallNode: Direct function call
- DispatchNode: Multiple dispatch call site
- InlineNode: Inlined function body
- ReturnNode: Function return
```

### 5. Relational Operation Nodes
Implement functional-relational operations:
```
- ProjectNode: Select columns
- FilterNode: Where clause
- JoinNode: Various join types
- AggregateNode: Group by operations
- OrderNode: Sort operations
```

### 6. Control Flow Nodes
Minimal control flow for pattern matching:
```
- IfNode: Conditional branches
- MatchNode: Pattern matching
- MergeNode: Control flow merge points
- LoopNode: For relational iterations
```

### 7. Memory Nodes
Track memory operations and effects:
```
- LoadNode: Read from relations
- StoreNode: Write to relations (only at boundaries)
- MemoryPhiNode: Memory state at merge points
- BarrierNode: Memory ordering constraints
```

## Graph Construction

### From AST to Sea of Nodes

1. **Initial Lowering**
   ```relic
   fn double(x: Int) -> Int {
       x * 2
   }
   ```
   Becomes:
   ```
   ParameterNode(x) → ConstantNode(2) → MultiplyNode → ReturnNode
   ```

2. **Value Construction**
   ```relic
   value EmailAddress(raw: String) {
       validate: raw contains "@" && raw.length > 3
       normalize: raw.toLowerCase()
   }
   ```
   Becomes:
   ```
   ParameterNode(raw) → ValidateNode(contains "@") → ValidateNode(length > 3) 
                      → NormalizeNode(toLowerCase) → ConstructNode(EmailAddress)
   ```

## Optimization Passes

### 1. Standard Optimizations

**Common Subexpression Elimination (CSE)**
- Identify identical subgraphs
- Replace with single computation
- Particularly effective for validation predicates

**Dead Code Elimination (DCE)**
- Remove unreachable nodes
- Eliminate unused computations
- Critical for removing redundant validations

**Constant Folding**
- Evaluate compile-time constants
- Propagate through the graph
- Fold validation predicates when possible

### 2. Relic-Specific Optimizations

**Validation Inlining**
```
Before:
ConstructNode(EmailAddress) → ValidateNode(complex predicate)

After (when type is known):
ConstructNode(EmailAddress) → [inlined validation nodes]
```

**Multiple Dispatch Devirtualization**
```
Before:
DispatchNode(join, [HashRelation, SortedRelation])

After:
DirectCallNode(hash_sort_join)
```

**Relational Operation Fusion**
```
Before:
FilterNode(x > 5) → FilterNode(x < 10) → ProjectNode(x, y)

After:
FilterNode(x > 5 && x < 10) → ProjectNode(x, y)
```

## Example Transformation

### Original Relic Code
```relic
fn processEmails(emails: Relation[String]) -> Relation[EmailAddress] {
    emails
    |> filter(e => e.length > 0)
    |> map(e => EmailAddress(e))
    |> filter(e => e != null)
}
```

### Initial Graph
```
ParameterNode(emails)
  → FilterNode(length > 0)
    → MapNode(
        → ConstructNode(EmailAddress)
          → ValidateNode
          → NormalizeNode
      )
    → FilterNode(!= null)
```

### After Optimization
```
ParameterNode(emails)
  → FilterNode(length > 0 && contains "@" && length > 3)
    → MapNode(
        → NormalizeNode(toLowerCase)
        → ConstructNode(EmailAddress)
      )
// Null check eliminated - constructor never returns null
// Validation hoisted into filter
```

## Memory Model

### Immutable Value Representation
- Values are allocated once and never modified
- Reference counting for memory management
- Copy-on-write for collections

### Relation Storage
- Column-oriented storage for cache efficiency
- Compressed representations for common patterns
- Zero-copy operations where possible

## Code Generation

### Target Architectures

1. **Bytecode Interpreter** (Initial)
   - Stack-based VM optimized for Relic
   - Specialized opcodes for value operations
   - Fast dispatch for common patterns

2. **LLVM Backend** (Production)
   - Generate LLVM IR from sea of nodes
   - Leverage LLVM's optimization passes
   - Native code for all platforms

3. **WebAssembly** (Web Deployment)
   - Direct compilation to WASM
   - Minimal runtime for browser execution
   - Streaming compilation support

### Optimization Levels

- **Level 0**: Direct interpretation (development)
- **Level 1**: Basic optimizations (CSE, DCE)
- **Level 2**: Relic-specific optimizations
- **Level 3**: Aggressive inlining and specialization
- **Level 4**: Profile-guided optimization

## Runtime Integration

### Adaptive Optimization
- Start with interpreter for cold code
- Identify hot paths through profiling
- Progressively optimize based on runtime behavior

### Deoptimization
- Guards for speculative optimizations
- Fallback to interpreter when assumptions fail
- Minimal overhead for uncommon cases

## Debugging Support

### Graph Visualization
- GraphViz export for IR inspection
- Interactive debugger for optimization passes
- Source mapping from nodes to original code

### Verification
- Type checking on IR
- Invariant checking between passes
- Formal verification of critical optimizations

## Performance Targets

### Optimization Goals
- Value construction: < 10ns overhead vs raw allocation
- Multiple dispatch: < 5ns for monomorphic calls
- Relational operations: Competitive with hand-optimized C
- Memory usage: Minimal overhead for immutability

### Benchmarks
- Micro-benchmarks for each optimization
- Real-world relational query performance
- Comparison with similar languages (Julia, Rust, Haskell)

## Future Directions

### Advanced Optimizations
- Partial evaluation for known inputs
- Supercompilation for complex patterns
- Machine learning-guided optimization

### Distributed Compilation
- Parallel graph construction
- Distributed optimization passes
- Cloud-based compilation cache

### Hardware Acceleration
- SIMD vectorization for relational operations
- GPU acceleration for large-scale queries
- Custom hardware targeting (FPGAs)

## Conclusion

The sea of nodes architecture provides Relic with a powerful foundation for optimization while maintaining the language's correctness guarantees. By representing programs as explicit dependency graphs, we can perform aggressive optimizations that would be difficult or impossible with traditional compiler architectures, all while preserving the parse-don't-validate pattern and functional-relational semantics that make Relic unique.