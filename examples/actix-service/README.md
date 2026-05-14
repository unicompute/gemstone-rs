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
cargo run --manifest-path examples/actix-service/Cargo.toml -- --host 127.0.0.1 --port 3000
```

Then in another shell:

```bash
curl -i http://127.0.0.1:3000/
curl -i http://127.0.0.1:3000/health/local
curl -i http://127.0.0.1:3000/health/gemstone
```

The GemStone health route opens a session inside `actix_web::web::block` and
evaluates `3 + 4`. Keep that pattern unless you introduce a deliberate session
worker or session pool for your deployment. Treat each GemStone `Session` as
thread-local and blocking.

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
