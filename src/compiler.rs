use crate::ast::*;
use crate::error::{Error, Result, ValidationError};
use crate::specialization::SpecializationCache;
use crate::value::{ValueConstructor, ValueRegistry};
use std::any::Any;
use std::collections::HashMap;

pub struct Compiler {
    registry: ValueRegistry,
    specialization_cache: SpecializationCache,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            registry: ValueRegistry::new(),
            specialization_cache: SpecializationCache::new(),
        }
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<()> {
        for declaration in &program.declarations {
            self.compile_declaration(declaration)?;
        }
        Ok(())
    }

    fn compile_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Value(value_decl) => self.compile_value_declaration(value_decl),
            Declaration::Function(func_decl) => self.compile_function_declaration(func_decl),
            Declaration::Method(method_decl) => {
                // For backward compatibility, compile methods as functions
                self.compile_method_declaration(method_decl)
            },
            Declaration::Relation(_relation_decl) => {
                // TODO: Implement relation compilation in Phase 4
                Ok(())
            }
        }
    }

    fn compile_value_declaration(&mut self, decl: &ValueDeclaration) -> Result<()> {
        let decl_clone = decl.clone();
        let name = decl.name.clone();

        // Create validator function
        let validator = if let Some(ref validate_expr) = decl.body.validate {
            let expr_clone = validate_expr.clone();
            let param_name = decl.parameter.name.clone();

            Box::new(move |input: &(dyn Any + Send + Sync)| -> Result<()> {
                // This is a simplified validator - in a real implementation,
                // we would compile the expression to executable code
                match &decl_clone.parameter.ty {
                    crate::types::Type::String => {
                        if let Some(s) = input.downcast_ref::<String>() {
                            if !evaluate_string_validation(s, &expr_clone, &param_name)? {
                                return Err(Error::Validation(ValidationError {
                                    message: "Validation failed".to_string(),
                                    value_type: name.clone(),
                                }));
                            }
                        }
                    }
                    crate::types::Type::Int => {
                        if let Some(n) = input.downcast_ref::<i64>() {
                            if !evaluate_int_validation(*n, &expr_clone, &param_name)? {
                                return Err(Error::Validation(ValidationError {
                                    message: "Validation failed".to_string(),
                                    value_type: name.clone(),
                                }));
                            }
                        }
                    }
                    _ => {}
                }
                Ok(())
            }) as Box<dyn Fn(&(dyn Any + Send + Sync)) -> Result<()> + Send + Sync>
        } else {
            Box::new(|_: &(dyn Any + Send + Sync)| Ok(()))
        };

        // Create normalizer function
        let normalizer = if decl.body.normalize.is_some() {
            Some(
                Box::new(move |_input: &mut (dyn Any + Send + Sync)| -> Result<()> {
                    // Normalization would be implemented here
                    Ok(())
                })
                    as Box<dyn Fn(&mut (dyn Any + Send + Sync)) -> Result<()> + Send + Sync>,
            )
        } else {
            None
        };

        let constructor = ValueConstructor {
            declaration: decl.clone(),
            validator,
            normalizer,
        };

        self.registry.register(decl.name.clone(), constructor);

        Ok(())
    }

    fn compile_function_declaration(&mut self, decl: &FunctionDeclaration) -> Result<()> {
        // For now, we'll store function declarations in the registry
        // In a full implementation, functions would be compiled to executable code
        self.registry.register_function(decl.clone());
        Ok(())
    }

    fn compile_method_declaration(&mut self, decl: &MethodDeclaration) -> Result<()> {
        // Store method declarations in the registry for multiple dispatch
        self.registry.register_method(decl.clone());
        Ok(())
    }

    pub fn get_registry(&self) -> &ValueRegistry {
        &self.registry
    }

    pub fn into_registry(self) -> ValueRegistry {
        self.registry
    }

    pub fn evaluate_expression(&self, expr: &Expression) -> Result<crate::evaluator::EvalValue> {
        // Use optimized evaluator when we have type information available
        // For now, fall back to regular evaluation
        crate::evaluator::evaluate_expression(expr, &HashMap::new(), &self.registry)
    }
    
    pub fn evaluate_expression_with_optimization(&mut self, expr: &Expression) -> Result<crate::evaluator::EvalValue> {
        // Pre-specialize function calls in the expression
        let mut expr_copy = expr.clone();
        crate::specialization::specialize_function_calls(
            &mut expr_copy,
            &HashMap::new(), // Type environment - would be populated from type checker
            &mut self.specialization_cache,
            &self.registry,
        );
        
        // Use optimized evaluator with specialization cache
        crate::optimized_evaluator::evaluate_expression_optimized(
            &expr_copy,
            &HashMap::new(),
            &self.registry,
            &self.specialization_cache,
            &HashMap::new(), // Type environment
        )
    }
}

// Evaluation context for let-bindings
#[derive(Clone, Debug)]
#[allow(dead_code)]
enum EvalValue {
    String(String),
    Integer(i64),
    Boolean(bool),
}

// Simplified expression evaluation functions
fn evaluate_string_validation(value: &str, expr: &Expression, param_name: &str) -> Result<bool> {
    evaluate_string_validation_with_context(value, expr, param_name, &HashMap::new())
}

fn evaluate_string_validation_with_context(
    value: &str,
    expr: &Expression,
    param_name: &str,
    context: &HashMap<String, EvalValue>,
) -> Result<bool> {
    match expr {
        Expression::Binary(BinaryOp::And, left, right) => {
            Ok(evaluate_string_validation_with_context(value, left, param_name, context)?
                && evaluate_string_validation_with_context(value, right, param_name, context)?)
        }
        Expression::Binary(BinaryOp::Or, left, right) => {
            Ok(evaluate_string_validation_with_context(value, left, param_name, context)?
                || evaluate_string_validation_with_context(value, right, param_name, context)?)
        }
        Expression::Comparison(ComparisonOp::Contains, left, right) => {
            // Get the left side value
            let left_str = if let Expression::Identifier(name) = &**left {
                if name == param_name {
                    value
                } else if let Some(EvalValue::String(s)) = context.get(name) {
                    s.as_str()
                } else {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            };
            
            // Get the right side value
            if let Expression::Literal(Literal::String(s)) = &**right {
                Ok(left_str.contains(s))
            } else {
                Ok(false)
            }
        }
        Expression::Comparison(ComparisonOp::Equal, left, right) => {
            // Evaluate both sides to string values
            let left_str = evaluate_string_expr(left, value, param_name, context)?;
            let right_str = evaluate_string_expr(right, value, param_name, context)?;
            Ok(left_str == right_str)
        }
        Expression::Comparison(ComparisonOp::NotEqual, left, right) => {
            // Evaluate both sides to string values
            let left_str = evaluate_string_expr(left, value, param_name, context)?;
            let right_str = evaluate_string_expr(right, value, param_name, context)?;
            Ok(left_str != right_str)
        }
        Expression::Comparison(ComparisonOp::Greater, left, right) => {
            // Evaluate left side
            let left_val = if let Expression::MemberAccess(obj, member) = &**left {
                if let Expression::Identifier(name) = &**obj {
                    if name == param_name && member == "length" {
                        value.len() as i64
                    } else if let Some(val) = context.get(name) {
                        match val {
                            EvalValue::String(s) if member == "length" => s.len() as i64,
                            EvalValue::Integer(n) => *n,
                            _ => return Ok(false),
                        }
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            } else if let Expression::Identifier(name) = &**left {
                if let Some(EvalValue::Integer(n)) = context.get(name) {
                    *n
                } else {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            };
            
            // Evaluate right side
            if let Expression::Literal(Literal::Integer(n)) = &**right {
                Ok(left_val > *n)
            } else if let Expression::Identifier(name) = &**right {
                if let Some(EvalValue::Integer(n)) = context.get(name) {
                    Ok(left_val > *n)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
        Expression::Pipeline(left, _right) => {
            // For now, just evaluate the left side
            // Full implementation would need to apply right as a function to left
            evaluate_string_validation_with_context(value, left, param_name, context)
        }
        Expression::Let(name, binding_value, body) => {
            // Evaluate the binding value
            let bound_value = evaluate_expression_to_value(binding_value, value, param_name, context)?;
            
            // Create new context with the bound value
            let mut new_context = context.clone();
            new_context.insert(name.clone(), bound_value);
            
            // Evaluate the body with the extended context
            evaluate_string_validation_with_context(value, body, param_name, &new_context)
        }
        Expression::Match(expr, arms) => {
            // For now, we only support matching on identifiers
            if let Expression::Identifier(name) = &**expr {
                if name == param_name || context.contains_key(name) {
                    // Evaluate the first arm (in a full implementation, we'd match patterns)
                    if let Some(arm) = arms.first() {
                        evaluate_string_validation_with_context(value, &arm.body, param_name, context)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
        _ => Ok(true),
    }
}

fn evaluate_int_validation(value: i64, expr: &Expression, param_name: &str) -> Result<bool> {
    evaluate_int_validation_with_context(value, expr, param_name, &HashMap::new())
}

fn evaluate_int_validation_with_context(
    value: i64,
    expr: &Expression,
    param_name: &str,
    context: &HashMap<String, EvalValue>,
) -> Result<bool> {
    match expr {
        Expression::Binary(BinaryOp::And, left, right) => {
            Ok(evaluate_int_validation_with_context(value, left, param_name, context)?
                && evaluate_int_validation_with_context(value, right, param_name, context)?)
        }
        Expression::Binary(BinaryOp::Or, left, right) => {
            Ok(evaluate_int_validation_with_context(value, left, param_name, context)?
                || evaluate_int_validation_with_context(value, right, param_name, context)?)
        }
        Expression::Comparison(ComparisonOp::Equal, left, right) => {
            let left_val = evaluate_int_expr(left, value, param_name, context)?;
            let right_val = evaluate_int_expr(right, value, param_name, context)?;
            Ok(left_val == right_val)
        }
        Expression::Comparison(ComparisonOp::NotEqual, left, right) => {
            let left_val = evaluate_int_expr(left, value, param_name, context)?;
            let right_val = evaluate_int_expr(right, value, param_name, context)?;
            Ok(left_val != right_val)
        }
        Expression::Comparison(ComparisonOp::Greater, left, right) => {
            let left_val = evaluate_int_expr(left, value, param_name, context)?;
            let right_val = evaluate_int_expr(right, value, param_name, context)?;
            Ok(left_val > right_val)
        }
        Expression::Comparison(ComparisonOp::GreaterEqual, left, right) => {
            let left_val = evaluate_int_expr(left, value, param_name, context)?;
            let right_val = evaluate_int_expr(right, value, param_name, context)?;
            Ok(left_val >= right_val)
        }
        Expression::Comparison(ComparisonOp::Less, left, right) => {
            let left_val = evaluate_int_expr(left, value, param_name, context)?;
            let right_val = evaluate_int_expr(right, value, param_name, context)?;
            Ok(left_val < right_val)
        }
        Expression::Comparison(ComparisonOp::LessEqual, left, right) => {
            let left_val = evaluate_int_expr(left, value, param_name, context)?;
            let right_val = evaluate_int_expr(right, value, param_name, context)?;
            Ok(left_val <= right_val)
        }
        Expression::Pipeline(left, _right) => {
            // For now, just evaluate the left side
            // Full implementation would need to apply right as a function to left
            evaluate_int_validation_with_context(value, left, param_name, context)
        }
        Expression::Let(name, binding_value, body) => {
            // Evaluate the binding value with current parameter as integer
            let bound_value = evaluate_expression_to_value_int(binding_value, value, param_name, context)?;
            
            // Create new context with the bound value
            let mut new_context = context.clone();
            new_context.insert(name.clone(), bound_value);
            
            // Evaluate the body with the extended context
            evaluate_int_validation_with_context(value, body, param_name, &new_context)
        }
        Expression::Match(expr, arms) => {
            // For now, we only support matching on identifiers
            if let Expression::Identifier(name) = &**expr {
                if name == param_name || context.contains_key(name) {
                    // Evaluate the first arm (in a full implementation, we'd match patterns)
                    if let Some(arm) = arms.first() {
                        // Create a new context with the pattern binding
                        // Create a new context with the pattern binding
                        let Pattern::Constructor(_, binding) = &arm.pattern;
                        let mut new_context = context.clone();
                        new_context.insert(binding.clone(), EvalValue::Integer(value));
                        evaluate_int_validation_with_context(value, &arm.body, param_name, &new_context)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
        _ => Ok(true),
    }
}

// Helper function to evaluate an integer expression
fn evaluate_int_expr(
    expr: &Expression,
    param_value: i64,
    param_name: &str,
    context: &HashMap<String, EvalValue>,
) -> Result<i64> {
    match expr {
        Expression::Literal(Literal::Integer(n)) => Ok(*n),
        Expression::Identifier(name) => {
            if name == param_name {
                Ok(param_value)
            } else if let Some(EvalValue::Integer(n)) = context.get(name) {
                Ok(*n)
            } else {
                Err(Error::Validation(ValidationError {
                    message: format!("Cannot evaluate {} as integer", name),
                    value_type: "".to_string(),
                }))
            }
        }
        Expression::MemberAccess(obj, member) => {
            if let Expression::Identifier(name) = &**obj {
                if name == param_name && member == "length" {
                    // This assumes param_value represents something with length
                    // In practice, we'd need type information here
                    Ok(0) // placeholder
                } else if let Some(val) = context.get(name) {
                    match val {
                        EvalValue::String(s) if member == "length" => Ok(s.len() as i64),
                        _ => Err(Error::Validation(ValidationError {
                            message: format!("Cannot access {} on {}", member, name),
                            value_type: "".to_string(),
                        })),
                    }
                } else {
                    Err(Error::Validation(ValidationError {
                        message: format!("Unknown identifier: {}", name),
                        value_type: "".to_string(),
                    }))
                }
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Complex member access not yet supported".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        _ => Err(Error::Validation(ValidationError {
            message: "Cannot evaluate expression as integer".to_string(),
            value_type: "".to_string(),
        })),
    }
}

// Helper function to evaluate a string expression
fn evaluate_string_expr(
    expr: &Expression,
    param_value: &str,
    param_name: &str,
    context: &HashMap<String, EvalValue>,
) -> Result<String> {
    match expr {
        Expression::Literal(Literal::String(s)) => Ok(s.clone()),
        Expression::Identifier(name) => {
            if name == param_name {
                Ok(param_value.to_string())
            } else if let Some(EvalValue::String(s)) = context.get(name) {
                Ok(s.clone())
            } else {
                Err(Error::Validation(ValidationError {
                    message: format!("Cannot evaluate {} as string", name),
                    value_type: "".to_string(),
                }))
            }
        }
        Expression::MethodCall(obj, method, _args) => {
            if let Expression::Identifier(name) = &**obj {
                if name == param_name && method == "toLowerCase" {
                    Ok(param_value.to_lowercase())
                } else if let Some(EvalValue::String(s)) = context.get(name) {
                    if method == "toLowerCase" {
                        Ok(s.to_lowercase())
                    } else {
                        Ok(s.clone())
                    }
                } else {
                    Err(Error::Validation(ValidationError {
                        message: format!("Cannot call method {} on {}", method, name),
                        value_type: "".to_string(),
                    }))
                }
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Complex method calls not yet supported".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        _ => Err(Error::Validation(ValidationError {
            message: "Cannot evaluate expression as string".to_string(),
            value_type: "".to_string(),
        })),
    }
}

// Helper function to evaluate an expression to a value (for string context)
fn evaluate_expression_to_value(
    expr: &Expression,
    param_value: &str,
    param_name: &str,
    context: &HashMap<String, EvalValue>,
) -> Result<EvalValue> {
    match expr {
        Expression::Literal(Literal::String(s)) => Ok(EvalValue::String(s.clone())),
        Expression::Literal(Literal::Integer(n)) => Ok(EvalValue::Integer(*n)),
        Expression::Literal(Literal::Boolean(b)) => Ok(EvalValue::Boolean(*b)),
        Expression::Identifier(name) => {
            if name == param_name {
                Ok(EvalValue::String(param_value.to_string()))
            } else if let Some(value) = context.get(name) {
                Ok(value.clone())
            } else {
                Err(Error::Validation(ValidationError {
                    message: format!("Unknown identifier: {}", name),
                    value_type: "".to_string(),
                }))
            }
        }
        Expression::MemberAccess(obj, member) => {
            if let Expression::Identifier(name) = &**obj {
                if name == param_name && member == "length" {
                    Ok(EvalValue::Integer(param_value.len() as i64))
                } else {
                    Err(Error::Validation(ValidationError {
                        message: format!("Unknown member access: {}.{}", name, member),
                        value_type: "".to_string(),
                    }))
                }
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Complex member access not supported".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        Expression::Binary(BinaryOp::Multiply, left, right) => {
            let left_val = evaluate_expression_to_value(left, param_value, param_name, context)?;
            let right_val = evaluate_expression_to_value(right, param_value, param_name, context)?;
            
            if let (EvalValue::Integer(l), EvalValue::Integer(r)) = (left_val, right_val) {
                Ok(EvalValue::Integer(l * r))
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Multiplication requires integers".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        Expression::Binary(BinaryOp::Divide, left, right) => {
            let left_val = evaluate_expression_to_value(left, param_value, param_name, context)?;
            let right_val = evaluate_expression_to_value(right, param_value, param_name, context)?;
            
            if let (EvalValue::Integer(l), EvalValue::Integer(r)) = (left_val, right_val) {
                if r != 0 {
                    Ok(EvalValue::Integer(l / r))
                } else {
                    Err(Error::Validation(ValidationError {
                        message: "Division by zero".to_string(),
                        value_type: "".to_string(),
                    }))
                }
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Division requires integers".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        Expression::Binary(BinaryOp::Add, left, right) => {
            let left_val = evaluate_expression_to_value(left, param_value, param_name, context)?;
            let right_val = evaluate_expression_to_value(right, param_value, param_name, context)?;
            
            if let (EvalValue::Integer(l), EvalValue::Integer(r)) = (left_val, right_val) {
                Ok(EvalValue::Integer(l + r))
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Addition requires integers".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        _ => Err(Error::Validation(ValidationError {
            message: "Expression type not supported in let binding".to_string(),
            value_type: "".to_string(),
        })),
    }
}

// Helper function to evaluate an expression to a value (for integer context)
fn evaluate_expression_to_value_int(
    expr: &Expression,
    param_value: i64,
    param_name: &str,
    context: &HashMap<String, EvalValue>,
) -> Result<EvalValue> {
    match expr {
        Expression::Literal(Literal::String(s)) => Ok(EvalValue::String(s.clone())),
        Expression::Literal(Literal::Integer(n)) => Ok(EvalValue::Integer(*n)),
        Expression::Literal(Literal::Boolean(b)) => Ok(EvalValue::Boolean(*b)),
        Expression::Identifier(name) => {
            if name == param_name {
                Ok(EvalValue::Integer(param_value))
            } else if let Some(value) = context.get(name) {
                Ok(value.clone())
            } else {
                Err(Error::Validation(ValidationError {
                    message: format!("Unknown identifier: {}", name),
                    value_type: "".to_string(),
                }))
            }
        }
        Expression::Binary(BinaryOp::Multiply, left, right) => {
            let left_val = evaluate_expression_to_value_int(left, param_value, param_name, context)?;
            let right_val = evaluate_expression_to_value_int(right, param_value, param_name, context)?;
            
            if let (EvalValue::Integer(l), EvalValue::Integer(r)) = (left_val, right_val) {
                Ok(EvalValue::Integer(l * r))
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Multiplication requires integers".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        Expression::Binary(BinaryOp::Divide, left, right) => {
            let left_val = evaluate_expression_to_value_int(left, param_value, param_name, context)?;
            let right_val = evaluate_expression_to_value_int(right, param_value, param_name, context)?;
            
            if let (EvalValue::Integer(l), EvalValue::Integer(r)) = (left_val, right_val) {
                if r != 0 {
                    Ok(EvalValue::Integer(l / r))
                } else {
                    Err(Error::Validation(ValidationError {
                        message: "Division by zero".to_string(),
                        value_type: "".to_string(),
                    }))
                }
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Division requires integers".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        Expression::Binary(BinaryOp::Add, left, right) => {
            let left_val = evaluate_expression_to_value_int(left, param_value, param_name, context)?;
            let right_val = evaluate_expression_to_value_int(right, param_value, param_name, context)?;
            
            if let (EvalValue::Integer(l), EvalValue::Integer(r)) = (left_val, right_val) {
                Ok(EvalValue::Integer(l + r))
            } else {
                Err(Error::Validation(ValidationError {
                    message: "Addition requires integers".to_string(),
                    value_type: "".to_string(),
                }))
            }
        }
        _ => Err(Error::Validation(ValidationError {
            message: "Expression type not supported in let binding".to_string(),
            value_type: "".to_string(),
        })),
    }
}
