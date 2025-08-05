use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Int,
    Bool,
    Value(String),
    Any,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    values: HashMap<String, ValueType>,
    // Unified storage: all functions can have multiple implementations
    functions: HashMap<String, Vec<FunctionType>>,
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub name: String,
    pub parameter_types: Vec<Type>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub parameter_types: Vec<Type>,
    pub return_type: Type,
    pub guards: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct ValueType {
    pub name: String,
    pub parameter_type: Type,
    pub constraints: Constraints,
}

#[derive(Debug, Clone)]
pub struct Constraints {
    pub validate: Option<String>,
    pub normalize: Option<String>,
    pub unique: bool,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn define_value(&mut self, name: String, value_type: ValueType) {
        self.values.insert(name, value_type);
    }

    pub fn get_value(&self, name: &str) -> Option<&ValueType> {
        self.values.get(name)
    }

    pub fn define_function(&mut self, name: String, parameter_types: Vec<Type>, return_type: Type) {
        let function_type = FunctionType {
            name: name.clone(),
            parameter_types,
            return_type,
        };
        self.functions.entry(name).or_insert_with(Vec::new).push(function_type);
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionType> {
        // For backward compatibility, return the first function if only one exists
        self.functions.get(name).and_then(|funcs| {
            if funcs.len() == 1 {
                funcs.first()
            } else {
                None
            }
        })
    }
    
    pub fn get_functions(&self, name: &str) -> Option<&Vec<FunctionType>> {
        self.functions.get(name)
    }
    
    // Methods are now just functions with multiple implementations
    pub fn define_method(&mut self, name: String, signature: MethodSignature) {
        let function_type = FunctionType {
            name: name.clone(),
            parameter_types: signature.parameter_types,
            return_type: signature.return_type,
        };
        self.functions.entry(name).or_insert_with(Vec::new).push(function_type);
    }
    
    pub fn get_methods(&self, _name: &str) -> Option<&Vec<MethodSignature>> {
        // For backward compatibility during transition
        None
    }
}
