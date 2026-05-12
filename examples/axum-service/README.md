# Axum Service Example

This is a Rust-native web-service sketch for teams that want `gemstone-rs` in
an Axum application. It is intentionally kept outside the workspace so the core
crate stays dependency-light.

Create a new service:

```bash
cargo new gemstone-rs-axum-demo
cd gemstone-rs-axum-demo
cargo add axum tokio --features tokio/full
cargo add gemstone-rs
```

Use a short handler that opens a GemStone session for the request:

```rust
use axum::{routing::get, Json, Router};
use gemstone_rs::{Config, Session, Value};
use serde_json::json;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "gemstone-rs Axum demo" }))
        .route("/health/gemstone", get(gemstone_health));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn gemstone_health() -> Json<serde_json::Value> {
    let result = tokio::task::spawn_blocking(|| -> gemstone_rs::Result<i64> {
        let mut session = Session::login(Config::from_env()?)?;
        let Value::SmallInt(value) = session.eval("3 + 4")? else {
            return Ok(-1);
        };
        session.logout()?;
        Ok(value)
    })
    .await
    .unwrap();

    match result {
        Ok(value) => Json(json!({ "result": value })),
        Err(err) => Json(json!({ "error": err.to_string() })),
    }
}
```

Keep one important rule: treat each GemStone `Session` as thread-local and
blocking. Use `spawn_blocking`, a worker thread, or an explicit session pool
after GCI threading behavior is fully proven for your deployment.
