use std::sync::Arc;

use axum::Json;
use axum::extract::{Multipart, State};
use storeit_domain::errors::DomainError;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;
use crate::dto::*;
use crate::error::AppError;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(identify))
        .routes(routes!(identify_correct))
}

#[utoipa::path(
    post,
    path = "/",
    tag = "identify",
    description = "Upload a photo for AI-powered item identification. Returns suggested name, category, and other metadata.",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, body = IdentificationResponse),
        (status = 400, body = ErrorResponse),
        (status = 501, body = ErrorResponse),
    ),
)]
pub async fn identify(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<IdentificationResponse>, AppError> {
    let identifier = state.item_identifier.as_ref().ok_or_else(|| {
        AppError(DomainError::Internal(
            "AI identification is not configured".into(),
        ))
    })?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_mime: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError(DomainError::Validation(format!("multipart error: {e}"))))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "photo" {
            file_mime = field.content_type().map(|s| s.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError(DomainError::Validation(e.to_string())))?;
            file_data = Some(bytes.to_vec());
        }
    }

    let file_data =
        file_data.ok_or_else(|| AppError(DomainError::Validation("missing photo field".into())))?;
    let mime_type = file_mime.unwrap_or_else(|| "image/jpeg".into());

    let result = identifier
        .identify(&file_data, &mime_type)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "AI identification failed");
            AppError(e)
        })?;
    Ok(Json(result.into()))
}

#[utoipa::path(
    post,
    path = "/correct",
    tag = "identify",
    description = "Re-identify a photo with a user-provided correction hint to improve the AI result.",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, body = IdentificationResponse),
        (status = 400, body = ErrorResponse),
        (status = 501, body = ErrorResponse),
    ),
)]
pub async fn identify_correct(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<IdentificationResponse>, AppError> {
    let identifier = state.item_identifier.as_ref().ok_or_else(|| {
        AppError(DomainError::Internal(
            "AI identification is not configured".into(),
        ))
    })?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_mime: Option<String> = None;
    let mut correction: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError(DomainError::Validation(format!("multipart error: {e}"))))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "photo" => {
                file_mime = field.content_type().map(|s| s.to_string());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError(DomainError::Validation(e.to_string())))?;
                file_data = Some(bytes.to_vec());
            }
            "correction" => {
                correction = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError(DomainError::Validation(e.to_string())))?,
                );
            }
            _ => {}
        }
    }

    let file_data =
        file_data.ok_or_else(|| AppError(DomainError::Validation("missing photo field".into())))?;
    let correction = correction
        .ok_or_else(|| AppError(DomainError::Validation("missing correction field".into())))?;
    let mime_type = file_mime.unwrap_or_else(|| "image/jpeg".into());

    let result = identifier
        .identify_with_correction(&file_data, &mime_type, &correction)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "AI identification with correction failed");
            AppError(e)
        })?;
    Ok(Json(result.into()))
}
