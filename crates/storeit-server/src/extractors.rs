use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use chrono::Utc;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::AppError;
use storeit_domain::errors::DomainError;

/// Name of the session cookie.
pub const SESSION_COOKIE: &str = "storeit_session";

/// Authenticated request context extracted from the session cookie.
///
/// Any handler that includes `AuthContext` as a parameter will require a
/// valid, non-expired session. Unauthenticated routes simply omit this
/// extractor.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub group_id: Uuid,
}

impl FromRequestParts<Arc<AppState>> for AuthContext {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Extract cookies from the request
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError(DomainError::Unauthenticated))?;

        // Read the session cookie
        let session_id = jar
            .get(SESSION_COOKIE)
            .map(|c| c.value().to_string())
            .ok_or(AppError(DomainError::Unauthenticated))?;

        // Look up the session in the database
        let session = state
            .session_repo
            .get(&session_id)
            .await
            .map_err(|e| AppError(DomainError::Internal(e.to_string())))?
            .ok_or(AppError(DomainError::Unauthenticated))?;

        // Check expiry
        if session.expires_at < Utc::now() {
            // Clean up the expired session
            let _ = state.session_repo.delete(&session_id).await;
            return Err(AppError(DomainError::Unauthenticated));
        }

        Ok(AuthContext {
            user_id: session.user_id,
            group_id: session.active_group_id,
        })
    }
}

/// Admin-only request context. Requires local auth mode and is_admin == true.
#[derive(Debug, Clone)]
pub struct AdminContext {
    pub user_id: Uuid,
    pub group_id: Uuid,
}

impl FromRequestParts<Arc<AppState>> for AdminContext {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // First, validate the session
        let auth = AuthContext::from_request_parts(parts, state).await?;

        // Admin endpoints only available in local auth mode
        if state.auth_mode != storeit_auth::AuthMode::Local {
            return Err(AppError(DomainError::Forbidden(
                "admin endpoints only available in local auth mode".into(),
            )));
        }

        // Check user is admin
        let user = state
            .user_repo
            .get(auth.user_id)
            .await
            .map_err(|e| AppError(DomainError::Internal(e.to_string())))?
            .ok_or(AppError(DomainError::Unauthenticated))?;

        if !user.is_admin {
            return Err(AppError(DomainError::Forbidden(
                "admin access required".into(),
            )));
        }

        Ok(AdminContext {
            user_id: auth.user_id,
            group_id: auth.group_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_cookie_name() {
        assert_eq!(SESSION_COOKIE, "storeit_session");
    }

    #[test]
    fn auth_context_debug() {
        let ctx = AuthContext {
            user_id: Uuid::nil(),
            group_id: Uuid::nil(),
        };
        let dbg = format!("{ctx:?}");
        assert!(dbg.contains("AuthContext"));
    }
}
