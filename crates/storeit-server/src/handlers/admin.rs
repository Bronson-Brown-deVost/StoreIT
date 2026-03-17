use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::app_state::{AppState, BackupJob, RestoreJob};
use crate::dto::*;
use crate::error::AppError;
use crate::extractors::AdminContext;

use storeit_domain::entities::{CreateLocalUser, GroupRole};
use storeit_domain::errors::DomainError;

/// Build the admin sub-router.
pub fn router() -> utoipa_axum::router::OpenApiRouter<Arc<AppState>> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(list_users))
        .routes(utoipa_axum::routes!(create_user))
        .routes(utoipa_axum::routes!(update_user))
        .routes(utoipa_axum::routes!(reset_password))
        .routes(utoipa_axum::routes!(delete_user))
        .routes(utoipa_axum::routes!(list_groups))
        .routes(utoipa_axum::routes!(create_group))
        .routes(utoipa_axum::routes!(delete_group))
        .routes(utoipa_axum::routes!(list_group_members))
        .routes(utoipa_axum::routes!(add_group_member))
        .routes(utoipa_axum::routes!(remove_group_member))
        .routes(utoipa_axum::routes!(get_settings, update_settings))
        .routes(utoipa_axum::routes!(start_backup))
        .routes(utoipa_axum::routes!(backup_status))
        .routes(utoipa_axum::routes!(download_backup))
        .routes(utoipa_axum::routes!(start_restore))
        .routes(utoipa_axum::routes!(restore_status))
        .routes(utoipa_axum::routes!(migrate_storage))
        .routes(utoipa_axum::routes!(schema_version))
}

// -----------------------------------------------------------------------
// Users
// -----------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/users",
    tag = "admin",
    responses(
        (status = 200, description = "List all users", body = Vec<AdminUserResponse>),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn list_users(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
) -> Result<Json<Vec<AdminUserResponse>>, AppError> {
    let users = state
        .user_repo
        .list_all()
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    Ok(Json(users.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/users",
    tag = "admin",
    request_body = CreateLocalUserRequest,
    responses(
        (status = 200, description = "User created", body = AdminUserResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn create_user(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Json(body): Json<CreateLocalUserRequest>,
) -> Result<Json<AdminUserResponse>, AppError> {
    if body.username.is_empty() {
        return Err(AppError(DomainError::Validation(
            "username is required".into(),
        )));
    }
    if body.password.is_empty() {
        return Err(AppError(DomainError::Validation(
            "password is required".into(),
        )));
    }

    let password_hash = storeit_auth::hash_password(&body.password)
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    let user = state
        .user_repo
        .create_local(CreateLocalUser {
            username: body.username,
            email: body.email,
            display_name: body.display_name,
            password_hash,
            is_admin: body.is_admin.unwrap_or(false),
        })
        .await?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    tag = "admin",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = UpdateLocalUserRequest,
    responses(
        (status = 200, description = "User updated", body = AdminUserResponse),
        (status = 404, description = "User not found"),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn update_user(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateLocalUserRequest>,
) -> Result<Json<AdminUserResponse>, AppError> {
    // Fetch existing user
    let mut user = state
        .user_repo
        .get(id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?
        .ok_or_else(|| {
            AppError(DomainError::NotFound {
                entity_type: "user".into(),
                id,
            })
        })?;

    // Update fields via upsert
    if let Some(email) = &body.email {
        user.email = email.clone();
    }
    if let Some(display_name) = &body.display_name {
        user.display_name = display_name.clone();
    }

    // Re-upsert to save email/display_name changes
    let updated = state
        .user_repo
        .upsert_by_external_id(storeit_domain::entities::CreateUser {
            external_id: user.external_id.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
        })
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    // Update admin flag if requested
    if let Some(is_admin) = body.is_admin {
        state
            .user_repo
            .set_admin(id, is_admin)
            .await
            .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;
    }

    // Re-fetch to get final state
    let final_user = state
        .user_repo
        .get(updated.id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?
        .ok_or_else(|| AppError(DomainError::Internal("user disappeared".into())))?;

    Ok(Json(final_user.into()))
}

#[utoipa::path(
    put,
    path = "/users/{id}/password",
    tag = "admin",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn reset_password(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Path(id): Path<Uuid>,
    Json(body): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if body.new_password.is_empty() {
        return Err(AppError(DomainError::Validation(
            "new_password is required".into(),
        )));
    }

    // Verify user exists
    state
        .user_repo
        .get(id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?
        .ok_or_else(|| {
            AppError(DomainError::NotFound {
                entity_type: "user".into(),
                id,
            })
        })?;

    let hash = storeit_auth::hash_password(&body.new_password)
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    state
        .user_repo
        .set_password_hash(id, &hash)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "admin",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User deleted"),
        (status = 400, description = "Cannot delete last admin or self"),
        (status = 404, description = "User not found"),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn delete_user(
    State(state): State<Arc<AppState>>,
    admin: AdminContext,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if id == admin.user_id {
        return Err(AppError(DomainError::Validation(
            "cannot delete your own account".into(),
        )));
    }

    state.user_repo.delete(id).await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

// -----------------------------------------------------------------------
// Groups
// -----------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/groups",
    tag = "admin",
    responses(
        (status = 200, description = "List all groups", body = Vec<AdminGroupResponse>),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn list_groups(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
) -> Result<Json<Vec<AdminGroupResponse>>, AppError> {
    let groups = state
        .group_repo
        .list_all()
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    Ok(Json(groups.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/groups",
    tag = "admin",
    request_body = CreateGroupRequest,
    responses(
        (status = 200, description = "Group created", body = AdminGroupResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn create_group(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Json(body): Json<CreateGroupRequest>,
) -> Result<Json<AdminGroupResponse>, AppError> {
    if body.name.is_empty() {
        return Err(AppError(DomainError::Validation(
            "group name is required".into(),
        )));
    }

    let group = state.group_repo.create(&body.name).await?;

    Ok(Json(group.into()))
}

#[utoipa::path(
    delete,
    path = "/groups/{id}",
    tag = "admin",
    params(("id" = Uuid, Path, description = "Group ID")),
    responses(
        (status = 200, description = "Group deleted"),
        (status = 400, description = "Cannot delete group with members or default group"),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn delete_group(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.group_repo.delete(id).await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

// -----------------------------------------------------------------------
// Group Members
// -----------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/groups/{id}/members",
    tag = "admin",
    params(("id" = Uuid, Path, description = "Group ID")),
    responses(
        (status = 200, description = "List group members", body = Vec<GroupMemberResponse>),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn list_group_members(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<GroupMemberResponse>>, AppError> {
    let members = state
        .user_group_repo
        .list_members_of_group(id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    Ok(Json(
        members
            .into_iter()
            .map(|(user, role)| GroupMemberResponse {
                user: user.into(),
                role: role.as_str().to_string(),
            })
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/groups/{id}/members",
    tag = "admin",
    params(("id" = Uuid, Path, description = "Group ID")),
    request_body = AddMemberRequest,
    responses(
        (status = 200, description = "Member added"),
        (status = 400, description = "Invalid role"),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn add_group_member(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Path(id): Path<Uuid>,
    Json(body): Json<AddMemberRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let role: GroupRole = body.role.parse().map_err(|_| {
        AppError(DomainError::Validation(format!(
            "invalid role: {}",
            body.role
        )))
    })?;

    state
        .user_group_repo
        .add_member(body.user_id, id, role)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    delete,
    path = "/groups/{group_id}/members/{user_id}",
    tag = "admin",
    params(
        ("group_id" = Uuid, Path, description = "Group ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Member removed"),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn remove_group_member(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    state
        .user_group_repo
        .remove_member(user_id, group_id)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    Ok(Json(serde_json::json!({"ok": true})))
}

// -----------------------------------------------------------------------
// Settings
// -----------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/settings",
    tag = "admin",
    responses(
        (status = 200, description = "Current admin settings", body = AdminSettingsResponse),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn get_settings(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
) -> Result<Json<AdminSettingsResponse>, AppError> {
    Ok(Json(AdminSettingsResponse {
        image_storage_path: state.image_storage_path(),
        image_storage_path_readonly: state.env_image_path,
    }))
}

#[utoipa::path(
    put,
    path = "/settings",
    tag = "admin",
    request_body = UpdateSettingsRequest,
    responses(
        (status = 200, description = "Settings updated", body = AdminSettingsResponse),
        (status = 403, description = "Admin access required"),
        (status = 409, description = "Setting is read-only (set via env var)"),
    ),
)]
async fn update_settings(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Json(body): Json<UpdateSettingsRequest>,
) -> Result<Json<AdminSettingsResponse>, AppError> {
    if state.env_image_path {
        return Err(AppError(DomainError::NotEmpty {
            entity_type: "setting".into(),
            id: Uuid::nil(),
            child_count: 0,
        }));
    }

    let new_path = body.image_storage_path.trim().to_string();
    if new_path.is_empty() {
        return Err(AppError(DomainError::Validation(
            "image_storage_path cannot be empty".into(),
        )));
    }

    // Create the directory if it doesn't exist
    let dir = std::path::Path::new(&new_path);
    std::fs::create_dir_all(dir).map_err(|e| {
        AppError(DomainError::Storage(format!(
            "failed to create directory: {e}"
        )))
    })?;

    // Persist to DB
    state
        .settings_repo
        .set("image_storage_path", &new_path)
        .await
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))?;

    // Swap the image storage backend
    let new_storage = Arc::new(storeit_storage_fs::FsImageStorage::new(&new_path));
    state.swap_image_storage(new_path.clone(), new_storage);

    Ok(Json(AdminSettingsResponse {
        image_storage_path: new_path,
        image_storage_path_readonly: false,
    }))
}

// -----------------------------------------------------------------------
// Backup
// -----------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/backup",
    tag = "admin",
    request_body = BackupRequest,
    responses(
        (status = 200, description = "Backup job started", body = BackupJobResponse),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn start_backup(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    Json(body): Json<BackupRequest>,
) -> Result<Json<BackupJobResponse>, AppError> {
    let job_id = Uuid::new_v4().to_string();
    let job = Arc::new(BackupJob::new());
    state.backup_jobs.insert(job_id.clone(), job.clone());

    let state_clone = state.clone();

    tokio::spawn(async move {
        let final_path =
            std::env::temp_dir().join(format!("storeit-backup-{}.storeit", Uuid::new_v4()));
        let options = crate::interchange::ExportOptions {
            include_images: body.include_images,
        };
        match crate::interchange::export_to_file(&state_clone, &final_path, &options, job.as_ref())
            .await
        {
            Ok(()) => job.set_complete(final_path),
            Err(e) => job.set_error(e.to_string()),
        }
    });

    Ok(Json(BackupJobResponse { job_id }))
}

#[utoipa::path(
    get,
    path = "/backup/{job_id}/status",
    tag = "admin",
    params(("job_id" = String, Path, description = "Backup job ID")),
    responses(
        (status = 200, description = "Job status", body = JobStatusResponse),
        (status = 404, description = "Job not found"),
    ),
)]
async fn backup_status(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<Json<JobStatusResponse>, AppError> {
    let job = state
        .backup_jobs
        .get(&job_id)
        .ok_or_else(|| {
            AppError(DomainError::NotFound {
                entity_type: "backup_job".into(),
                id: Uuid::nil(),
            })
        })?
        .clone();

    Ok(Json(JobStatusResponse {
        status: job.status(),
        progress: job.progress.load(std::sync::atomic::Ordering::Relaxed),
        total: job.total.load(std::sync::atomic::Ordering::Relaxed),
        error: job.error.lock().unwrap().clone(),
    }))
}

#[utoipa::path(
    get,
    path = "/backup/{job_id}/download",
    tag = "admin",
    params(("job_id" = String, Path, description = "Backup job ID")),
    responses(
        (status = 200, description = "Download backup archive"),
        (status = 404, description = "Job not found or not complete"),
    ),
)]
async fn download_backup(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let job = state
        .backup_jobs
        .get(&job_id)
        .ok_or_else(|| {
            AppError(DomainError::NotFound {
                entity_type: "backup_job".into(),
                id: Uuid::nil(),
            })
        })?
        .clone();

    if job.status() != "complete" {
        return Err(AppError(DomainError::Validation(
            "backup job is not complete".into(),
        )));
    }

    let archive_path = job
        .archive_path
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| AppError(DomainError::Internal("archive path missing".into())))?;

    let data = tokio::fs::read(&archive_path)
        .await
        .map_err(|e| AppError(DomainError::Storage(e.to_string())))?;

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let filename = format!("storeit-backup-{timestamp}.storeit");

    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        Body::from(data),
    ))
}

// -----------------------------------------------------------------------
// Restore
// -----------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/restore",
    tag = "admin",
    responses(
        (status = 200, description = "Restore job started", body = BackupJobResponse),
        (status = 400, description = "Invalid archive"),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn start_restore(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<BackupJobResponse>, AppError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut options: Option<RestoreOptions> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError(DomainError::Validation(e.to_string())))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError(DomainError::Validation(e.to_string())))?
                        .to_vec(),
                );
            }
            "options" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError(DomainError::Validation(e.to_string())))?;
                options = Some(
                    serde_json::from_str(&text)
                        .map_err(|e| AppError(DomainError::Validation(e.to_string())))?,
                );
            }
            _ => {}
        }
    }

    let file_data =
        file_data.ok_or_else(|| AppError(DomainError::Validation("missing file field".into())))?;
    let options =
        options.ok_or_else(|| AppError(DomainError::Validation("missing options field".into())))?;

    if options.mode != "replace" && options.mode != "merge" {
        return Err(AppError(DomainError::Validation(format!(
            "invalid mode: {}, must be 'replace' or 'merge'",
            options.mode
        ))));
    }

    let job_id = Uuid::new_v4().to_string();
    let job = Arc::new(RestoreJob::new());
    state.restore_jobs.insert(job_id.clone(), job.clone());

    let state_clone = state.clone();

    tokio::spawn(async move {
        let import_options = crate::interchange::ImportOptions {
            mode: options.mode,
            image_storage_path: options.image_storage_path,
        };
        match crate::interchange::import_from_bytes(
            &state_clone,
            &file_data,
            &import_options,
            job.as_ref(),
        )
        .await
        {
            Ok(()) => job.set_complete(),
            Err(e) => job.set_error(e.to_string()),
        }
    });

    Ok(Json(BackupJobResponse { job_id }))
}

#[utoipa::path(
    get,
    path = "/restore/{job_id}/status",
    tag = "admin",
    params(("job_id" = String, Path, description = "Restore job ID")),
    responses(
        (status = 200, description = "Job status", body = JobStatusResponse),
        (status = 404, description = "Job not found"),
    ),
)]
async fn restore_status(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<Json<JobStatusResponse>, AppError> {
    let job = state
        .restore_jobs
        .get(&job_id)
        .ok_or_else(|| {
            AppError(DomainError::NotFound {
                entity_type: "restore_job".into(),
                id: Uuid::nil(),
            })
        })?
        .clone();

    Ok(Json(JobStatusResponse {
        status: job.status(),
        progress: job.progress.load(std::sync::atomic::Ordering::Relaxed),
        total: job.total.load(std::sync::atomic::Ordering::Relaxed),
        error: job.error.lock().unwrap().clone(),
    }))
}

// -----------------------------------------------------------------------
// Schema Version
// -----------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/schema-version",
    tag = "admin",
    responses(
        (status = 200, description = "Schema version info", body = serde_json::Value),
    ),
)]
async fn schema_version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "schema_version": storeit_db_sqlite::SqliteDb::EXPECTED_SCHEMA_VERSION,
        "expected_version": storeit_db_sqlite::SqliteDb::EXPECTED_SCHEMA_VERSION,
        "app_version": env!("CARGO_PKG_VERSION"),
    }))
}

// -----------------------------------------------------------------------
// Storage Migration
// -----------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/migrate-storage",
    tag = "admin",
    responses(
        (status = 200, description = "Migration summary", body = serde_json::Value),
        (status = 403, description = "Admin access required"),
    ),
)]
async fn migrate_storage(
    State(state): State<Arc<AppState>>,
    _admin: AdminContext,
) -> Result<Json<serde_json::Value>, AppError> {
    let photos = state.photo_repo.list_all().await?;
    let storage = state.image_storage();

    let mut migrated: u64 = 0;
    let mut skipped: u64 = 0;
    let mut errors: u64 = 0;

    for photo in &photos {
        // Keys already containing '/' are new-format — skip
        if photo.storage_key.contains('/') {
            skipped += 1;
            continue;
        }

        // Legacy UUID-based key — migrate
        match storage.retrieve(&photo.storage_key).await {
            Ok((data, _)) => {
                // Compute new content-addressable key
                let keys = storage.store(&data, &photo.mime_type).await.map_err(|e| {
                    AppError(DomainError::Storage(format!(
                        "failed to store migrated file: {e}"
                    )))
                })?;

                // Update DB record
                if let Err(e) = state
                    .photo_repo
                    .update_storage_key(photo.id, &keys.storage_key)
                    .await
                {
                    tracing::warn!("failed to update storage key for {}: {e}", photo.id);
                    errors += 1;
                    continue;
                }

                // Delete old file (best-effort)
                if keys.storage_key != photo.storage_key
                    && let Err(e) = storage.delete(&photo.storage_key).await
                {
                    tracing::warn!("failed to delete old file {}: {e}", photo.storage_key);
                }

                migrated += 1;
            }
            Err(e) => {
                tracing::warn!(
                    "failed to retrieve photo {} ({}): {e}",
                    photo.id,
                    photo.storage_key
                );
                errors += 1;
            }
        }
    }

    Ok(Json(serde_json::json!({
        "migrated": migrated,
        "skipped": skipped,
        "errors": errors,
    })))
}
