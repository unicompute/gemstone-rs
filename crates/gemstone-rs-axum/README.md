# gemstone-rs-axum

`gemstone-rs-axum` provides small Axum adapter helpers for Rust services that
talk directly to GemStone/S through `gemstone-rs`.

```rust,no_run
use gemstone_rs_axum::{pool_from_env, router_with_name};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = pool_from_env(2)?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, router_with_name(pool, "my GemStone service")).await?;
    Ok(())
}
```

The router exposes:

- `GET /`
- `GET /health/local`
- `GET /health/gemstone`

`/health/gemstone` uses a `SessionWorkerPool` and returns `{"result":7}` when
credentials are configured and the stone is reachable.

For development tools or container startup checks, use `router_from_env(2)` or
`router_from_env_with_name(2, "my service")`. Those functions let the service
start even when credentials are missing; `/health/gemstone` returns a `503`
JSON error until the GemStone pool can be created.
