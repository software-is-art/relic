use crate::ast::*;
use crate::types::Type;
use crate::value::ValueRegistry;

/// Register all standard library functions
pub fn register_stdlib(registry: &mut ValueRegistry) {
    // Register the single built-in function: all(t: Type) -> List[t]
    // This is the ONLY built-in needed for the Type-as-Relation model
    register_all_function(registry);
}

/// Register the all(t: Type) -> List[t] built-in function
/// This is the foundational function for the Type-as-Relation model
fn register_all_function(registry: &mut ValueRegistry) {
    let all_function = FunctionDeclaration {
        name: "all".to_string(),
        parameters: vec![ParameterWithGuard {
            name: "t".to_string(),
            ty: Type::Type,
            guard: None,
        }],
        return_type: Type::List(Box::new(Type::Any)), // List of elements of the type
        // The body is not used for built-ins - they are handled specially in the evaluator
        body: Expression::Literal(Literal::String("built-in".to_string())),
    };
    
    registry.register_function(all_function);
}