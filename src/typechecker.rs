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
            Declaration::Function(func_decl) => self.check_function_declaration(func_decl),
            Declaration::Method(method_decl) => self.check_method_declaration(method_decl),
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

    fn check_function_declaration(&mut self, decl: &FunctionDeclaration) -> Result<()> {
        // With unified syntax and multiple dispatch, we allow multiple implementations
        // with the same parameter types (they may have different guards)

        // Set up local environment for checking the function body
        self.locals.clear();
        for param in &decl.parameters {
            self.locals.insert(param.name.clone(), param.ty.clone());
        }
        
        // Check guards if present
        for param in &decl.parameters {
            if let Some(ref guard) = param.guard {
                let guard_type = self.check_expression(guard)?;
                if guard_type != Type::Bool {
                    return Err(Error::Type(TypeError {
                        message: format!("Function guard must return Bool, found {:?}", guard_type),
                    }));
                }
            }
        }

        // Type check the function body
        let body_type = self.check_expression(&decl.body)?;
        
        // Ensure body type matches declared return type
        if body_type != decl.return_type {
            return Err(Error::Type(TypeError {
                message: format!(
                    "Function body returns {:?} but declared return type is {:?}",
                    body_type, decl.return_type
                ),
            }));
        }

        // Register the function in the environment
        let param_types: Vec<Type> = decl.parameters.iter().map(|p| p.ty.clone()).collect();
        self.env.define_function(
            decl.name.clone(),
            param_types,
            decl.return_type.clone(),
        );

        Ok(())
    }

    fn check_method_declaration(&mut self, decl: &MethodDeclaration) -> Result<()> {
        // With unified syntax and multiple dispatch, we allow multiple implementations
        // with the same parameter types (they may have different guards)
        
        // Set up local environment for checking the method body
        self.locals.clear();
        for param in &decl.parameters {
            self.locals.insert(param.name.clone(), param.ty.clone());
        }
        
        // Check guards if present
        for param in &decl.parameters {
            if let Some(ref guard) = param.guard {
                let guard_type = self.check_expression(guard)?;
                if guard_type != Type::Bool {
                    return Err(Error::Type(TypeError {
                        message: format!("Method guard must return Bool, found {:?}", guard_type),
                    }));
                }
            }
        }

        // Check method body
        let body_type = self.check_expression(&decl.body)?;
        if body_type != decl.return_type {
            return Err(Error::Type(TypeError {
                message: format!(
                    "Method body returns {:?} but declared return type is {:?}",
                    body_type, decl.return_type
                ),
            }));
        }
        
        // Check for ambiguity with existing methods
        if let Some(existing_methods) = self.env.get_methods(&decl.name) {
            let new_param_types: Vec<_> = decl.parameters.iter().map(|p| &p.ty).collect();
            
            for existing in existing_methods {
                // Check if parameter types match exactly (potential ambiguity)
                if existing.parameter_types.len() == new_param_types.len() {
                    let all_match = existing.parameter_types.iter()
                        .zip(&new_param_types)
                        .all(|(existing_ty, new_ty)| existing_ty == *new_ty);
                        
                    if all_match {
                        return Err(Error::Type(TypeError {
                            message: format!(
                                "Ambiguous method definition: method '{}' with the same parameter types already exists",
                                decl.name
                            ),
                        }));
                    }
                }
            }
        }

        // Register the method in the environment
        let param_types: Vec<Type> = decl.parameters.iter().map(|p| p.ty.clone()).collect();
        let guards: Vec<Option<String>> = decl.parameters.iter()
            .map(|p| p.guard.as_ref().map(|_| "custom".to_string()))
            .collect();
        
        use crate::types::MethodSignature;
        let signature = MethodSignature {
            parameter_types: param_types,
            return_type: decl.return_type.clone(),
            guards,
        };
        
        self.env.define_method(decl.name.clone(), signature);

        Ok(())
    }

    pub fn check_expression(&self, expr: &Expression) -> Result<Type> {
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
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => {
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

            Expression::FunctionCall(name, args) => {
                // With unified syntax, all functions can have multiple implementations
                if let Some(functions) = self.env.get_functions(name) {
                    // Collect argument types
                    let arg_types: Vec<Type> = args.iter()
                        .map(|arg| self.check_expression(arg))
                        .collect::<Result<Vec<_>>>()?;
                    
                    // If only one function, use simple type checking
                    if functions.len() == 1 {
                        let func_type = &functions[0];
                        // Check argument count
                        if args.len() != func_type.parameter_types.len() {
                            return Err(Error::Type(TypeError {
                                message: format!(
                                    "Function '{}' expects {} arguments, but {} provided",
                                    name,
                                    func_type.parameter_types.len(),
                                    args.len()
                                ),
                            }));
                        }
                        // Check argument types
                        for (i, (actual, expected)) in arg_types.iter().zip(&func_type.parameter_types).enumerate() {
                            if actual != expected {
                                return Err(Error::Type(TypeError {
                                    message: format!(
                                        "Function '{}' parameter {} expects {:?}, but {:?} provided",
                                        name, i + 1, expected, actual
                                    ),
                                }));
                            }
                        }
                        Ok(func_type.return_type.clone())
                    } else {
                        // Multiple implementations - find matching one
                        for func_type in functions {
                            if func_type.parameter_types.len() != arg_types.len() {
                                continue;
                            }
                            
                            // Check if all parameter types match
                            let matches = func_type.parameter_types.iter()
                                .zip(&arg_types)
                                .all(|(expected, actual)| expected == actual);
                                
                            if matches {
                                return Ok(func_type.return_type.clone());
                            }
                        }
                        
                        Err(Error::Type(TypeError {
                            message: format!(
                                "No matching function '{}' found for argument types {:?}",
                                name, arg_types
                            ),
                        }))
                    }
                } else if let Some(methods) = self.env.get_methods(name) {
                    // Handle as a method call with multiple dispatch
                    // Collect argument types
                    let arg_types: Vec<Type> = args.iter()
                        .map(|arg| self.check_expression(arg))
                        .collect::<Result<Vec<_>>>()?;
                    
                    // Find the best matching method
                    let mut best_match = None;
                    for method in methods {
                        if method.parameter_types.len() != arg_types.len() {
                            continue;
                        }
                        
                        // Check if all parameter types match
                        let matches = method.parameter_types.iter()
                            .zip(&arg_types)
                            .all(|(expected, actual)| expected == actual);
                            
                        if matches {
                            best_match = Some(method);
                            break; // For now, take the first exact match
                        }
                    }
                    
                    if let Some(method) = best_match {
                        Ok(method.return_type.clone())
                    } else {
                        Err(Error::Type(TypeError {
                            message: format!(
                                "No matching method '{}' found for argument types {:?}",
                                name, arg_types
                            ),
                        }))
                    }
                } else {
                    Err(Error::Type(TypeError {
                        message: format!("Undefined function or method: {}", name),
                    }))
                }
            },

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
                // Get the object type first
                let object_type = self.check_expression(object)?;
                
                // Collect all argument types (object type + arg types)
                let mut all_arg_types = vec![object_type.clone()];
                for arg in args {
                    all_arg_types.push(self.check_expression(arg)?);
                }
                
                // With unified syntax, check for all functions (UFC syntax)
                if let Some(functions) = self.env.get_functions(method) {
                    // If only one function implementation
                    if functions.len() == 1 {
                        let func_type = &functions[0];
                        // Transform x.f(y, z) into f(x, y, z) for type checking
                        // Check that the function can accept the object as first parameter
                        if func_type.parameter_types.is_empty() {
                            return Err(Error::Type(TypeError {
                                message: format!("Function {} takes no parameters", method),
                            }));
                        }
                        
                        if func_type.parameter_types[0] != object_type {
                            return Err(Error::Type(TypeError {
                                message: format!(
                                    "Cannot call {} on type {:?}, expected {:?}",
                                    method, object_type, func_type.parameter_types[0]
                                ),
                            }));
                        }
                        
                        // Check remaining arguments
                        if args.len() != func_type.parameter_types.len() - 1 {
                            return Err(Error::Type(TypeError {
                                message: format!(
                                    "Function {} expects {} arguments, got {}",
                                    method,
                                    func_type.parameter_types.len() - 1,
                                    args.len()
                                ),
                            }));
                        }
                        
                        for (i, arg_type) in all_arg_types[1..].iter().enumerate() {
                            let expected_type = &func_type.parameter_types[i + 1];
                            if arg_type != expected_type {
                                return Err(Error::Type(TypeError {
                                    message: format!(
                                        "Function {} parameter {} type mismatch: expected {:?}, got {:?}",
                                        method, i + 2, expected_type, arg_type
                                    ),
                                }));
                            }
                        }
                        
                        return Ok(func_type.return_type.clone());
                    } else {
                        // Multiple implementations - find matching one
                        for func_type in functions {
                            if func_type.parameter_types.len() != all_arg_types.len() {
                                continue;
                            }
                            
                            // Check if all parameter types match
                            let matches = func_type.parameter_types.iter()
                                .zip(&all_arg_types)
                                .all(|(expected, actual)| expected == actual);
                                
                            if matches {
                                return Ok(func_type.return_type.clone());
                            }
                        }
                        
                        return Err(Error::Type(TypeError {
                            message: format!(
                                "No matching function '{}' found for argument types {:?}",
                                method, all_arg_types
                            ),
                        }));
                    }
                }
                
                // For backward compatibility, check methods
                if let Some(methods) = self.env.get_methods(method) {
                    // Find the best matching method
                    let mut best_match = None;
                    for method_sig in methods {
                        if method_sig.parameter_types.len() != all_arg_types.len() {
                            continue;
                        }
                        
                        // Check if all parameter types match
                        let matches = method_sig.parameter_types.iter()
                            .zip(&all_arg_types)
                            .all(|(expected, actual)| expected == actual);
                            
                        if matches {
                            best_match = Some(method_sig);
                            break; // For now, take the first exact match
                        }
                    }
                    
                    if let Some(method_sig) = best_match {
                        return Ok(method_sig.return_type.clone());
                    }
                }
                
                // Otherwise, handle built-in methods
                match (&object_type, method.as_str()) {
                    (Type::String, "toLowerCase") => {
                        if !args.is_empty() {
                            return Err(Error::Type(TypeError {
                                message: "toLowerCase takes no arguments".to_string(),
                            }));
                        }
                        Ok(Type::String)
                    }
                    (Type::String, "toUpperCase") => {
                        if !args.is_empty() {
                            return Err(Error::Type(TypeError {
                                message: "toUpperCase takes no arguments".to_string(),
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
            
            Expression::Match(expr, arms) => {
                let expr_type = self.check_expression(expr)?;
                
                // Check that we're matching on a value type
                let value_name = match &expr_type {
                    Type::Value(name) => name,
                    _ => return Err(Error::Type(TypeError {
                        message: format!("Can only match on value types, found {:?}", expr_type),
                    })),
                };
                
                // Get the value type definition
                let value_type = self.env.get_value(value_name).ok_or_else(|| {
                    Error::Type(TypeError {
                        message: format!("Unknown value type: {}", value_name),
                    })
                })?;
                
                // All arms must have the same result type
                let mut result_type = None;
                
                for arm in arms {
                    match &arm.pattern {
                        Pattern::Constructor(constructor, binding) => {
                            // Check that the constructor matches the value type
                            if constructor != value_name {
                                return Err(Error::Type(TypeError {
                                    message: format!(
                                        "Pattern constructor '{}' doesn't match value type '{}'",
                                        constructor, value_name
                                    ),
                                }));
                            }
                            
                            // Create environment with pattern binding
                            let mut extended_checker = TypeChecker {
                                env: self.env.clone(),
                                locals: self.locals.clone(),
                            };
                            extended_checker.locals.insert(binding.clone(), value_type.parameter_type.clone());
                            
                            // Check arm body
                            let arm_type = extended_checker.check_expression(&arm.body)?;
                            
                            // Ensure all arms have the same type
                            match &result_type {
                                None => result_type = Some(arm_type),
                                Some(expected) => {
                                    if arm_type != *expected {
                                        return Err(Error::Type(TypeError {
                                            message: format!(
                                                "Match arms have different types: {:?} and {:?}",
                                                expected, arm_type
                                            ),
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
                
                result_type.ok_or_else(|| Error::Type(TypeError {
                    message: "Match expression has no arms".to_string(),
                }))
            }
        }
    }

    pub fn get_environment(&self) -> &TypeEnvironment {
        &self.env
    }
}
