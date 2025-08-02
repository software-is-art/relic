use crate::ast::ValueDeclaration;
use crate::error::{Error, Result, ValidationError};
use std::any::Any;
use std::collections::HashMap;

pub trait ValueObject: Any + Send + Sync {
    fn validate(&self) -> Result<()>;
    fn normalize(&mut self) -> Result<()>;
    fn type_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

pub struct ValueConstructor {
    pub declaration: ValueDeclaration,
    pub validator: Box<dyn Fn(&(dyn Any + Send + Sync)) -> Result<()> + Send + Sync>,
    pub normalizer: Option<Box<dyn Fn(&mut (dyn Any + Send + Sync)) -> Result<()> + Send + Sync>>,
}

pub struct ValueRegistry {
    constructors: HashMap<String, ValueConstructor>,
}

impl ValueRegistry {
    pub fn new() -> Self {
        Self {
            constructors: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, constructor: ValueConstructor) {
        self.constructors.insert(name, constructor);
    }

    pub fn construct(
        &self,
        type_name: &str,
        input: Box<dyn Any + Send + Sync>,
    ) -> Result<Box<dyn ValueObject>> {
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

        Ok(value)
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

struct GenericValueObject {
    type_name: String,
    data: Box<dyn Any + Send + Sync>,
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
}

// Example implementation for EmailAddress value type
pub struct EmailAddress {
    value: String,
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
}
