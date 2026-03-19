use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use storeit_domain::entities::{CreateItem, EntityType, MoveTarget, UpdateItem};
use storeit_domain::services::build_item_search_text;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;
use crate::dto::*;
use crate::error::AppError;
use crate::extractors::AuthContext;

use super::not_found;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(list_items, create_item))
        .routes(routes!(create_item_batch))
        .routes(routes!(get_item, update_item, delete_item))
        .routes(routes!(move_item))
}

#[utoipa::path(
    get,
    path = "/",
    tag = "items",
    description = "List all items.",
    responses(
        (status = 200, body = Vec<ItemResponse>),
    ),
)]
pub async fn list_items(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
) -> Result<Json<Vec<ItemResponse>>, AppError> {
    let gid = auth.group_id;
    let items = state.item_repo.list_all(gid).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/",
    tag = "items",
    description = "Create a new item inside a location or container.",
    request_body = CreateItemRequest,
    responses(
        (status = 201, body = ItemResponse),
        (status = 400, body = ErrorResponse),
    ),
)]
pub async fn create_item(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(req): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<ItemResponse>), AppError> {
    let gid = auth.group_id;
    let parent = req.to_parent_ref()?;
    let input = CreateItem {
        parent,
        name: req.name,
        description: req.description,
        aliases: req.aliases,
        keywords: req.keywords,
        category: req.category,
        barcode: req.barcode,
        material: req.material,
        color: req.color,
        condition_notes: req.condition_notes,
        quantity: req.quantity,
    };
    let item = state.item_repo.create(gid, input).await?;
    let text = build_item_search_text(&item);
    let _ = state
        .search_repo
        .index(EntityType::Item, item.id, gid, &text)
        .await;
    Ok((StatusCode::CREATED, Json(item.into())))
}

#[utoipa::path(
    post,
    path = "/batch",
    tag = "items",
    description = "Create multiple items in a single request.",
    request_body = Vec<CreateItemRequest>,
    responses(
        (status = 201, body = Vec<ItemResponse>),
        (status = 400, body = ErrorResponse),
    ),
)]
pub async fn create_item_batch(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Json(reqs): Json<Vec<CreateItemRequest>>,
) -> Result<(StatusCode, Json<Vec<ItemResponse>>), AppError> {
    let gid = auth.group_id;
    let mut inputs = Vec::with_capacity(reqs.len());
    for req in reqs {
        let parent = req.to_parent_ref()?;
        inputs.push(CreateItem {
            parent,
            name: req.name,
            description: req.description,
            aliases: req.aliases,
            keywords: req.keywords,
            category: req.category,
            barcode: req.barcode,
            material: req.material,
            color: req.color,
            condition_notes: req.condition_notes,
            quantity: req.quantity,
        });
    }
    let items = state.item_repo.create_batch(gid, inputs).await?;
    for item in &items {
        let text = build_item_search_text(item);
        let _ = state
            .search_repo
            .index(EntityType::Item, item.id, gid, &text)
            .await;
    }
    Ok((
        StatusCode::CREATED,
        Json(items.into_iter().map(Into::into).collect()),
    ))
}

#[utoipa::path(
    get,
    path = "/{id}",
    tag = "items",
    description = "Get a single item by ID.",
    params(("id" = Uuid, Path, description = "Item ID")),
    responses(
        (status = 200, body = ItemResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn get_item(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ItemResponse>, AppError> {
    let gid = auth.group_id;
    let item = state
        .item_repo
        .get(id, gid)
        .await?
        .ok_or_else(|| not_found("item", id))?;
    Ok(Json(item.into()))
}

#[utoipa::path(
    put,
    path = "/{id}",
    tag = "items",
    description = "Update an item's properties (name, description, category, quantity, etc.).",
    params(("id" = Uuid, Path, description = "Item ID")),
    request_body = UpdateItemRequest,
    responses(
        (status = 200, body = ItemResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn update_item(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateItemRequest>,
) -> Result<Json<ItemResponse>, AppError> {
    let gid = auth.group_id;
    let input = UpdateItem {
        name: req.name,
        description: req.description,
        aliases: req.aliases,
        keywords: req.keywords,
        category: req.category,
        barcode: req.barcode,
        material: req.material,
        color: req.color,
        condition_notes: req.condition_notes,
        quantity: req.quantity,
    };
    let item = state.item_repo.update(id, gid, input).await?;
    let text = build_item_search_text(&item);
    let _ = state
        .search_repo
        .index(EntityType::Item, item.id, gid, &text)
        .await;
    Ok(Json(item.into()))
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "items",
    description = "Delete an item.",
    params(("id" = Uuid, Path, description = "Item ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let gid = auth.group_id;
    state.item_repo.delete(id, gid).await?;
    let _ = state.search_repo.remove(EntityType::Item, id).await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/{id}/move",
    tag = "items",
    description = "Move an item to a different parent location or container.",
    params(("id" = Uuid, Path, description = "Item ID")),
    request_body = MoveRequest,
    responses(
        (status = 200, body = ItemResponse),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn move_item(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(req): Json<MoveRequest>,
) -> Result<Json<ItemResponse>, AppError> {
    let gid = auth.group_id;
    let target = MoveTarget {
        target: req.to_parent_ref()?,
    };
    let item = state.item_repo.move_to(id, gid, target).await?;
    Ok(Json(item.into()))
}
