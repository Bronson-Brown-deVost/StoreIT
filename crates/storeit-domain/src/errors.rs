use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("entity not found: {entity_type} {id}")]
    NotFound { entity_type: String, id: Uuid },

    #[error("entity not empty: {entity_type} {id} contains {child_count} children")]
    NotEmpty {
        entity_type: String,
        id: Uuid,
        child_count: i64,
    },

    #[error("circular reference detected: {entity_type} {id} cannot be its own ancestor")]
    CircularReference { entity_type: String, id: Uuid },

    #[error("invalid parent: a {child_type} cannot have a parent of type {parent_type}")]
    InvalidParent {
        child_type: String,
        parent_type: String,
    },

    #[error("invalid entity type: {0}")]
    InvalidEntityType(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("authentication required")]
    Unauthenticated,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("auth provider error: {0}")]
    AuthProvider(String),

    #[error("invalid username or password")]
    InvalidCredentials,
}

pub type Result<T> = std::result::Result<T, DomainError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display() {
        let id = Uuid::nil();
        let err = DomainError::NotFound {
            entity_type: "item".into(),
            id,
        };
        assert_eq!(err.to_string(), format!("entity not found: item {id}"));
    }

    #[test]
    fn not_empty_display() {
        let id = Uuid::nil();
        let err = DomainError::NotEmpty {
            entity_type: "container".into(),
            id,
            child_count: 5,
        };
        assert!(err.to_string().contains("5 children"));
    }

    #[test]
    fn circular_reference_display() {
        let id = Uuid::nil();
        let err = DomainError::CircularReference {
            entity_type: "container".into(),
            id,
        };
        assert!(err.to_string().contains("circular reference"));
    }

    #[test]
    fn invalid_parent_display() {
        let err = DomainError::InvalidParent {
            child_type: "item".into(),
            parent_type: "item".into(),
        };
        assert!(err.to_string().contains("item"));
    }

    #[test]
    fn invalid_entity_type_display() {
        let err = DomainError::InvalidEntityType("bogus".into());
        assert!(err.to_string().contains("bogus"));
    }

    #[test]
    fn validation_display() {
        let err = DomainError::Validation("name required".into());
        assert!(err.to_string().contains("name required"));
    }

    #[test]
    fn storage_display() {
        let err = DomainError::Storage("disk full".into());
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn database_display() {
        let err = DomainError::Database("connection lost".into());
        assert!(err.to_string().contains("connection lost"));
    }

    #[test]
    fn internal_display() {
        let err = DomainError::Internal("unexpected".into());
        assert!(err.to_string().contains("unexpected"));
    }

    #[test]
    fn unauthenticated_display() {
        let err = DomainError::Unauthenticated;
        assert_eq!(err.to_string(), "authentication required");
    }

    #[test]
    fn forbidden_display() {
        let err = DomainError::Forbidden("not a member".into());
        assert!(err.to_string().contains("not a member"));
    }

    #[test]
    fn auth_provider_display() {
        let err = DomainError::AuthProvider("OIDC timeout".into());
        assert!(err.to_string().contains("OIDC timeout"));
    }

    #[test]
    fn invalid_credentials_display() {
        let err = DomainError::InvalidCredentials;
        assert_eq!(err.to_string(), "invalid username or password");
    }

    #[test]
    fn result_type_alias_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn result_type_alias_err() {
        let result: Result<i32> = Err(DomainError::Validation("bad".into()));
        assert!(result.is_err());
    }
}
