use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
)]
#[diesel(table_name = crate::schema::generators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Generator {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: String,
    pub description: Option<String>,
    pub pak_hash: Option<String>,
    pub last_active_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::generators)]
pub struct NewGenerator {
    pub name: String,
    pub description: Option<String>,
}

impl NewGenerator {
    pub fn new(name: String, description: Option<String>) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Generator name cannot be empty".to_string());
        }

        Ok(NewGenerator { name, description })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generator_success() {
        let name = "Test Generator".to_string();
        let description = Some("A test generator".to_string());

        let result = NewGenerator::new(name.clone(), description.clone());

        assert!(
            result.is_ok(),
            "NewGenerator creation should succeed with valid inputs"
        );
        let new_generator = result.unwrap();
        assert_eq!(new_generator.name, name);
        assert_eq!(new_generator.description, description);
    }

    #[test]
    fn test_new_generator_empty_name() {
        let result = NewGenerator::new("".to_string(), None);
        assert!(
            result.is_err(),
            "NewGenerator creation should fail with empty name"
        );
        assert_eq!(
            result.unwrap_err(),
            "Generator name cannot be empty",
            "Error message should indicate empty name"
        );
    }
}
