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
