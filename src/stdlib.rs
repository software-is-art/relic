use crate::ast::*;
use crate::types::Type;
use crate::value::ValueRegistry;

/// Register all standard library functions
pub fn register_stdlib(registry: &mut ValueRegistry) {
    // For now, we'll need to handle relationOf specially in the evaluator
    // since it needs to create new value constructors dynamically.
    // In the future, we could implement this more elegantly with:
    // - First-class functions that can create value types
    // - Macros or compile-time function evaluation
    
    // For now, let's not register it here and handle it as a special case
    // in the evaluator when we don't find a regular function with that name.
}