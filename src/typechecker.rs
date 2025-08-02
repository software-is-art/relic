use crate::ast::*;
use crate::error::{Error, Result, TypeError};
use crate::types::{Constraints, Type, TypeEnvironment, ValueType};
use std::collections::HashMap;

pub struct TypeChecker {
    env: TypeEnvironment,
    locals: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnvironment::new(),
            locals: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<()> {
        for declaration in &program.declarations {
            self.check_declaration(declaration)?;
        }
        Ok(())
    }

    fn check_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        match declaration {
            Declaration::Value(value_decl) => self.check_value_declaration(value_decl),
        }
    }

    fn check_value_declaration(&mut self, decl: &ValueDeclaration) -> Result<()> {
        // Check if value type already exists
        if self.env.get_value(&decl.name).is_some() {
            return Err(Error::Type(TypeError {
                message: format!("Value type '{}' is already defined", decl.name),
            }));
        }

        // Set up local environment for checking the value body
        self.locals.clear();
        self.locals
            .insert(decl.parameter.name.clone(), decl.parameter.ty.clone());

        // Check validation expression if present
        if let Some(ref validate_expr) = decl.body.validate {
            let validate_type = self.check_expression(validate_expr)?;
            if validate_type != Type::Bool {
                return Err(Error::Type(TypeError {
                    message: format!(
                        "Validation expression must return Bool, found {:?}",
                        validate_type
                    ),
                }));
            }
        }

        // Check normalization expression if present
        if let Some(ref normalize_expr) = decl.body.normalize {
            let normalize_type = self.check_expression(normalize_expr)?;
            // Normalization should return the same type as the parameter
            if normalize_type != decl.parameter.ty {
                return Err(Error::Type(TypeError {
                    message: format!(
                        "Normalization expression must return {:?}, found {:?}",
                        decl.parameter.ty, normalize_type
                    ),
                }));
            }
        }

        // Register the value type
        let value_type = ValueType {
            name: decl.name.clone(),
            parameter_type: decl.parameter.ty.clone(),
            constraints: Constraints {
                validate: decl.body.validate.as_ref().map(|_| "custom".to_string()),
                normalize: decl.body.normalize.as_ref().map(|_| "custom".to_string()),
                unique: decl.body.unique.unwrap_or(false),
            },
        };

        self.env.define_value(decl.name.clone(), value_type);

        Ok(())
    }

    fn check_expression(&self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Binary(op, left, right) => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                match op {
                    BinaryOp::And | BinaryOp::Or => {
                        if left_type != Type::Bool || right_type != Type::Bool {
                            return Err(Error::Type(TypeError {
                                message: format!(
                                    "Logical operators require Bool operands, found {:?} and {:?}",
                                    left_type, right_type
                                ),
                            }));
                        }
                        Ok(Type::Bool)
                    }
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        if left_type != Type::Int || right_type != Type::Int {
                            return Err(Error::Type(TypeError {
                                message: format!(
                                    "Arithmetic operators require Int operands, found {:?} and {:?}",
                                    left_type, right_type
                                ),
                            }));
                        }
                        Ok(Type::Int)
                    }
                }
            }

            Expression::Unary(op, operand) => {
                let operand_type = self.check_expression(operand)?;

                match op {
                    UnaryOp::Not => {
                        if operand_type != Type::Bool {
                            return Err(Error::Type(TypeError {
                                message: format!(
                                    "Not operator requires Bool operand, found {:?}",
                                    operand_type
                                ),
                            }));
                        }
                        Ok(Type::Bool)
                    }
                    UnaryOp::Minus => {
                        if operand_type != Type::Int {
                            return Err(Error::Type(TypeError {
                                message: format!(
                                    "Unary minus requires Int operand, found {:?}",
                                    operand_type
                                ),
                            }));
                        }
                        Ok(Type::Int)
                    }
                }
            }

            Expression::Literal(lit) => match lit {
                Literal::String(_) => Ok(Type::String),
                Literal::Integer(_) => Ok(Type::Int),
                Literal::Boolean(_) => Ok(Type::Bool),
            },

            Expression::Identifier(name) => self.locals.get(name).cloned().ok_or_else(|| {
                Error::Type(TypeError {
                    message: format!("Undefined identifier: {}", name),
                })
            }),

            Expression::MemberAccess(object, member) => {
                let object_type = self.check_expression(object)?;

                // Handle built-in members
                match (&object_type, member.as_str()) {
                    (Type::String, "length") => Ok(Type::Int),
                    _ => Err(Error::Type(TypeError {
                        message: format!("Type {:?} has no member '{}'", object_type, member),
                    })),
                }
            }

            Expression::MethodCall(object, method, args) => {
                let object_type = self.check_expression(object)?;

                // Handle built-in methods
                match (&object_type, method.as_str()) {
                    (Type::String, "toLowerCase") => {
                        if !args.is_empty() {
                            return Err(Error::Type(TypeError {
                                message: "toLowerCase takes no arguments".to_string(),
                            }));
                        }
                        Ok(Type::String)
                    }
                    _ => Err(Error::Type(TypeError {
                        message: format!("Type {:?} has no method '{}'", object_type, method),
                    })),
                }
            }

            Expression::Comparison(op, left, right) => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                match op {
                    ComparisonOp::Contains => {
                        // Special case for 'contains' operator
                        if left_type != Type::String || right_type != Type::String {
                            return Err(Error::Type(TypeError {
                                message: format!(
                                    "Contains operator requires String operands, found {:?} and {:?}",
                                    left_type, right_type
                                ),
                            }));
                        }
                        Ok(Type::Bool)
                    }
                    _ => {
                        // For other comparisons, types must match
                        if left_type != right_type {
                            return Err(Error::Type(TypeError {
                                message: format!(
                                    "Comparison requires matching types, found {:?} and {:?}",
                                    left_type, right_type
                                ),
                            }));
                        }
                        Ok(Type::Bool)
                    }
                }
            }

            Expression::Pipeline(left, right) => {
                let _left_type = self.check_expression(left)?;
                // For pipeline, the right side should be a function that takes the left type
                // For now, we'll just ensure the right side can accept the left type
                // This is a simplified implementation - a full implementation would need
                // function types and proper application checking
                self.check_expression(right)
            }

            Expression::Let(name, value, body) => {
                let value_type = self.check_expression(value)?;
                
                // Create a new type checker with extended locals
                let mut extended_checker = TypeChecker {
                    env: self.env.clone(),
                    locals: self.locals.clone(),
                };
                extended_checker.locals.insert(name.clone(), value_type);
                
                // Check the body with the extended environment
                extended_checker.check_expression(body)
            }
        }
    }

    pub fn get_environment(&self) -> &TypeEnvironment {
        &self.env
    }
}
