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

use actix_web::{http::StatusCode, web as actix, HttpResponse, Responder, Scope};
use gemstone_rs::{web as gemstone_web, Config, Result, SessionWorkerPool};

/// Route contract exposed by [`scope`] and [`scope_with_name`].
pub const ROUTES: &[&str] = &["GET /", "GET /health/local", "GET /health/gemstone"];

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
pub async fn root(state: actix::Data<AppState>) -> impl Responder {
    actix_response(gemstone_web::index_response(&state.service_name))
}

/// Local process health route handler.
pub async fn health_local() -> impl Responder {
    actix_response(gemstone_web::local_health_response())
}

/// Live GemStone health route handler.
pub async fn health_gemstone(state: actix::Data<AppState>) -> impl Responder {
    let health = state.health.clone();
    let response = match actix::block(move || health.gemstone_health_response()).await {
        Ok(response) => response,
        Err(err) => gemstone_web::JsonResponse::error(500, err.to_string()),
    };
    actix_response(response)
}

/// Convert a shared gemstone-rs JSON response into an Actix response.
pub fn actix_response(response: gemstone_web::JsonResponse) -> HttpResponse {
    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    HttpResponse::build(status)
        .content_type("application/json")
        .body(response.body)
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
    fn actix_response_accepts_shared_response() {
        let response = gemstone_web::JsonResponse::ok(r#"{"ok":true}"#.to_string());
        let actual = actix_response(response);
        assert_eq!(actual.status(), StatusCode::OK);
    }

    #[test]
    fn unavailable_health_pool_is_supported() {
        let _ = scope_with_health_pool(
            gemstone_web::HealthPool::unavailable("missing test credentials"),
            "test service",
        );
    }
}
