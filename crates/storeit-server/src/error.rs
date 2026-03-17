use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use storeit_domain::errors::DomainError;

use crate::dto::ErrorResponse;

pub struct AppError(pub DomainError);

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        AppError(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self.0 {
            DomainError::NotFound { .. } => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND".to_string(),
                self.0.to_string(),
            ),
            DomainError::NotEmpty { .. } => (
                StatusCode::CONFLICT,
                "NOT_EMPTY".to_string(),
                self.0.to_string(),
            ),
            DomainError::CircularReference { .. } => (
                StatusCode::CONFLICT,
                "CIRCULAR_REFERENCE".to_string(),
                self.0.to_string(),
            ),
            DomainError::InvalidParent { .. } => (
                StatusCode::BAD_REQUEST,
                "INVALID_PARENT".to_string(),
                self.0.to_string(),
            ),
            DomainError::InvalidEntityType(_) => (
                StatusCode::BAD_REQUEST,
                "INVALID_ENTITY_TYPE".to_string(),
                self.0.to_string(),
            ),
            DomainError::Validation(_) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR".to_string(),
                self.0.to_string(),
            ),
            DomainError::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHENTICATED".to_string(),
                self.0.to_string(),
            ),
            DomainError::Forbidden(_) => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN".to_string(),
                self.0.to_string(),
            ),
            DomainError::AuthProvider(_) => (
                StatusCode::BAD_GATEWAY,
                "AUTH_PROVIDER_ERROR".to_string(),
                "auth provider error".to_string(),
            ),
            DomainError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS".to_string(),
                "Invalid username or password".to_string(),
            ),
            DomainError::Storage(_) | DomainError::Database(_) | DomainError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR".to_string(),
                "internal server error".to_string(),
            ),
        };

        (status, Json(ErrorResponse::new(code, message))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn not_found_maps_to_404() {
        let err = AppError(DomainError::NotFound {
            entity_type: "item".into(),
            id: Uuid::nil(),
        });
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn not_empty_maps_to_409() {
        let err = AppError(DomainError::NotEmpty {
            entity_type: "container".into(),
            id: Uuid::nil(),
            child_count: 3,
        });
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn circular_maps_to_409() {
        let err = AppError(DomainError::CircularReference {
            entity_type: "container".into(),
            id: Uuid::nil(),
        });
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn validation_maps_to_400() {
        let err = AppError(DomainError::Validation("bad".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn invalid_parent_maps_to_400() {
        let err = AppError(DomainError::InvalidParent {
            child_type: "item".into(),
            parent_type: "item".into(),
        });
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn invalid_entity_type_maps_to_400() {
        let err = AppError(DomainError::InvalidEntityType("bogus".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn storage_error_maps_to_500() {
        let err = AppError(DomainError::Storage("fail".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn database_error_maps_to_500() {
        let err = AppError(DomainError::Database("fail".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn internal_error_maps_to_500() {
        let err = AppError(DomainError::Internal("fail".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn unauthenticated_maps_to_401() {
        let err = AppError(DomainError::Unauthenticated);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn forbidden_maps_to_403() {
        let err = AppError(DomainError::Forbidden("not a member".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn auth_provider_maps_to_502() {
        let err = AppError(DomainError::AuthProvider("OIDC down".into()));
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn from_domain_error() {
        let domain_err = DomainError::Validation("test".into());
        let _app_err: AppError = domain_err.into();
    }
}
