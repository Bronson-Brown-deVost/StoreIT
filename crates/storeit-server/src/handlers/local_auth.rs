use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::dto::*;
use crate::error::AppError;
use crate::extractors::SESSION_COOKIE;
use crate::handlers::auth::generate_session_id;

use storeit_domain::entities::{GroupRole, Session};
use storeit_domain::errors::DomainError;

/// Build the local auth sub-router.
pub fn router() -> utoipa_axum::router::OpenApiRouter<Arc<AppState>> {
    utoipa_axum::router::OpenApiRouter::new().routes(utoipa_axum::routes!(local_login))
}

// -----------------------------------------------------------------------
// POST /auth/local/login
// -----------------------------------------------------------------------
#[utoipa::path(
    post,
    path = "/login",
    tag = "auth",
    request_body = LocalLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = MeResponse),
        (status = 401, description = "Invalid credentials"),
    ),
)]
async fn local_login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<LocalLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let external_id = format!("local:{}", body.username);

    // Always fetch the hash first for constant-time behavior
    let hash = state
        .user_repo
        .get_password_hash(&external_id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    let hash = match hash {
        Some(h) => h,
        None => {
            // User doesn't exist — do a dummy verify to prevent timing attacks
            let _ = storeit_auth::verify_password(
                &body.password,
                "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            );
            return Err(AppError(DomainError::InvalidCredentials));
        }
    };

    let valid = storeit_auth::verify_password(&body.password, &hash)
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    if !valid {
        return Err(AppError(DomainError::InvalidCredentials));
    }

    // Fetch the user
    let user = state
        .user_repo
        .get_by_external_id(&external_id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?
        .ok_or(AppError(DomainError::InvalidCredentials))?;

    // Get user's groups
    let group_list = state
        .user_group_repo
        .list_groups_for_user(user.id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    // Pick active group
    let active_group_id = if let Some((group, _)) = group_list.first() {
        group.id
    } else {
        // Fallback to default group, and add user as member
        let default_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        state
            .user_group_repo
            .add_member(user.id, default_id, GroupRole::Member)
            .await
            .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;
        default_id
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
    state
        .session_repo
        .create(session)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    // Set session cookie
    let session_cookie = axum_extra::extract::cookie::Cookie::build((SESSION_COOKIE, session_id))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .max_age(time::Duration::hours(state.session_ttl_hours as i64))
        .build();

    let groups: Vec<GroupResponse> = group_list
        .into_iter()
        .map(|(g, role)| GroupResponse {
            id: g.id,
            name: g.name,
            role: role.as_str().to_string(),
        })
        .collect();

    // If groups was empty, we added the default group above
    let groups = if groups.is_empty() {
        vec![GroupResponse {
            id: active_group_id,
            name: "default".to_string(),
            role: "member".to_string(),
        }]
    } else {
        groups
    };

    Ok((
        jar.add(session_cookie),
        Json(MeResponse {
            user: user.into(),
            groups,
            active_group_id,
        }),
    ))
}
