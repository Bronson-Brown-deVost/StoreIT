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
        .routes(routes!(get_photo_thumbnail))
        .routes(routes!(rotate_photo))
}

#[utoipa::path(
    post,
    path = "/",
    tag = "photos",
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

    let storage_key = state.image_storage().store(&file_data, &mime_type).await?;

    let input = CreatePhoto {
        entity_type,
        entity_id,
        mime_type,
    };
    let photo = state.photo_repo.create(input, storage_key).await?;

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
    params(("id" = Uuid, Path, description = "Photo ID")),
    responses(
        (status = 200, description = "Image file"),
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
    path = "/{id}/thumbnail",
    tag = "photos",
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

    let (data, mime) = state
        .image_storage()
        .retrieve_thumbnail(&photo.storage_key)
        .await?;

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
    if let Err(e) = state
        .image_storage()
        .delete_if_unreferenced(&photo.storage_key, ref_count)
        .await
    {
        tracing::warn!("failed to clean up photo file {}: {e}", photo.storage_key);
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/{id}/rotate",
    tag = "photos",
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
