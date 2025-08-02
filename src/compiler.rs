use crate::ast::*;
use crate::error::{Error, Result, ValidationError};
use crate::value::{ValueConstructor, ValueRegistry};
use std::any::Any;

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

// Simplified expression evaluation functions
fn evaluate_string_validation(value: &str, expr: &Expression, param_name: &str) -> Result<bool> {
    match expr {
        Expression::Binary(BinaryOp::And, left, right) => {
            Ok(evaluate_string_validation(value, left, param_name)?
                && evaluate_string_validation(value, right, param_name)?)
        }
        Expression::Comparison(ComparisonOp::Contains, left, right) => {
            if let Expression::Identifier(name) = &**left {
                if name == param_name {
                    if let Expression::Literal(Literal::String(s)) = &**right {
                        return Ok(value.contains(s));
                    }
                }
            }
            Ok(false)
        }
        Expression::Comparison(ComparisonOp::Greater, left, right) => {
            if let Expression::MemberAccess(obj, member) = &**left {
                if let Expression::Identifier(name) = &**obj {
                    if name == param_name && member == "length" {
                        if let Expression::Literal(Literal::Integer(n)) = &**right {
                            return Ok(value.len() > *n as usize);
                        }
                    }
                }
            }
            Ok(false)
        }
        Expression::Pipeline(left, _right) => {
            // For now, just evaluate the left side
            // Full implementation would need to apply right as a function to left
            evaluate_string_validation(value, left, param_name)
        }
        _ => Ok(true),
    }
}

fn evaluate_int_validation(value: i64, expr: &Expression, param_name: &str) -> Result<bool> {
    match expr {
        Expression::Comparison(ComparisonOp::Greater, left, right) => {
            if let Expression::Identifier(name) = &**left {
                if name == param_name {
                    if let Expression::Literal(Literal::Integer(n)) = &**right {
                        return Ok(value > *n);
                    }
                }
            }
            Ok(false)
        }
        Expression::Pipeline(left, _right) => {
            // For now, just evaluate the left side
            // Full implementation would need to apply right as a function to left
            evaluate_int_validation(value, left, param_name)
        }
        _ => Ok(true),
    }
}
