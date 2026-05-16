# gemstone-rs-actix

`gemstone-rs-actix` provides small Actix Web adapter helpers for Rust services
that talk directly to GemStone/S through `gemstone-rs`.

```rust,no_run
use actix_web::{App, HttpServer};
use gemstone_rs_actix::{pool_from_env, scope_with_name};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = pool_from_env(2)?;
    HttpServer::new(move || {
        App::new().service(scope_with_name(pool.clone(), "my GemStone service"))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await?;
    Ok(())
}
```

The scope exposes:

- `GET /`
- `GET /health/local`
- `GET /health/gemstone`

`/health/gemstone` uses a `SessionWorkerPool` and returns `{"result":7}` when
credentials are configured and the stone is reachable.

For development tools or container startup checks, use `scope_from_env(2)` or
`scope_from_env_with_name(2, "my service")`. Those functions let the service
start even when credentials are missing; `/health/gemstone` returns a `503`
JSON error until the GemStone pool can be created.

Every adapter response also includes diagnostic headers:

- `x-gemstone-rs-adapter: actix`
- `x-gemstone-rs-route: root`, `health.local`, or `health.gemstone`
- `x-gemstone-rs-request-id`
- `x-gemstone-rs-request-method`
- `x-gemstone-rs-request-path`
- `x-gemstone-rs-request-lifecycle: received,handled`
- `x-gemstone-rs-request-duration-us`

If the caller sends `x-request-id`, the adapter propagates that value into
`x-gemstone-rs-request-id`; otherwise it generates an `actix-*` request id.
`x-gemstone-rs-request-duration-us` measures the packaged adapter handler in
microseconds, so route smoke tests and local proxies can assert request
lifecycle behavior without adding framework-specific middleware.
