use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use rand::Rng;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::dto::*;
use crate::error::AppError;
use crate::extractors::{AuthContext, SESSION_COOKIE};

use storeit_auth::AuthMode;
use storeit_domain::entities::{CreateUser, GroupRole, Session};
use storeit_domain::errors::DomainError;

/// Build the auth sub-router.
pub fn router() -> utoipa_axum::router::OpenApiRouter<Arc<AppState>> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(auth_mode))
        .routes(utoipa_axum::routes!(login))
        .routes(utoipa_axum::routes!(callback))
        .routes(utoipa_axum::routes!(logout))
        .routes(utoipa_axum::routes!(me))
        .routes(utoipa_axum::routes!(switch_active_group))
}

// -----------------------------------------------------------------------
// GET /auth/mode — return current auth mode (unauthenticated)
// -----------------------------------------------------------------------
#[utoipa::path(
    get,
    path = "/mode",
    tag = "auth",
    description = "Get the current authentication mode (OIDC or local). Unauthenticated.",
    responses(
        (status = 200, description = "Current auth mode", body = AuthModeResponse),
    ),
)]
async fn auth_mode(State(state): State<Arc<AppState>>) -> Json<AuthModeResponse> {
    let mode = match state.auth_mode {
        AuthMode::Oidc => "oidc",
        AuthMode::Local => "local",
    };
    Json(AuthModeResponse {
        mode: mode.to_string(),
    })
}

// -----------------------------------------------------------------------
// GET /auth/login — start the OIDC flow (unauthenticated, OIDC only)
// -----------------------------------------------------------------------
#[utoipa::path(
    get,
    path = "/login",
    tag = "auth",
    description = "Start the OIDC login flow. Redirects to the identity provider. OIDC mode only.",
    responses(
        (status = 302, description = "Redirect to OIDC provider"),
        (status = 502, description = "Auth provider error"),
    ),
)]
async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let provider = state
        .auth_provider
        .as_ref()
        .ok_or_else(|| AppError(DomainError::Forbidden("OIDC not configured".into())))?;

    let result = provider
        .start_auth()
        .map_err(|e| AppError(DomainError::AuthProvider(e.to_string())))?;

    // Store CSRF state + PKCE verifier in a cookie so we can validate the callback.
    let cookie_value = format!("{}|{}", result.csrf_state, result.pkce_verifier);
    let cookie = axum_extra::extract::cookie::Cookie::build(("storeit_auth_pending", cookie_value))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .max_age(time::Duration::minutes(10))
        .build();

    Ok((jar.add(cookie), Redirect::to(&result.authorize_url)))
}

// -----------------------------------------------------------------------
// GET /auth/callback — OIDC callback (unauthenticated, OIDC only)
// -----------------------------------------------------------------------
#[utoipa::path(
    get,
    path = "/callback",
    tag = "auth",
    description = "OIDC callback endpoint. Exchanges the authorization code for a session. OIDC mode only.",
    params(AuthCallbackQuery),
    responses(
        (status = 302, description = "Redirect to app after successful auth"),
        (status = 401, description = "Invalid CSRF state or auth code"),
    ),
)]
async fn callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthCallbackQuery>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let provider = state
        .auth_provider
        .as_ref()
        .ok_or_else(|| AppError(DomainError::Forbidden("OIDC not configured".into())))?;

    // Read the pending auth cookie
    let pending = jar
        .get("storeit_auth_pending")
        .ok_or(AppError(DomainError::Unauthenticated))?;

    let parts: Vec<&str> = pending.value().splitn(2, '|').collect();
    if parts.len() != 2 {
        return Err(AppError(DomainError::Unauthenticated));
    }
    let expected_csrf = parts[0];
    let pkce_verifier = parts[1];

    // Validate CSRF
    if query.state != expected_csrf {
        return Err(AppError(DomainError::Unauthenticated));
    }

    // Exchange code for user info
    let user_info = provider
        .exchange_code(&query.code, pkce_verifier)
        .await
        .map_err(|e| AppError(DomainError::AuthProvider(e.to_string())))?;

    // Upsert user
    let user = state
        .user_repo
        .upsert_by_external_id(CreateUser {
            external_id: user_info.external_id,
            email: user_info.email,
            display_name: user_info.display_name,
        })
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    // Filter groups by prefix, deduplicate, get-or-create each, set memberships
    tracing::debug!(raw_groups = ?user_info.groups, "groups from OIDC provider");
    let mut filtered_group_names = provider.filter_groups(&user_info.groups);
    filtered_group_names.sort();
    filtered_group_names.dedup();
    tracing::debug!(filtered_groups = ?filtered_group_names, "groups after prefix filter");

    let mut memberships = Vec::new();
    for group_name in &filtered_group_names {
        let group = state
            .group_repo
            .get_or_create_by_name(group_name)
            .await
            .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;
        memberships.push((group.id, GroupRole::Member));
    }

    if !memberships.is_empty() {
        state
            .user_group_repo
            .set_memberships(user.id, memberships.clone())
            .await
            .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;
    }

    // Pick active group: first membership or the hardcoded default group
    let active_group_id = if let Some((gid, _)) = memberships.first() {
        *gid
    } else {
        // Fallback to the seeded "default" group
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    };

    // Create session
    let session_id = generate_session_id();
    let session = Session {
        id: session_id.clone(),
        user_id: user.id,
        active_group_id,
        expires_at: Utc::now() + Duration::hours(state.session_ttl_hours as i64),
        created_at: Utc::now(),
    };
    tracing::debug!(user_id = %user.id, active_group_id = %active_group_id, "creating session");
    state.session_repo.create(session).await.map_err(|e| {
        tracing::error!(error = %e, "session creation failed");
        AppError(DomainError::Internal(e.to_string()))
    })?;

    // Set session cookie and clear the pending auth cookie
    let session_cookie = axum_extra::extract::cookie::Cookie::build((SESSION_COOKIE, session_id))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .max_age(time::Duration::hours(state.session_ttl_hours as i64))
        .build();

    let remove_pending = axum_extra::extract::cookie::Cookie::build(("storeit_auth_pending", ""))
        .path("/")
        .max_age(time::Duration::ZERO)
        .build();

    let jar = jar.add(session_cookie).add(remove_pending);

    Ok((jar, Redirect::to("/")))
}

// -----------------------------------------------------------------------
// POST /auth/logout
// -----------------------------------------------------------------------
#[utoipa::path(
    post,
    path = "/logout",
    tag = "auth",
    description = "End the current session and clear the session cookie.",
    responses(
        (status = 200, description = "Logged out"),
        (status = 401, description = "Not authenticated"),
    ),
)]
async fn logout(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    // Delete session from the session cookie value
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        let _ = state.session_repo.delete(cookie.value()).await;
    }

    let remove = axum_extra::extract::cookie::Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .max_age(time::Duration::ZERO)
        .build();

    // Suppress unused variable warning — `auth` ensures the user is authenticated
    let _ = auth;

    Ok(jar.add(remove))
}

// -----------------------------------------------------------------------
// GET /auth/me
// -----------------------------------------------------------------------
#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    description = "Get the current authenticated user's profile, group memberships, and active group.",
    responses(
        (status = 200, description = "Current user info", body = MeResponse),
        (status = 401, description = "Not authenticated"),
    ),
)]
async fn me(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<MeResponse>, AppError> {
    let user = state
        .user_repo
        .get(auth.user_id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?
        .ok_or(AppError(DomainError::Unauthenticated))?;

    let group_list = state
        .user_group_repo
        .list_groups_for_user(auth.user_id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    let groups: Vec<GroupResponse> = group_list
        .into_iter()
        .map(|(g, role)| GroupResponse {
            id: g.id,
            name: g.name,
            role: role.as_str().to_string(),
        })
        .collect();

    Ok(Json(MeResponse {
        user: user.into(),
        groups,
        active_group_id: auth.group_id,
    }))
}

// -----------------------------------------------------------------------
// PUT /auth/me/active-group
// -----------------------------------------------------------------------
#[utoipa::path(
    put,
    path = "/me/active-group",
    tag = "auth",
    description = "Switch the active group for the current session. Must be a member of the target group.",
    request_body = SwitchGroupRequest,
    responses(
        (status = 200, description = "Active group switched", body = MeResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Not a member of requested group"),
    ),
)]
async fn switch_active_group(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    jar: CookieJar,
    Json(body): Json<SwitchGroupRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check membership
    let is_member = state
        .user_group_repo
        .is_member(auth.user_id, body.group_id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    if !is_member {
        return Err(AppError(DomainError::Forbidden(
            "not a member of this group".into(),
        )));
    }

    // Update session
    let session_id = jar
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or(AppError(DomainError::Unauthenticated))?;

    state
        .session_repo
        .update_active_group(&session_id, body.group_id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    // Return updated /me response
    let user = state
        .user_repo
        .get(auth.user_id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?
        .ok_or(AppError(DomainError::Unauthenticated))?;

    let group_list = state
        .user_group_repo
        .list_groups_for_user(auth.user_id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    let groups: Vec<GroupResponse> = group_list
        .into_iter()
        .map(|(g, role)| GroupResponse {
            id: g.id,
            name: g.name,
            role: role.as_str().to_string(),
        })
        .collect();

    Ok(Json(MeResponse {
        user: user.into(),
        groups,
        active_group_id: body.group_id,
    }))
}

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

pub fn generate_session_id() -> String {
    use rand::distributions::Alphanumeric;
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_length() {
        let id = generate_session_id();
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn session_id_unique() {
        let a = generate_session_id();
        let b = generate_session_id();
        assert_ne!(a, b);
    }
}
