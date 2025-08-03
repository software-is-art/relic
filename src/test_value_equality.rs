#[cfg(test)]
mod tests {
    use crate::value::{EmailAddress, ValueObject};

    #[test]
    fn test_email_address_equality() {
        let email1 = EmailAddress::from("Test@Example.com".to_string()).unwrap();
        let email2 = EmailAddress::from("test@example.com".to_string()).unwrap();
        let email3 = EmailAddress::from("other@example.com".to_string()).unwrap();

        // Test equality - should be equal after normalization
        assert_eq!(email1, email2);
        assert_ne!(email1, email3);
    }

    #[test]
    fn test_email_address_hash() {
        use std::collections::HashSet;

        let email1 = EmailAddress::from("Test@Example.com".to_string()).unwrap();
        let email2 = EmailAddress::from("test@example.com".to_string()).unwrap();
        let email3 = EmailAddress::from("other@example.com".to_string()).unwrap();

        let mut set = HashSet::new();
        set.insert(email1.clone());
        
        // Should not insert duplicate (normalized to same value)
        assert!(!set.insert(email2.clone()));
        
        // Should insert different email
        assert!(set.insert(email3));
        
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_value_object_trait_equality() {
        let email1 = EmailAddress::from("test@example.com".to_string()).unwrap();
        let email2 = EmailAddress::from("TEST@EXAMPLE.COM".to_string()).unwrap();
        let email3 = EmailAddress::from("other@example.com".to_string()).unwrap();

        // Test through trait object
        let value1: &dyn ValueObject = &email1;
        let value2: &dyn ValueObject = &email2;
        let value3: &dyn ValueObject = &email3;

        assert!(value1.equals(value2));
        assert!(!value1.equals(value3));
    }

    #[test]
    fn test_value_object_trait_hash() {
        let email = EmailAddress::from("test@example.com".to_string()).unwrap();
        let value: &dyn ValueObject = &email;

        // Should produce consistent hash
        let hash1 = value.hash_value();
        let hash2 = value.hash_value();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_display() {
        let email = EmailAddress::from("test@example.com".to_string()).unwrap();
        assert_eq!(format!("{}", email), "EmailAddress(test@example.com)");
    }
}