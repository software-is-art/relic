use crate::error::{Error, Result, ValidationError};
use crate::value::ValueObject;
use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Schema describes the structure of rows in a relation
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub fields: Vec<(String, String)>, // (field_name, type_name)
}

/// Relation is a value type that holds a collection of validated rows
#[derive(Clone)]
pub struct Relation {
    schema: Schema,
    rows: Vec<Arc<HashMap<String, Box<dyn ValueObject>>>>,
    key_field: Option<String>,
    unique_fields: Vec<String>,
}

impl Relation {
    /// Create a new empty relation with a schema
    pub fn new(schema: Schema) -> Self {
        Relation {
            schema,
            rows: vec![],
            key_field: None,
            unique_fields: vec![],
        }
    }
    
    /// Set the key field
    pub fn with_key(mut self, key: String) -> Self {
        self.key_field = Some(key);
        self
    }
    
    /// Add unique constraints
    pub fn with_unique(mut self, fields: Vec<String>) -> Self {
        self.unique_fields = fields;
        self
    }
    
    /// Add a row to the relation (returns new relation - immutable)
    pub fn add_row(&self, row: HashMap<String, Box<dyn ValueObject>>) -> Result<Relation> {
        // Validate row matches schema
        self.validate_row(&row)?;
        
        // Check key uniqueness if applicable
        if let Some(ref key) = self.key_field {
            if let Some(key_value) = row.get(key) {
                for existing in &self.rows {
                    if let Some(existing_key) = existing.get(key) {
                        if self.values_equal(key_value.as_ref(), existing_key.as_ref()) {
                            return Err(Error::Validation(ValidationError {
                                message: format!("Duplicate key value for field '{}'", key),
                                value_type: "Relation".to_string(),
                            }));
                        }
                    }
                }
            }
        }
        
        // Check unique constraints
        for unique_field in &self.unique_fields {
            if let Some(value) = row.get(unique_field) {
                for existing in &self.rows {
                    if let Some(existing_value) = existing.get(unique_field) {
                        if self.values_equal(value.as_ref(), existing_value.as_ref()) {
                            return Err(Error::Validation(ValidationError {
                                message: format!("Duplicate value for unique field '{}'", unique_field),
                                value_type: "Relation".to_string(),
                            }));
                        }
                    }
                }
            }
        }
        
        // Create new relation with added row
        let mut new_rows = self.rows.clone();
        new_rows.push(Arc::new(row));
        
        Ok(Relation {
            schema: self.schema.clone(),
            rows: new_rows,
            key_field: self.key_field.clone(),
            unique_fields: self.unique_fields.clone(),
        })
    }
    
    /// Validate that a row matches the schema
    fn validate_row(&self, row: &HashMap<String, Box<dyn ValueObject>>) -> Result<()> {
        // Check all required fields are present
        for (field_name, _field_type) in &self.schema.fields {
            if !row.contains_key(field_name) {
                return Err(Error::Validation(ValidationError {
                    message: format!("Missing required field '{}'", field_name),
                    value_type: "Relation".to_string(),
                }));
            }
        }
        
        // Check no extra fields
        for field_name in row.keys() {
            if !self.schema.fields.iter().any(|(name, _)| name == field_name) {
                return Err(Error::Validation(ValidationError {
                    message: format!("Unknown field '{}'", field_name),
                    value_type: "Relation".to_string(),
                }));
            }
        }
        
        // TODO: Type checking when we have runtime type information
        
        Ok(())
    }
    
    /// Compare two values for equality
    fn values_equal(&self, a: &dyn ValueObject, b: &dyn ValueObject) -> bool {
        a.equals(b)
    }
    
    /// Get the rows of the relation
    pub fn rows(&self) -> &[Arc<HashMap<String, Box<dyn ValueObject>>>] {
        &self.rows
    }
    
    /// Get the schema of the relation
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

impl ValueObject for Relation {
    fn validate(&self) -> Result<()> {
        // Relations are always valid after construction
        Ok(())
    }
    
    fn normalize(&mut self) -> Result<()> {
        // Relations don't need normalization
        Ok(())
    }
    
    fn type_name(&self) -> &'static str {
        "Relation"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn equals(&self, other: &dyn ValueObject) -> bool {
        if let Some(other_rel) = other.as_any().downcast_ref::<Relation>() {
            // Compare schemas and rows
            self.schema == other_rel.schema &&
            self.rows.len() == other_rel.rows.len()
            // TODO: Proper row comparison
        } else {
            false
        }
    }
    
    fn hash_value(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        // Hash schema and row count
        for (name, typ) in &self.schema.fields {
            name.hash(&mut hasher);
            typ.hash(&mut hasher);
        }
        self.rows.len().hash(&mut hasher);
        hasher.finish()
    }
}

impl std::fmt::Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Relation({} rows)", self.rows.len())
    }
}

impl std::fmt::Debug for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Relation(schema: {:?}, {} rows)", self.schema, self.rows.len())
    }
}