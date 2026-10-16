//! Embedded frontend (`frontend/build`) as compile-time blobs — served from memory.

#![allow(clippy::same_name_method)] // rust-embed `Embed` derive

use std::borrow::Cow;

use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "frontend/build/"]
struct Frontend;

/// SPA fallback: exact asset → else `index.html` (client routes).
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let candidate = if path.is_empty() { "index.html" } else { path };

    if let Some(response) = serve_file(candidate) {
        return response;
    }

    // Client-side routes (no file extension) fall back to the shell.
    if !candidate.contains('.')
        && let Some(response) = serve_file("index.html")
    {
        return response;
    }

    (StatusCode::NOT_FOUND, "not found").into_response()
}

fn serve_file(path: &str) -> Option<Response> {
    let file = Frontend::get(path)?;
    let mime = content_type(path);
    let cache = cache_control(path);
    let body = match file.data {
        Cow::Borrowed(bytes) => Body::from(bytes),
        Cow::Owned(bytes) => Body::from(bytes),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, cache)
        .body(body)
        .ok()
}

fn content_type(path: &str) -> HeaderValue {
    let mime = match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    };
    HeaderValue::from_static(mime)
}

fn cache_control(path: &str) -> HeaderValue {
    // Vite/SvelteKit hashed assets under `_app/immutable/` are content-addressed.
    if path.contains("/_app/immutable/") || path.starts_with("_app/immutable/") {
        HeaderValue::from_static("public, max-age=31536000, immutable")
    } else {
        HeaderValue::from_static("no-cache")
    }
}
