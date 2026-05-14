# Actix Service Example

This is a checked Rust-native Actix Web service for teams that want
`gemstone-rs` inside an async web application. It is intentionally kept outside
the main workspace so the core crate stays dependency-light.

It exposes the same minimal contract used by the Python FastAPI/Litestar
examples:

```text
GET /
GET /health/local
GET /health/gemstone
```

Check the route map without starting a server:

```bash
cargo run --manifest-path examples/actix-service/Cargo.toml -- --routes
```

Run the local service:

```bash
cargo run --manifest-path examples/actix-service/Cargo.toml -- --host 127.0.0.1 --port 3000 --workers 2
```

Then in another shell:

```bash
curl -i http://127.0.0.1:3000/
curl -i http://127.0.0.1:3000/health/local
curl -i http://127.0.0.1:3000/health/gemstone
```

The service starts a bounded `SessionWorkerPool` and the GemStone health route
uses the shared `gemstone_rs::web` response helper inside
`actix_web::web::block`. Each underlying GemStone `Session` stays pinned to one
worker thread.

Use the installed scaffold when you want to start a separate application:

```bash
gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service
```

The dependency-free source-checkout service remains useful for checking the same
route contract without Axum, Actix, Tokio, or serde:

```bash
cargo run -p gemstone-rs --example http_service -- --routes
cargo run -p gemstone-rs --example http_service -- --port 3000
```
