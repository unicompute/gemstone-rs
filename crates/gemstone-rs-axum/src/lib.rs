//! Axum helpers for `gemstone-rs` services.
//!
//! The adapter exposes a small, reusable route set for service startup checks:
//! `/`, `/health/local`, and `/health/gemstone`.
//!
//! ```no_run
//! # async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! let pool = gemstone_rs_axum::pool_from_env(2)?;
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
//! axum::serve(listener, gemstone_rs_axum::router(pool)).await?;
//! # Ok(())
//! # }
//! ```
//!
//! Use [`router_from_env`] when the service should start before GemStone
//! credentials are configured. `/health/gemstone` will report a `503` JSON
//! error until the pool can be created.

use axum::{
    extract::OriginalUri,
    extract::State,
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use gemstone_rs::{web as gemstone_web, Config, Result, SessionWorkerPool};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

/// Adapter name exposed through diagnostic response headers.
pub const ADAPTER_NAME: &str = "axum";

/// Header reporting which adapter produced the response.
pub const ADAPTER_HEADER: &str = "x-gemstone-rs-adapter";

/// Header reporting which route handler produced the response.
pub const ROUTE_HEADER: &str = "x-gemstone-rs-route";

/// Incoming request id header accepted by the tracing helpers.
pub const INCOMING_REQUEST_ID_HEADER: &str = "x-request-id";

/// Header reporting the request id used for route tracing.
pub const REQUEST_ID_HEADER: &str = "x-gemstone-rs-request-id";

/// Header reporting the request method used for route tracing.
pub const REQUEST_METHOD_HEADER: &str = "x-gemstone-rs-request-method";

/// Header reporting the request path used for route tracing.
pub const REQUEST_PATH_HEADER: &str = "x-gemstone-rs-request-path";

/// Header reporting the request lifecycle marker used by the adapter.
pub const REQUEST_LIFECYCLE_HEADER: &str = "x-gemstone-rs-request-lifecycle";

/// Stable lifecycle value emitted after an adapter route is handled.
pub const REQUEST_LIFECYCLE_VALUE: &str = "received,handled";

/// Header reporting the adapter route handler duration in microseconds.
pub const REQUEST_DURATION_US_HEADER: &str = "x-gemstone-rs-request-duration-us";

/// Route contract exposed by [`router`] and [`router_with_name`].
pub const ROUTES: &[&str] = &["GET /", "GET /health/local", "GET /health/gemstone"];

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Shared Axum state for the GemStone health router.
#[derive(Clone)]
pub struct AppState {
    /// Health backend used by `/health/gemstone`.
    pub health: gemstone_web::HealthPool,
    /// Service label returned by `/`.
    pub service_name: String,
}

impl AppState {
    /// Build reusable route state.
    pub fn new(health: gemstone_web::HealthPool, service_name: impl Into<String>) -> Self {
        Self {
            health,
            service_name: service_name.into(),
        }
    }
}

/// Start a [`SessionWorkerPool`] from `GS_*` environment variables.
pub fn pool_from_env(workers: usize) -> Result<SessionWorkerPool> {
    SessionWorkerPool::start(Config::from_env()?, workers)
}

/// Build a health backend from `GS_*` environment variables without failing
/// service startup.
pub fn health_pool_from_env(workers: usize) -> gemstone_web::HealthPool {
    gemstone_web::HealthPool::start_from_env(workers)
}

/// Build the default GemStone health router.
pub fn router(pool: SessionWorkerPool) -> Router {
    router_with_name(pool, "gemstone-rs Axum service")
}

/// Build the default router from environment, keeping local routes available
/// even when GemStone is not configured yet.
pub fn router_from_env(workers: usize) -> Router {
    router_from_env_with_name(workers, "gemstone-rs Axum service")
}

/// Build a named router from environment, keeping local routes available even
/// when GemStone is not configured yet.
pub fn router_from_env_with_name(workers: usize, service_name: impl Into<String>) -> Router {
    router_with_health_pool(health_pool_from_env(workers), service_name)
}

/// Build the GemStone health router with a custom service label.
pub fn router_with_name(pool: SessionWorkerPool, service_name: impl Into<String>) -> Router {
    router_with_health_pool(gemstone_web::HealthPool::ready(pool), service_name)
}

/// Build the GemStone health router with an already-created health backend.
pub fn router_with_health_pool(
    health: gemstone_web::HealthPool,
    service_name: impl Into<String>,
) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health/local", get(health_local))
        .route("/health/gemstone", get(health_gemstone))
        .with_state(AppState::new(health, service_name))
}

/// Root route handler.
pub async fn root(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> impl IntoResponse {
    json_response_with_route_and_trace(
        gemstone_web::index_response(&state.service_name),
        "root",
        RequestTrace::from_parts(&method, uri.path(), &headers),
    )
}

/// Local process health route handler.
pub async fn health_local(
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> impl IntoResponse {
    json_response_with_route_and_trace(
        gemstone_web::local_health_response(),
        "health.local",
        RequestTrace::from_parts(&method, uri.path(), &headers),
    )
}

/// Live GemStone health route handler.
pub async fn health_gemstone(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> impl IntoResponse {
    let health = state.health.clone();
    let response = health.gemstone_health_response_async().await;
    json_response_with_route_and_trace(
        response,
        "health.gemstone",
        RequestTrace::from_parts(&method, uri.path(), &headers),
    )
}

/// Convert a shared gemstone-rs JSON response into an Axum response tuple.
pub fn json_response(response: gemstone_web::JsonResponse) -> Response {
    json_response_with_route(response, "generic")
}

/// Convert a shared gemstone-rs JSON response into an Axum response tuple with
/// diagnostic adapter headers.
pub fn json_response_with_route(
    response: gemstone_web::JsonResponse,
    route: &'static str,
) -> Response {
    json_response_with_route_and_trace(response, route, RequestTrace::generated("GET", "/"))
}

/// Convert a shared gemstone-rs JSON response into an Axum response with
/// diagnostic adapter and request trace headers.
pub fn json_response_with_route_and_trace(
    response: gemstone_web::JsonResponse,
    route: &'static str,
    trace: RequestTrace,
) -> Response {
    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut response = (
        status,
        [
            (header::CONTENT_TYPE, "application/json"),
            (HeaderName::from_static(ADAPTER_HEADER), ADAPTER_NAME),
            (HeaderName::from_static(ROUTE_HEADER), route),
        ],
        response.body,
    )
        .into_response();
    trace.insert_headers(response.headers_mut());
    response
}

/// Request trace metadata emitted as response headers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestTrace {
    /// Request id propagated from `x-request-id`, or generated locally.
    pub request_id: String,
    /// HTTP method observed by the adapter.
    pub method: String,
    /// Request path observed by the adapter.
    pub path: String,
    started_at: Instant,
}

impl RequestTrace {
    /// Build request trace metadata from Axum request parts.
    pub fn from_parts(method: &Method, path: &str, headers: &HeaderMap) -> Self {
        let request_id = headers
            .get(INCOMING_REQUEST_ID_HEADER)
            .or_else(|| headers.get(REQUEST_ID_HEADER))
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("axum-{}", NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)));
        Self {
            request_id,
            method: method.as_str().to_string(),
            path: path.to_string(),
            started_at: Instant::now(),
        }
    }

    /// Build generated trace metadata for generic response conversions.
    pub fn generated(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            request_id: format!("axum-{}", NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)),
            method: method.into(),
            path: path.into(),
            started_at: Instant::now(),
        }
    }

    fn insert_headers(&self, headers: &mut HeaderMap) {
        headers.insert(
            HeaderName::from_static(REQUEST_ID_HEADER),
            header_value(&self.request_id),
        );
        headers.insert(
            HeaderName::from_static(REQUEST_METHOD_HEADER),
            header_value(&self.method),
        );
        headers.insert(
            HeaderName::from_static(REQUEST_PATH_HEADER),
            header_value(&self.path),
        );
        headers.insert(
            HeaderName::from_static(REQUEST_LIFECYCLE_HEADER),
            HeaderValue::from_static(REQUEST_LIFECYCLE_VALUE),
        );
        headers.insert(
            HeaderName::from_static(REQUEST_DURATION_US_HEADER),
            header_value(&self.started_at.elapsed().as_micros().to_string()),
        );
    }
}

fn header_value(value: &str) -> HeaderValue {
    HeaderValue::from_str(value).unwrap_or_else(|_| HeaderValue::from_static("invalid"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_are_stable() {
        assert_eq!(
            ROUTES,
            &["GET /", "GET /health/local", "GET /health/gemstone"]
        );
        assert_eq!(ADAPTER_HEADER, "x-gemstone-rs-adapter");
        assert_eq!(ROUTE_HEADER, "x-gemstone-rs-route");
        assert_eq!(REQUEST_ID_HEADER, "x-gemstone-rs-request-id");
        assert_eq!(REQUEST_METHOD_HEADER, "x-gemstone-rs-request-method");
        assert_eq!(REQUEST_PATH_HEADER, "x-gemstone-rs-request-path");
        assert_eq!(REQUEST_LIFECYCLE_HEADER, "x-gemstone-rs-request-lifecycle");
        assert_eq!(
            REQUEST_DURATION_US_HEADER,
            "x-gemstone-rs-request-duration-us"
        );
    }

    #[test]
    fn json_response_accepts_shared_response() {
        let response = gemstone_web::JsonResponse::ok(r#"{"ok":true}"#.to_string());
        let _ = json_response(response);
    }

    #[test]
    fn request_trace_propagates_request_id() {
        let mut headers = HeaderMap::new();
        headers.insert(
            INCOMING_REQUEST_ID_HEADER,
            HeaderValue::from_static("smoke-123"),
        );
        let trace = RequestTrace::from_parts(&Method::GET, "/health/local", &headers);
        assert_eq!(trace.request_id, "smoke-123");
        assert_eq!(trace.method, "GET");
        assert_eq!(trace.path, "/health/local");
    }

    #[test]
    fn response_includes_lifecycle_headers() {
        let response = json_response_with_route_and_trace(
            gemstone_web::JsonResponse::ok(r#"{"ok":true}"#.to_string()),
            "health.local",
            RequestTrace::generated("GET", "/health/local"),
        );
        assert_eq!(
            response
                .headers()
                .get(REQUEST_LIFECYCLE_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some(REQUEST_LIFECYCLE_VALUE)
        );
        let duration = response
            .headers()
            .get(REQUEST_DURATION_US_HEADER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u128>().ok());
        assert!(duration.is_some());
    }

    #[test]
    fn unavailable_health_pool_is_supported() {
        let _ = router_with_health_pool(
            gemstone_web::HealthPool::unavailable("missing test credentials"),
            "test service",
        );
    }
}
