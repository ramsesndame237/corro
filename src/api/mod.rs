use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::Request,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
    routing::get,
};
use tower_http::{catch_panic::CatchPanicLayer, trace::TraceLayer};

use crate::error::CorroError;
use crate::storage::StorageBackend;

pub mod health;

/// Shared application state injected into every handler.
pub type AppState = Arc<dyn StorageBackend>;

/// Build the full axum router with all middleware layers.
///
/// Middleware stack (outermost → innermost):
/// 1. CatchPanic   — never crash the process on a handler panic
/// 2. Timeout      — enforce per-request deadline (axum middleware)
/// 3. Trace        — structured request/response logging
pub fn build_router(backend: AppState, timeout_secs: u64) -> Router {
    Router::new()
        // ── Observability endpoints (unauthenticated) ──
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        // ── Fallback: all unrecognised routes return S3 NotImplemented ──
        .fallback(not_implemented_handler)
        // ── Middleware stack ──
        // Note: layers are applied innermost-first (last .layer() = outermost).
        .layer(CatchPanicLayer::new())
        .layer(middleware::from_fn(move |req, next| {
            timeout_middleware(req, next, timeout_secs)
        }))
        .layer(TraceLayer::new_for_http())
        .with_state(backend)
}

/// Enforce a per-request timeout. Returns 408 Request Timeout on expiry.
async fn timeout_middleware(req: Request, next: Next, timeout_secs: u64) -> Response {
    match tokio::time::timeout(Duration::from_secs(timeout_secs), next.run(req)).await {
        Ok(response) => response,
        Err(_elapsed) => {
            tracing::warn!(timeout_secs, "Request timed out");
            (
                axum::http::StatusCode::REQUEST_TIMEOUT,
                [("content-type", "application/xml; charset=utf-8")],
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Error><Code>RequestTimeout</Code>\
<Message>Your socket connection to the server was not read from or written to within the timeout period.</Message>\
</Error>",
            )
                .into_response()
        }
    }
}

async fn not_implemented_handler() -> CorroError {
    CorroError::NotImplemented
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::NullBackend;
    use axum::{body::to_bytes, http::Request};
    use tower::ServiceExt;

    fn test_app() -> Router {
        build_router(Arc::new(NullBackend), 30)
    }

    #[tokio::test]
    async fn health_route_exists() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn unknown_route_returns_not_implemented_xml() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/unknown-s3-route")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 501);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let xml = std::str::from_utf8(&body).unwrap();
        assert!(xml.contains("<Code>NotImplemented</Code>"));
    }
}
