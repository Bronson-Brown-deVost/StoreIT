use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use storeit_domain::entities::{CreateLocation, EntityType, UpdateLocation};
use storeit_domain::services::build_location_search_text;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;
use crate::dto::*;
use crate::error::AppError;
use crate::extractors::AuthContext;

use super::not_found;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_roots, create_location))
        .routes(routes!(get_tree))
        .routes(routes!(get_location, update_location, delete_location))
        .routes(routes!(list_children))
        .routes(routes!(list_location_containers))
        .routes(routes!(list_location_items))
}

#[utoipa::path(
    get,
    path = "/",
    tag = "locations",
    responses(
        (status = 200, body = Vec<LocationResponse>),
    ),
)]
pub async fn list_roots(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<Vec<LocationResponse>>, AppError> {
    let gid = auth.group_id;
    let locations = state.location_repo.list_roots(gid).await?;
    Ok(Json(locations.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/",
    tag = "locations",
    request_body = CreateLocationRequest,
    responses(
        (status = 201, body = LocationResponse),
        (status = 400, body = ErrorResponse),
    ),
)]
pub async fn create_location(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<CreateLocationRequest>,
) -> Result<(StatusCode, Json<LocationResponse>), AppError> {
    let gid = auth.group_id;
    let input = CreateLocation {
        parent_id: req.parent_id,
        name: req.name,
        description: req.description,
        latitude: req.latitude,
        longitude: req.longitude,
    };
    let location = state.location_repo.create(gid, input).await?;
    let text = build_location_search_text(&location);
    let _ = state
        .search_repo
        .index(EntityType::Location, location.id, gid, &text)
        .await;
    Ok((StatusCode::CREATED, Json(location.into())))
}

#[utoipa::path(
    get,
    path = "/tree",
    tag = "locations",
    responses(
        (status = 200, body = Vec<LocationTreeNodeResponse>),
    ),
)]
pub async fn get_tree(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<Vec<LocationTreeNodeResponse>>, AppError> {
    let gid = auth.group_id;
    let tree = state.location_repo.get_tree(gid).await?;
    Ok(Json(tree.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/{id}",
    tag = "locations",
    params(("id" = Uuid, Path, description = "Location ID")),
    responses(
        (status = 200, body = LocationResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn get_location(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<LocationResponse>, AppError> {
    let gid = auth.group_id;
    let location = state
        .location_repo
        .get(id, gid)
        .await?
        .ok_or_else(|| not_found("location", id))?;
    Ok(Json(location.into()))
}

#[utoipa::path(
    put,
    path = "/{id}",
    tag = "locations",
    params(("id" = Uuid, Path, description = "Location ID")),
    request_body = UpdateLocationRequest,
    responses(
        (status = 200, body = LocationResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn update_location(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<Json<LocationResponse>, AppError> {
    let gid = auth.group_id;
    let input = UpdateLocation {
        name: req.name,
        description: req.description,
        latitude: req.latitude,
        longitude: req.longitude,
    };
    let location = state.location_repo.update(id, gid, input).await?;
    let text = build_location_search_text(&location);
    let _ = state
        .search_repo
        .index(EntityType::Location, location.id, gid, &text)
        .await;
    Ok(Json(location.into()))
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "locations",
    params(("id" = Uuid, Path, description = "Location ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse),
    ),
)]
pub async fn delete_location(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let gid = auth.group_id;
    state.location_repo.delete(id, gid).await?;
    let _ = state.search_repo.remove(EntityType::Location, id).await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/{id}/children",
    tag = "locations",
    params(("id" = Uuid, Path, description = "Location ID")),
    responses(
        (status = 200, body = Vec<LocationResponse>),
    ),
)]
pub async fn list_children(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<LocationResponse>>, AppError> {
    let gid = auth.group_id;
    let children = state.location_repo.list_children(id, gid).await?;
    Ok(Json(children.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/{id}/containers",
    tag = "locations",
    params(("id" = Uuid, Path, description = "Location ID")),
    responses(
        (status = 200, body = Vec<ContainerResponse>),
    ),
)]
pub async fn list_location_containers(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ContainerResponse>>, AppError> {
    let gid = auth.group_id;
    let containers = state.container_repo.list_by_location(id, gid).await?;
    Ok(Json(containers.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/{id}/items",
    tag = "locations",
    params(("id" = Uuid, Path, description = "Location ID")),
    responses(
        (status = 200, body = Vec<ItemResponse>),
    ),
)]
pub async fn list_location_items(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ItemResponse>>, AppError> {
    let gid = auth.group_id;
    let items = state.item_repo.list_by_location(id, gid).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}
