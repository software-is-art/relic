use crate::ast::*;
use crate::error::{Error, Result, ValidationError};
// use crate::relation::{Relation, Schema}; // Unused for now
use crate::value::ValueRegistry;
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::Arc;
use std::any::Any;

// Cache key for dispatch decisions
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DispatchKey {
    function_name: String,
    arg_type_signatures: Vec<String>,
}

// Cache entry storing the resolved function
type DispatchCache = Arc<RwLock<HashMap<DispatchKey, usize>>>; // Stores index into function list

// Create a thread-safe dispatch cache
lazy_static::lazy_static! {
    static ref DISPATCH_CACHE: DispatchCache = Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Clone, Debug)]
pub enum EvalValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Value {
        type_name: String,
        fields: HashMap<String, EvalValue>,
    },
    // First-class Type value for Type-as-Relation
    Type(String), // Type name
    // List value for relational operations
    List(Vec<EvalValue>),
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
            // First check if it's in the context
            if let Some(value) = context.get(name) {
                Ok(value.clone())
            } else if registry.constructors.contains_key(name) {
                // If it's a type name, return a Type value for Type-as-Relation
                Ok(EvalValue::Type(name.clone()))
            } else {
                Err(Error::Validation(ValidationError {
                    message: format!("Unknown identifier: {}", name),
                    value_type: "".to_string(),
                }))
            }
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
            
            // Handle built-in functions first
            if name == "all" && arg_values.len() == 1 {
                if let EvalValue::Type(type_name) = &arg_values[0] {
                    // Get all instances of the type and return as List
                    let instances = registry.get_all_instances(type_name);
                    let eval_instances: Vec<EvalValue> = instances
                        .into_iter()
                        .map(|instance| {
                            let mut fields = HashMap::new();
                            
                            // Try to extract field value based on the constructor definition
                            if let Some(constructor) = registry.constructors.get(type_name) {
                                let param_name = &constructor.declaration.parameter.name;
                                
                                // Try to downcast and extract the value
                                let any_ref = instance.as_any();
                                if let Some(generic_obj) = any_ref.downcast_ref::<crate::value::GenericValueObject>() {
                                    // Access the data field directly
                                    let data_ref = &*generic_obj.data;
                                    
                                    // Try to extract based on parameter type
                                    match &constructor.declaration.parameter.ty {
                                        crate::types::Type::String => {
                                            if let Some(s) = data_ref.downcast_ref::<String>() {
                                                fields.insert(param_name.clone(), EvalValue::String(s.clone()));
                                            }
                                        }
                                        crate::types::Type::Int => {
                                            if let Some(n) = data_ref.downcast_ref::<i64>() {
                                                fields.insert(param_name.clone(), EvalValue::Integer(*n));
                                            }
                                        }
                                        crate::types::Type::Bool => {
                                            if let Some(b) = data_ref.downcast_ref::<bool>() {
                                                fields.insert(param_name.clone(), EvalValue::Boolean(*b));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            
                            EvalValue::Value {
                                type_name: instance.type_name().to_string(),
                                fields,
                            }
                        })
                        .collect();
                    return Ok(EvalValue::List(eval_instances));
                } else {
                    return Err(Error::Validation(ValidationError {
                        message: "all() expects a Type argument".to_string(),
                        value_type: "function".to_string(),
                    }));
                }
            }
            
            // First check if it's a value constructor
            if registry.constructors.contains_key(name) {
                // Handle value construction
                if arg_values.len() != 1 {
                    return Err(Error::Validation(ValidationError {
                        message: format!(
                            "Value constructor {} expects 1 argument, got {}",
                            name,
                            arg_values.len()
                        ),
                        value_type: "constructor".to_string(),
                    }));
                }
                
                // Convert the argument to a form the constructor can use
                let arg = &arg_values[0];
                let input: Box<dyn Any + Send + Sync> = match arg {
                    EvalValue::String(s) => Box::new(s.clone()),
                    EvalValue::Integer(n) => Box::new(*n),
                    EvalValue::Boolean(b) => Box::new(*b),
                    _ => return Err(Error::Validation(ValidationError {
                        message: format!("Invalid argument type for value constructor {}", name),
                        value_type: "constructor".to_string(),
                    })),
                };
                
                // Construct the value
                let _value_obj = registry.construct(name, input)?;
                
                // Extract the field value for the EvalValue
                let mut fields = HashMap::new();
                if let Some(constructor) = registry.constructors.get(name) {
                    let param_name = &constructor.declaration.parameter.name;
                    fields.insert(param_name.clone(), arg.clone());
                }
                
                Ok(EvalValue::Value {
                    type_name: name.clone(),
                    fields,
                })
            }
            // With unified syntax, all functions can have multiple implementations
            else if let Some(functions) = registry.get_functions(name) {
                // If only one function, execute it directly
                if functions.len() == 1 {
                    let func_decl = &functions[0];
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
                } else {
                    // Multiple implementations - use dispatch
                    dispatch_function(name, functions, &arg_values, context, registry)
                }
            } else if let Some(methods) = registry.get_methods(name) {
                // Handle as a method call with multiple dispatch
                // Find the best matching method based on argument types and specificity
                let mut candidates = Vec::new();
                
                for method in methods {
                    if method.parameters.len() != arg_values.len() {
                        continue;
                    }
                    
                    // Check if all parameters match
                    let matches = method.parameters.iter()
                        .zip(&arg_values)
                        .all(|(param, value)| {
                            matches_type(&param.ty, value)
                        });
                        
                    if matches {
                        // Create context for guard evaluation
                        let mut guard_context = HashMap::new();
                        for (param, value) in method.parameters.iter().zip(arg_values.iter()) {
                            guard_context.insert(param.name.clone(), value.clone());
                        }
                        
                        // Check if all guards are satisfied
                        let guards_satisfied = method.parameters.iter()
                            .all(|param| {
                                match &param.guard {
                                    Some(guard_expr) => {
                                        // Evaluate the guard expression
                                        match evaluate_expression(guard_expr, &guard_context, registry) {
                                            Ok(EvalValue::Boolean(true)) => true,
                                            _ => false,
                                        }
                                    }
                                    None => true, // No guard means it's satisfied
                                }
                            });
                            
                        if guards_satisfied {
                            // Calculate specificity score for this method
                            let specificity = calculate_method_specificity(method, &arg_values);
                            candidates.push((method, specificity));
                        }
                    }
                }
                
                // Sort by specificity (higher is more specific)
                candidates.sort_by(|a, b| b.1.cmp(&a.1));
                
                // Check for ambiguity - if top two have same specificity
                if candidates.len() >= 2 && candidates[0].1 == candidates[1].1 {
                    return Err(Error::Validation(ValidationError {
                        message: format!("Ambiguous method call '{}' - multiple methods with same specificity", name),
                        value_type: "method".to_string(),
                    }));
                }
                
                let best_match = candidates.first().map(|(method, _)| *method);
                
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
            } else if name == "relationOf" {
                // Special handling for relationOf
                // TODO: Implement relationOf for Type-as-Relation
                Err(Error::Validation(ValidationError {
                    message: "relationOf is not yet implemented in Type-as-Relation model".to_string(),
                    value_type: "function".to_string(),
                }))
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
            match &obj_val {
                EvalValue::String(s) => match member.as_str() {
                    "length" => Ok(EvalValue::Integer(s.len() as i64)),
                    _ => Err(Error::Validation(ValidationError {
                        message: format!("String has no member '{}'", member),
                        value_type: "String".to_string(),
                    })),
                },
                EvalValue::Value { type_name, fields } => {
                    fields.get(member).cloned().ok_or_else(|| {
                        Error::Validation(ValidationError {
                            message: format!("Value type '{}' has no member '{}'", type_name, member),
                            value_type: type_name.clone(),
                        })
                    })
                },
                _ => Err(Error::Validation(ValidationError {
                    message: format!("Cannot access member {} on primitive value", member),
                    value_type: "".to_string(),
                })),
            }
        }
        
        Expression::MethodCall(obj, method, args) => {
            // Check if this is a Type method call (e.g., User.all())
            if let Expression::Identifier(type_name) = &**obj {
                // Check if this identifier is a type name in the registry
                if registry.constructors.contains_key(type_name) {
                    // Handle Type-as-Relation methods by delegating to built-in functions
                    match method.as_str() {
                        "all" if args.is_empty() => {
                            // Delegate to the built-in all() function
                            let instances = registry.get_all_instances(type_name);
                            let eval_instances: Vec<EvalValue> = instances
                                .into_iter()
                                .map(|instance| {
                                    let mut fields = HashMap::new();
                                    
                                    // Try to extract field value based on the constructor definition
                                    if let Some(constructor) = registry.constructors.get(type_name) {
                                        let param_name = &constructor.declaration.parameter.name;
                                        
                                        // Try to downcast and extract the value
                                        let any_ref = instance.as_any();
                                        if let Some(generic_obj) = any_ref.downcast_ref::<crate::value::GenericValueObject>() {
                                            // Access the data field directly
                                            let data_ref = &*generic_obj.data;
                                            
                                            // Try to extract based on parameter type
                                            match &constructor.declaration.parameter.ty {
                                                crate::types::Type::String => {
                                                    if let Some(s) = data_ref.downcast_ref::<String>() {
                                                        fields.insert(param_name.clone(), EvalValue::String(s.clone()));
                                                    }
                                                }
                                                crate::types::Type::Int => {
                                                    if let Some(n) = data_ref.downcast_ref::<i64>() {
                                                        fields.insert(param_name.clone(), EvalValue::Integer(*n));
                                                    }
                                                }
                                                crate::types::Type::Bool => {
                                                    if let Some(b) = data_ref.downcast_ref::<bool>() {
                                                        fields.insert(param_name.clone(), EvalValue::Boolean(*b));
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    
                                    EvalValue::Value {
                                        type_name: instance.type_name().to_string(),
                                        fields,
                                    }
                                })
                                .collect();
                            Ok(EvalValue::List(eval_instances))
                        }
                        "count" if args.is_empty() => {
                            // For now, keep count as special case until we implement pure Relic functions
                            let count = registry.count_instances(type_name);
                            Ok(EvalValue::Integer(count as i64))
                        }
                        _ => Err(Error::Validation(ValidationError {
                            message: format!("Unknown type method {} or wrong arguments", method),
                            value_type: type_name.to_string(),
                        })),
                    }
                } else {
                    // Not a type, check if it's in the context
                    if context.contains_key(type_name) {
                        // Continue with normal method evaluation
                        // With unified syntax, check if this is a user-defined function (UFC syntax)
                        if let Some(_functions) = registry.get_functions(method) {
                            // Transform x.f(y, z) into f(x, y, z)
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
                            (EvalValue::List(items), "length") if args.is_empty() => {
                                Ok(EvalValue::Integer(items.len() as i64))
                            }
                            (EvalValue::List(_items), "filter") if args.len() == 1 => {
                                // For now, filter is not implemented
                                // We need function values/lambdas for this
                                Err(Error::Validation(ValidationError {
                                    message: "List.filter() not yet implemented - requires lambda support".to_string(),
                                    value_type: "method".to_string(),
                                }))
                            }
                            (EvalValue::List(_items), "find") if args.len() == 1 => {
                                // For now, find is not implemented
                                // We need function values/lambdas for this
                                Err(Error::Validation(ValidationError {
                                    message: "List.find() not yet implemented - requires lambda support".to_string(),
                                    value_type: "method".to_string(),
                                }))
                            }
                            _ => Err(Error::Validation(ValidationError {
                                message: format!("Unknown method {} or wrong arguments", method),
                                value_type: "".to_string(),
                            })),
                        }
                    } else {
                        Err(Error::Validation(ValidationError {
                            message: format!("Unknown identifier: {}", type_name),
                            value_type: "".to_string(),
                        }))
                    }
                }
            } else {
                // With unified syntax, check if this is a user-defined function (UFC syntax)
                if let Some(_functions) = registry.get_functions(method) {
                    // Transform x.f(y, z) into f(x, y, z)
                    let mut full_args = vec![obj.as_ref().clone()];
                    full_args.extend(args.clone());
                    return evaluate_expression(
                        &Expression::FunctionCall(method.clone(), full_args),
                        context,
                        registry,
                    );
                }
                
                // For backward compatibility, check if this is a method
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
                    (EvalValue::List(items), "length") if args.is_empty() => {
                        Ok(EvalValue::Integer(items.len() as i64))
                    }
                    (EvalValue::List(_items), "filter") if args.len() == 1 => {
                        // For now, filter is not implemented
                        // We need function values/lambdas for this
                        Err(Error::Validation(ValidationError {
                            message: "List.filter() not yet implemented - requires lambda support".to_string(),
                            value_type: "method".to_string(),
                        }))
                    }
                    (EvalValue::List(_items), "find") if args.len() == 1 => {
                        // For now, find is not implemented
                        // We need function values/lambdas for this
                        Err(Error::Validation(ValidationError {
                            message: "List.find() not yet implemented - requires lambda support".to_string(),
                            value_type: "method".to_string(),
                        }))
                    }
                    _ => Err(Error::Validation(ValidationError {
                        message: format!("Unknown method {} or wrong arguments", method),
                        value_type: "".to_string(),
                    })),
                }
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

        Expression::TypeLiteral(type_name) => {
            // Return a Type value for Type-as-Relation
            Ok(EvalValue::Type(type_name.clone()))
        }
    }
}

// Helper to convert EvalValue back to Expression for pipeline operations
fn value_to_expression(val: EvalValue) -> Result<Expression> {
    match val {
        EvalValue::String(s) => Ok(Expression::Literal(Literal::String(s))),
        EvalValue::Integer(n) => Ok(Expression::Literal(Literal::Integer(n))),
        EvalValue::Boolean(b) => Ok(Expression::Literal(Literal::Boolean(b))),
        EvalValue::Value { type_name, .. } => {
            // For now, we can't convert value objects back to expressions
            Err(Error::Validation(ValidationError {
                message: format!("Cannot convert value type '{}' to expression", type_name),
                value_type: type_name,
            }))
        }
        EvalValue::Type(type_name) => Ok(Expression::TypeLiteral(type_name)),
        EvalValue::List(_items) => {
            // For now, create a placeholder - in a full implementation we'd need list literals
            Err(Error::Validation(ValidationError {
                message: "Cannot convert List to expression".to_string(),
                value_type: "List".to_string(),
            }))
        }
    }
}

// Check if a runtime value matches a type
fn matches_type(ty: &crate::types::Type, value: &EvalValue) -> bool {
    match (ty, value) {
        (crate::types::Type::Int, EvalValue::Integer(_)) => true,
        (crate::types::Type::String, EvalValue::String(_)) => true,
        (crate::types::Type::Bool, EvalValue::Boolean(_)) => true,
        (crate::types::Type::Value(type_name), EvalValue::Value { type_name: val_type, .. }) => {
            type_name == val_type
        },
        (crate::types::Type::Type, EvalValue::Type(_)) => true,
        (crate::types::Type::List(_), EvalValue::List(_)) => true, // TODO: Check element types
        (crate::types::Type::Any, _) => true, // Any matches everything
        _ => false,
    }
}

// Get type signature for an EvalValue (used for cache keys)
fn get_value_type_signature(value: &EvalValue) -> String {
    match value {
        EvalValue::String(_) => "String".to_string(),
        EvalValue::Integer(_) => "Int".to_string(),
        EvalValue::Boolean(_) => "Bool".to_string(),
        EvalValue::Value { type_name, .. } => type_name.clone(),
        EvalValue::Type(_) => "Type".to_string(),
        EvalValue::List(_) => "List".to_string(),
    }
}

// Create a cache key from function name and argument types
fn create_dispatch_key(name: &str, arg_values: &[EvalValue]) -> DispatchKey {
    DispatchKey {
        function_name: name.to_string(),
        arg_type_signatures: arg_values.iter()
            .map(get_value_type_signature)
            .collect(),
    }
}

// Calculate specificity score for a method based on parameter types
// Higher score means more specific
fn dispatch_function(
    name: &str,
    functions: &[crate::ast::FunctionDeclaration],
    arg_values: &[EvalValue],
    _context: &HashMap<String, EvalValue>,
    registry: &ValueRegistry,
) -> Result<EvalValue> {
    // Create cache key
    let cache_key = create_dispatch_key(name, arg_values);
    
    // Check cache first
    {
        let cache = DISPATCH_CACHE.read().unwrap();
        if let Some(&func_index) = cache.get(&cache_key) {
            if func_index < functions.len() {
                let func = &functions[func_index];
                // Create new context with function parameters
                let mut func_context = HashMap::new();
                for (param, value) in func.parameters.iter().zip(arg_values.iter()) {
                    func_context.insert(param.name.clone(), value.clone());
                }
                
                // Evaluate function body (cached path)
                return evaluate_expression(&func.body, &func_context, registry);
            }
        }
    }
    
    // Cache miss - perform full dispatch resolution
    // Find the best matching function based on argument types and specificity
    let mut candidates = Vec::new();
    
    for (index, func) in functions.iter().enumerate() {
        if func.parameters.len() != arg_values.len() {
            continue;
        }
        
        // Check if all parameters match
        let matches = func.parameters.iter()
            .zip(arg_values)
            .all(|(param, value)| matches_type(&param.ty, value));
            
        if matches {
            // Create context for guard evaluation
            let mut guard_context = HashMap::new();
            for (param, value) in func.parameters.iter().zip(arg_values.iter()) {
                guard_context.insert(param.name.clone(), value.clone());
            }
            
            // Check if all guards are satisfied
            let guards_satisfied = func.parameters.iter()
                .all(|param| {
                    match &param.guard {
                        Some(guard_expr) => {
                            // Evaluate the guard expression
                            match evaluate_expression(guard_expr, &guard_context, registry) {
                                Ok(EvalValue::Boolean(true)) => true,
                                _ => false,
                            }
                        }
                        None => true, // No guard means it's satisfied
                    }
                });
                
            if guards_satisfied {
                // Calculate specificity score for this function
                let specificity = calculate_function_specificity(func, arg_values);
                candidates.push((index, func, specificity));
            }
        }
    }
    
    // Sort by specificity (higher is more specific)
    candidates.sort_by(|a, b| b.2.cmp(&a.2));
    
    // Check for ambiguity - if top two have same specificity
    if candidates.len() >= 2 && candidates[0].2 == candidates[1].2 {
        return Err(Error::Validation(ValidationError {
            message: format!("Ambiguous function call '{}' - multiple functions with same specificity", name),
            value_type: "function".to_string(),
        }));
    }
    
    let best_match = candidates.first().map(|(index, func, _)| (*index, *func));
    
    if let Some((func_index, func)) = best_match {
        // Store in cache for future lookups
        {
            let mut cache = DISPATCH_CACHE.write().unwrap();
            cache.insert(cache_key, func_index);
        }
        
        // Create new context with function parameters
        let mut func_context = HashMap::new();
        for (param, value) in func.parameters.iter().zip(arg_values.iter()) {
            func_context.insert(param.name.clone(), value.clone());
        }
        
        // Evaluate function body
        evaluate_expression(&func.body, &func_context, registry)
    } else {
        Err(Error::Validation(ValidationError {
            message: format!("No matching function '{}' found for given arguments", name),
            value_type: "function".to_string(),
        }))
    }
}

fn calculate_function_specificity(func: &crate::ast::FunctionDeclaration, arg_values: &[EvalValue]) -> u32 {
    let mut score = 0;
    
    for (param, _value) in func.parameters.iter().zip(arg_values) {
        score += match &param.ty {
            crate::types::Type::Int => 3,     // Specific types get higher scores
            crate::types::Type::String => 3,
            crate::types::Type::Bool => 3,
            crate::types::Type::Value(_) => 3,
            crate::types::Type::Type => 3,
            crate::types::Type::List(_) => 3,
            crate::types::Type::Any => 1,     // Any is least specific
            crate::types::Type::Unknown => 0,
        };
        
        // Add bonus for having a guard (more specific)
        if param.guard.is_some() {
            score += 2;
        }
    }
    
    score
}

fn calculate_method_specificity(method: &crate::ast::MethodDeclaration, arg_values: &[EvalValue]) -> u32 {
    let mut score = 0;
    
    for (param, _value) in method.parameters.iter().zip(arg_values) {
        score += match &param.ty {
            crate::types::Type::Int => 3,     // Specific types get higher scores
            crate::types::Type::String => 3,
            crate::types::Type::Bool => 3,
            crate::types::Type::Value(_) => 3,
            crate::types::Type::Type => 3,
            crate::types::Type::List(_) => 3,
            crate::types::Type::Any => 1,     // Any is least specific
            crate::types::Type::Unknown => 0,
        };
        
        // Add bonus for having a guard (more specific)
        if param.guard.is_some() {
            score += 2;
        }
    }
    
    score
}

impl std::fmt::Display for EvalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalValue::String(s) => write!(f, "{}", s),
            EvalValue::Integer(n) => write!(f, "{}", n),
            EvalValue::Boolean(b) => write!(f, "{}", b),
            EvalValue::Value { type_name, fields } => {
                write!(f, "{}", type_name)?;
                if !fields.is_empty() {
                    write!(f, "(")?;
                    let mut first = true;
                    // For single-parameter values, just show the value
                    if fields.len() == 1 {
                        for (_name, value) in fields {
                            write!(f, "{}", value)?;
                        }
                    } else {
                        // For multi-parameter values (future), show name=value pairs
                        for (name, value) in fields {
                            if !first {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}={}", name, value)?;
                            first = false;
                        }
                    }
                    write!(f, ")")?;
                } else {
                    write!(f, "()")?;
                }
                Ok(())
            },
            EvalValue::Type(type_name) => write!(f, "Type({})", type_name),
            EvalValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
        }
    }
}