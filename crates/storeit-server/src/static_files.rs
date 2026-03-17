use axum::http::{StatusCode, Uri, header};
use axum::response::{Html, IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../frontend/dist"]
struct FrontendAssets;

pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try to serve the exact file first
    if !path.is_empty()
        && let Some(file) = FrontendAssets::get(path)
    {
        return serve_file(path, &file);
    }

    // SPA fallback: serve index.html for any unmatched route
    match FrontendAssets::get("index.html") {
        Some(file) => {
            let html = String::from_utf8_lossy(&file.data);
            (
                [(header::CACHE_CONTROL, "no-cache")],
                Html(html.into_owned()),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "Frontend not embedded").into_response(),
    }
}

fn serve_file(path: &str, file: &rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let cache = if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };

    (
        [
            (header::CONTENT_TYPE, mime.as_ref().to_string()),
            (header::CACHE_CONTROL, cache.to_string()),
        ],
        file.data.clone(),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn static_handler_root_returns_response() {
        let uri: Uri = "/".parse().unwrap();
        let resp = static_handler(uri).await;
        // If frontend is embedded we get 200 with HTML, otherwise 404
        let status = resp.status();
        assert!(
            status == StatusCode::OK || status == StatusCode::NOT_FOUND,
            "expected 200 or 404, got {status}"
        );
    }

    #[tokio::test]
    async fn static_handler_unknown_path_returns_spa_fallback_or_404() {
        let uri: Uri = "/some/random/path".parse().unwrap();
        let resp = static_handler(uri).await;
        let status = resp.status();
        // SPA fallback serves index.html (200) or 404 if no frontend embedded
        assert!(
            status == StatusCode::OK || status == StatusCode::NOT_FOUND,
            "expected 200 or 404, got {status}"
        );
    }

    #[test]
    fn frontend_assets_iter_does_not_panic() {
        // Just ensure the RustEmbed derive works and iteration doesn't panic
        let _files: Vec<_> = FrontendAssets::iter().collect();
    }
}
