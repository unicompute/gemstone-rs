//! Actix Web helpers for `gemstone-rs` services.
//!
//! The adapter exposes a small, reusable route set for service startup checks:
//! `/`, `/health/local`, and `/health/gemstone`.
//!
//! ```no_run
//! # async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! let pool = gemstone_rs_actix::pool_from_env(2)?;
//! let server = actix_web::HttpServer::new(move || {
//!     actix_web::App::new().service(gemstone_rs_actix::scope(pool.clone()))
//! })
//! .bind("127.0.0.1:3000")?;
//! server.run().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Use [`scope_from_env`] when the service should start before GemStone
//! credentials are configured. `/health/gemstone` will report a `503` JSON
//! error until the pool can be created.

use actix_web::{http::StatusCode, web as actix, HttpRequest, HttpResponse, Responder, Scope};
use gemstone_rs::{web as gemstone_web, Config, Result, SessionWorkerPool};
use std::sync::atomic::{AtomicU64, Ordering};

/// Adapter name exposed through diagnostic response headers.
pub const ADAPTER_NAME: &str = "actix";

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

/// Route contract exposed by [`scope`] and [`scope_with_name`].
pub const ROUTES: &[&str] = &["GET /", "GET /health/local", "GET /health/gemstone"];

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Shared Actix state for the GemStone health scope.
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

/// Build the default GemStone health scope.
pub fn scope(pool: SessionWorkerPool) -> Scope {
    scope_with_name(pool, "gemstone-rs Actix service")
}

/// Build the default scope from environment, keeping local routes available
/// even when GemStone is not configured yet.
pub fn scope_from_env(workers: usize) -> Scope {
    scope_from_env_with_name(workers, "gemstone-rs Actix service")
}

/// Build a named scope from environment, keeping local routes available even
/// when GemStone is not configured yet.
pub fn scope_from_env_with_name(workers: usize, service_name: impl Into<String>) -> Scope {
    scope_with_health_pool(health_pool_from_env(workers), service_name)
}

/// Build the GemStone health scope with a custom service label.
pub fn scope_with_name(pool: SessionWorkerPool, service_name: impl Into<String>) -> Scope {
    scope_with_health_pool(gemstone_web::HealthPool::ready(pool), service_name)
}

/// Build the GemStone health scope with an already-created health backend.
pub fn scope_with_health_pool(
    health: gemstone_web::HealthPool,
    service_name: impl Into<String>,
) -> Scope {
    actix::scope("")
        .app_data(actix::Data::new(AppState::new(health, service_name)))
        .route("/", actix::get().to(root))
        .route("/health/local", actix::get().to(health_local))
        .route("/health/gemstone", actix::get().to(health_gemstone))
}

/// Root route handler.
pub async fn root(state: actix::Data<AppState>, request: HttpRequest) -> impl Responder {
    actix_response_with_route_and_trace(
        gemstone_web::index_response(&state.service_name),
        "root",
        RequestTrace::from_request(&request),
    )
}

/// Local process health route handler.
pub async fn health_local(request: HttpRequest) -> impl Responder {
    actix_response_with_route_and_trace(
        gemstone_web::local_health_response(),
        "health.local",
        RequestTrace::from_request(&request),
    )
}

/// Live GemStone health route handler.
pub async fn health_gemstone(state: actix::Data<AppState>, request: HttpRequest) -> impl Responder {
    let health = state.health.clone();
    let response = match actix::block(move || health.gemstone_health_response()).await {
        Ok(response) => response,
        Err(err) => gemstone_web::JsonResponse::error(500, err.to_string()),
    };
    actix_response_with_route_and_trace(
        response,
        "health.gemstone",
        RequestTrace::from_request(&request),
    )
}

/// Convert a shared gemstone-rs JSON response into an Actix response.
pub fn actix_response(response: gemstone_web::JsonResponse) -> HttpResponse {
    actix_response_with_route(response, "generic")
}

/// Convert a shared gemstone-rs JSON response into an Actix response with
/// diagnostic adapter headers.
pub fn actix_response_with_route(
    response: gemstone_web::JsonResponse,
    route: &'static str,
) -> HttpResponse {
    actix_response_with_route_and_trace(response, route, RequestTrace::generated("GET", "/"))
}

/// Convert a shared gemstone-rs JSON response into an Actix response with
/// diagnostic adapter and request trace headers.
pub fn actix_response_with_route_and_trace(
    response: gemstone_web::JsonResponse,
    route: &'static str,
    trace: RequestTrace,
) -> HttpResponse {
    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    HttpResponse::build(status)
        .content_type("application/json")
        .insert_header((ADAPTER_HEADER, ADAPTER_NAME))
        .insert_header((ROUTE_HEADER, route))
        .insert_header((REQUEST_ID_HEADER, trace.request_id))
        .insert_header((REQUEST_METHOD_HEADER, trace.method))
        .insert_header((REQUEST_PATH_HEADER, trace.path))
        .body(response.body)
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
}

impl RequestTrace {
    /// Build request trace metadata from an Actix request.
    pub fn from_request(request: &HttpRequest) -> Self {
        let request_id = request
            .headers()
            .get(INCOMING_REQUEST_ID_HEADER)
            .or_else(|| request.headers().get(REQUEST_ID_HEADER))
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                format!("actix-{}", NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed))
            });
        Self {
            request_id,
            method: request.method().as_str().to_string(),
            path: request.path().to_string(),
        }
    }

    /// Build generated trace metadata for generic response conversions.
    pub fn generated(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            request_id: format!("actix-{}", NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)),
            method: method.into(),
            path: path.into(),
        }
    }
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
    }

    #[test]
    fn actix_response_accepts_shared_response() {
        let response = gemstone_web::JsonResponse::ok(r#"{"ok":true}"#.to_string());
        let actual = actix_response(response);
        assert_eq!(actual.status(), StatusCode::OK);
    }

    #[test]
    fn generated_request_trace_is_stable() {
        let trace = RequestTrace::generated("GET", "/health/local");
        assert!(trace.request_id.starts_with("actix-"));
        assert_eq!(trace.method, "GET");
        assert_eq!(trace.path, "/health/local");
    }

    #[test]
    fn unavailable_health_pool_is_supported() {
        let _ = scope_with_health_pool(
            gemstone_web::HealthPool::unavailable("missing test credentials"),
            "test service",
        );
    }
}
