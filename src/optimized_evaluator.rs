use crate::ast::*;
use crate::error::{Error, Result, ValidationError};
use crate::evaluator::{EvalValue, evaluate_expression};
use crate::specialization::SpecializationCache;
use crate::types::Type;
use crate::value::ValueRegistry;
use std::collections::HashMap;

/// Optimized expression evaluator that uses compile-time specialization when possible
pub fn evaluate_expression_optimized(
    expr: &Expression,
    context: &HashMap<String, EvalValue>,
    registry: &ValueRegistry,
    specialization_cache: &SpecializationCache,
    type_env: &HashMap<String, Type>,
) -> Result<EvalValue> {
    match expr {
        Expression::FunctionCall(name, args) => {
            // Evaluate arguments first
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(evaluate_expression_optimized(
                    arg, context, registry, specialization_cache, type_env
                )?);
            }

            // Try to use specialized dispatch if available
            let arg_types: Vec<Type> = args.iter()
                .map(|arg| infer_runtime_type(arg, context, type_env))
                .collect();

            // Check if we have a cached specialization for these exact types
            if let Some(specialized_idx) = specialization_cache.get_specialization(name, &arg_types) {
                // Use the pre-computed best function directly
                if let Some(functions) = registry.get_functions(name) {
                    if let Some(func) = functions.get(specialized_idx) {
                        // Fast path: directly call the specialized function
                        let mut func_context = HashMap::new();
                        for (param, value) in func.parameters.iter().zip(arg_values.iter()) {
                            func_context.insert(param.name.clone(), value.clone());
                        }
                        return evaluate_expression_optimized(
                            &func.body, &func_context, registry, specialization_cache, type_env
                        );
                    }
                }
            }

            // Fall back to regular dynamic dispatch
            evaluate_function_call(name, args, &arg_values, context, registry)
        }
        
        Expression::MethodCall(receiver, method_name, args) => {
            // Similar optimization for method calls
            let receiver_value = evaluate_expression_optimized(
                receiver, context, registry, specialization_cache, type_env
            )?;
            
            let mut arg_values = vec![receiver_value];
            for arg in args {
                arg_values.push(evaluate_expression_optimized(
                    arg, context, registry, specialization_cache, type_env
                )?);
            }

            // Build complete type signature for specialization lookup
            let receiver_type = infer_runtime_type(receiver, context, type_env);
            let mut all_types = vec![receiver_type];
            all_types.extend(args.iter().map(|arg| infer_runtime_type(arg, context, type_env)));

            // Check for specialized dispatch
            if let Some(specialized_idx) = specialization_cache.get_specialization(method_name, &all_types) {
                if let Some(functions) = registry.get_functions(method_name) {
                    if let Some(func) = functions.get(specialized_idx) {
                        let mut func_context = HashMap::new();
                        for (param, value) in func.parameters.iter().zip(arg_values.iter()) {
                            func_context.insert(param.name.clone(), value.clone());
                        }
                        return evaluate_expression_optimized(
                            &func.body, &func_context, registry, specialization_cache, type_env
                        );
                    }
                }
            }

            // Fall back to dynamic dispatch
            let mut call_args = args.to_vec();
            call_args.insert(0, *receiver.clone());
            let function_call = Expression::FunctionCall(method_name.clone(), call_args);
            evaluate_expression(&function_call, context, registry)
        }
        
        // For other expression types, recurse with optimization
        Expression::Binary(op, left, right) => {
            let left_val = evaluate_expression_optimized(left, context, registry, specialization_cache, type_env)?;
            let right_val = evaluate_expression_optimized(right, context, registry, specialization_cache, type_env)?;
            evaluate_binary_op(op, left_val, right_val)
        }
        
        Expression::Let(name, binding, body) => {
            let mut new_context = context.clone();
            let mut new_type_env = type_env.clone();
            
            let value = evaluate_expression_optimized(binding, &new_context, registry, specialization_cache, &new_type_env)?;
            let ty = type_from_value(&value);
            new_context.insert(name.clone(), value);
            new_type_env.insert(name.clone(), ty);
            
            evaluate_expression_optimized(body, &new_context, registry, specialization_cache, &new_type_env)
        }
        
        // For simple expressions, use regular evaluation
        _ => evaluate_expression(expr, context, registry),
    }
}

/// Helper function to evaluate function calls (shared logic)
fn evaluate_function_call(
    name: &str,
    args: &[Expression],
    _arg_values: &[EvalValue],
    context: &HashMap<String, EvalValue>,
    registry: &ValueRegistry,
) -> Result<EvalValue> {
    // This is simplified - in reality, we'd call the dispatch logic
    let call_args = args.to_vec();
    let function_call = Expression::FunctionCall(name.to_string(), call_args);
    evaluate_expression(&function_call, context, registry)
}

/// Evaluate binary operations
fn evaluate_binary_op(op: &BinaryOp, left: EvalValue, right: EvalValue) -> Result<EvalValue> {
    match (op, left, right) {
        (BinaryOp::Add, EvalValue::Integer(l), EvalValue::Integer(r)) => {
            Ok(EvalValue::Integer(l + r))
        }
        (BinaryOp::Subtract, EvalValue::Integer(l), EvalValue::Integer(r)) => {
            Ok(EvalValue::Integer(l - r))
        }
        (BinaryOp::Multiply, EvalValue::Integer(l), EvalValue::Integer(r)) => {
            Ok(EvalValue::Integer(l * r))
        }
        (BinaryOp::Divide, EvalValue::Integer(l), EvalValue::Integer(r)) => {
            if r != 0 {
                Ok(EvalValue::Integer(l / r))
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Division by zero".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        (BinaryOp::Modulo, EvalValue::Integer(l), EvalValue::Integer(r)) => {
            if r != 0 {
                Ok(EvalValue::Integer(l % r))
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Modulo by zero".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        (BinaryOp::And, EvalValue::Boolean(l), EvalValue::Boolean(r)) => {
            Ok(EvalValue::Boolean(l && r))
        }
        (BinaryOp::Or, EvalValue::Boolean(l), EvalValue::Boolean(r)) => {
            Ok(EvalValue::Boolean(l || r))
        }
        _ => Err(Error::Validation(ValidationError {
            message: "Type mismatch in binary operation".to_string(),
            value_type: "".to_string(),
        })),
    }
}

/// Infer type from runtime value
fn type_from_value(value: &EvalValue) -> Type {
    match value {
        EvalValue::Integer(_) => Type::Int,
        EvalValue::String(_) => Type::String,
        EvalValue::Boolean(_) => Type::Bool,
        EvalValue::Value { type_name, .. } => Type::Value(type_name.clone()),
        EvalValue::Type(_) => Type::Type,
        EvalValue::List(_) => Type::List(Box::new(Type::Any)), // TODO: Infer element type
    }
}

/// Best-effort runtime type inference
fn infer_runtime_type(
    expr: &Expression,
    context: &HashMap<String, EvalValue>,
    type_env: &HashMap<String, Type>,
) -> Type {
    match expr {
        Expression::Literal(Literal::Integer(_)) => Type::Int,
        Expression::Literal(Literal::String(_)) => Type::String,
        Expression::Literal(Literal::Boolean(_)) => Type::Bool,
        Expression::Identifier(name) => {
            // Try type environment first, then runtime context
            if let Some(ty) = type_env.get(name) {
                ty.clone()
            } else if let Some(value) = context.get(name) {
                type_from_value(value)
            } else {
                Type::Unknown
            }
        }
        Expression::FunctionCall(name, _) if name.chars().next().unwrap_or('a').is_uppercase() => {
            // Heuristic: uppercase function names are likely value constructors
            Type::Value(name.clone())
        }
        _ => Type::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialization::SpecializationCache;

    #[test]
    fn test_optimized_function_call() {
        // This is a placeholder test - in a real implementation,
        // we would test that specialized calls are faster than dynamic dispatch
        let _cache = SpecializationCache::new();
        let _registry = ValueRegistry::new();
        let _context: HashMap<String, EvalValue> = HashMap::new();
        let _type_env: HashMap<String, Type> = HashMap::new();
        
        // Would test optimization here
        assert!(true);
    }
}