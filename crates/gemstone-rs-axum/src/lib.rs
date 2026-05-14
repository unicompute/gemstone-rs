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
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use gemstone_rs::{web as gemstone_web, Config, Result, SessionWorkerPool};

/// Route contract exposed by [`router`] and [`router_with_name`].
pub const ROUTES: &[&str] = &["GET /", "GET /health/local", "GET /health/gemstone"];

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
pub async fn root(State(state): State<AppState>) -> impl IntoResponse {
    json_response(gemstone_web::index_response(&state.service_name))
}

/// Local process health route handler.
pub async fn health_local() -> impl IntoResponse {
    json_response(gemstone_web::local_health_response())
}

/// Live GemStone health route handler.
pub async fn health_gemstone(State(state): State<AppState>) -> impl IntoResponse {
    let health = state.health.clone();
    let response =
        match tokio::task::spawn_blocking(move || health.gemstone_health_response()).await {
            Ok(response) => response,
            Err(err) => gemstone_web::JsonResponse::error(500, err.to_string()),
        };
    json_response(response)
}

/// Convert a shared gemstone-rs JSON response into an Axum response tuple.
pub fn json_response(response: gemstone_web::JsonResponse) -> impl IntoResponse {
    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        response.body,
    )
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
    }

    #[test]
    fn json_response_accepts_shared_response() {
        let response = gemstone_web::JsonResponse::ok(r#"{"ok":true}"#.to_string());
        let _ = json_response(response);
    }

    #[test]
    fn unavailable_health_pool_is_supported() {
        let _ = router_with_health_pool(
            gemstone_web::HealthPool::unavailable("missing test credentials"),
            "test service",
        );
    }
}
