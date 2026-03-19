use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use storeit_domain::entities::{CreateContainer, EntityType, MoveTarget, UpdateContainer};
use storeit_domain::services::{build_container_search_text, validate_container_move};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;
use crate::dto::*;
use crate::error::AppError;
use crate::extractors::AuthContext;

use super::not_found;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_containers, create_container))
        .routes(routes!(get_container, update_container, delete_container))
        .routes(routes!(move_container))
        .routes(routes!(get_ancestry))
        .routes(routes!(list_child_containers))
        .routes(routes!(list_container_items))
}

#[utoipa::path(
    get,
    path = "/",
    tag = "containers",
    description = "List all containers.",
    responses(
        (status = 200, body = Vec<ContainerResponse>),
    ),
)]
pub async fn list_containers(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<Vec<ContainerResponse>>, AppError> {
    let gid = auth.group_id;
    let containers = state.container_repo.list_all(gid).await?;
    Ok(Json(containers.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/",
    tag = "containers",
    description = "Create a new container inside a location or another container.",
    request_body = CreateContainerRequest,
    responses(
        (status = 201, body = ContainerResponse),
        (status = 400, body = ErrorResponse),
    ),
)]
pub async fn create_container(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<CreateContainerRequest>,
) -> Result<(StatusCode, Json<ContainerResponse>), AppError> {
    let gid = auth.group_id;
    let parent = req.to_parent_ref()?;
    let input = CreateContainer {
        parent,
        name: req.name,
        description: req.description,
        color: req.color,
    };
    let container = state.container_repo.create(gid, input).await?;
    let text = build_container_search_text(&container);
    let _ = state
        .search_repo
        .index(EntityType::Container, container.id, gid, &text)
        .await;
    Ok((StatusCode::CREATED, Json(container.into())))
}

#[utoipa::path(
    get,
    path = "/{id}",
    tag = "containers",
    description = "Get a single container by ID.",
    params(("id" = Uuid, Path, description = "Container ID")),
    responses(
        (status = 200, body = ContainerResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn get_container(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ContainerResponse>, AppError> {
    let gid = auth.group_id;
    let container = state
        .container_repo
        .get(id, gid)
        .await?
        .ok_or_else(|| not_found("container", id))?;
    Ok(Json(container.into()))
}

#[utoipa::path(
    put,
    path = "/{id}",
    tag = "containers",
    description = "Update a container's name, description, or color.",
    params(("id" = Uuid, Path, description = "Container ID")),
    request_body = UpdateContainerRequest,
    responses(
        (status = 200, body = ContainerResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn update_container(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateContainerRequest>,
) -> Result<Json<ContainerResponse>, AppError> {
    let gid = auth.group_id;
    let input = UpdateContainer {
        name: req.name,
        description: req.description,
        color: req.color,
    };
    let container = state.container_repo.update(id, gid, input).await?;
    let text = build_container_search_text(&container);
    let _ = state
        .search_repo
        .index(EntityType::Container, container.id, gid, &text)
        .await;
    Ok(Json(container.into()))
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "containers",
    description = "Delete a container. Fails with 409 if it still contains children.",
    params(("id" = Uuid, Path, description = "Container ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse),
    ),
)]
pub async fn delete_container(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let gid = auth.group_id;
    state.container_repo.delete(id, gid).await?;
    let _ = state.search_repo.remove(EntityType::Container, id).await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/{id}/move",
    tag = "containers",
    description = "Move a container to a different parent location or container. Validates against circular nesting.",
    params(("id" = Uuid, Path, description = "Container ID")),
    request_body = MoveRequest,
    responses(
        (status = 200, body = ContainerResponse),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse),
    ),
)]
pub async fn move_container(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<MoveRequest>,
) -> Result<Json<ContainerResponse>, AppError> {
    let gid = auth.group_id;
    let target = MoveTarget {
        target: req.to_parent_ref()?,
    };
    validate_container_move(state.container_repo.as_ref(), id, &target, gid).await?;
    let container = state.container_repo.move_to(id, gid, target).await?;
    Ok(Json(container.into()))
}

#[utoipa::path(
    get,
    path = "/{id}/ancestry",
    tag = "containers",
    description = "Get the full ancestry (breadcrumb path) from root location to this container.",
    params(("id" = Uuid, Path, description = "Container ID")),
    responses(
        (status = 200, body = Vec<AncestryNodeResponse>),
    ),
)]
pub async fn get_ancestry(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AncestryNodeResponse>>, AppError> {
    let gid = auth.group_id;
    let ancestry = state.container_repo.get_ancestry(id, gid).await?;
    Ok(Json(ancestry.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/{id}/containers",
    tag = "containers",
    description = "List containers nested inside this container.",
    params(("id" = Uuid, Path, description = "Container ID")),
    responses(
        (status = 200, body = Vec<ContainerResponse>),
    ),
)]
pub async fn list_child_containers(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ContainerResponse>>, AppError> {
    let gid = auth.group_id;
    let containers = state.container_repo.list_by_container(id, gid).await?;
    Ok(Json(containers.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/{id}/items",
    tag = "containers",
    description = "List items inside this container.",
    params(("id" = Uuid, Path, description = "Container ID")),
    responses(
        (status = 200, body = Vec<ItemResponse>),
    ),
)]
pub async fn list_container_items(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ItemResponse>>, AppError> {
    let gid = auth.group_id;
    let items = state.item_repo.list_by_container(id, gid).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}
