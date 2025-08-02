use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Int,
    Bool,
    Value(String),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    values: HashMap<String, ValueType>,
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
        }
    }

    pub fn define_value(&mut self, name: String, value_type: ValueType) {
        self.values.insert(name, value_type);
    }

    pub fn get_value(&self, name: &str) -> Option<&ValueType> {
        self.values.get(name)
    }
}
