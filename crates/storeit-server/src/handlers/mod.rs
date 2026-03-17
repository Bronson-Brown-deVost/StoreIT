pub mod admin;
pub mod auth;
pub mod containers;
pub mod identify;
pub mod items;
pub mod local_auth;
pub mod locations;
pub mod nfc;
pub mod photos;
pub mod search;

use storeit_domain::errors::DomainError;
use uuid::Uuid;

use crate::error::AppError;

pub fn not_found(entity_type: &str, id: Uuid) -> AppError {
    AppError(DomainError::NotFound {
        entity_type: entity_type.to_string(),
        id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn not_found_returns_404() {
        let id = Uuid::nil();
        let err = not_found("item", id);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn not_found_preserves_entity_type() {
        let id = Uuid::nil();
        let err = not_found("container", id);
        match err.0 {
            DomainError::NotFound {
                entity_type,
                id: err_id,
            } => {
                assert_eq!(entity_type, "container");
                assert_eq!(err_id, id);
            }
            _ => panic!("expected NotFound variant"),
        }
    }
}
