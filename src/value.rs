use crate::ast::{ValueDeclaration, FunctionDeclaration, MethodDeclaration};
use crate::error::{Error, Result, ValidationError};
use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::fmt::{Debug, Display};
use std::sync::{Arc, RwLock};

pub trait ValueObject: Any + Send + Sync + Debug + Display {
    fn validate(&self) -> Result<()>;
    fn normalize(&mut self) -> Result<()>;
    fn type_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn equals(&self, other: &dyn ValueObject) -> bool;
    fn hash_value(&self) -> u64;
}

pub struct ValueConstructor {
    pub declaration: ValueDeclaration,
    pub validator: Box<dyn Fn(&(dyn Any + Send + Sync)) -> Result<()> + Send + Sync>,
    pub normalizer: Option<Box<dyn Fn(&mut (dyn Any + Send + Sync)) -> Result<()> + Send + Sync>>,
}

pub struct ValueRegistry {
    pub(crate) constructors: HashMap<String, ValueConstructor>,
    // Unified storage: all functions can have multiple implementations
    functions: HashMap<String, Vec<FunctionDeclaration>>,
    // Type-as-Relation: Track all instances by type name
    // Using strong references to keep instances indefinitely
    instances: Arc<RwLock<HashMap<String, Vec<Arc<dyn ValueObject>>>>>,
}

impl ValueRegistry {
    pub fn new() -> Self {
        Self {
            constructors: HashMap::new(),
            functions: HashMap::new(),
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&mut self, name: String, constructor: ValueConstructor) {
        self.constructors.insert(name, constructor);
    }

    pub fn register_function(&mut self, func_decl: FunctionDeclaration) {
        self.functions.entry(func_decl.name.clone())
            .or_insert_with(Vec::new)
            .push(func_decl);
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionDeclaration> {
        // For backward compatibility, return the first function if only one exists
        self.functions.get(name).and_then(|funcs| {
            if funcs.len() == 1 {
                funcs.first()
            } else {
                None
            }
        })
    }
    
    pub fn get_functions(&self, name: &str) -> Option<&Vec<FunctionDeclaration>> {
        self.functions.get(name)
    }
    
    pub fn register_method(&mut self, method_decl: MethodDeclaration) {
        // Convert method to function for unified storage
        let func_decl = FunctionDeclaration {
            name: method_decl.name.clone(),
            parameters: method_decl.parameters.clone(),
            return_type: method_decl.return_type,
            body: method_decl.body,
        };
        self.register_function(func_decl);
    }
    
    pub fn get_methods(&self, _name: &str) -> Option<&Vec<MethodDeclaration>> {
        // For backward compatibility during transition
        None
    }

    pub fn execute_function(&self, name: &str, args: Vec<Box<dyn Any + Send + Sync>>) -> Result<Box<dyn Any + Send + Sync>> {
        let _func = self.get_function(name).ok_or_else(|| {
            Error::Validation(ValidationError {
                message: format!("Unknown function: {}", name),
                value_type: "function".to_string(),
            })
        })?;

        // For now, we'll just return a placeholder result
        // In a full implementation, this would interpret the function body
        Ok(Box::new(format!("Function {} called with {} arguments", name, args.len())))
    }

    pub fn construct(
        &self,
        type_name: &str,
        input: Box<dyn Any + Send + Sync>,
    ) -> Result<Arc<dyn ValueObject>> {
        let constructor = self.constructors.get(type_name).ok_or_else(|| {
            Error::Validation(ValidationError {
                message: format!("Unknown value type: {}", type_name),
                value_type: type_name.to_string(),
            })
        })?;

        // Validate the input
        (constructor.validator)(&*input)?;

        // Create the value object
        let value = self.create_value_object(type_name, input)?;
        let value_arc: Arc<dyn ValueObject> = Arc::from(value);

        // Register the instance for Type-as-Relation
        self.register_instance(type_name, value_arc.clone());

        Ok(value_arc)
    }

    fn register_instance(&self, type_name: &str, instance: Arc<dyn ValueObject>) {
        if let Ok(mut instances) = self.instances.write() {
            instances.entry(type_name.to_string())
                .or_insert_with(Vec::new)
                .push(instance);
        }
    }

    // Type-as-Relation query methods
    pub fn get_all_instances(&self, type_name: &str) -> Vec<Arc<dyn ValueObject>> {
        if let Ok(instances) = self.instances.read() {
            if let Some(type_instances) = instances.get(type_name) {
                // Simply return a clone of the Arc references
                return type_instances.clone();
            }
        }
        Vec::new()
    }

    pub fn count_instances(&self, type_name: &str) -> usize {
        self.get_all_instances(type_name).len()
    }

    fn create_value_object(
        &self,
        type_name: &str,
        input: Box<dyn Any + Send + Sync>,
    ) -> Result<Box<dyn ValueObject>> {
        // This would be expanded to create specific value object types
        // For now, we'll create a generic implementation
        Ok(Box::new(GenericValueObject {
            type_name: type_name.to_string(),
            data: input,
        }))
    }
}

#[derive(Debug)]
struct GenericValueObject {
    type_name: String,
    data: Box<dyn Any + Send + Sync>,
}

impl Display for GenericValueObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?})", self.type_name, self.data)
    }
}

impl ValueObject for GenericValueObject {
    fn validate(&self) -> Result<()> {
        // Validation already performed during construction
        Ok(())
    }

    fn normalize(&mut self) -> Result<()> {
        // Normalization would be performed here
        Ok(())
    }

    fn type_name(&self) -> &'static str {
        Box::leak(self.type_name.clone().into_boxed_str())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn equals(&self, other: &dyn ValueObject) -> bool {
        // First check if types match
        if self.type_name() != other.type_name() {
            return false;
        }

        // For now, we compare the debug representation
        // In a real implementation, we'd have type-specific comparison
        format!("{:?}", self.data) == format!("{:?}", other.as_any())
    }

    fn hash_value(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.type_name.hash(&mut hasher);
        // Hash the debug representation for now
        format!("{:?}", self.data).hash(&mut hasher);
        hasher.finish()
    }
}

// Example implementation for EmailAddress value type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress {
    value: String,
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EmailAddress({})", self.value)
    }
}

impl EmailAddress {
    pub fn from(raw: String) -> Result<Self> {
        // Validate
        if !raw.contains('@') || raw.len() <= 3 {
            return Err(Error::Validation(ValidationError {
                message: "Invalid email address".to_string(),
                value_type: "EmailAddress".to_string(),
            }));
        }

        // Normalize
        let normalized = raw.to_lowercase();

        Ok(EmailAddress { value: normalized })
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl ValueObject for EmailAddress {
    fn validate(&self) -> Result<()> {
        if !self.value.contains('@') || self.value.len() <= 3 {
            return Err(Error::Validation(ValidationError {
                message: "Invalid email address".to_string(),
                value_type: "EmailAddress".to_string(),
            }));
        }
        Ok(())
    }

    fn normalize(&mut self) -> Result<()> {
        self.value = self.value.to_lowercase();
        Ok(())
    }

    fn type_name(&self) -> &'static str {
        "EmailAddress"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn equals(&self, other: &dyn ValueObject) -> bool {
        if let Some(other_email) = other.as_any().downcast_ref::<EmailAddress>() {
            self == other_email
        } else {
            false
        }
    }

    fn hash_value(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
