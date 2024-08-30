use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::stacks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StackDB {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::stacks)]
pub struct NewStackDB {
    pub name: String,
    pub description: Option<String>,
}

impl NewStackDB {
    pub fn new(name: String, description: Option<String>) -> Result<Self, String> {
        // Validation
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if name.len() > 255 {
            return Err("Name cannot exceed 255 characters".to_string());
        }

        Ok(NewStackDB {
            name,
            description,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_success() {
        let name = "Test Stack".to_string();
        let description = Some("A test stack".to_string());

        let new_stack = NewStackDB::new(name.clone(), description.clone()).unwrap();

        assert_eq!(new_stack.name, name);
        assert_eq!(new_stack.description, description);
    }

    #[test]
    fn test_new_stack_without_description() {
        let name = "Test Stack".to_string();

        let new_stack = NewStackDB::new(name.clone(), None).unwrap();

        assert_eq!(new_stack.name, name);
        assert_eq!(new_stack.description, None);
    }

    #[test]
    fn test_new_stack_empty_name() {
        let result = NewStackDB::new("".to_string(), None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Name cannot be empty");
    }

    #[test]
    fn test_new_stack_name_too_long() {
        let result = NewStackDB::new("a".repeat(256), None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Name cannot exceed 255 characters");
    }
}