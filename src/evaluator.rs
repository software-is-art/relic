use crate::ast::*;
use crate::error::{Error, Result, ValidationError};
use crate::value::ValueRegistry;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum EvalValue {
    String(String),
    Integer(i64),
    Boolean(bool),
}

// General expression evaluator that can handle all expression types including function calls
pub fn evaluate_expression(
    expr: &Expression,
    context: &HashMap<String, EvalValue>,
    registry: &ValueRegistry,
) -> Result<EvalValue> {
    match expr {
        Expression::Literal(Literal::String(s)) => Ok(EvalValue::String(s.clone())),
        Expression::Literal(Literal::Integer(n)) => Ok(EvalValue::Integer(*n)),
        Expression::Literal(Literal::Boolean(b)) => Ok(EvalValue::Boolean(*b)),
        
        Expression::Identifier(name) => {
            context.get(name).cloned().ok_or_else(|| {
                Error::Validation(ValidationError {
                    message: format!("Unknown identifier: {}", name),
                    value_type: "".to_string(),
                })
            })
        }
        
        Expression::Binary(op, left, right) => {
            let left_val = evaluate_expression(left, context, registry)?;
            let right_val = evaluate_expression(right, context, registry)?;
            
            match (op, left_val, right_val) {
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
        
        Expression::Unary(op, expr) => {
            let val = evaluate_expression(expr, context, registry)?;
            match (op, val) {
                (UnaryOp::Not, EvalValue::Boolean(b)) => Ok(EvalValue::Boolean(!b)),
                (UnaryOp::Minus, EvalValue::Integer(n)) => Ok(EvalValue::Integer(-n)),
                _ => Err(Error::Validation(ValidationError {
                    message: "Type mismatch in unary operation".to_string(),
                    value_type: "".to_string(),
                })),
            }
        }
        
        Expression::Comparison(op, left, right) => {
            let left_val = evaluate_expression(left, context, registry)?;
            let right_val = evaluate_expression(right, context, registry)?;
            
            let result = match (op, left_val, right_val) {
                (ComparisonOp::Equal, EvalValue::Integer(l), EvalValue::Integer(r)) => l == r,
                (ComparisonOp::NotEqual, EvalValue::Integer(l), EvalValue::Integer(r)) => l != r,
                (ComparisonOp::Less, EvalValue::Integer(l), EvalValue::Integer(r)) => l < r,
                (ComparisonOp::Greater, EvalValue::Integer(l), EvalValue::Integer(r)) => l > r,
                (ComparisonOp::LessEqual, EvalValue::Integer(l), EvalValue::Integer(r)) => l <= r,
                (ComparisonOp::GreaterEqual, EvalValue::Integer(l), EvalValue::Integer(r)) => l >= r,
                (ComparisonOp::Equal, EvalValue::String(ref l), EvalValue::String(ref r)) => l == r,
                (ComparisonOp::NotEqual, EvalValue::String(ref l), EvalValue::String(ref r)) => l != r,
                (ComparisonOp::Contains, EvalValue::String(ref l), EvalValue::String(ref r)) => l.contains(r),
                (ComparisonOp::Equal, EvalValue::Boolean(l), EvalValue::Boolean(r)) => l == r,
                (ComparisonOp::NotEqual, EvalValue::Boolean(l), EvalValue::Boolean(r)) => l != r,
                _ => return Err(Error::Validation(ValidationError {
                    message: "Type mismatch in comparison".to_string(),
                    value_type: "".to_string(),
                })),
            };
            
            Ok(EvalValue::Boolean(result))
        }
        
        Expression::FunctionCall(name, args) => {
            // Evaluate arguments first
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(evaluate_expression(arg, context, registry)?);
            }
            
            // First check if it's a function
            if let Some(func_decl) = registry.get_function(name) {
                // Handle as a function call
                // Check argument count
                if arg_values.len() != func_decl.parameters.len() {
                    return Err(Error::Validation(ValidationError {
                        message: format!(
                            "Function {} expects {} arguments, got {}",
                            name,
                            func_decl.parameters.len(),
                            arg_values.len()
                        ),
                        value_type: "function".to_string(),
                    }));
                }
                
                // Create new context with function parameters
                let mut func_context = HashMap::new();
                for (param, value) in func_decl.parameters.iter().zip(arg_values.iter()) {
                    func_context.insert(param.name.clone(), value.clone());
                }
                
                // Evaluate function body
                evaluate_expression(&func_decl.body, &func_context, registry)
            } else if let Some(methods) = registry.get_methods(name) {
                // Handle as a method call with multiple dispatch
                // Find the best matching method based on argument types
                let mut best_match = None;
                
                for method in methods {
                    if method.parameters.len() != arg_values.len() {
                        continue;
                    }
                    
                    // For now, we do simple type matching based on runtime values
                    // In the future, we could use more sophisticated type matching
                    let matches = method.parameters.iter()
                        .zip(&arg_values)
                        .all(|(param, value)| {
                            match (&param.ty, value) {
                                (crate::types::Type::Int, EvalValue::Integer(_)) => true,
                                (crate::types::Type::String, EvalValue::String(_)) => true,
                                (crate::types::Type::Bool, EvalValue::Boolean(_)) => true,
                                _ => false,
                            }
                        });
                        
                    if matches {
                        best_match = Some(method);
                        break; // Take first exact match for now
                    }
                }
                
                if let Some(method) = best_match {
                    // Create new context with method parameters
                    let mut method_context = HashMap::new();
                    for (param, value) in method.parameters.iter().zip(arg_values.iter()) {
                        method_context.insert(param.name.clone(), value.clone());
                    }
                    
                    // Evaluate method body
                    evaluate_expression(&method.body, &method_context, registry)
                } else {
                    Err(Error::Validation(ValidationError {
                        message: format!("No matching method '{}' found for given arguments", name),
                        value_type: "method".to_string(),
                    }))
                }
            } else {
                Err(Error::Validation(ValidationError {
                    message: format!("Unknown function or method: {}", name),
                    value_type: "function".to_string(),
                }))
            }
        }
        
        Expression::Let(name, binding, body) => {
            let bound_value = evaluate_expression(binding, context, registry)?;
            let mut new_context = context.clone();
            new_context.insert(name.clone(), bound_value);
            evaluate_expression(body, &new_context, registry)
        }
        
        Expression::Pipeline(left, right) => {
            // Evaluate the left expression
            let left_val = evaluate_expression(left, context, registry)?;
            
            // The right side should be a function call or identifier
            match &**right {
                Expression::Identifier(func_name) => {
                    // Transform into a function call with left_val as argument
                    let func_call = Expression::FunctionCall(
                        func_name.clone(),
                        vec![value_to_expression(left_val)?],
                    );
                    evaluate_expression(&func_call, context, registry)
                }
                Expression::FunctionCall(func_name, args) => {
                    // Prepend left_val to the arguments
                    let mut new_args = vec![value_to_expression(left_val)?];
                    new_args.extend(args.clone());
                    let func_call = Expression::FunctionCall(func_name.clone(), new_args);
                    evaluate_expression(&func_call, context, registry)
                }
                _ => Err(Error::Validation(ValidationError {
                    message: "Pipeline right side must be a function".to_string(),
                    value_type: "".to_string(),
                })),
            }
        }
        
        Expression::MemberAccess(obj, member) => {
            let obj_val = evaluate_expression(obj, context, registry)?;
            match (&obj_val, member.as_str()) {
                (EvalValue::String(s), "length") => Ok(EvalValue::Integer(s.len() as i64)),
                _ => Err(Error::Validation(ValidationError {
                    message: format!("Cannot access member {} on value", member),
                    value_type: "".to_string(),
                })),
            }
        }
        
        Expression::MethodCall(obj, method, args) => {
            // First check if this is a user-defined function (UFC syntax)
            if let Some(_func_decl) = registry.get_function(method) {
                // Transform x.f(y, z) into f(x, y, z)
                let mut full_args = vec![obj.as_ref().clone()];
                full_args.extend(args.clone());
                return evaluate_expression(
                    &Expression::FunctionCall(method.clone(), full_args),
                    context,
                    registry,
                );
            }
            
            // Check if this is a method (UFC syntax for methods)
            if let Some(_methods) = registry.get_methods(method) {
                // Transform x.f(y, z) into f(x, y, z) for method dispatch
                let mut full_args = vec![obj.as_ref().clone()];
                full_args.extend(args.clone());
                return evaluate_expression(
                    &Expression::FunctionCall(method.clone(), full_args),
                    context,
                    registry,
                );
            }
            
            // Otherwise, handle built-in methods
            let obj_val = evaluate_expression(obj, context, registry)?;
            match (&obj_val, method.as_str()) {
                (EvalValue::String(s), "toLowerCase") if args.is_empty() => {
                    Ok(EvalValue::String(s.to_lowercase()))
                }
                (EvalValue::String(s), "toUpperCase") if args.is_empty() => {
                    Ok(EvalValue::String(s.to_uppercase()))
                }
                _ => Err(Error::Validation(ValidationError {
                    message: format!("Unknown method {} or wrong arguments", method),
                    value_type: "".to_string(),
                })),
            }
        }
        
        Expression::Match(expr, arms) => {
            let val = evaluate_expression(expr, context, registry)?;
            
            // For now, just evaluate the first arm's body
            // Full pattern matching would be more complex
            if let Some(arm) = arms.first() {
                // Add pattern binding to context if needed
                let mut new_context = context.clone();
                match &arm.pattern {
                    Pattern::Constructor(_, binding) => {
                        new_context.insert(binding.clone(), val);
                    }
                }
                evaluate_expression(&arm.body, &new_context, registry)
            } else {
                Err(Error::Validation(ValidationError {
                    message: "No match arms".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
    }
}

// Helper to convert EvalValue back to Expression for pipeline operations
fn value_to_expression(val: EvalValue) -> Result<Expression> {
    match val {
        EvalValue::String(s) => Ok(Expression::Literal(Literal::String(s))),
        EvalValue::Integer(n) => Ok(Expression::Literal(Literal::Integer(n))),
        EvalValue::Boolean(b) => Ok(Expression::Literal(Literal::Boolean(b))),
    }
}

impl std::fmt::Display for EvalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalValue::String(s) => write!(f, "{}", s),
            EvalValue::Integer(n) => write!(f, "{}", n),
            EvalValue::Boolean(b) => write!(f, "{}", b),
        }
    }
}