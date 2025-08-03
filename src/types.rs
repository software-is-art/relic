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
    functions: HashMap<String, FunctionType>,
    methods: HashMap<String, Vec<MethodSignature>>,
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
            methods: HashMap::new(),
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
        self.functions.insert(name, function_type);
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionType> {
        self.functions.get(name)
    }
    
    pub fn define_method(&mut self, name: String, signature: MethodSignature) {
        self.methods.entry(name).or_insert_with(Vec::new).push(signature);
    }
    
    pub fn get_methods(&self, name: &str) -> Option<&Vec<MethodSignature>> {
        self.methods.get(name)
    }
}
