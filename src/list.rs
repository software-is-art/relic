use crate::value::ValueObject;
use crate::error::Result;
use std::any::Any;
use std::sync::Arc;

/// Minimal List implementation for the Type-as-Relation model
/// This supports the essential operations needed for relational programming
#[derive(Debug, Clone)]
pub struct List {
    pub items: Vec<Arc<dyn ValueObject>>,
    pub element_type: String, // Type name of elements
}

impl List {
    pub fn new(element_type: String) -> Self {
        Self {
            items: Vec::new(),
            element_type,
        }
    }

    pub fn from_items(items: Vec<Arc<dyn ValueObject>>, element_type: String) -> Self {
        Self {
            items,
            element_type,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn push(&mut self, item: Arc<dyn ValueObject>) {
        self.items.push(item);
    }

    pub fn get(&self, index: usize) -> Option<&Arc<dyn ValueObject>> {
        self.items.get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<Arc<dyn ValueObject>> {
        self.items.iter()
    }

    // Essential methods for functional programming
    pub fn filter<F>(&self, predicate: F) -> List
    where
        F: Fn(&Arc<dyn ValueObject>) -> bool,
    {
        let filtered_items: Vec<Arc<dyn ValueObject>> = self.items
            .iter()
            .filter(|item| predicate(item))
            .cloned()
            .collect();

        List::from_items(filtered_items, self.element_type.clone())
    }

    pub fn map<F>(&self, mapper: F) -> List
    where
        F: Fn(&Arc<dyn ValueObject>) -> Arc<dyn ValueObject>,
    {
        let mapped_items: Vec<Arc<dyn ValueObject>> = self.items
            .iter()
            .map(|item| mapper(item))
            .collect();

        // Note: mapping might change the element type, but for now we keep it simple
        List::from_items(mapped_items, self.element_type.clone())
    }

    pub fn find<F>(&self, predicate: F) -> Option<Arc<dyn ValueObject>>
    where
        F: Fn(&Arc<dyn ValueObject>) -> bool,
    {
        self.items
            .iter()
            .find(|item| predicate(item))
            .cloned()
    }

    pub fn any<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Arc<dyn ValueObject>) -> bool,
    {
        self.items.iter().any(|item| predicate(item))
    }

    pub fn all<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Arc<dyn ValueObject>) -> bool,
    {
        self.items.iter().all(|item| predicate(item))
    }
}

impl std::fmt::Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "List[{}]([", self.element_type)?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "])")
    }
}

// Implement ValueObject for List so it can be used in the system
impl ValueObject for List {
    fn validate(&self) -> Result<()> {
        // Lists are always valid - validation happens on individual elements
        Ok(())
    }

    fn normalize(&mut self) -> Result<()> {
        // No normalization needed for lists themselves
        Ok(())
    }

    fn type_name(&self) -> &'static str {
        "List"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn equals(&self, other: &dyn ValueObject) -> bool {
        if let Some(other_list) = other.as_any().downcast_ref::<List>() {
            if self.element_type != other_list.element_type || self.items.len() != other_list.items.len() {
                return false;
            }
            
            for (a, b) in self.items.iter().zip(other_list.items.iter()) {
                if !a.equals(&**b) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    fn hash_value(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.element_type.hash(&mut hasher);
        self.items.len().hash(&mut hasher);
        for item in &self.items {
            item.hash_value().hash(&mut hasher);
        }
        hasher.finish()
    }
}