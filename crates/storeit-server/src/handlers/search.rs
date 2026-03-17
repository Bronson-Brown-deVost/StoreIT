use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;
use crate::dto::*;
use crate::error::AppError;
use crate::extractors::AuthContext;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(search))
}

#[utoipa::path(
    get,
    path = "/",
    tag = "search",
    params(SearchQuery),
    responses(
        (status = 200, body = SearchResponse),
    ),
)]
pub async fn search(
    State(state): State<Arc<AppState>>,
    auth: AuthContext,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, AppError> {
    let gid = auth.group_id;
    let limit = query.limit.unwrap_or(20);
    let results = state.search_repo.search(gid, &query.q, limit).await?;
    Ok(Json(SearchResponse {
        results: results.into_iter().map(Into::into).collect(),
    }))
}
