use crate::ast::*;
use crate::error::{Error, Result, ValidationError};
use crate::value::{ValueConstructor, ValueRegistry};
use std::any::Any;
use std::collections::HashMap;

pub struct Compiler {
    registry: ValueRegistry,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            registry: ValueRegistry::new(),
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

    pub fn get_registry(&self) -> &ValueRegistry {
        &self.registry
    }

    pub fn into_registry(self) -> ValueRegistry {
        self.registry
    }
}

// Evaluation context for let-bindings
#[derive(Clone)]
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
        Expression::Comparison(ComparisonOp::Greater, left, right) => {
            // Evaluate left side
            let left_val = if let Expression::Identifier(name) = &**left {
                if name == param_name {
                    value
                } else if let Some(EvalValue::Integer(n)) = context.get(name) {
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
        _ => Ok(true),
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
