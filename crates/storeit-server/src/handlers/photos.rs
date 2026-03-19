use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::Response;
use uuid::Uuid;

use storeit_domain::entities::{CreatePhoto, EntityType};
use storeit_domain::errors::DomainError;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;
use crate::dto::*;
use crate::error::AppError;

use super::not_found;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(upload_photo))
        .routes(routes!(list_entity_photos))
        .routes(routes!(get_photo, delete_photo))
        .routes(routes!(get_photo_file))
        .routes(routes!(get_photo_large))
        .routes(routes!(get_photo_thumbnail))
        .routes(routes!(rotate_photo))
}

#[utoipa::path(
    post,
    path = "/",
    tag = "photos",
    description = "Upload a photo and attach it to an entity. Thumbnails and large variants are generated server-side.",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 201, body = PhotoResponse),
        (status = 400, body = ErrorResponse),
    ),
)]
pub async fn upload_photo(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<PhotoResponse>), AppError> {
    let mut entity_type_str: Option<String> = None;
    let mut entity_id_str: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_mime: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError(DomainError::Validation(format!("multipart error: {e}"))))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "entity_type" => {
                entity_type_str = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError(DomainError::Validation(e.to_string())))?,
                );
            }
            "entity_id" => {
                entity_id_str = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError(DomainError::Validation(e.to_string())))?,
                );
            }
            "file" => {
                file_mime = field.content_type().map(|s| s.to_string());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError(DomainError::Validation(e.to_string())))?;
                file_data = Some(bytes.to_vec());
            }
            _ => {}
        }
    }

    let entity_type_str = entity_type_str
        .ok_or_else(|| AppError(DomainError::Validation("missing entity_type field".into())))?;
    let entity_id_str = entity_id_str
        .ok_or_else(|| AppError(DomainError::Validation("missing entity_id field".into())))?;
    let file_data =
        file_data.ok_or_else(|| AppError(DomainError::Validation("missing file field".into())))?;
    let mime_type = file_mime.unwrap_or_else(|| "application/octet-stream".into());

    let entity_type: EntityType = entity_type_str.parse().map_err(AppError::from)?;
    let entity_id: Uuid = entity_id_str
        .parse()
        .map_err(|_| AppError(DomainError::Validation("invalid entity_id".into())))?;

    let keys = state.image_storage().store(&file_data, &mime_type).await?;

    let input = CreatePhoto {
        entity_type,
        entity_id,
        mime_type,
    };
    let photo = state
        .photo_repo
        .create(input, keys.storage_key, keys.thumbnail_key, keys.large_key)
        .await?;

    Ok((StatusCode::CREATED, Json(photo.into())))
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct PhotoListQuery {
    pub entity_type: String,
    pub entity_id: Uuid,
}

#[utoipa::path(
    get,
    path = "/by-entity",
    tag = "photos",
    description = "List all photos attached to a specific entity (location, container, or item).",
    params(PhotoListQuery),
    responses(
        (status = 200, body = Vec<PhotoResponse>),
        (status = 400, body = ErrorResponse),
    ),
)]
pub async fn list_entity_photos(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PhotoListQuery>,
) -> Result<Json<Vec<PhotoResponse>>, AppError> {
    let entity_type: EntityType = query.entity_type.parse().map_err(AppError::from)?;
    let photos = state
        .photo_repo
        .list_by_entity(entity_type, query.entity_id)
        .await?;
    Ok(Json(photos.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/{id}",
    tag = "photos",
    description = "Get photo metadata by ID.",
    params(("id" = Uuid, Path, description = "Photo ID")),
    responses(
        (status = 200, body = PhotoResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn get_photo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PhotoResponse>, AppError> {
    let photo = state
        .photo_repo
        .get(id)
        .await?
        .ok_or_else(|| not_found("photo", id))?;
    Ok(Json(photo.into()))
}

#[utoipa::path(
    get,
    path = "/{id}/file",
    tag = "photos",
    description = "Download the original full-resolution image file.",
    params(("id" = Uuid, Path, description = "Photo ID")),
    responses(
        (status = 200, description = "Original image file"),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn get_photo_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let photo = state
        .photo_repo
        .get(id)
        .await?
        .ok_or_else(|| not_found("photo", id))?;

    let (data, _) = state.image_storage().retrieve(&photo.storage_key).await?;

    Response::builder()
        .header(header::CONTENT_TYPE, photo.mime_type.as_str())
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(Body::from(data))
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))
}

#[utoipa::path(
    get,
    path = "/{id}/large",
    tag = "photos",
    description = "Download the large display variant (~1200px max dimension, WebP). Falls back to the original if no large variant exists.",
    params(("id" = Uuid, Path, description = "Photo ID")),
    responses(
        (status = 200, description = "Large display image (~1200px)"),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn get_photo_large(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let photo = state
        .photo_repo
        .get(id)
        .await?
        .ok_or_else(|| not_found("photo", id))?;

    // Prefer stored large_key; fall back to original file
    let key = photo.large_key.as_deref().unwrap_or(&photo.storage_key);
    let (data, mime) = state.image_storage().retrieve(key).await?;

    Response::builder()
        .header(header::CONTENT_TYPE, mime.as_str())
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(Body::from(data))
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))
}

#[utoipa::path(
    get,
    path = "/{id}/thumbnail",
    tag = "photos",
    description = "Download the thumbnail variant (~200px, WebP). Used for grid views and cards.",
    params(("id" = Uuid, Path, description = "Photo ID")),
    responses(
        (status = 200, description = "Thumbnail image"),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn get_photo_thumbnail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let photo = state
        .photo_repo
        .get(id)
        .await?
        .ok_or_else(|| not_found("photo", id))?;

    // Prefer stored thumbnail_key; fall back to legacy derived key
    let (data, mime) = if let Some(ref thumb_key) = photo.thumbnail_key {
        state.image_storage().retrieve(thumb_key).await?
    } else {
        state
            .image_storage()
            .retrieve_thumbnail(&photo.storage_key)
            .await?
    };

    Response::builder()
        .header(header::CONTENT_TYPE, mime.as_str())
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .body(Body::from(data))
        .map_err(|e| AppError(DomainError::Internal(e.to_string())))
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "photos",
    description = "Delete a photo and clean up storage files if no other photos reference them.",
    params(("id" = Uuid, Path, description = "Photo ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn delete_photo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let photo = state
        .photo_repo
        .get(id)
        .await?
        .ok_or_else(|| not_found("photo", id))?;

    state.photo_repo.delete(id).await?;

    // Best-effort storage cleanup — only delete file if no other photos reference it
    let ref_count = state
        .photo_repo
        .count_by_storage_key(&photo.storage_key)
        .await
        .unwrap_or(1);
    let storage = state.image_storage();
    if let Err(e) = storage
        .delete_if_unreferenced(&photo.storage_key, ref_count)
        .await
    {
        tracing::warn!("failed to clean up photo file {}: {e}", photo.storage_key);
    }
    // Clean up variant files (best-effort, ignore errors)
    if ref_count == 0 {
        if let Some(ref key) = photo.thumbnail_key {
            let _ = storage.delete(key).await;
        }
        if let Some(ref key) = photo.large_key {
            let _ = storage.delete(key).await;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/{id}/rotate",
    tag = "photos",
    description = "Rotate a photo by the given degrees (accumulated, mod 360). Applied client-side via CSS transform.",
    params(("id" = Uuid, Path, description = "Photo ID")),
    request_body = RotatePhotoRequest,
    responses(
        (status = 200, body = PhotoResponse),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn rotate_photo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<RotatePhotoRequest>,
) -> Result<Json<PhotoResponse>, AppError> {
    let photo = state
        .photo_repo
        .get(id)
        .await?
        .ok_or_else(|| not_found("photo", id))?;

    // Compute new rotation (accumulate, mod 360; 0 means no rotation)
    let new_rotation = (photo.rotation + req.degrees as i32) % 360;

    state.photo_repo.set_rotation(id, new_rotation).await?;

    let updated = state
        .photo_repo
        .get(id)
        .await?
        .ok_or_else(|| not_found("photo", id))?;

    Ok(Json(updated.into()))
}
