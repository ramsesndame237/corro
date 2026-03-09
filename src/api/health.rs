use axum::{http::StatusCode, response::Json};
use serde_json::{Value, json};

/// Liveness probe — returns 200 as long as the process is alive.
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Readiness probe — returns 200 when the server is ready to handle S3 traffic.
///
/// Will check storage backend reachability in issue #47.
pub async fn ready() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ready",
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::to_bytes, http::Request, routing::get};
    use tower::ServiceExt;

    fn test_router() -> Router {
        Router::new()
            .route("/health", get(health))
            .route("/ready", get(ready))
    }

    #[tokio::test]
    async fn health_returns_200() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_body_contains_status_ok() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn ready_returns_200() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
