use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub description: Option<String>,
    // Atomic Habits: Make it Attractive — identity-based habits
    pub identity: Option<String>,  // "I am someone who..." statement
}

impl Category {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            identity: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_identity(mut self, identity: String) -> Self {
        self.identity = Some(identity);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_new_defaults() {
        let cat = Category::new("work".to_string());
        assert_eq!(cat.name, "work");
        assert!(cat.description.is_none());
        assert!(cat.identity.is_none());
    }

    #[test]
    fn test_with_description() {
        let cat = Category::new("work".to_string()).with_description("Work tasks".to_string());
        assert_eq!(cat.description, Some("Work tasks".to_string()));
    }

    #[test]
    fn test_with_identity() {
        let cat = Category::new("health".to_string())
            .with_identity("I am someone who moves every day".to_string());
        assert_eq!(cat.identity, Some("I am someone who moves every day".to_string()));
    }

    #[test]
    fn test_all_fields() {
        let cat = Category::new("fitness".to_string())
            .with_description("Physical activities".to_string())
            .with_identity("I am an athlete".to_string());
        assert_eq!(cat.name, "fitness");
        assert_eq!(cat.description, Some("Physical activities".to_string()));
        assert_eq!(cat.identity, Some("I am an athlete".to_string()));
    }
}
