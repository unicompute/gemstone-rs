# Axum Service Example

This is a checked Rust-native Axum service for teams that want `gemstone-rs`
inside an async web application. It uses the reusable `gemstone-rs-axum`
adapter crate so application code does not copy health-route handlers.

It exposes the same minimal contract used by the Python FastAPI/Litestar
examples:

```text
GET /
GET /health/local
GET /health/gemstone
```

Check the route map without starting a server:

```bash
cargo run --manifest-path examples/axum-service/Cargo.toml -- --routes
```

Run the local service:

```bash
cargo run --manifest-path examples/axum-service/Cargo.toml -- --host 127.0.0.1 --port 3000 --workers 2
```

Then in another shell:

```bash
curl -i http://127.0.0.1:3000/
curl -i http://127.0.0.1:3000/health/local
curl -i http://127.0.0.1:3000/health/gemstone
```

The service starts a bounded `SessionWorkerPool` and passes it to
`gemstone_rs_axum::router_with_name`. The adapter wraps the shared
`gemstone_rs::web` response helper inside `tokio::task::spawn_blocking`. Each
underlying GemStone `Session` stays pinned to one worker thread.

Use the installed scaffold when you want to start a separate application:

```bash
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
```

The dependency-free source-checkout service remains useful for checking the
same route contract without Axum/Tokio:

```bash
cargo run -p gemstone-rs --example http_service -- --routes
cargo run -p gemstone-rs --example http_service -- --port 3000
```
