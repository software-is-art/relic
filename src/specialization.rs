use crate::ast::*;
use crate::types::Type;
use crate::value::ValueRegistry;
use std::collections::HashMap;

/// Represents a specialized dispatch site where we can optimize away dynamic dispatch
#[derive(Debug, Clone)]
pub struct SpecializedCall {
    pub function_name: String,
    pub arg_types: Vec<Type>,
    pub target_function: usize, // Index into the function implementations
}

/// Compile-time specialization cache
pub struct SpecializationCache {
    /// Maps (function_name, arg_types) to the index of the best matching function
    cache: HashMap<(String, Vec<Type>), usize>,
}

impl SpecializationCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Try to specialize a function call at compile time
    pub fn try_specialize(
        &mut self,
        function_name: &str,
        arg_types: &[Type],
        registry: &ValueRegistry,
    ) -> Option<SpecializedCall> {
        // Check if we've already computed this specialization
        let key = (function_name.to_string(), arg_types.to_vec());
        if let Some(&target_idx) = self.cache.get(&key) {
            return Some(SpecializedCall {
                function_name: function_name.to_string(),
                arg_types: arg_types.to_vec(),
                target_function: target_idx,
            });
        }

        // Get all implementations for this function
        let functions = registry.get_functions(function_name)?;
        
        // If there's only one implementation, always use it (fast path)
        if functions.len() == 1 {
            self.cache.insert(key, 0);
            return Some(SpecializedCall {
                function_name: function_name.to_string(),
                arg_types: arg_types.to_vec(),
                target_function: 0,
            });
        }

        // For multiple implementations, try to find the best match at compile time
        let mut candidates = Vec::new();
        
        for (idx, func) in functions.iter().enumerate() {
            // Check if parameter count matches
            if func.parameters.len() != arg_types.len() {
                continue;
            }

            // Check if all parameter types are compatible
            let mut compatible = true;
            let mut specificity = 0u32;
            
            for (param, arg_type) in func.parameters.iter().zip(arg_types.iter()) {
                if !types_compatible(&param.ty, arg_type) {
                    compatible = false;
                    break;
                }
                
                // Calculate specificity (higher is more specific)
                specificity += type_specificity(&param.ty);
                
                // Guards make a function more specific, but we can't evaluate them at compile time
                // So we give a smaller bonus for having guards
                if param.guard.is_some() {
                    specificity += 1;
                }
            }
            
            if compatible {
                candidates.push((idx, specificity));
            }
        }

        // Sort by specificity (highest first)
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        // Check for ambiguity
        if candidates.len() >= 2 && candidates[0].1 == candidates[1].1 {
            // Ambiguous at compile time - fall back to runtime dispatch
            return None;
        }

        if let Some((best_idx, _)) = candidates.first() {
            self.cache.insert(key, *best_idx);
            Some(SpecializedCall {
                function_name: function_name.to_string(),
                arg_types: arg_types.to_vec(),
                target_function: *best_idx,
            })
        } else {
            None
        }
    }

    /// Get cached specialization
    pub fn get_specialization(
        &self,
        function_name: &str,
        arg_types: &[Type],
    ) -> Option<usize> {
        let key = (function_name.to_string(), arg_types.to_vec());
        self.cache.get(&key).copied()
    }
}

/// Check if two types are compatible (source type can be passed to parameter type)
fn types_compatible(param_type: &Type, arg_type: &Type) -> bool {
    match (param_type, arg_type) {
        // Any type accepts anything
        (Type::Any, _) => true,
        // Exact match
        (Type::Int, Type::Int) => true,
        (Type::String, Type::String) => true,
        (Type::Bool, Type::Bool) => true,
        (Type::Value(n1), Type::Value(n2)) => n1 == n2,
        // Unknown at compile time - conservative approach
        (_, Type::Unknown) => true,
        (Type::Unknown, _) => true,
        // Otherwise not compatible
        _ => false,
    }
}

/// Calculate type specificity score (higher is more specific)
fn type_specificity(ty: &Type) -> u32 {
    match ty {
        Type::Int | Type::String | Type::Bool | Type::Value(_) | Type::Type | Type::List(_) => 3,
        Type::Any => 1,
        Type::Unknown => 0,
    }
}

/// Optimization pass that specializes function calls where possible
pub fn specialize_function_calls(
    expr: &mut Expression,
    type_env: &HashMap<String, Type>,
    specialization_cache: &mut SpecializationCache,
    registry: &ValueRegistry,
) {
    match expr {
        Expression::FunctionCall(name, args) => {
            // First, recursively specialize arguments
            for arg in args.iter_mut() {
                specialize_function_calls(arg, type_env, specialization_cache, registry);
            }

            // Try to determine argument types
            let arg_types: Vec<Type> = args.iter()
                .map(|arg| infer_expression_type(arg, type_env))
                .collect();

            // All types must be known for specialization
            if arg_types.iter().all(|t| !matches!(t, Type::Unknown)) {
                if let Some(spec) = specialization_cache.try_specialize(name, &arg_types, registry) {
                    // In a real implementation, we would transform this into a specialized call
                    // For now, we just cache the specialization for the evaluator to use
                    // The evaluator can check the cache before doing dynamic dispatch
                }
            }
        }
        Expression::MethodCall(receiver, method_name, args) => {
            // Specialize receiver and arguments
            specialize_function_calls(receiver, type_env, specialization_cache, registry);
            for arg in args.iter_mut() {
                specialize_function_calls(arg, type_env, specialization_cache, registry);
            }

            // For method calls, we need the receiver type plus argument types
            let receiver_type = infer_expression_type(receiver, type_env);
            let mut all_types = vec![receiver_type];
            all_types.extend(args.iter().map(|arg| infer_expression_type(arg, type_env)));

            if all_types.iter().all(|t| !matches!(t, Type::Unknown)) {
                specialization_cache.try_specialize(method_name, &all_types, registry);
            }
        }
        Expression::Binary(_, left, right) => {
            specialize_function_calls(left, type_env, specialization_cache, registry);
            specialize_function_calls(right, type_env, specialization_cache, registry);
        }
        Expression::Unary(_, expr) => {
            specialize_function_calls(expr, type_env, specialization_cache, registry);
        }
        Expression::Comparison(_, left, right) => {
            specialize_function_calls(left, type_env, specialization_cache, registry);
            specialize_function_calls(right, type_env, specialization_cache, registry);
        }
        Expression::Let(name, binding, body) => {
            specialize_function_calls(binding, type_env, specialization_cache, registry);
            let mut new_env = type_env.clone();
            let ty = infer_expression_type(binding, type_env);
            new_env.insert(name.clone(), ty);
            specialize_function_calls(body, &new_env, specialization_cache, registry);
        }
        Expression::Pipeline(expr, right) => {
            specialize_function_calls(expr, type_env, specialization_cache, registry);
            specialize_function_calls(right, type_env, specialization_cache, registry);
        }
        Expression::Match(expr, branches) => {
            specialize_function_calls(expr, type_env, specialization_cache, registry);
            // Note: branches are not mutable here, would need different approach for real implementation
        }
        // No ValueConstruction variant in current AST
        Expression::MemberAccess(expr, _) => {
            specialize_function_calls(expr, type_env, specialization_cache, registry);
        }
        Expression::Literal(_) | Expression::Identifier(_) | Expression::TypeLiteral(_) => {
            // No function calls to specialize
        }
    }
}

/// Simple type inference for expressions (best effort)
fn infer_expression_type(expr: &Expression, type_env: &HashMap<String, Type>) -> Type {
    match expr {
        Expression::Literal(Literal::Integer(_)) => Type::Int,
        Expression::Literal(Literal::String(_)) => Type::String,
        Expression::Literal(Literal::Boolean(_)) => Type::Bool,
        Expression::Identifier(name) => {
            type_env.get(name).cloned().unwrap_or(Type::Unknown)
        }
        Expression::Binary(op, _, _) => {
            match op {
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | 
                BinaryOp::Divide | BinaryOp::Modulo => Type::Int,
                BinaryOp::And | BinaryOp::Or => Type::Bool,
            }
        }
        Expression::Unary(op, _) => {
            match op {
                UnaryOp::Not => Type::Bool,
                UnaryOp::Minus => Type::Int,
            }
        }
        Expression::Comparison(_, _, _) => Type::Bool,
        Expression::TypeLiteral(_) => Type::Type,
        // Value construction is through function calls
        // For complex expressions, we can't determine the type statically
        _ => Type::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_implementation_specialization() {
        let mut cache = SpecializationCache::new();
        let mut registry = ValueRegistry::new();
        
        // Register a single function
        let func = FunctionDeclaration {
            name: "double".to_string(),
            parameters: vec![
                ParameterWithGuard {
                    name: "x".to_string(),
                    ty: Type::Int,
                    guard: None,
                }
            ],
            return_type: Type::Int,
            body: Expression::Literal(Literal::Integer(0)), // Dummy body
        };
        registry.register_function(func);

        // Should specialize immediately for single implementation
        let spec = cache.try_specialize("double", &[Type::Int], &registry);
        assert!(spec.is_some());
        assert_eq!(spec.unwrap().target_function, 0);
    }

    #[test]
    fn test_type_compatibility() {
        assert!(types_compatible(&Type::Any, &Type::Int));
        assert!(types_compatible(&Type::Int, &Type::Int));
        assert!(!types_compatible(&Type::Int, &Type::String));
        assert!(types_compatible(&Type::Int, &Type::Unknown));
    }
}