use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;
use crate::dto::*;
use crate::handlers;
use crate::static_files;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "StoreIT API",
        version = "0.1.0",
        description = "Home inventory management API"
    ),
    tags(
        (name = "auth", description = "Authentication"),
        (name = "admin", description = "Admin management"),
        (name = "locations", description = "Location management"),
        (name = "containers", description = "Container management"),
        (name = "items", description = "Item management"),
        (name = "photos", description = "Photo management"),
        (name = "search", description = "Full-text search"),
        (name = "identify", description = "AI item identification"),
        (name = "nfc", description = "NFC tag management"),
    ),
    components(schemas(
        LocationResponse, CreateLocationRequest, UpdateLocationRequest, LocationTreeNodeResponse,
        ContainerResponse, CreateContainerRequest, UpdateContainerRequest,
        MoveRequest, AncestryNodeResponse,
        ItemResponse, CreateItemRequest, UpdateItemRequest,
        PhotoResponse,
        SearchResultItem, SearchResponse,
        UserResponse, GroupResponse, MeResponse, SwitchGroupRequest,
        AuthModeResponse, LocalLoginRequest,
        AdminUserResponse, CreateLocalUserRequest, UpdateLocalUserRequest, ResetPasswordRequest,
        CreateGroupRequest, AdminGroupResponse, AddMemberRequest, GroupMemberResponse,
        AdminSettingsResponse, UpdateSettingsRequest,
        BackupRequest, BackupJobResponse, JobStatusResponse, RestoreOptions,
        IdentificationResponse,
        NfcTagResponse, CreateNfcTagRequest, AssignNfcTagRequest, NfcResolveResponse,
        ErrorResponse, ErrorDetail,
    ))
)]
struct ApiDoc;

pub fn build_router(state: Arc<AppState>) -> Router {
    let (router, api) = OpenApiRouter::<Arc<AppState>>::with_openapi(ApiDoc::openapi())
        .nest(
            "/api/v1/auth",
            handlers::auth::router().merge(
                utoipa_axum::router::OpenApiRouter::<Arc<AppState>>::new()
                    .nest("/local", handlers::local_auth::router()),
            ),
        )
        .nest("/api/v1/admin", handlers::admin::router())
        .nest("/api/v1/locations", handlers::locations::router())
        .nest("/api/v1/containers", handlers::containers::router())
        .nest("/api/v1/items", handlers::items::router())
        .nest("/api/v1/photos", handlers::photos::router())
        .nest("/api/v1/search", handlers::search::router())
        .nest("/api/v1/identify", handlers::identify::router())
        .nest("/api/v1/nfc-tags", handlers::nfc::router())
        .split_for_parts();

    router
        .with_state(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .fallback(static_files::static_handler)
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB for camera photos
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
