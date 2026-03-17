use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use storeit_domain::entities::EntityType;
use storeit_domain::errors::DomainError;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::dto::*;
use crate::error::AppError;
use crate::extractors::AuthContext;

use super::not_found;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(create_nfc_tag, list_nfc_tags))
        .routes(routes!(resolve_nfc_tag))
        .routes(routes!(resolve_nfc_tag_by_uid))
        .routes(routes!(register_and_assign_nfc_tag))
        .routes(routes!(get_nfc_tag, delete_nfc_tag))
        .routes(routes!(assign_nfc_tag))
        .routes(routes!(unassign_nfc_tag))
}

#[utoipa::path(
    post,
    path = "/",
    tag = "nfc",
    request_body = CreateNfcTagRequest,
    responses(
        (status = 201, body = NfcTagResponse),
        (status = 400, body = ErrorResponse),
    ),
)]
pub async fn create_nfc_tag(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<CreateNfcTagRequest>,
) -> Result<(StatusCode, Json<NfcTagResponse>), AppError> {
    let tag = state
        .nfc_tag_repo
        .create(auth.group_id, req.tag_uri)
        .await?;
    Ok((StatusCode::CREATED, Json(tag.into())))
}

#[utoipa::path(
    get,
    path = "/",
    tag = "nfc",
    responses(
        (status = 200, body = Vec<NfcTagResponse>),
    ),
)]
pub async fn list_nfc_tags(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<Vec<NfcTagResponse>>, AppError> {
    let tags = state.nfc_tag_repo.list_by_group(auth.group_id).await?;
    Ok(Json(tags.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/resolve/{tag_uri}",
    tag = "nfc",
    params(("tag_uri" = String, Path, description = "NFC tag URI")),
    responses(
        (status = 200, body = NfcResolveResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn resolve_nfc_tag(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(tag_uri): Path<String>,
) -> Result<Json<NfcResolveResponse>, AppError> {
    // Axum's Path extractor already percent-decodes the URI
    let tag = state
        .nfc_tag_repo
        .get_by_uri(&tag_uri)
        .await?
        .ok_or_else(|| {
            AppError(DomainError::NotFound {
                entity_type: "nfc_tag".into(),
                id: Uuid::nil(),
            })
        })?;

    let entity_type = tag.entity_type.ok_or_else(|| {
        AppError(DomainError::Validation(
            "NFC tag is not assigned to any entity".into(),
        ))
    })?;
    let entity_id = tag.entity_id.ok_or_else(|| {
        AppError(DomainError::Validation(
            "NFC tag is not assigned to any entity".into(),
        ))
    })?;

    let gid = auth.group_id;

    let (entity_name, location_path) = match entity_type {
        EntityType::Container => {
            let container = state
                .container_repo
                .get(entity_id, gid)
                .await?
                .ok_or_else(|| not_found("container", entity_id))?;
            let ancestry = state.container_repo.get_ancestry(entity_id, gid).await?;
            let path: Vec<String> = ancestry.iter().map(|n| n.name.clone()).collect();
            (container.name, path)
        }
        EntityType::Location => {
            let location = state
                .location_repo
                .get(entity_id, gid)
                .await?
                .ok_or_else(|| not_found("location", entity_id))?;
            (location.name, vec![])
        }
        EntityType::Item => {
            return Err(AppError(DomainError::Validation(
                "NFC tags cannot be assigned to items".into(),
            )));
        }
    };

    Ok(Json(NfcResolveResponse {
        tag_id: tag.id,
        entity_type: entity_type.as_str().to_string(),
        entity_id,
        entity_name,
        location_path,
    }))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct UidQuery {
    /// NFC tag UID (e.g. "04A3B2C1D5E6F7")
    uid: String,
}

/// Resolve an NFC tag by its hardware UID. Returns status: "assigned", "unassigned", or "unknown".
#[utoipa::path(
    get,
    path = "/resolve-uid",
    tag = "nfc",
    params(UidQuery),
    responses(
        (status = 200, body = NfcUidResolveResponse),
    ),
)]
pub async fn resolve_nfc_tag_by_uid(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Query(query): Query<UidQuery>,
) -> Result<Json<NfcUidResolveResponse>, AppError> {
    let tag = state.nfc_tag_repo.get_by_uri(&query.uid).await?;

    let Some(tag) = tag else {
        return Ok(Json(NfcUidResolveResponse {
            status: "unknown".into(),
            tag_id: None,
            entity_type: None,
            entity_id: None,
            entity_name: None,
            location_path: None,
        }));
    };

    let (Some(entity_type), Some(entity_id)) = (tag.entity_type, tag.entity_id) else {
        return Ok(Json(NfcUidResolveResponse {
            status: "unassigned".into(),
            tag_id: Some(tag.id),
            entity_type: None,
            entity_id: None,
            entity_name: None,
            location_path: None,
        }));
    };

    let gid = auth.group_id;
    let (entity_name, location_path) = match entity_type {
        EntityType::Container => {
            let container = state
                .container_repo
                .get(entity_id, gid)
                .await?
                .ok_or_else(|| not_found("container", entity_id))?;
            let ancestry = state.container_repo.get_ancestry(entity_id, gid).await?;
            let path: Vec<String> = ancestry.iter().map(|n| n.name.clone()).collect();
            (container.name, path)
        }
        EntityType::Location => {
            let location = state
                .location_repo
                .get(entity_id, gid)
                .await?
                .ok_or_else(|| not_found("location", entity_id))?;
            (location.name, vec![])
        }
        EntityType::Item => {
            return Err(AppError(DomainError::Validation(
                "NFC tags cannot be assigned to items".into(),
            )));
        }
    };

    Ok(Json(NfcUidResolveResponse {
        status: "assigned".into(),
        tag_id: Some(tag.id),
        entity_type: Some(entity_type.as_str().to_string()),
        entity_id: Some(entity_id),
        entity_name: Some(entity_name),
        location_path: Some(location_path),
    }))
}

/// Register a new NFC tag by UID and assign it to an entity in one step.
#[utoipa::path(
    post,
    path = "/register-and-assign",
    tag = "nfc",
    request_body = RegisterAndAssignNfcRequest,
    responses(
        (status = 201, body = NfcTagResponse),
        (status = 400, body = ErrorResponse),
    ),
)]
pub async fn register_and_assign_nfc_tag(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<RegisterAndAssignNfcRequest>,
) -> Result<(StatusCode, Json<NfcTagResponse>), AppError> {
    let entity_type: EntityType = req
        .entity_type
        .parse()
        .map_err(|e: DomainError| AppError(e))?;

    if entity_type == EntityType::Item {
        return Err(AppError(DomainError::Validation(
            "NFC tags can only be assigned to locations or containers".into(),
        )));
    }

    // Create or find existing tag
    let tag = match state.nfc_tag_repo.get_by_uri(&req.tag_uri).await? {
        Some(existing) => existing,
        None => {
            state
                .nfc_tag_repo
                .create(auth.group_id, req.tag_uri)
                .await?
        }
    };

    // Assign it
    let tag = state
        .nfc_tag_repo
        .assign(tag.id, entity_type, req.entity_id)
        .await?;

    Ok((StatusCode::CREATED, Json(tag.into())))
}

#[utoipa::path(
    get,
    path = "/{id}",
    tag = "nfc",
    params(("id" = Uuid, Path, description = "NFC tag ID")),
    responses(
        (status = 200, body = NfcTagResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn get_nfc_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<NfcTagResponse>, AppError> {
    let tag = state
        .nfc_tag_repo
        .get(id)
        .await?
        .ok_or_else(|| not_found("nfc_tag", id))?;
    Ok(Json(tag.into()))
}

#[utoipa::path(
    put,
    path = "/{id}/assign",
    tag = "nfc",
    params(("id" = Uuid, Path, description = "NFC tag ID")),
    request_body = AssignNfcTagRequest,
    responses(
        (status = 200, body = NfcTagResponse),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn assign_nfc_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignNfcTagRequest>,
) -> Result<Json<NfcTagResponse>, AppError> {
    let entity_type: EntityType = req
        .entity_type
        .parse()
        .map_err(|e: storeit_domain::errors::DomainError| AppError(e))?;

    if entity_type == EntityType::Item {
        return Err(AppError(DomainError::Validation(
            "NFC tags can only be assigned to locations or containers".into(),
        )));
    }

    let tag = state
        .nfc_tag_repo
        .assign(id, entity_type, req.entity_id)
        .await?;
    Ok(Json(tag.into()))
}

#[utoipa::path(
    put,
    path = "/{id}/unassign",
    tag = "nfc",
    params(("id" = Uuid, Path, description = "NFC tag ID")),
    responses(
        (status = 204, description = "Unassigned"),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn unassign_nfc_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.nfc_tag_repo.unassign(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "nfc",
    params(("id" = Uuid, Path, description = "NFC tag ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn delete_nfc_tag(
    State(state): State<Arc<AppState>>,
    _auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    state.nfc_tag_repo.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
